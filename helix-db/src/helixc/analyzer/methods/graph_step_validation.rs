//! Semantic analyzer for Helix‑QL.
use crate::{
    helix_engine::graph_core::ops::source::add_e::EdgeType,
    helixc::{
        analyzer::{
            analyzer::Ctx,
            diagnostic::Diagnostic,
            errors::{push_query_err, push_query_err_with_fix, push_query_warn, push_schema_err},
            methods::{
                traversal_validation::check_traversal,
                query_validation::check_query,
                schema_methods::{build_field_lookups, check_schema},
            },
            types::Type,
            utils::{
                gen_id_access_or_param, gen_identifier_or_param, is_valid_identifier, type_in_scope,
            },
        },
        generator::{
            bool_op::{BoolOp, Eq, Gt, Gte, Lt, Lte, Neq},
            generator_types::{
                Assignment as GeneratedAssignment, BoExp, Drop as GeneratedDrop,
                ForEach as GeneratedForEach, ForLoopInVariable, ForVariable,
                Parameter as GeneratedParameter, Query as GeneratedQuery, ReturnType, ReturnValue,
                ReturnValueExpr, Source as GeneratedSource, Statement as GeneratedStatement,
            },
            object_remapping_generation::{
                ExcludeField, IdentifierRemapping, ObjectRemapping, Remapping, RemappingType,
                TraversalRemapping, ValueRemapping,
            },
            source_steps::{
                AddE, AddN, AddV, EFromID, EFromType, NFromID, NFromIndex, NFromType, SearchBM25,
                SearchVector as GeneratedSearchVector, SourceStep,
            },
            traversal_steps::{
                In as GeneratedIn, InE as GeneratedInE, OrderBy, Out as GeneratedOut,
                OutE as GeneratedOutE, Range, SearchVectorStep,
                ShortestPath as GeneratedShortestPath, ShouldCollect, Step as GeneratedStep,
                Traversal as GeneratedTraversal, TraversalType, Where, WhereExists, WhereRef,
            },
            utils::{GenRef, GeneratedValue, Order, Separator, VecData},
        },
        parser::{helix_parser::*, location::Loc},
    },
    protocol::{date::Date, value::Value},
    utils::styled_string::StyledString,
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    convert::Infallible,
};

