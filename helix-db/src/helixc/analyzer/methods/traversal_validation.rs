use crate::{
    helix_engine::graph_core::ops::source::add_e::EdgeType,
    helixc::{
        analyzer::{
            analyzer::Ctx,
            diagnostic::Diagnostic,
            errors::{push_query_err, push_query_err_with_fix, push_query_warn, push_schema_err},
            methods::{
                graph_step_validation::apply_graph_step, exclude_validation::validate_exclude, infer_expr_type::infer_expr_type, object_validation::validate_object, query_validation::check_query, schema_methods::{build_field_lookups, check_schema}
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

// -----------------------------------------------------
// Traversal‑specific checks
// -----------------------------------------------------
pub(crate) fn check_traversal<'a>(
    ctx: &mut Ctx<'a>,
    tr: &'a Traversal,
    scope: &mut HashMap<&'a str, Type>,
    q: &'a Query,
    parent_ty: Option<Type>,
    gen_traversal: &mut GeneratedTraversal,
    gen_query: Option<&mut GeneratedQuery>,
) -> Type {
    let mut previous_step = None;
    let mut cur_ty = match &tr.start {
        StartNode::Node { node_type, ids } => {
            if !ctx.node_set.contains(node_type.as_str()) {
                push_query_err(
                    ctx,
                    q,
                    tr.loc.clone(),
                    format!("unknown node type `{}`", node_type),
                    format!("declare N::{} in the schema first", node_type),
                );
            }
            if let Some(ids) = ids {
                assert!(ids.len() == 1, "multiple ids not supported yet");
                // check id exists in scope
                match ids[0].clone() {
                    IdType::ByIndex { index, value, loc } => {
                        is_valid_identifier(ctx, q, loc.clone(), index.to_string().as_str());
                        let corresponding_field = ctx.node_fields.get(node_type.as_str()).cloned();
                        match corresponding_field {
                            Some(node_fields) => {
                                match node_fields
                                    .iter()
                                    .find(|(name, _)| name.to_string() == *index.to_string())
                                {
                                    Some((_, field)) => {
                                        if !field.is_indexed() {
                                            push_query_err(
                                                ctx,
                                                q,
                                                loc.clone(),
                                                format!(
                                                    "field `{}` has not been indexed for node type `{}`",
                                                    index, node_type
                                                ),
                                                format!(
                                                    "use a field that has been indexed with `INDEX` in the schema for node type `{}`",
                                                    node_type
                                                ),
                                            );
                                        } else {
                                            if let ValueType::Literal { ref value, ref loc } =
                                                *value
                                            {
                                                if !field.field_type.eq(value) {
                                                    push_query_err(
                                                        ctx,
                                                        q,
                                                        loc.clone(),
                                                        format!(
                                                            "value `{}` is of type `{}`, expected `{}`",
                                                            value.to_string(),
                                                            value,
                                                            field.field_type
                                                        ),
                                                        format!(
                                                            "use a value of type `{}`",
                                                            field.field_type
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    None => {
                                        push_query_err(
                                            ctx,
                                            q,
                                            loc.clone(),
                                            format!(
                                                "field `{}` has not been defined and/or indexed for node type `{}`",
                                                index, node_type
                                            ),
                                            format!(
                                                "use a field that has been defined and indexed with `INDEX` in the schema for node type `{}`",
                                                node_type
                                            ),
                                        );
                                    }
                                }
                            }
                            None => unreachable!(),
                        };
                        gen_traversal.source_step = Separator::Period(SourceStep::NFromIndex(
                            NFromIndex {
                                index: GenRef::Literal(match *index {
                                    IdType::Identifier { value, loc: _ } => value,
                                    // would be caught by the parser
                                    _ => unreachable!(),
                                }),
                                key: match *value {
                                    ValueType::Identifier { value, loc } => {
                                        if is_valid_identifier(ctx, q, loc.clone(), value.as_str())
                                        {
                                            if !scope.contains_key(value.as_str()) {
                                                push_query_err(
                                                    ctx,
                                                    q,
                                                    loc,
                                                    format!(
                                                        "variable named `{}` is not in scope",
                                                        value
                                                    ),
                                                    format!(
                                                        "declare {} in the current scope or fix the typo",
                                                        value
                                                    ),
                                                );
                                            }
                                        }
                                        gen_identifier_or_param(
                                            ctx,
                                            q,
                                            value.as_str(),
                                            true,
                                            false,
                                        )
                                    }
                                    ValueType::Literal { value, loc: _ } => {
                                        GeneratedValue::Primitive(GenRef::Std(match value {
                                            Value::String(s) => format!("\"{}\"", s),
                                            Value::I8(i) => i.to_string(),
                                            Value::I16(i) => i.to_string(),
                                            Value::I32(i) => i.to_string(),
                                            Value::I64(i) => i.to_string(),
                                            Value::U8(i) => i.to_string(),
                                            Value::U16(i) => i.to_string(),
                                            Value::U32(i) => i.to_string(),
                                            Value::U64(i) => i.to_string(),
                                            Value::U128(i) => i.to_string(),
                                            Value::F32(i) => i.to_string(),
                                            Value::F64(i) => i.to_string(),
                                            Value::Boolean(b) => b.to_string(),
                                            _ => unreachable!(),
                                        }))
                                    }
                                    _ => unreachable!(),
                                },
                            },
                        ));
                        gen_traversal.should_collect = ShouldCollect::ToVal;
                        gen_traversal.traversal_type = TraversalType::Ref;
                        Type::Node(Some(node_type.to_string()))
                    }
                    IdType::Identifier { value: i, loc } => {
                        if is_valid_identifier(ctx, q, loc.clone(), i.as_str()) {
                            if !scope.contains_key(i.as_str()) {
                                push_query_err(
                                    ctx,
                                    q,
                                    loc.clone(),
                                    format!("variable named `{}` is not in scope", i),
                                    format!("declare {} in the current scope or fix the typo", i),
                                );
                            }
                        }
                        gen_traversal.source_step =
                            Separator::Period(SourceStep::NFromID(NFromID {
                                id: GenRef::Ref(format!("data.{}", i)),
                                label: GenRef::Literal(node_type.clone()),
                            }));
                        gen_traversal.traversal_type = TraversalType::Ref;
                        gen_traversal.should_collect = ShouldCollect::ToVal;
                        Type::Node(Some(node_type.to_string()))
                    }
                    IdType::Literal { value: s, loc: _ } => {
                        gen_traversal.source_step =
                            Separator::Period(SourceStep::NFromID(NFromID {
                                id: GenRef::Ref(s),
                                label: GenRef::Literal(node_type.clone()),
                            }));
                        gen_traversal.traversal_type = TraversalType::Ref;
                        gen_traversal.should_collect = ShouldCollect::ToVal;
                        Type::Node(Some(node_type.to_string()))
                    }
                }
            } else {
                gen_traversal.source_step = Separator::Period(SourceStep::NFromType(NFromType {
                    label: GenRef::Literal(node_type.clone()),
                }));
                gen_traversal.traversal_type = TraversalType::Ref;
                Type::Nodes(Some(node_type.to_string()))
            }
        }
        StartNode::Edge { edge_type, ids } => {
            if !ctx.edge_map.contains_key(edge_type.as_str()) {
                push_query_err(
                    ctx,
                    q,
                    tr.loc.clone(),
                    format!("unknown edge type `{}`", edge_type),
                    format!("declare E::{} in the schema first", edge_type),
                );
            }
            if let Some(ids) = ids {
                assert!(ids.len() == 1, "multiple ids not supported yet");
                gen_traversal.source_step = Separator::Period(SourceStep::EFromID(EFromID {
                    id: match ids[0].clone() {
                        IdType::Identifier { value: i, loc } => {
                            if is_valid_identifier(ctx, q, loc.clone(), i.as_str()) {
                                if !scope.contains_key(i.as_str()) {
                                    push_query_err(
                                        ctx,
                                        q,
                                        loc,
                                        format!("variable named `{}` is not in scope", i),
                                        format!(
                                            "declare {} in the current scope or fix the typo",
                                            i
                                        ),
                                    );
                                }
                            }
                            GenRef::Std(format!("&data.{}", i))
                        }
                        IdType::Literal { value: s, loc: _ } => GenRef::Std(s),
                        _ => unreachable!(),
                    },
                    label: GenRef::Literal(edge_type.clone()),
                }));
                gen_traversal.traversal_type = TraversalType::Ref;
                Type::Edge(Some(edge_type.to_string()))
            } else {
                gen_traversal.source_step = Separator::Period(SourceStep::EFromType(EFromType {
                    label: GenRef::Literal(edge_type.clone()),
                }));
                gen_traversal.traversal_type = TraversalType::Ref;
                Type::Edges(Some(edge_type.to_string()))
            }
        }

        StartNode::Identifier(identifier) => {
            match is_valid_identifier(ctx, q, tr.loc.clone(), identifier.as_str()) {
                true => scope.get(identifier.as_str()).cloned().map_or_else(
                    || {
                        push_query_err(
                            ctx,
                            q,
                            tr.loc.clone(),
                            format!("variable named `{}` is not in scope", identifier),
                            format!(
                                "declare {} in the current scope or fix the typo",
                                identifier
                            ),
                        );
                        Type::Unknown
                    },
                    |var_type| {
                        gen_traversal.traversal_type =
                            TraversalType::FromVar(GenRef::Std(identifier.clone()));
                        gen_traversal.source_step = Separator::Empty(SourceStep::Identifier(
                            GenRef::Std(identifier.clone()),
                        ));
                        var_type.clone()
                    },
                ),
                false => Type::Unknown,
            }
        }
        // anonymous will be the traversal type rather than the start type
        StartNode::Anonymous => {
            let parent = parent_ty.unwrap();
            gen_traversal.traversal_type = TraversalType::FromVar(GenRef::Std("val".to_string())); // TODO: ensure this default is stable
            gen_traversal.source_step = Separator::Empty(SourceStep::Anonymous);
            parent
        }
    };

    // Track excluded fields for property validation
    let mut excluded: HashMap<&str, Loc> = HashMap::new();

    // Stream through the steps
    let number_of_steps = match tr.steps.len() {
        0 => 0,
        n => n - 1,
    };

    for (i, graph_step) in tr.steps.iter().enumerate() {
        let step = &graph_step.step;
        match step {
            StepType::Node(gs) | StepType::Edge(gs) => {
                match apply_graph_step(ctx, &gs, &cur_ty, q, gen_traversal, scope) {
                    Some(new_ty) => {
                        cur_ty = new_ty;
                    }
                    None => { /* error already recorded */ }
                }
                excluded.clear(); // Traversal to a new element resets exclusions
            }

            StepType::Count => {
                cur_ty = Type::Scalar(FieldType::I64);
                excluded.clear();
                gen_traversal
                    .steps
                    .push(Separator::Period(GeneratedStep::Count));
                gen_traversal.should_collect = ShouldCollect::No;
            }

            StepType::Exclude(ex) => {
                // checks if exclude is either the last step or the step before an object remapping or closure
                // i.e. you cant have `N<Type>::!{field1}::Out<Label>`
                if !(i == number_of_steps
                    || (i != number_of_steps - 1
                        && (!matches!(tr.steps[i + 1].step, StepType::Closure(_))
                            || !matches!(tr.steps[i + 1].step, StepType::Object(_)))))
                {
                    push_query_err(
                        ctx,
                        q,
                        ex.loc.clone(),
                        "exclude is only valid as the last step in a traversal,
                            or as the step before an object remapping or closure"
                            .to_string(),
                        "move exclude steps to the end of the traversal,
                            or remove the traversal steps following the exclude"
                            .to_string(),
                    );
                }
                validate_exclude(ctx, &cur_ty, tr, ex, &excluded, q);
                for (_, key) in &ex.fields {
                    excluded.insert(key.as_str(), ex.loc.clone());
                }
                gen_traversal
                    .steps
                    .push(Separator::Period(GeneratedStep::Remapping(Remapping {
                        variable_name: "item".to_string(), // TODO: Change to start var
                        is_inner: false,
                        should_spread: false,
                        remappings: vec![RemappingType::ExcludeField(ExcludeField {
                            variable_name: "item".to_string(), // TODO: Change to start var
                            fields_to_exclude: ex
                                .fields
                                .iter()
                                .map(|(_, field)| GenRef::Literal(field.clone()))
                                .collect(),
                        })],
                    })));
            }

            StepType::Object(obj) => {
                // TODO: Fix issue with step count being incorrect (i think its counting each field as a step)
                // if i != number_of_steps {
                //     println!("{} {}", i, number_of_steps);
                //     push_query_err(ctx,
                //         q,
                //         obj.loc.clone(),
                //         "object is only valid as the last step in a traversal".to_string(),
                //         "move the object to the end of the traversal",
                //     );
                // }
                validate_object(ctx,
                    &cur_ty,
                    tr,
                    obj,
                    &excluded,
                    q,
                    gen_traversal,
                    None,
                    scope,
                    None,
                );
            }

            StepType::Where(expr) => {
                let (_, stmt) = infer_expr_type(ctx, expr, scope, q, Some(cur_ty.clone()), None);
                // Where/boolean ops don't change the element type,
                // so `cur_ty` stays the same.
                assert!(stmt.is_some());
                let stmt = stmt.unwrap();
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
            }
            StepType::BooleanOperation(b_op) => {
                let step = previous_step.unwrap();
                let property_type = match &b_op.op {
                    BooleanOpType::LessThanOrEqual(expr)
                    | BooleanOpType::LessThan(expr)
                    | BooleanOpType::GreaterThanOrEqual(expr)
                    | BooleanOpType::GreaterThan(expr)
                    | BooleanOpType::Equal(expr)
                    | BooleanOpType::NotEqual(expr) => {
                        match infer_expr_type(ctx, expr, scope, q, Some(cur_ty.clone()), None) {
                            (Type::Scalar(ft), _) => ft.clone(),
                            (field_type, _) => {
                                push_query_err(
                                    ctx,
                                    q,
                                    b_op.loc.clone(),
                                    format!(
                                        "boolean operation `{}` cannot be applied to `{}`",
                                        b_op.loc.span,
                                        field_type.kind_str()
                                    ),
                                    "make sure the expression evaluates to a number or a string"
                                        .to_string(),
                                );
                                return field_type;
                            }
                        }
                    }
                    _ => return cur_ty.clone(),
                };

                // get type of field name
                let field_name = match step {
                    StepType::Object(obj) => {
                        let fields = obj.fields;
                        assert!(fields.len() == 1);
                        Some(fields[0].value.value.clone())
                    }
                    _ => None,
                };
                if let Some(FieldValueType::Identifier(field_name)) = &field_name {
                    is_valid_identifier(ctx, q, b_op.loc.clone(), field_name.as_str());
                    match &cur_ty {
                        Type::Nodes(Some(node_ty)) | Type::Node(Some(node_ty)) => {
                            let field_set = ctx.node_fields.get(node_ty.as_str()).cloned();
                            if let Some(field_set) = field_set {
                                match field_set.get(field_name.as_str()) {
                                    Some(field) => {
                                        if field.field_type != property_type {
                                            push_query_err(ctx,
                                                    q,
                                                    b_op.loc.clone(),
                                                    format!("property `{field_name}` is of type `{}` (from node type `{node_ty}::{{{field_name}}}`), which does not match type of compared value `{}`", field.field_type, property_type),
                                                    "make sure comparison value is of the same type as the property".to_string(),
                                                );
                                        }
                                    }
                                    None => {
                                        push_query_err(
                                            ctx,
                                            q,
                                            b_op.loc.clone(),
                                            format!(
                                                "`{}` is not a field of {} `{}`",
                                                field_name, "node", node_ty
                                            ),
                                            "check the schema field names",
                                        );
                                    }
                                }
                            }
                        }
                        Type::Edges(Some(edge_ty)) | Type::Edge(Some(edge_ty)) => {
                            let field_set = ctx.edge_fields.get(edge_ty.as_str()).cloned();
                            if let Some(field_set) = field_set {
                                match field_set.get(field_name.as_str()) {
                                    Some(field) => {
                                        if field.field_type != property_type {
                                            push_query_err(ctx,
                                                    q,
                                                    b_op.loc.clone(),
                                                    format!("property `{field_name}` is of type `{}` (from edge type `{edge_ty}::{{{field_name}}}`), which does not match type of compared value `{}`", field.field_type, property_type),
                                                    "make sure comparison value is of the same type as the property".to_string(),
                                                );
                                        }
                                    }
                                    None => {
                                        push_query_err(
                                            ctx,
                                            q,
                                            b_op.loc.clone(),
                                            format!(
                                                "`{}` is not a field of {} `{}`",
                                                field_name, "edge", edge_ty
                                            ),
                                            "check the schema field names",
                                        );
                                    }
                                }
                            }
                        }
                        Type::Vectors(Some(sv)) | Type::Vector(Some(sv)) => {
                            let field_set = ctx.vector_fields.get(sv.as_str()).cloned();
                            if let Some(field_set) = field_set {
                                match field_set.get(field_name.as_str()) {
                                    Some(field) => {
                                        if field.field_type != property_type {
                                            push_query_err(ctx,
                                                    q,
                                                    b_op.loc.clone(),
                                                    format!("property `{field_name}` is of type `{}` (from vector type `{sv}::{{{field_name}}}`), which does not match type of compared value `{}`", field.field_type, property_type),
                                                    "make sure comparison value is of the same type as the property".to_string(),
                                                );
                                        }
                                    }
                                    None => {
                                        push_query_err(
                                            ctx,
                                            q,
                                            b_op.loc.clone(),
                                            format!(
                                                "`{}` is not a field of {} `{}`",
                                                field_name, "vector", sv
                                            ),
                                            "check the schema field names",
                                        );
                                    }
                                }
                            }
                        }
                        _ => {
                            push_query_err(
                                ctx,
                                q,
                                b_op.loc.clone(),
                                "boolean operation can only be applied to scalar values"
                                    .to_string(),
                                "make sure the expression evaluates to a number or a string"
                                    .to_string(),
                            );
                        }
                    }
                }

                // ctx.infer_expr_type(expr, scope, q);
                // Where/boolean ops don't change the element type,
                // so `cur_ty` stays the same.
                let op = match &b_op.op {
                    BooleanOpType::LessThanOrEqual(expr) => {
                        // assert!()
                        let v = match &expr.expr {
                            ExpressionType::IntegerLiteral(i) => {
                                GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                            }
                            ExpressionType::FloatLiteral(f) => {
                                GeneratedValue::Primitive(GenRef::Std(f.to_string()))
                            }
                            ExpressionType::Identifier(i) => {
                                is_valid_identifier(ctx, q, expr.loc.clone(), i.as_str());
                                gen_identifier_or_param(ctx, q, i.as_str(), false, true)
                            }
                            _ => unreachable!("Cannot reach here"),
                        };
                        BoolOp::Lte(Lte { value: v })
                    }
                    BooleanOpType::LessThan(expr) => {
                        let v = match &expr.expr {
                            ExpressionType::IntegerLiteral(i) => {
                                GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                            }
                            ExpressionType::FloatLiteral(f) => {
                                GeneratedValue::Primitive(GenRef::Std(f.to_string()))
                            }
                            ExpressionType::Identifier(i) => {
                                is_valid_identifier(ctx, q, expr.loc.clone(), i.as_str());
                                gen_identifier_or_param(ctx, q, i.as_str(), false, true)
                            }
                            _ => unreachable!("Cannot reach here"),
                        };
                        BoolOp::Lt(Lt { value: v })
                    }
                    BooleanOpType::GreaterThanOrEqual(expr) => {
                        let v = match &expr.expr {
                            ExpressionType::IntegerLiteral(i) => {
                                GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                            }
                            ExpressionType::FloatLiteral(f) => {
                                GeneratedValue::Primitive(GenRef::Std(f.to_string()))
                            }
                            ExpressionType::Identifier(i) => {
                                is_valid_identifier(ctx, q, expr.loc.clone(), i.as_str());
                                gen_identifier_or_param(ctx, q, i.as_str(), false, true)
                            }
                            _ => unreachable!("Cannot reach here"),
                        };
                        BoolOp::Gte(Gte { value: v })
                    }
                    BooleanOpType::GreaterThan(expr) => {
                        let v = match &expr.expr {
                            ExpressionType::IntegerLiteral(i) => {
                                GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                            }
                            ExpressionType::FloatLiteral(f) => {
                                GeneratedValue::Primitive(GenRef::Std(f.to_string()))
                            }
                            ExpressionType::Identifier(i) => {
                                is_valid_identifier(ctx, q, expr.loc.clone(), i.as_str());
                                gen_identifier_or_param(ctx, q, i.as_str(), false, true)
                            }
                            _ => unreachable!("Cannot reach here"),
                        };
                        BoolOp::Gt(Gt { value: v })
                    }
                    BooleanOpType::Equal(expr) => {
                        let v = match &expr.expr {
                            ExpressionType::BooleanLiteral(b) => {
                                GeneratedValue::Primitive(GenRef::Std(b.to_string()))
                            }
                            ExpressionType::IntegerLiteral(i) => {
                                GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                            }
                            ExpressionType::FloatLiteral(f) => {
                                GeneratedValue::Primitive(GenRef::Std(f.to_string()))
                            }
                            ExpressionType::StringLiteral(s) => {
                                GeneratedValue::Primitive(GenRef::Literal(s.to_string()))
                            }
                            ExpressionType::Identifier(i) => {
                                is_valid_identifier(ctx, q, expr.loc.clone(), i.as_str());
                                gen_identifier_or_param(ctx, q, i.as_str(), false, true)
                            }
                            _ => unreachable!("Cannot reach here"),
                        };
                        BoolOp::Eq(Eq { value: v })
                    }
                    BooleanOpType::NotEqual(expr) => {
                        let v = match &expr.expr {
                            ExpressionType::BooleanLiteral(b) => {
                                GeneratedValue::Primitive(GenRef::Std(b.to_string()))
                            }
                            ExpressionType::IntegerLiteral(i) => {
                                GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                            }
                            ExpressionType::FloatLiteral(f) => {
                                GeneratedValue::Primitive(GenRef::Std(f.to_string()))
                            }
                            ExpressionType::StringLiteral(s) => {
                                GeneratedValue::Primitive(GenRef::Literal(s.to_string()))
                            }
                            ExpressionType::Identifier(i) => {
                                is_valid_identifier(ctx, q, expr.loc.clone(), i.as_str());
                                gen_identifier_or_param(ctx, q, i.as_str(), false, true)
                            }
                            _ => unreachable!("Cannot reach here"),
                        };
                        BoolOp::Neq(Neq { value: v })
                    }
                    _ => unreachable!("shouldve been caught eariler"),
                };
                gen_traversal
                    .steps
                    .push(Separator::Period(GeneratedStep::BoolOp(op)));
                gen_traversal.should_collect = ShouldCollect::No;
            }

            StepType::Update(update) => {
                // if type == node, edge, vector then update is valid
                // otherwise it is invalid

                // Update returns the same type (nodes/edges) it started with.
                match tr.steps.iter().nth_back(1) {
                    Some(step) => match &step.step {
                        StepType::Node(gs) => {
                            let node_ty = gs.get_item_type().unwrap();
                            let field_set = ctx.node_fields.get(node_ty.as_str()).cloned();
                            if let Some(field_set) = field_set {
                                for FieldAddition { key, value, loc } in &update.fields {
                                    if !field_set.contains_key(key.as_str()) {
                                        push_query_err(
                                            ctx,
                                            q,
                                            loc.clone(),
                                            format!(
                                                "`{}` is not a field of node `{}`",
                                                key, node_ty
                                            ),
                                            "check the schema field names",
                                        );
                                    }
                                }
                            }
                        }

                        StepType::Edge(gs) => {
                            let edge_ty = gs.get_item_type().unwrap();
                            let field_set = ctx.edge_fields.get(edge_ty.as_str()).cloned();
                            if let Some(field_set) = field_set {
                                for FieldAddition { key, value, loc } in &update.fields {
                                    if !field_set.contains_key(key.as_str()) {
                                        push_query_err(
                                            ctx,
                                            q,
                                            loc.clone(),
                                            format!(
                                                "`{}` is not a field of edge `{}`",
                                                key, edge_ty
                                            ),
                                            "check the schema field names",
                                        );
                                    }
                                }
                            }
                        }
                        _ => {
                            push_query_err(
                                ctx,
                                q,
                                update.loc.clone(),
                                "update is only valid on nodes or edges".to_string(),
                                "update is only valid on nodes or edges".to_string(),
                            );
                            return cur_ty.clone();
                        }
                    },
                    None => match &tr.start {
                        StartNode::Node { node_type, .. } => {
                            let node_ty = node_type.as_str();
                            let field_set = ctx.node_fields.get(node_ty).cloned();
                            if let Some(field_set) = field_set {
                                for FieldAddition { key, value, loc } in &update.fields {
                                    if !field_set.contains_key(key.as_str()) {
                                        push_query_err(
                                            ctx,
                                            q,
                                            loc.clone(),
                                            format!(
                                                "`{}` is not a field of node `{}`",
                                                key, node_ty
                                            ),
                                            "check the schema field names",
                                        );
                                    }
                                }
                            }
                        }
                        StartNode::Edge { edge_type, .. } => {
                            let edge_ty = edge_type.as_str();
                            let field_set = ctx.edge_fields.get(edge_ty).cloned();
                            if let Some(field_set) = field_set {
                                for FieldAddition { key, value, loc } in &update.fields {
                                    if !field_set.contains_key(key.as_str()) {
                                        push_query_err(
                                            ctx,
                                            q,
                                            loc.clone(),
                                            format!(
                                                "`{}` is not a field of edge `{}`",
                                                key, edge_ty
                                            ),
                                            "check the schema field names",
                                        );
                                    }
                                }
                            }
                        }
                        _ => {
                            push_query_err(
                                ctx,
                                q,
                                update.loc.clone(),
                                "update is only valid on nodes or edges".to_string(),
                                "update is only valid on nodes or edges".to_string(),
                            );
                            return cur_ty.clone();
                        }
                    },
                };
                gen_traversal.traversal_type = TraversalType::Update(Some(
                    update
                        .fields
                        .iter()
                        .map(|field| {
                            (
                                field.key.clone(),
                                match &field.value.value {
                                    FieldValueType::Identifier(i) => {
                                        is_valid_identifier(
                                            ctx,
                                            q,
                                            field.value.loc.clone(),
                                            i.as_str(),
                                        );
                                        gen_identifier_or_param(ctx, q, i.as_str(), true, true)
                                    }
                                    FieldValueType::Literal(l) => match l {
                                        Value::String(s) => {
                                            GeneratedValue::Literal(GenRef::Literal(s.clone()))
                                        }
                                        other => GeneratedValue::Primitive(GenRef::Std(
                                            other.to_string(),
                                        )),
                                    },
                                    FieldValueType::Expression(e) => match &e.expr {
                                        ExpressionType::Identifier(i) => {
                                            is_valid_identifier(ctx, q, e.loc.clone(), i.as_str());
                                            gen_identifier_or_param(ctx, q, i.as_str(), true, true)
                                        }
                                        ExpressionType::StringLiteral(i) => {
                                            GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                                        }

                                        ExpressionType::IntegerLiteral(i) => {
                                            GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                                        }
                                        ExpressionType::FloatLiteral(i) => {
                                            GeneratedValue::Primitive(GenRef::Std(i.to_string()))
                                        }
                                        v => {
                                            println!("ID {:?}", v);
                                            panic!("expr be primitive or value")
                                        }
                                    },
                                    v => {
                                        println!("{:?}", v);
                                        panic!("Should be primitive or value")
                                    }
                                },
                            )
                        })
                        .collect(),
                ));
                gen_traversal.should_collect = ShouldCollect::No;
                excluded.clear();
            }

            StepType::AddEdge(add) => {
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
                }
                cur_ty = Type::Edges(add.edge_type.clone());
                excluded.clear();
            }

            StepType::Range((start, end)) => {
                let (start, end) = match (&start.expr, &end.expr) {
                    (ExpressionType::Identifier(i), ExpressionType::Identifier(j)) => {
                        is_valid_identifier(ctx, q, start.loc.clone(), i.as_str());
                        is_valid_identifier(ctx, q, end.loc.clone(), j.as_str());

                        let ty = type_in_scope(ctx, q, start.loc.clone(), scope, i.as_str());
                        match ty {
                            Some(ty) => {
                                if !ty.is_integer() {
                                    push_query_err(
                                        ctx,
                                        q,
                                        start.loc.clone(),
                                        format!(
                                            "index of range must be an integer, got {:?}",
                                            ty.get_type_name()
                                        ),
                                        "start and end of range must be integers".to_string(),
                                    );
                                    return cur_ty.clone(); // Not sure if this should be here
                                }
                            }
                            None => {}
                        };
                        let ty = type_in_scope(ctx, q, end.loc.clone(), scope, j.as_str());
                        match ty {
                            Some(ty) => {
                                if !ty.is_integer() {
                                    push_query_err(
                                        ctx,
                                        q,
                                        end.loc.clone(),
                                        format!(
                                            "index of range must be an integer, got {:?}",
                                            ty.get_type_name()
                                        ),
                                        "start and end of range must be integers".to_string(),
                                    );
                                    return cur_ty.clone(); // Not sure if this should be here
                                }
                            }
                            None => {}
                        }
                        (
                            gen_identifier_or_param(ctx, q, i.as_str(), false, true),
                            gen_identifier_or_param(ctx, q, j.as_str(), false, true),
                        )
                    }
                    (ExpressionType::IntegerLiteral(i), ExpressionType::IntegerLiteral(j)) => (
                        GeneratedValue::Primitive(GenRef::Std(i.to_string())),
                        GeneratedValue::Primitive(GenRef::Std(j.to_string())),
                    ),
                    (ExpressionType::Identifier(i), ExpressionType::IntegerLiteral(j)) => {
                        is_valid_identifier(ctx, q, start.loc.clone(), i.as_str());

                        let ty = type_in_scope(ctx, q, start.loc.clone(), scope, i.as_str());
                        match ty {
                            Some(ty) => {
                                if !ty.is_integer() {
                                    push_query_err(
                                        ctx,
                                        q,
                                        start.loc.clone(),
                                        format!(
                                            "index of range must be an integer, got {:?}",
                                            ty.get_type_name()
                                        ),
                                        "start and end of range must be integers".to_string(),
                                    );
                                    return cur_ty.clone(); // Not sure if this should be here
                                }
                            }
                            None => {}
                        }

                        (
                            gen_identifier_or_param(ctx, q, i.as_str(), false, true),
                            GeneratedValue::Primitive(GenRef::Std(j.to_string())),
                        )
                    }
                    (ExpressionType::IntegerLiteral(i), ExpressionType::Identifier(j)) => {
                        is_valid_identifier(ctx, q, end.loc.clone(), j.as_str());
                        let ty = type_in_scope(ctx, q, end.loc.clone(), scope, j.as_str());
                        match ty {
                            Some(ty) => {
                                if !ty.is_integer() {
                                    push_query_err(
                                        ctx,
                                        q,
                                        end.loc.clone(),
                                        format!(
                                            "index of range must be an integer, got {:?}",
                                            ty.get_type_name()
                                        ),
                                        "start and end of range must be integers".to_string(),
                                    );
                                    return cur_ty.clone();
                                }
                            }
                            None => {}
                        }
                        (
                            GeneratedValue::Primitive(GenRef::Std(i.to_string())),
                            gen_identifier_or_param(ctx, q, j.as_str(), false, true),
                        )
                    }
                    (ExpressionType::Identifier(_) | ExpressionType::IntegerLiteral(_), other) => {
                        push_query_err(
                            ctx,
                            q,
                            start.loc.clone(),
                            format!("{:?} does not resolve to an integer value", other),
                            "start and end of range must be integers".to_string(),
                        );
                        return cur_ty.clone();
                    }
                    _ => {
                        push_query_err(
                            ctx,
                            q,
                            start.loc.clone(),
                            format!(
                                "start and end of range must be integers, got {:?} and {:?}",
                                start, end
                            ),
                            "start and end of range must be integers".to_string(),
                        );
                        return cur_ty.clone();
                    }
                };
                gen_traversal
                    .steps
                    .push(Separator::Period(GeneratedStep::Range(Range {
                        start: start,
                        end: end,
                    })));
            }
            StepType::OrderByAsc(expr) => {
                // verify property access
                let (_, stmt) = infer_expr_type(ctx, expr, scope, q, Some(cur_ty.clone()), None);

                assert!(stmt.is_some());
                match stmt.unwrap() {
                    GeneratedStatement::Traversal(traversal) => {
                        let property = match &traversal.steps.last() {
                            Some(step) => match &step.inner() {
                                GeneratedStep::PropertyFetch(property) => property.clone(),
                                _ => unreachable!("Cannot reach here"),
                            },
                            None => unreachable!("Cannot reach here"),
                        };
                        gen_traversal
                            .steps
                            .push(Separator::Period(GeneratedStep::OrderBy(OrderBy {
                                property,
                                order: Order::Asc,
                            })));
                        gen_traversal.should_collect = ShouldCollect::Try;
                    }
                    _ => unreachable!("Cannot reach here"),
                }
            }
            StepType::OrderByDesc(expr) => {
                // verify property access
                let (_, stmt) = infer_expr_type(ctx, expr, scope, q, Some(cur_ty.clone()), None);

                assert!(stmt.is_some());
                match stmt.unwrap() {
                    GeneratedStatement::Traversal(traversal) => {
                        let property = match &traversal.steps.last() {
                            Some(step) => match &step.inner() {
                                GeneratedStep::PropertyFetch(property) => property.clone(),
                                _ => unreachable!("Cannot reach here"),
                            },
                            None => unreachable!("Cannot reach here"),
                        };
                        gen_traversal
                            .steps
                            .push(Separator::Period(GeneratedStep::OrderBy(OrderBy {
                                property,
                                order: Order::Desc,
                            })));
                        gen_traversal.should_collect = ShouldCollect::Try;
                    }
                    _ => unreachable!("Cannot reach here"),
                }
            }
            StepType::Closure(cl) => {
                if i != number_of_steps {
                    push_query_err(
                        ctx,
                        q,
                        cl.loc.clone(),
                        "closure is only valid as the last step in a traversal".to_string(),
                        "move the closure to the end of the traversal",
                    );
                }
                // Add identifier to a temporary scope so inner uses pass
                scope.insert(cl.identifier.as_str(), cur_ty.clone()); // If true then already exists so return error
                let obj = &cl.object;
                validate_object(ctx,
                    &cur_ty,
                    tr,
                    obj,
                    &excluded,
                    q,
                    gen_traversal,
                    None,
                    scope,
                    Some(&cl.identifier),
                );

                // gen_traversal
                //     .steps
                //     .push(Separator::Period(GeneratedStep::Remapping(Remapping {
                //         is_inner: false,
                //         should_spread: false,
                //         variable_name: cl.identifier.clone(),
                //         remappings: (),
                //     })));
                scope.remove(cl.identifier.as_str());
                // gen_traversal.traversal_type =
                //     TraversalType::Nested(GenRef::Std(var));
            }
        }
        previous_step = Some(step.clone());
    }
    match gen_traversal.traversal_type {
        TraversalType::Mut | TraversalType::Update(_) => {
            if let Some(gen_query) = gen_query {
                gen_query.is_mut = true;
            }
        }
        _ => {}
    }
    cur_ty
}
