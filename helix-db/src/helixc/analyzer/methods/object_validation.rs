//! Semantic analyzer for Helix‑QL.
use crate::{
    helixc::{
        analyzer::{
            analyzer::Ctx, errors::push_query_err, methods::traversal_validation::check_traversal, types::Type, utils::is_valid_identifier
        },
        generator::{

            generator_types::{

                Query as GeneratedQuery,
            },
            object_remapping_generation::{
                IdentifierRemapping, ObjectRemapping, Remapping, RemappingType,
                TraversalRemapping, ValueRemapping,
            },
            source_steps::SourceStep,
            traversal_steps::{
                ShouldCollect, Step as GeneratedStep,
                Traversal as GeneratedTraversal, TraversalType,
            },
            utils::{GenRef, Separator},
        },
        parser::{helix_parser::*, location::Loc},
    },
};
use std::{borrow::Cow, collections::HashMap};

pub(crate) fn validate_object<'a>(
        ctx: &mut Ctx<'a>,
        cur_ty: &Type,
        tr: &Traversal,
        obj: &'a Object,
        excluded: &HashMap<&str, Loc>,
        q: &'a Query,
        gen_traversal: &mut GeneratedTraversal,
        gen_query: Option<&mut GeneratedQuery>,
        scope: &mut HashMap<&'a str, Type>,
        var_name: Option<&str>,
    ) {
        match &cur_ty {
            Type::Node(Some(node_ty)) | Type::Nodes(Some(node_ty)) => {
                validate_property_access(ctx, obj, q, scope, var_name, gen_traversal, cur_ty, ctx.node_fields.get(node_ty.as_str()).cloned());
            }
            Type::Edge(Some(edge_ty)) | Type::Edges(Some(edge_ty)) => {
                validate_property_access(ctx, obj, q, scope, var_name, gen_traversal, cur_ty, ctx.edge_fields.get(edge_ty.as_str()).cloned());
            }
            Type::Vector(Some(vector_ty)) | Type::Vectors(Some(vector_ty)) => {
                validate_property_access(ctx, obj, q, scope, var_name, gen_traversal, cur_ty, ctx.vector_fields.get(vector_ty.as_str()).cloned());
            }
            Type::Anonymous(ty) => {
                validate_object(ctx,
                    ty,
                    tr,
                    obj,
                    excluded,
                    q,
                    gen_traversal,
                    gen_query,
                    scope,
                    var_name,
                );
            }
            _ => {
                push_query_err(
                    ctx,
                    q,
                    obj.fields[0].value.loc.clone(),
                    "cannot access properties on this type".to_string(),
                    "property access is only valid on nodes, edges and vectors",
                );
            }
        }
    }

   
    pub(crate) fn parse_object_remapping<'a>(
        ctx: &mut Ctx<'a>,
        obj: &'a Vec<FieldAddition>,
        q: &'a Query,
        is_inner: bool,
        scope: &mut HashMap<&'a str, Type>,
        var_name: &str,
        parent_ty: Type,
    ) -> Remapping {
        let remappings = obj
            .into_iter()
            .map(|FieldAddition { key, value, .. }| {
                match &value.value {
                    // if the field value is a traversal then it is a TraversalRemapping
                    FieldValueType::Traversal(traversal) => {
                        let mut inner_traversal = GeneratedTraversal::default();
                        check_traversal(
                            ctx,
                            &traversal,
                            scope,
                            q,
                            Some(parent_ty.clone()),
                            &mut inner_traversal,
                            None,
                        );
                        match &traversal.start {
                            StartNode::Identifier(name) => {
                                if name.to_string() == var_name {
                                    inner_traversal.traversal_type = TraversalType::NestedFrom(
                                        GenRef::Std(var_name.to_string()),
                                    );
                                } else {
                                    inner_traversal.traversal_type =
                                        TraversalType::FromVar(GenRef::Std(name.to_string()));
                                }
                            }
                            _ => {
                                inner_traversal.traversal_type =
                                    TraversalType::NestedFrom(GenRef::Std(var_name.to_string()));
                            }
                        };

                        match &traversal.steps.last() {
                            Some(step) => {
                                match step.step  {
                                    StepType::Count | StepType::BooleanOperation(_) => {
                                        return RemappingType::ValueRemapping(ValueRemapping {
                                            variable_name: var_name.to_string(),
                                            field_name: key.clone(),
                                            value: GenRef::Std(inner_traversal.to_string()),
                                        })
                                    }
                                    _ => {
                                        RemappingType::TraversalRemapping(TraversalRemapping {
                                            variable_name: var_name.to_string(),
                                            new_field: key.clone(),
                                            new_value: inner_traversal,
                                        })
                                    }
                                }
                            }
                            None => {
                                RemappingType::TraversalRemapping(TraversalRemapping {
                                    variable_name: var_name.to_string(),
                                    new_field: key.clone(),
                                    new_value: inner_traversal,
                                })
                            }
                        }
                    }
                    FieldValueType::Expression(expr) => {
                        match &expr.expr {
                            ExpressionType::Traversal(traversal) => {
                                let mut inner_traversal = GeneratedTraversal::default();
                                check_traversal(
                                    ctx,
                                    &traversal,
                                    scope,
                                    q,
                                    Some(parent_ty.clone()),
                                    &mut inner_traversal,
                                    None,
                                );
                                match &traversal.start {
                                    StartNode::Identifier(name) => {
                                        if name.to_string() == var_name {
                                            inner_traversal.traversal_type =
                                                TraversalType::NestedFrom(GenRef::Std(
                                                    var_name.to_string(),
                                                ));
                                        } else {
                                            inner_traversal.traversal_type = TraversalType::FromVar(
                                                GenRef::Std(name.to_string()),
                                            );
                                        }
                                    }
                                    _ => {
                                        inner_traversal.traversal_type = TraversalType::NestedFrom(
                                            GenRef::Std(var_name.to_string()),
                                        );
                                    }
                                };
                                RemappingType::TraversalRemapping(TraversalRemapping {
                                    variable_name: var_name.to_string(),
                                    new_field: key.clone(),
                                    new_value: inner_traversal,
                                })
                            }
                            ExpressionType::Exists(exists) => {
                                todo!()
                            }
                            ExpressionType::BooleanLiteral(bo_lit) => {
                                RemappingType::ValueRemapping(ValueRemapping {
                                    variable_name: var_name.to_string(),
                                    field_name: key.clone(),
                                    value: GenRef::Literal(bo_lit.to_string()), 
                                })
                            }
                            ExpressionType::FloatLiteral(float) => {
                                RemappingType::ValueRemapping(ValueRemapping {
                                    variable_name: var_name.to_string(),
                                    field_name: key.clone(),
                                    value: GenRef::Literal(float.to_string()), 
                                })
                            }
                            ExpressionType::StringLiteral(string) => {
                                RemappingType::ValueRemapping(ValueRemapping {
                                    variable_name: var_name.to_string(),
                                    field_name: key.clone(),
                                    value: GenRef::Literal(string.clone()), 
                                })
                            }
                            ExpressionType::IntegerLiteral(integer) => {
                                RemappingType::ValueRemapping(ValueRemapping {
                                    variable_name: var_name.to_string(),
                                    field_name: key.clone(),
                                    value: GenRef::Literal(integer.to_string()), 
                                })
                            }
                            ExpressionType::Identifier(identifier) => {
                                is_valid_identifier(ctx, q, value.loc.clone(), identifier.as_str());
                                if scope.contains_key(identifier.as_str()) {
                                    return RemappingType::IdentifierRemapping(
                                        IdentifierRemapping {
                                            variable_name: var_name.to_string(),
                                            field_name: key.clone(),
                                            identifier_value: identifier.into(),
                                        },
                                    );
                                } else {
                                    let (is_valid_field, item_type) = match &parent_ty {
                                        Type::Nodes(Some(ty)) | Type::Node(Some(ty)) => (ctx
                                            .node_fields
                                            .get(ty.as_str())
                                            .unwrap()
                                            .contains_key(identifier.as_str()), ty.as_str()),
                                        Type::Edges(Some(ty)) | Type::Edge(Some(ty)) => (ctx
                                            .edge_fields
                                            .get(ty.as_str())
                                            .unwrap()
                                            .contains_key(identifier.as_str()), ty.as_str()),
                                        Type::Vectors(Some(ty)) | Type::Vector(Some(ty)) => (ctx
                                            .vector_fields
                                            .get(ty.as_str())
                                            .unwrap()
                                            .contains_key(identifier.as_str()), ty.as_str()),
                                        _ => unreachable!(),
                                    };
                                    match is_valid_field {
                                        true => {
                                            RemappingType::TraversalRemapping(TraversalRemapping {
                                                variable_name: var_name.to_string(),
                                                new_field: key.clone(),
                                                new_value: GeneratedTraversal {
                                                    traversal_type: TraversalType::NestedFrom(
                                                        GenRef::Std(var_name.to_string()),
                                                    ),
                                                    source_step: Separator::Empty(
                                                        SourceStep::Anonymous,
                                                    ),
                                                    steps: vec![Separator::Period(
                                                        GeneratedStep::PropertyFetch(
                                                            GenRef::Literal(identifier.to_string()),
                                                        ),
                                                    )],
                                                    should_collect: ShouldCollect::ToVal,
                                                },
                                            })
                                        }
                                        false => {
                                            push_query_err(ctx,
                                                q,
                                                expr.loc.clone(),
                                                format!(
                                                    "`{}` is not a field of type `{}` or is not a variable in scope",
                                                    identifier, item_type
                                                ),
                                                "check the schema field names or declare the variable".to_string(),
                                            );
                                            RemappingType::Empty
                                        }
                                    }
                                }
                            }
                            _ => {
                                push_query_err(ctx,
                                    q,
                                    expr.loc.clone(),
                                    "invalid expression".to_string(),
                                    "invalid expression".to_string(),
                                );
                                RemappingType::Empty
                            }
                        }
                    }
                    // if field value is identifier then push field remapping
                    FieldValueType::Literal(lit) => {
                        RemappingType::ValueRemapping(ValueRemapping {
                            variable_name: var_name.to_string(),
                            field_name: key.clone(),
                            value: GenRef::from(lit.clone()), // TODO: Implement
                        })
                    }
                    FieldValueType::Identifier(identifier) => {
                        is_valid_identifier(ctx, q, value.loc.clone(), identifier.as_str());
                        if scope.contains_key(identifier.as_str()) {
                            return RemappingType::IdentifierRemapping(IdentifierRemapping {
                                variable_name: var_name.to_string(),
                                field_name: key.clone(),
                                identifier_value: identifier.into(), // TODO: Implement
                            });
                        } else {
                            let (is_valid_field, item_type) = match &parent_ty {
                                Type::Nodes(Some(ty)) | Type::Node(Some(ty)) => (ctx
                                        .node_fields
                                        .get(ty.as_str())
                                        .unwrap()
                                        .contains_key(identifier.as_str()),
                                    ty.as_str()),
                                Type::Edges(Some(ty)) | Type::Edge(Some(ty)) => (ctx
                                        .edge_fields
                                        .get(ty.as_str())
                                        .unwrap()
                                        .contains_key(identifier.as_str()),
                                    ty.as_str()),
                                Type::Vectors(Some(ty)) | Type::Vector(Some(ty)) => (ctx
                                        .vector_fields
                                        .get(ty.as_str())
                                        .unwrap()
                                        .contains_key(identifier.as_str()),
                                    ty.as_str()),
                                _ => unreachable!(),
                            };
                            match is_valid_field {
                                true => RemappingType::TraversalRemapping(TraversalRemapping {
                                    variable_name: var_name.to_string(),
                                    new_field: key.clone(),
                                    new_value: GeneratedTraversal {
                                        traversal_type: TraversalType::NestedFrom(GenRef::Std(
                                            var_name.to_string(),
                                        )),
                                        source_step: Separator::Empty(SourceStep::Anonymous),
                                        steps: vec![Separator::Period(
                                            GeneratedStep::PropertyFetch(GenRef::Literal(
                                                identifier.to_string(),
                                            )),
                                        )],
                                        should_collect: ShouldCollect::ToVec,
                                    },
                                }),
                                false => {
                                    push_query_err(ctx,
                                        q,
                                        value.loc.clone(),
                                        format!(
                                                    "`{}` is not a field of type `{}` or is not a variable in scope",
                                            identifier, item_type
                                        ),
                                        "check the schema field names or declare the variable".to_string(),
                                    );
                                    RemappingType::Empty
                                }
                            }
                        }
                    }
                    // if the field value is another object or closure then recurse (sub mapping would go where traversal would go)
                    FieldValueType::Fields(fields) => {
                        let remapping = parse_object_remapping(ctx,
                            &fields,
                            q,
                            true,
                            scope,
                            var_name,
                            parent_ty.clone(),
                        );
                        RemappingType::ObjectRemapping(ObjectRemapping {
                            variable_name: var_name.to_string(),
                            field_name: key.clone(),
                            remapping,
                        })
                    } // object or closure
                    FieldValueType::Empty => {
                        push_query_err(ctx,
                            q,
                            obj[0].loc.clone(),
                            "field value is empty".to_string(),
                            "field value must be a literal, identifier, traversal,or object"
                                .to_string(),
                        );
                        RemappingType::Empty
                    } // err
                }
                // cast to a remapping type
            })
            .collect();

        Remapping {
            variable_name: var_name.to_string(),
            is_inner,
            remappings,
            should_spread: false,
        }
    }