/// Check that a graph‑navigation step is allowed for the current element
/// kind and return the post‑step kind.
pub(crate) fn apply_graph_step<'a>(
    ctx: &mut Ctx<'a>,
    gs: &'a GraphStep,
    cur_ty: &Type,
    q: &'a Query,
    traversal: &mut GeneratedTraversal,
    scope: &mut HashMap<&'a str, Type>,
) -> Option<Type> {
    use GraphStepType::*;
    match (&gs.step, cur_ty.base()) {
        // Node‑to‑Edge
        (
            OutE(label),
            Type::Nodes(Some(node_label))
            | Type::Node(Some(node_label))
            | Type::Vectors(Some(node_label))
            | Type::Vector(Some(node_label)),
        ) => {
            traversal
                .steps
                .push(Separator::Period(GeneratedStep::OutE(GeneratedOutE {
                    label: GenRef::Literal(label.clone()),
                })));
            traversal.should_collect = ShouldCollect::ToVec;
            let edge = ctx.edge_map.get(label.as_str());
            if edge.is_none() {
                push_query_err(
                    ctx,
                    q,
                    gs.loc.clone(),
                    format!("Edge of type `{}` does not exist", label),
                    "check the schema for valid edge types",
                );
                return None;
            }
            match edge.unwrap().from.1 == node_label.clone() {
                true => Some(Type::Edges(Some(label.to_string()))),
                false => {
                    push_query_err(
                        ctx,
                        q,
                        gs.loc.clone(),
                        format!(
                            "Edge of type `{}` exists but it is not a valid outgoing edge type for node of type `{}`",
                            label, node_label
                        ),
                        "check the schema for valid edge types",
                    );
                    None
                }
            }
        }
        (
            InE(label),
            Type::Nodes(Some(node_label))
            | Type::Node(Some(node_label))
            | Type::Vectors(Some(node_label))
            | Type::Vector(Some(node_label)),
        ) => {
            traversal
                .steps
                .push(Separator::Period(GeneratedStep::InE(GeneratedInE {
                    label: GenRef::Literal(label.clone()),
                })));
            traversal.should_collect = ShouldCollect::ToVec;
            let edge = ctx.edge_map.get(label.as_str());
            if edge.is_none() {
                push_query_err(
                    ctx,
                    q,
                    gs.loc.clone(),
                    format!("Edge of type `{}` does not exist", label),
                    "check the schema for valid edge types",
                );
                return None;
            }

            match edge.unwrap().to.1 == node_label.clone() {
                true => Some(Type::Edges(Some(label.to_string()))),
                false => {
                    push_query_err(
                        ctx,
                        q,
                        gs.loc.clone(),
                        format!("Edge of type `{}` does not exist", label),
                        "check the schema for valid edge types",
                    );
                    None
                }
            }
        }

        // Node‑to‑Node
        (
            Out(label),
            Type::Nodes(Some(node_label))
            | Type::Node(Some(node_label))
            | Type::Vectors(Some(node_label))
            | Type::Vector(Some(node_label)),
        ) => {
            let edge_type = match ctx.edge_map.get(label.as_str()) {
                Some(ref edge) => {
                    if ctx.node_set.contains(edge.to.1.as_str()) {
                        EdgeType::Node
                    } else if ctx.vector_set.contains(edge.to.1.as_str()) {
                        EdgeType::Vec
                    } else {
                        panic!("Edge of type `{}` does not exist", label);
                    }
                }
                None => {
                    push_query_err(
                        ctx,
                        q,
                        gs.loc.clone(),
                        format!("Edge of type `{}` does not exist", label),
                        "check the schema for valid edge types",
                    );
                    return None;
                }
            };
            traversal
                .steps
                .push(Separator::Period(GeneratedStep::Out(GeneratedOut {
                    edge_type: GenRef::Ref(edge_type.to_string()),
                    label: GenRef::Literal(label.clone()),
                })));
            traversal.should_collect = ShouldCollect::ToVec;
            let edge = ctx.edge_map.get(label.as_str());
            // assert!(edge.is_some()); // make sure is caught
            if edge.is_none() {
                push_query_err(
                    ctx,
                    q,
                    gs.loc.clone(),
                    format!("Edge of type `{}` does not exist", label),
                    "check the schema for valid edge types",
                );
                return None;
            }
            match edge.unwrap().from.1 == node_label.clone() {
                true => {
                    if EdgeType::Node == edge_type {
                        Some(Type::Nodes(Some(edge.unwrap().to.1.clone())))
                    } else if EdgeType::Vec == edge_type {
                        Some(Type::Vectors(Some(edge.unwrap().to.1.clone())))
                    } else {
                        None
                    }
                }
                false => {
                    push_query_err(
                        ctx,
                        q,
                        gs.loc.clone(),
                        format!(
                            "Edge of type `{}` exists but it is not a valid outgoing edge type for node of type `{}`",
                            label, node_label
                        ),
                        "check the schema for valid edge types",
                    );
                    None
                }
            }
        }

        (
            In(label),
            Type::Nodes(Some(node_label))
            | Type::Node(Some(node_label))
            | Type::Vectors(Some(node_label))
            | Type::Vector(Some(node_label)),
        ) => {
            let edge_type = match ctx.edge_map.get(label.as_str()) {
                Some(ref edge) => {
                    if ctx.node_set.contains(edge.from.1.as_str()) {
                        EdgeType::Node
                    } else if ctx.vector_set.contains(edge.from.1.as_str()) {
                        EdgeType::Vec
                    } else {
                        push_query_err(
                            ctx,
                            q,
                            gs.loc.clone(),
                            format!("Edge of type `{}` does not exist", label),
                            "check the schema for valid edge types",
                        );
                        return None;
                    }
                }
                None => {
                    unreachable!()
                }
            };

            traversal
                .steps
                .push(Separator::Period(GeneratedStep::In(GeneratedIn {
                    edge_type: GenRef::Ref(edge_type.to_string()),
                    label: GenRef::Literal(label.clone()),
                })));
            traversal.should_collect = ShouldCollect::ToVec;
            let edge = ctx.edge_map.get(label.as_str());
            // assert!(edge.is_some());
            if edge.is_none() {
                push_query_err(
                    ctx,
                    q,
                    gs.loc.clone(),
                    format!("Edge of type `{}` does not exist", label),
                    "check the schema for valid edge types",
                );
                return None;
            }

            match edge.unwrap().to.1 == node_label.clone() {
                true => {
                    if EdgeType::Node == edge_type {
                        Some(Type::Nodes(Some(edge.unwrap().from.1.clone())))
                    } else if EdgeType::Vec == edge_type {
                        Some(Type::Vectors(Some(edge.unwrap().from.1.clone())))
                    } else {
                        None
                    }
                }
                false => {
                    push_query_err(
                        ctx,
                        q,
                        gs.loc.clone(),
                        format!(
                            "Edge of type `{}` exists but it is not a valid incoming edge type for node of type `{}`",
                            label, node_label
                        ),
                        "check the schema for valid edge types",
                    );
                    None
                }
            }
        }

        // Edge‑to‑Node
        (FromN, Type::Edges(Some(edge_ty)) | Type::Edge(Some(edge_ty))) => {
            let new_ty = if let Some(edge_schema) = ctx.edge_map.get(edge_ty.as_str()) {
                let node_type = &edge_schema.from.1;
                if !ctx.node_set.contains(node_type.as_str()) {
                    push_query_err(
                        ctx,
                        q,
                        gs.loc.clone(),
                        format!(
                            "edge type `{}` does not have a node type as its `From` source",
                            edge_ty
                        ),
                        format!("set the `From` type of the edge to a node type"),
                    );
                }
                match cur_ty {
                    Type::Edges(_) => Some(Type::Nodes(Some(node_type.clone()))),
                    Type::Edge(_) => Some(Type::Node(Some(node_type.clone()))),
                    _ => None,
                }
            } else {
                None
            };
            traversal
                .steps
                .push(Separator::Period(GeneratedStep::FromN));
            traversal.should_collect = ShouldCollect::ToVec;
            new_ty
        }
        (ToN, Type::Edges(Some(edge_ty)) | Type::Edge(Some(edge_ty))) => {
            let new_ty = if let Some(edge_schema) = ctx.edge_map.get(edge_ty.as_str()) {
                let node_type = &edge_schema.to.1;
                if !ctx.node_set.contains(node_type.as_str()) {
                    push_query_err(
                        ctx,
                        q,
                        gs.loc.clone(),
                        format!(
                            "edge type `{}` does not have a node type as its `To` target",
                            edge_ty
                        ),
                        format!("set the `To` type of the edge to a node type"),
                    );
                }
                match cur_ty {
                    Type::Edges(_) => Some(Type::Nodes(Some(node_type.clone()))),
                    Type::Edge(_) => Some(Type::Node(Some(node_type.clone()))),
                    _ => None,
                }
            } else {
                None
            };
            traversal.steps.push(Separator::Period(GeneratedStep::ToN));
            traversal.should_collect = ShouldCollect::ToVec;
            new_ty
        }
        (FromV, Type::Edges(Some(edge_ty)) | Type::Edge(Some(edge_ty))) => {
            // Get the source vector type from the edge schema
            let new_ty = if let Some(edge_schema) = ctx.edge_map.get(edge_ty.as_str()) {
                let source_type = &edge_schema.from.1;
                if !ctx.vector_set.contains(source_type.as_str()) {
                    push_query_err(
                        ctx,
                        q,
                        gs.loc.clone(),
                        format!(
                            "edge type `{}` does not have a vector type as its `From` source",
                            edge_ty
                        ),
                        format!("set the `From` type of the edge to a vector type"),
                    );
                }
                match cur_ty {
                    Type::Edges(_) => Some(Type::Vectors(Some(source_type.clone()))),
                    Type::Edge(_) => Some(Type::Vector(Some(source_type.clone()))),
                    _ => None,
                }
            } else {
                None
            };
            traversal
                .steps
                .push(Separator::Period(GeneratedStep::FromV));
            traversal.should_collect = ShouldCollect::ToVec;
            new_ty
        }
        (ToV, Type::Edges(Some(edge_ty)) | Type::Edge(Some(edge_ty))) => {
            // Get the target vector type from the edge schema
            let new_ty = if let Some(edge_schema) = ctx.edge_map.get(edge_ty.as_str()) {
                let target_type = &edge_schema.to.1;
                if !ctx.vector_set.contains(target_type.as_str()) {
                    push_query_err(
                        ctx,
                        q,
                        gs.loc.clone(),
                        format!(
                            "edge type `{}` does not have a vector type as its `To` target",
                            edge_ty
                        ),
                        format!("set the `To` type of the edge to a vector type"),
                    );
                }
                match cur_ty {
                    Type::Edges(_) => Some(Type::Vectors(Some(target_type.clone()))),
                    Type::Edge(_) => Some(Type::Vector(Some(target_type.clone()))),
                    _ => None,
                }
            } else {
                None
            };
            traversal.steps.push(Separator::Period(GeneratedStep::ToV));
            traversal.should_collect = ShouldCollect::ToVec;
            new_ty
        }
        (ShortestPath(sp), Type::Nodes(_) | Type::Node(_)) => {
            let type_arg = match sp.type_arg.clone() {
                Some(type_arg) => Some(GenRef::Literal(type_arg)),
                None => None,
            };
            // check edge type is valid
            traversal
                .steps
                .push(Separator::Period(GeneratedStep::ShortestPath(
                    match (sp.from.clone(), sp.to.clone()) {
                        // TODO: get rid of clone
                        (Some(from), Some(to)) => GeneratedShortestPath {
                            label: type_arg,
                            from: Some(GenRef::from(from)),
                            to: Some(GenRef::from(to)),
                        },
                        (Some(from), None) => GeneratedShortestPath {
                            label: type_arg,
                            from: Some(GenRef::from(from)),
                            to: None,
                        },
                        (None, Some(to)) => GeneratedShortestPath {
                            label: type_arg,
                            from: None,
                            to: Some(GenRef::from(to)),
                        },
                        (None, None) => panic!("Invalid shortest path"),
                    },
                )));
            traversal.should_collect = ShouldCollect::ToVec;
            Some(Type::Unknown)
        }
        (SearchVector(sv), Type::Vectors(Some(vector_ty)) | Type::Vector(Some(vector_ty))) => {
            if !(matches!(cur_ty, Type::Vector(_)) || matches!(cur_ty, Type::Vectors(_))) {
                push_query_err(
                    ctx,
                    q,
                    sv.loc.clone(),
                    format!(
                        "`SearchVector` must be used on a vector type, got `{}`, which is of type `{}`",
                        cur_ty.get_type_name(),
                        cur_ty.kind_str()
                    ),
                    "ensure the result of the previous step is a vector type",
                );
            }
            if let Some(ref ty) = sv.vector_type {
                if !ctx.vector_set.contains(ty.as_str()) {
                    push_query_err(
                        ctx,
                        q,
                        sv.loc.clone(),
                        format!("vector type `{}` has not been declared", ty),
                        format!("add a `V::{}` schema first", ty),
                    );
                }
            }
            let vec = match &sv.data {
                Some(VectorData::Vector(v)) => {
                    VecData::Standard(GeneratedValue::Literal(GenRef::Ref(format!(
                        "[{}]",
                        v.iter()
                            .map(|f| f.to_string())
                            .collect::<Vec<String>>()
                            .join(",")
                    ))))
                }
                Some(VectorData::Identifier(i)) => {
                    is_valid_identifier(ctx, q, sv.loc.clone(), i.as_str());
                    // if is in params then use data.
                    if let Some(_) = q.parameters.iter().find(|p| p.name.1 == *i) {
                        VecData::Standard(GeneratedValue::Identifier(GenRef::Ref(format!(
                            "data.{}",
                            i.to_string()
                        ))))
                    } else if let Some(_) = scope.get(i.as_str()) {
                        VecData::Standard(GeneratedValue::Identifier(GenRef::Ref(i.to_string())))
                    } else {
                        push_query_err(
                            ctx,
                            q,
                            sv.loc.clone(),
                            format!("variable named `{}` is not in scope", i),
                            "declare {} in the current scope or fix the typo",
                        );
                        VecData::Standard(GeneratedValue::Unknown)
                    }
                }
                Some(VectorData::Embed(e)) => match &e.value {
                    EvaluatesToString::Identifier(i) => {
                        VecData::Embed(gen_identifier_or_param(ctx, q, &i, true, false))
                    }
                    EvaluatesToString::StringLiteral(s) => {
                        VecData::Embed(GeneratedValue::Literal(GenRef::Ref(s.clone())))
                    }
                },
                _ => {
                    push_query_err(
                        ctx,
                        q,
                        sv.loc.clone(),
                        "`SearchVector` must have a vector data".to_string(),
                        "add a vector data",
                    );
                    VecData::Standard(GeneratedValue::Unknown)
                }
            };
            let k = match &sv.k {
                Some(k) => match &k.value {
                    EvaluatesToNumberType::I8(i) => {
                        GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                    }
                    EvaluatesToNumberType::I16(i) => {
                        GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                    }
                    EvaluatesToNumberType::I32(i) => {
                        GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                    }
                    EvaluatesToNumberType::I64(i) => {
                        GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                    }
                    EvaluatesToNumberType::U8(i) => {
                        GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                    }
                    EvaluatesToNumberType::U16(i) => {
                        GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                    }
                    EvaluatesToNumberType::U32(i) => {
                        GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                    }
                    EvaluatesToNumberType::U64(i) => {
                        GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                    }
                    EvaluatesToNumberType::U128(i) => {
                        GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                    }
                    EvaluatesToNumberType::Identifier(i) => {
                        is_valid_identifier(ctx, q, sv.loc.clone(), i.as_str());
                        // is param
                        if let Some(_) = q.parameters.iter().find(|p| p.name.1 == *i) {
                            GeneratedValue::Identifier(GenRef::Std(format!("data.{} as usize", i)))
                        } else {
                            GeneratedValue::Identifier(GenRef::Std(i.to_string()))
                        }
                    }
                    _ => {
                        push_query_err(
                            ctx,
                            q,
                            sv.loc.clone(),
                            "`SearchVector` must have a limit of vectors to return".to_string(),
                            "add a limit",
                        );
                        GeneratedValue::Unknown
                    }
                },
                None => {
                    push_query_err(
                        ctx,
                        q,
                        sv.loc.clone(),
                        "`SearchV` must have a limit of vectors to return".to_string(),
                        "add a limit",
                    );
                    GeneratedValue::Unknown
                }
            };

            // Search returns nodes that contain the vectors

            // Some(GeneratedStatement::Traversal(GeneratedTraversal {
            //     traversal_type: TraversalType::Ref,
            //     steps: vec![],
            //     should_collect: ShouldCollect::ToVec,
            //     source_step: Separator::Period(SourceStep::SearchVector(
            //         GeneratedSearchVector { vec, k, pre_filter },
            //     )),
            // }))
            traversal
                .steps
                .push(Separator::Period(GeneratedStep::SearchVector(
                    SearchVectorStep { vec, k },
                )));
            // traversal.traversal_type = TraversalType::Ref;
            traversal.should_collect = ShouldCollect::ToVec;
            Some(Type::Vectors(Some(vector_ty.clone())))
        }
        // Anything else is illegal
        _ => {
            push_query_err(
                ctx,
                q,
                gs.loc.clone(),
                format!(
                    "traversal step `{}` cannot follow a step that returns {}",
                    gs.loc
                        .span
                        .trim_matches(|c: char| c == '"' || c.is_whitespace() || c == '\n')
                        .bold(),
                    cur_ty
                        .kind_str()
                        .trim_matches(|c: char| c == '"' || c.is_whitespace() || c == '\n')
                        .bold()
                ),
                get_traversal_step_hint(cur_ty, &gs.step).as_str(),
            );
            None
        }
    }
}

pub(crate) fn get_traversal_step_hint<'a>(
    current_step: &Type,
    next_step: &GraphStepType,
) -> String {
    match (current_step, next_step) {
        (
            Type::Nodes(Some(span))
            | Type::Node(Some(span))
            | Type::Vectors(Some(span))
            | Type::Vector(Some(span)),
            GraphStepType::ToN | GraphStepType::FromN,
        ) => {
            format!(
                "\n{}\n{}",
                format!(
                    "      • Use `OutE` or `InE` to traverse edges from `{}`",
                    span
                ),
                format!(
                    "      • Use `Out` or `In` to traverse nodes from `{}`",
                    span
                ),
            )
        }
        (Type::Edges(Some(span)), GraphStepType::OutE(_) | GraphStepType::InE(_)) => {
            format!("use `FromN` or `ToN` to traverse nodes from `{}`", span)
        }
        (Type::Edges(Some(span)), GraphStepType::Out(_) | GraphStepType::In(_)) => {
            format!("use `FromN` or `ToN` to traverse nodes from `{}`", span)
        }

        (_, _) => {
            println!(
                "get_traversal_step_hint: {:?}, {:?}",
                current_step, next_step
            );
            "re-order the traversal or remove the invalid step".to_string()
        }
    }
}
