//! Semantic analyzer for Helix‑QL.
use crate::{
    helixc::{
        analyzer::{
            analyzer::Ctx,
            errors::push_query_err,
            methods::traversal_validation::check_traversal,
            types::Type,
            utils::{gen_id_access_or_param, gen_identifier_or_param, is_valid_identifier},
        },
        generator::{
            generator_types::{BoExp, Query as GeneratedQuery, Statement as GeneratedStatement},
            source_steps::{
                AddE, AddN, AddV, SearchBM25, SearchVector as GeneratedSearchVector, SourceStep,
            },
            traversal_steps::{
                ShouldCollect, Step as GeneratedStep, Traversal as GeneratedTraversal,
                TraversalType, Where, WhereRef,
            },
            utils::{GenRef, GeneratedValue, Separator, VecData},
        },
        parser::helix_parser::*,
    },
    protocol::date::Date,
};
use std::collections::HashMap;

pub(crate) fn infer_expr_type<'a>(
    ctx: &mut Ctx<'a>,
    expression: &'a Expression,
    scope: &mut HashMap<&'a str, Type>,
    q: &'a Query,
    parent_ty: Option<Type>,
    gen_query: &mut GeneratedQuery,
) -> (Type, Option<GeneratedStatement>) {
    // TODO: Look at returning statement as well or passing mut query to push to
    use ExpressionType::*;
    let expr: &ExpressionType = &expression.expr;
    match expr {
        Identifier(name) => {
            is_valid_identifier(ctx, q, expression.loc.clone(), name.as_str());
            match scope.get(name.as_str()) {
                Some(t) => (
                    t.clone(),
                    Some(GeneratedStatement::Identifier(GenRef::Std(name.clone()))),
                ),

                None => {
                    push_query_err(
                        ctx,
                        q,
                        expression.loc.clone(),
                        format!("variable named `{}` is not in scope", name),
                        "declare it earlier or fix the typo",
                    );
                    (Type::Unknown, None)
                }
            }
        }

        IntegerLiteral(i) => (
            Type::Scalar(FieldType::I32),
            Some(GeneratedStatement::Literal(GenRef::Literal(i.to_string()))),
        ),
        FloatLiteral(f) => (
            Type::Scalar(FieldType::F64),
            Some(GeneratedStatement::Literal(GenRef::Literal(f.to_string()))),
        ),
        StringLiteral(s) => (
            Type::Scalar(FieldType::String),
            Some(GeneratedStatement::Literal(GenRef::Literal(s.to_string()))),
        ),
        BooleanLiteral(b) => (
            Type::Boolean,
            Some(GeneratedStatement::Literal(GenRef::Literal(b.to_string()))),
        ),

        Traversal(tr) => {
            let mut gen_traversal = GeneratedTraversal::default();
            let final_ty =
                check_traversal(ctx, tr, scope, q, parent_ty, &mut gen_traversal, gen_query);
            // push query
            let stmt = GeneratedStatement::Traversal(gen_traversal);

            if matches!(expr, Exists(_)) {
                (Type::Boolean, Some(stmt))
            } else {
                (final_ty, Some(stmt))
            }
        }

        AddNode(add) => {
            if let Some(ref ty) = add.node_type {
                if !ctx.node_set.contains(ty.as_str()) {
                    push_query_err(
                        ctx,
                        q,
                        add.loc.clone(),
                        format!("`AddN<{}>` refers to unknown node type", ty),
                        "declare the node schema first",
                    );
                }
                let label = GenRef::Literal(ty.clone());

                let node_in_schema = ctx
                    .output
                    .nodes
                    .iter()
                    .find(|n| n.name == ty.as_str())
                    .unwrap()
                    .clone();

                // Validate fields if both type and fields are present
                if let Some(fields) = &add.fields {
                    // Get the field set before validation
                    // TODO: Check field types
                    let field_set = ctx.node_fields.get(ty.as_str()).cloned();
                    if let Some(field_set) = field_set {
                        for (field_name, value) in fields {
                            if !field_set.contains_key(field_name.as_str()) {
                                push_query_err(
                                    ctx,
                                    q,
                                    add.loc.clone(),
                                    format!("`{}` is not a field of node `{}`", field_name, ty),
                                    "check the schema field names",
                                );
                            }
                            match value {
                                ValueType::Identifier { value, loc } => {
                                    if is_valid_identifier(ctx, q, loc.clone(), value.as_str()) {
                                        if !scope.contains_key(value.as_str()) {
                                            push_query_err(
                                                ctx,
                                                q,
                                                loc.clone(),
                                                format!("`{}` is not in scope", value),
                                                "declare it earlier or fix the typo",
                                            );
                                        }
                                    };
                                }
                                ValueType::Literal { value, loc } => {
                                    // check against type
                                    let field_type = ctx
                                        .node_fields
                                        .get(ty.as_str())
                                        .unwrap()
                                        .get(field_name.as_str())
                                        .unwrap()
                                        .field_type
                                        .clone();
                                    if field_type != *value {
                                        push_query_err(ctx,
                                             q,
                                             loc.clone(),
                                             format!("value `{}` is of type `{}`, which does not match type {} declared in the schema for field `{}` on node type `{}`", value, GenRef::from(value.clone()), field_type, field_name, ty),
                                             "ensure the value is of the same type as the field declared in the schema".to_string(),
                                         );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    let mut properties: HashMap<String, GeneratedValue> = fields
                        .iter()
                        .map(|(field_name, value)| {
                            (
                                field_name.clone(),
                                match value {
                                    ValueType::Literal { value, loc } => {
                                        match ctx
                                            .node_fields
                                            .get(ty.as_str())
                                            .unwrap()
                                            .get(field_name.as_str())
                                            .unwrap()
                                            .field_type
                                            == FieldType::Date
                                        {
                                            true => match Date::new(value) {
                                                Ok(date) => GeneratedValue::Literal(
                                                    GenRef::Literal(date.to_rfc3339()),
                                                ),
                                                Err(e) => {
                                                    push_query_err(
                                                        ctx,
                                                        q,
                                                        loc.clone(),
                                                        e.to_string(),
                                                        "ensure the value is a valid date",
                                                    );
                                                    GeneratedValue::Unknown
                                                }
                                            },
                                            false => {
                                                GeneratedValue::Literal(GenRef::from(value.clone()))
                                            }
                                        }
                                    }
                                    ValueType::Identifier { value, loc } => {
                                        is_valid_identifier(ctx, q, loc.clone(), value.as_str());
                                        gen_identifier_or_param(q, value, true, false)
                                    }
                                    v => {
                                        push_query_err(
                                            ctx,
                                            q,
                                            add.loc.clone(),
                                            format!("`{:?}` is not a valid field value", v),
                                            "use a literal or identifier",
                                        );
                                        GeneratedValue::Unknown
                                    }
                                },
                            )
                        })
                        .collect();

                    let default_properties = node_in_schema
                        .properties
                        .iter()
                        .filter_map(|p| p.default_value.clone().map(|v| (p.name.clone(), v)))
                        .collect::<Vec<(String, GeneratedValue)>>();

                    for (field_name, default_value) in default_properties {
                        if !properties.contains_key(field_name.as_str()) {
                            properties.insert(field_name, default_value);
                        }
                    }

                    let secondary_indices = {
                        let secondary_indices = node_in_schema
                            .properties
                            .iter()
                            .filter_map(|p| {
                                matches!(p.is_index, FieldPrefix::Index).then_some(p.name.clone())
                            })
                            .collect::<Vec<_>>();
                        match secondary_indices.is_empty() {
                            true => None,
                            false => Some(secondary_indices),
                        }
                    };

                    let add_n = AddN {
                        label,
                        properties: Some(properties.into_iter().collect()),
                        secondary_indices,
                    };

                    let stmt = GeneratedStatement::Traversal(GeneratedTraversal {
                        source_step: Separator::Period(SourceStep::AddN(add_n)),
                        steps: vec![],
                        traversal_type: TraversalType::Mut,
                        should_collect: ShouldCollect::ToVal,
                    });
                    gen_query.is_mut = true;
                    return (Type::Node(Some(ty.to_string())), Some(stmt));
                }
            }
            push_query_err(
                ctx,
                q,
                add.loc.clone(),
                "`AddN` must have a node type".to_string(),
                "add a node type",
            );
            return (Type::Node(None), None);
        }
        AddEdge(add) => {
            if let Some(ref ty) = add.edge_type {
                if !ctx.edge_map.contains_key(ty.as_str()) {
                    push_query_err(
                        ctx,
                        q,
                        add.loc.clone(),
                        format!("`AddE<{}>` refers to unknown edge type", ty),
                        "declare the edge schema first",
                    );
                }
                let label = GenRef::Literal(ty.clone());
                // Validate fields if both type and fields are present
                let properties = match &add.fields {
                    Some(fields) => {
                        // Get the field set before validation
                        let field_set = ctx.edge_fields.get(ty.as_str()).cloned();
                        if let Some(field_set) = field_set {
                            for (field_name, value) in fields {
                                if !field_set.contains_key(field_name.as_str()) {
                                    push_query_err(
                                        ctx,
                                        q,
                                        add.loc.clone(),
                                        format!("`{}` is not a field of edge `{}`", field_name, ty),
                                        "check the schema field names",
                                    );
                                }

                                match value {
                                    ValueType::Identifier { value, loc } => {
                                        if is_valid_identifier(ctx, q, loc.clone(), value.as_str())
                                        {
                                            if !scope.contains_key(value.as_str()) {
                                                push_query_err(
                                                    ctx,
                                                    q,
                                                    loc.clone(),
                                                    format!("`{}` is not in scope", value),
                                                    "declare it earlier or fix the typo",
                                                );
                                            }
                                        };
                                    }
                                    ValueType::Literal { value, loc } => {
                                        // check against type
                                        let field_type = ctx
                                            .edge_fields
                                            .get(ty.as_str())
                                            .unwrap()
                                            .get(field_name.as_str())
                                            .unwrap()
                                            .field_type
                                            .clone();
                                        if field_type != *value {
                                            push_query_err(ctx,
                                                 q,
                                                 loc.clone(),
                                                 format!("value `{}` is of type `{}`, which does not match type {} declared in the schema for field `{}` on node type `{}`", value, GenRef::from(value.clone()), field_type, field_name, ty),
                                                 "ensure the value is of the same type as the field declared in the schema".to_string(),
                                             );
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Some(
                            fields
                                .iter()
                                .map(|(field_name, value)| {
                                    (
                                        field_name.clone(),
                                        match value {
                                            ValueType::Literal { value, loc } => {
                                                match ctx
                                                    .edge_fields
                                                    .get(ty.as_str())
                                                    .unwrap()
                                                    .get(field_name.as_str())
                                                    .unwrap()
                                                    .field_type
                                                    == FieldType::Date
                                                {
                                                    true => match Date::new(value) {
                                                        Ok(date) => GeneratedValue::Literal(
                                                            GenRef::Literal(date.to_rfc3339()),
                                                        ),
                                                        Err(e) => {
                                                            push_query_err(
                                                                ctx,
                                                                q,
                                                                loc.clone(),
                                                                e.to_string(),
                                                                "ensure the value is a valid date",
                                                            );
                                                            GeneratedValue::Unknown
                                                        }
                                                    },
                                                    false => GeneratedValue::Literal(GenRef::from(
                                                        value.clone(),
                                                    )),
                                                }
                                            }
                                            ValueType::Identifier { value, loc } => {
                                                is_valid_identifier(
                                                    ctx,
                                                    q,
                                                    loc.clone(),
                                                    value.as_str(),
                                                );
                                                gen_identifier_or_param(
                                                    q,
                                                    value.as_str(),
                                                    false,
                                                    true,
                                                )
                                            }
                                            v => {
                                                push_query_err(
                                                    ctx,
                                                    q,
                                                    add.loc.clone(),
                                                    format!("`{:?}` is not a valid field value", v),
                                                    "use a literal or identifier",
                                                );
                                                GeneratedValue::Unknown
                                            }
                                        },
                                    )
                                })
                                .collect(),
                        )
                    }
                    None => None,
                };

                let to = match &add.connection.to_id {
                    Some(id) => match id {
                        IdType::Identifier { value, loc } => {
                            is_valid_identifier(ctx, q, loc.clone(), value.as_str());
                            gen_id_access_or_param(q, value.as_str())
                        }
                        IdType::Literal { value, loc: _ } => {
                            GeneratedValue::Literal(GenRef::Literal(value.clone()))
                        }
                        _ => unreachable!(),
                    },
                    _ => {
                        push_query_err(
                            ctx,
                            q,
                            add.loc.clone(),
                            "`AddE` must have a to id".to_string(),
                            "add a to id",
                        );
                        GeneratedValue::Unknown
                    }
                };
                let from = match &add.connection.from_id {
                    Some(id) => match id {
                        IdType::Identifier { value, loc } => {
                            is_valid_identifier(ctx, q, loc.clone(), value.as_str());
                            gen_id_access_or_param(q, value.as_str())
                        }
                        IdType::Literal { value, loc: _ } => {
                            GeneratedValue::Literal(GenRef::Literal(value.clone()))
                        }
                        _ => unreachable!(),
                    },
                    _ => {
                        push_query_err(
                            ctx,
                            q,
                            add.loc.clone(),
                            "`AddE` must have a from id".to_string(),
                            "add a from id",
                        );
                        GeneratedValue::Unknown
                    }
                };
                let add_e = AddE {
                    to,
                    from,
                    label,
                    properties,
                    // secondary_indices: None, // TODO: Add secondary indices by checking against labeled `INDEX` fields in schema
                };
                let stmt = GeneratedStatement::Traversal(GeneratedTraversal {
                    source_step: Separator::Period(SourceStep::AddE(add_e)),
                    steps: vec![],
                    traversal_type: TraversalType::Mut,
                    should_collect: ShouldCollect::ToVal,
                });
                gen_query.is_mut = true;
                return (Type::Edge(Some(ty.to_string())), Some(stmt));
            }
            push_query_err(
                ctx,
                q,
                add.loc.clone(),
                "`AddE` must have an edge type".to_string(),
                "add an edge type",
            );
            (Type::Edge(None), None)
        }
        AddVector(add) => {
            if let Some(ref ty) = add.vector_type {
                if !ctx.vector_set.contains(ty.as_str()) {
                    push_query_err(
                        ctx,
                        q,
                        add.loc.clone(),
                        format!("vector type `{}` has not been declared", ty),
                        format!("add a `V::{}` schema first", ty),
                    );
                }
                // Validate vector fields
                let (label, properties) = match &add.fields {
                    Some(fields) => {
                        let field_set = ctx.vector_fields.get(ty.as_str()).cloned();
                        if let Some(field_set) = field_set {
                            for (field_name, value) in fields {
                                if !field_set.contains_key(field_name.as_str()) {
                                    push_query_err(
                                        ctx,
                                        q,
                                        add.loc.clone(),
                                        format!(
                                            "`{}` is not a field of vector `{}`",
                                            field_name, ty
                                        ),
                                        "check the schema field names",
                                    );
                                }
                                match value {
                                    ValueType::Identifier { value, loc } => {
                                        if is_valid_identifier(ctx, q, loc.clone(), value.as_str())
                                        {
                                            if !scope.contains_key(value.as_str()) {
                                                push_query_err(
                                                    ctx,
                                                    q,
                                                    loc.clone(),
                                                    format!("`{}` is not in scope", value),
                                                    "declare it earlier or fix the typo",
                                                );
                                            }
                                        };
                                    }
                                    ValueType::Literal { value, loc } => {
                                        // check against type
                                        let field_type = ctx
                                            .vector_fields
                                            .get(ty.as_str())
                                            .unwrap()
                                            .get(field_name.as_str())
                                            .unwrap()
                                            .field_type
                                            .clone();
                                        if field_type != *value {
                                            push_query_err(ctx,
                                                 q,
                                                 loc.clone(),
                                                 format!("value `{}` is of type `{}`, which does not match type {} declared in the schema for field `{}` on node type `{}`", value, GenRef::from(value.clone()), field_type, field_name, ty),
                                                 "ensure the value is of the same type as the field declared in the schema".to_string(),
                                             );
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        let label = GenRef::Literal(ty.clone());
                        let properties = fields
                            .iter()
                            .map(|(field_name, value)| {
                                (
                                    field_name.clone(),
                                    match value {
                                        ValueType::Literal { value, loc } => {
                                            match ctx
                                                .vector_fields
                                                .get(ty.as_str())
                                                .unwrap()
                                                .get(field_name.as_str())
                                                .unwrap()
                                                .field_type
                                                == FieldType::Date
                                            {
                                                true => match Date::new(value) {
                                                    Ok(date) => GeneratedValue::Literal(
                                                        GenRef::Literal(date.to_rfc3339()),
                                                    ),
                                                    Err(e) => {
                                                        push_query_err(
                                                            ctx,
                                                            q,
                                                            loc.clone(),
                                                            e.to_string(),
                                                            "ensure the value is a valid date",
                                                        );
                                                        GeneratedValue::Unknown
                                                    }
                                                },
                                                false => GeneratedValue::Literal(GenRef::from(
                                                    value.clone(),
                                                )),
                                            }
                                        }
                                        ValueType::Identifier { value, loc } => {
                                            is_valid_identifier(
                                                ctx,
                                                q,
                                                loc.clone(),
                                                value.as_str(),
                                            );
                                            gen_identifier_or_param(q, value.as_str(), false, true)
                                        }
                                        v => {
                                            push_query_err(
                                                ctx,
                                                q,
                                                add.loc.clone(),
                                                format!("`{:?}` is not a valid field value", v),
                                                "use a literal or identifier",
                                            );
                                            GeneratedValue::Unknown
                                        }
                                    },
                                )
                            })
                            .collect();
                        (label, Some(properties))
                    }
                    None => (GenRef::Literal(ty.clone()), None),
                };
                if let Some(vec_data) = &add.data {
                    let vec = match vec_data {
                        VectorData::Vector(v) => {
                            VecData::Standard(GeneratedValue::Literal(GenRef::Ref(format!(
                                "[{}]",
                                v.iter()
                                    .map(|f| f.to_string())
                                    .collect::<Vec<String>>()
                                    .join(",")
                            ))))
                        }
                        VectorData::Identifier(i) => {
                            is_valid_identifier(ctx, q, add.loc.clone(), i.as_str());
                            let id = gen_identifier_or_param(q, i.as_str(), true, false);
                            VecData::Standard(id)
                        }
                        VectorData::Embed(e) => match &e.value {
                            EvaluatesToString::Identifier(i) => {
                                VecData::Embed(gen_identifier_or_param(q, i.as_str(), true, false))
                            }
                            EvaluatesToString::StringLiteral(s) => {
                                VecData::Embed(GeneratedValue::Literal(GenRef::Ref(s.clone())))
                            }
                        },
                    };
                    let add_v = AddV {
                        vec,
                        label,
                        properties,
                    };
                    let stmt = GeneratedStatement::Traversal(GeneratedTraversal {
                        source_step: Separator::Period(SourceStep::AddV(add_v)),
                        steps: vec![],
                        traversal_type: TraversalType::Mut,
                        should_collect: ShouldCollect::ToVal,
                    });
                    gen_query.is_mut = true;
                    return (Type::Vector(Some(ty.to_string())), Some(stmt));
                }
            }
            push_query_err(
                ctx,
                q,
                add.loc.clone(),
                "`AddV` must have a vector type".to_string(),
                "add a vector type",
            );
            (Type::Vector(None), None)
        }
        // BatchAddVector(add) => {
        //     if let Some(ref ty) = add.vector_type {
        //         if !ctx.vector_set.contains(ty.as_str()) {
        //             push_query_err(ctx,
        //                 q,
        //                 add.loc.clone(),
        //                 format!("vector type `{}` has not been declared", ty),
        //                 format!("add a `V::{}` schema first", ty),
        //             );
        //         }
        //     }
        //     Type::Vector(add.vector_type.as_deref())
        // }
        SearchVector(sv) => {
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
                Some(VectorData::Vector(v)) => GeneratedValue::Literal(GenRef::Ref(format!(
                    "[{}]",
                    v.iter()
                        .map(|f| f.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                ))),
                Some(VectorData::Identifier(i)) => {
                    is_valid_identifier(ctx, q, sv.loc.clone(), i.as_str());
                    // if is in params then use data.
                    if let Some(_) = q.parameters.iter().find(|p| p.name.1 == *i) {
                        GeneratedValue::Identifier(GenRef::Ref(format!("data.{}", i.to_string())))
                    } else if let Some(_) = scope.get(i.as_str()) {
                        GeneratedValue::Identifier(GenRef::Ref(i.to_string()))
                    } else {
                        push_query_err(
                            ctx,
                            q,
                            sv.loc.clone(),
                            format!("variable named `{}` is not in scope", i),
                            "declare {} in the current scope or fix the typo",
                        );
                        GeneratedValue::Unknown
                    }
                }
                _ => {
                    push_query_err(
                        ctx,
                        q,
                        sv.loc.clone(),
                        "`SearchVector` must have a vector data".to_string(),
                        "add a vector data",
                    );
                    GeneratedValue::Unknown
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
                            "`SearchV` must have a limit of vectors to return".to_string(),
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

            let pre_filter: Option<Vec<BoExp>> = match &sv.pre_filter {
                Some(expr) => {
                    let (_, stmt) = infer_expr_type(
                        ctx,
                        expr,
                        scope,
                        q,
                        Some(Type::Vector(sv.vector_type.clone())),
                        gen_query,
                    );
                    // Where/boolean ops don't change the element type,
                    // so `cur_ty` stays the same.
                    assert!(stmt.is_some());
                    let stmt = stmt.unwrap();
                    let mut gen_traversal = GeneratedTraversal {
                        traversal_type: TraversalType::NestedFrom(GenRef::Std("v".to_string())),
                        steps: vec![],
                        should_collect: ShouldCollect::ToVec,
                        source_step: Separator::Empty(SourceStep::Anonymous),
                    };
                    match stmt {
                        GeneratedStatement::Traversal(tr) => {
                            gen_traversal
                                .steps
                                .push(Separator::Period(GeneratedStep::Where(Where::Ref(
                                    WhereRef {
                                        expr: BoExp::Expr(tr),
                                    },
                                ))));
                        }
                        GeneratedStatement::BoExp(expr) => {
                            gen_traversal
                                .steps
                                .push(Separator::Period(GeneratedStep::Where(match expr {
                                    BoExp::Exists(mut tr) => {
                                        tr.should_collect = ShouldCollect::No;
                                        Where::Ref(WhereRef {
                                            expr: BoExp::Exists(tr),
                                        })
                                    }
                                    _ => Where::Ref(WhereRef { expr }),
                                })));
                        }
                        _ => unreachable!(),
                    }
                    Some(vec![BoExp::Expr(gen_traversal)])
                }
                None => None,
            };

            // Search returns nodes that contain the vectors
            (
                Type::Vectors(sv.vector_type.clone()),
                Some(GeneratedStatement::Traversal(GeneratedTraversal {
                    traversal_type: TraversalType::Ref,
                    steps: vec![],
                    should_collect: ShouldCollect::ToVec,
                    source_step: Separator::Period(SourceStep::SearchVector(
                        GeneratedSearchVector { vec, k, pre_filter },
                    )),
                })),
            )
        }
        And(v) => {
            let exprs = v
                .iter()
                .map(|expr| {
                    let (_, stmt) =
                        infer_expr_type(ctx, expr, scope, q, parent_ty.clone(), gen_query);
                    assert!(
                        stmt.is_some(),
                        "incorrect stmt should've been caught by `infer_expr_type`"
                    );

                    match stmt.unwrap() {
                        GeneratedStatement::BoExp(expr) => {
                            match expr {
                                BoExp::Exists(mut tr) => {
                                    // keep as iterator
                                    tr.should_collect = ShouldCollect::No;
                                    BoExp::Exists(tr)
                                }
                                _ => expr,
                            }
                        }
                        GeneratedStatement::Traversal(tr) => BoExp::Expr(tr),
                        _ => unreachable!(),
                    }
                })
                .collect::<Vec<_>>();
            (
                Type::Boolean,
                Some(GeneratedStatement::BoExp(BoExp::And(exprs))),
            )
        }
        Or(v) => {
            let exprs = v
                .iter()
                .map(|expr| {
                    let (_, stmt) =
                        infer_expr_type(ctx, expr, scope, q, parent_ty.clone(), gen_query);
                    assert!(
                        stmt.is_some(),
                        "incorrect stmt should've been caught by `infer_expr_type`"
                    );
                    match stmt.unwrap() {
                        GeneratedStatement::BoExp(expr) => match expr {
                            BoExp::Exists(mut tr) => {
                                tr.should_collect = ShouldCollect::No;
                                BoExp::Exists(tr)
                            }
                            _ => expr,
                        },
                        GeneratedStatement::Traversal(tr) => BoExp::Expr(tr),
                        _ => unreachable!(),
                    }
                })
                .collect::<Vec<_>>();
            (
                Type::Boolean,
                Some(GeneratedStatement::BoExp(BoExp::Or(exprs))),
            )
        }
        Exists(expr) => {
            let (_, stmt) = infer_expr_type(ctx, expr, scope, q, parent_ty, gen_query);
            assert!(stmt.is_some());
            assert!(matches!(stmt, Some(GeneratedStatement::Traversal(_))));
            let expr = match stmt.unwrap() {
                GeneratedStatement::Traversal(mut tr) => {
                    tr.traversal_type = TraversalType::NestedFrom(GenRef::Std("val".to_string()));
                    tr
                }
                _ => unreachable!(),
            };
            (
                Type::Boolean,
                Some(GeneratedStatement::BoExp(BoExp::Exists(expr))),
            )
        }
        Empty => (Type::Unknown, Some(GeneratedStatement::Empty)),
        BM25Search(bm25_search) => {
            // TODO: look into how best do type checking for type passed in
            if let Some(ref ty) = bm25_search.type_arg {
                if !ctx.node_set.contains(ty.as_str()) {
                    push_query_err(
                        ctx,
                        q,
                        bm25_search.loc.clone(),
                        format!("vector type `{}` has not been declared", ty),
                        format!("add a `V::{}` schema first", ty),
                    );
                }
            }
            let vec = match &bm25_search.data {
                Some(ValueType::Literal { value, loc: _ }) => {
                    GeneratedValue::Literal(GenRef::Std(value.to_string()))
                }
                Some(ValueType::Identifier { value: i, loc: _ }) => {
                    is_valid_identifier(ctx, q, bm25_search.loc.clone(), i.as_str());
                    // if is in params then use data.
                    if let Some(_) = q.parameters.iter().find(|p| p.name.1 == *i) {
                        GeneratedValue::Identifier(GenRef::Ref(format!("data.{}", i.to_string())))
                    } else if let Some(_) = scope.get(i.as_str()) {
                        GeneratedValue::Identifier(GenRef::Ref(i.to_string()))
                    } else {
                        push_query_err(
                            ctx,
                            q,
                            bm25_search.loc.clone(),
                            format!("variable named `{}` is not in scope", i),
                            "declare {} in the current scope or fix the typo",
                        );
                        GeneratedValue::Unknown
                    }
                }
                _ => {
                    push_query_err(
                        ctx,
                        q,
                        bm25_search.loc.clone(),
                        "`SearchVector` must have a vector data".to_string(),
                        "add a vector data",
                    );
                    GeneratedValue::Unknown
                }
            };
            let k = match &bm25_search.k {
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
                        is_valid_identifier(ctx, q, bm25_search.loc.clone(), i.as_str());
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
                            bm25_search.loc.clone(),
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
                        bm25_search.loc.clone(),
                        "`SearchV` must have a limit of vectors to return".to_string(),
                        "add a limit",
                    );
                    GeneratedValue::Unknown
                }
            };

            let search_bm25 = SearchBM25 {
                type_arg: GenRef::Literal(bm25_search.type_arg.clone().unwrap()),
                query: vec,
                k,
            };
            (
                Type::Nodes(bm25_search.type_arg.clone()),
                Some(GeneratedStatement::Traversal(GeneratedTraversal {
                    traversal_type: TraversalType::Ref,
                    steps: vec![],
                    should_collect: ShouldCollect::ToVec,
                    source_step: Separator::Period(SourceStep::SearchBM25(search_bm25)),
                })),
            )
        }
        _ => {
            println!("Unknown expression: {:?}", expr);
            todo!()
        }
    }
}