fn validate_property_access<'a>(
    ctx: &mut Ctx<'a>,
    obj: &'a Object,
    q: &'a Query,
    scope: &mut HashMap<&'a str, Type>,
    var_name: Option<&str>,
    gen_traversal: &mut GeneratedTraversal,
    cur_ty: &Type,
    fields: Option<HashMap<&'a str, Cow<'a, Field>>>,
) {
    assert!(fields.is_some());
    if let Some(fields) = fields {
    if let Some(_) = fields.get(cur_ty.get_type_name().as_str()).cloned() {
        // if there is only one field then it is a property access
        if obj.fields.len() == 1
            && matches!(obj.fields[0].value.value, FieldValueType::Identifier(_))
        {
            match &obj.fields[0].value.value {
                FieldValueType::Identifier(lit) => {
                    is_valid_identifier(
                        ctx,
                        q,
                        obj.fields[0].value.loc.clone(),
                        lit.as_str(),
                    );
                    gen_traversal.steps.push(Separator::Period(
                        GeneratedStep::PropertyFetch(GenRef::Literal(lit.clone())),
                    ));
                    match cur_ty {
                        Type::Nodes(_) | Type::Edges(_) | Type::Vectors(_) => {
                            gen_traversal.should_collect = ShouldCollect::ToVec;
                        }
                        Type::Node(_) | Type::Edge(_) | Type::Vector(_) => {
                            gen_traversal.should_collect = ShouldCollect::ToVal;
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                _ => unreachable!(),
            }
        } else if obj.fields.len() > 0 {
            // if there are multiple fields then it is a field remapping
            // push object remapping where
            let remapping = match var_name {
                Some(var_name) => parse_object_remapping(ctx,
                    &obj.fields,
                    q,
                    false,
                    scope,
                    var_name,
                    cur_ty.clone(),
                ),
                None => parse_object_remapping(ctx,
                    &obj.fields,
                    q,
                    false,
                    scope,
                    "item",
                    cur_ty.clone(),
                ),
            };

            gen_traversal
                .steps
                .push(Separator::Period(GeneratedStep::Remapping(remapping)));
        } else {
            // error
            push_query_err(
                ctx,
                q,
                obj.fields[0].value.loc.clone(),
                "object must have at least one field".to_string(),
                "object must have at least one field".to_string(),
            );
        }
    }
    }

}