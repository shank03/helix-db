//! Semantic analyzer for Helix‑QL.

use crate::helixc::{
    analyzer::{
        analyzer::Ctx, errors::push_query_err, methods::infer_expr_type::infer_expr_type,
        types::Type, utils::is_valid_identifier,
    },
    generator::{
        generator_types::{
            Assignment as GeneratedAssignment, Drop as GeneratedDrop, ForEach as GeneratedForEach,
            ForLoopInVariable, ForVariable, Query as GeneratedQuery,
            Statement as GeneratedStatement,
        },
        utils::GenRef,
    },
    parser::helix_parser::*,
};
use std::collections::HashMap;

pub(crate) fn validate_statements<'a>(
    ctx: &mut Ctx<'a>,
    scope: &mut HashMap<&'a str, Type>,
    q: &'a Query,
    query: &mut GeneratedQuery,
    statement: &'a Statement,
) -> Option<GeneratedStatement> {
    use StatementType::*;
    match &statement.statement {
        Assignment(assign) => {
            if scope.contains_key(assign.variable.as_str()) {
                push_query_err(
                    ctx,
                    q,
                    assign.loc.clone(),
                    format!("variable `{}` is already declared", assign.variable),
                    "rename the new variable or remove the previous definition",
                );
            }

            let (rhs_ty, stmt) = infer_expr_type(ctx, &assign.value, scope, q, None, query);
            scope.insert(assign.variable.as_str(), rhs_ty);
            assert!(stmt.is_some(), "Assignment statement should be generated");

            let assignment = GeneratedStatement::Assignment(GeneratedAssignment {
                variable: GenRef::Std(assign.variable.clone()),
                value: Box::new(stmt.unwrap()),
            });
            Some(assignment)
        }

        Drop(expr) => {
            let (_, stmt) = infer_expr_type(ctx, expr, scope, q, None, query);
            assert!(stmt.is_some());
            query.is_mut = true;
            if let Some(GeneratedStatement::Traversal(tr)) = stmt {
                Some(GeneratedStatement::Drop(GeneratedDrop { expression: tr }))
            } else {
                panic!("Drop should only be applied to traversals");
            }
        }

        Expression(expr) => {
            let (_, stmt) = infer_expr_type(ctx, expr, scope, q, None, query);
            stmt
        }

        ForLoop(fl) => {
            // Ensure the collection exists
            if !scope.contains_key(fl.in_variable.1.as_str()) {
                push_query_err(
                    ctx,
                    q,
                    fl.loc.clone(),
                    format!("`{}` is not defined in the current scope", fl.in_variable.1),
                    "add a statement assigning it before the loop",
                );
            }
            // Add loop vars to new child scope and walk the body
            let mut body_scope = HashMap::new();
            let mut for_loop_in_variable: ForLoopInVariable = ForLoopInVariable::Empty;
            // find param from fl.in_variable
            let param = q.parameters.iter().find(|p| p.name.1 == fl.in_variable.1);

            let mut for_variable: ForVariable = ForVariable::Empty;

            match &fl.variable {
                ForLoopVars::Identifier { name, loc: _ } => {
                    is_valid_identifier(ctx, q, fl.loc.clone(), name.as_str());
                    body_scope.insert(name.as_str(), Type::Unknown);
                    scope.insert(name.as_str(), Type::Unknown);
                    for_variable = ForVariable::Identifier(GenRef::Std(name.clone()));
                }
                ForLoopVars::ObjectAccess {
                    name: _,
                    field: _,
                    loc: _,
                } => {
                    // body_scope.insert(name.as_str(), Type::Unknown);
                    // for_variable =
                    //     ForVariable::ObjectDestructure(vec![GenRef::Std(name.clone())]);
                    unreachable!()
                }
                ForLoopVars::ObjectDestructuring { fields, loc: _ } => {
                    // TODO: check if fields are valid
                    match &param {
                        Some(p) => {
                            for_loop_in_variable =
                                ForLoopInVariable::Parameter(GenRef::Std(p.name.1.clone()));
                            match &p.param_type.1 {
                                FieldType::Array(inner) => match inner.as_ref() {
                                    FieldType::Object(param_fields) => {
                                        for (field_loc, field_name) in fields {
                                            if !param_fields.contains_key(field_name.as_str()) {
                                                push_query_err(
                                                    ctx,
                                                    q,
                                                    field_loc.clone(),
                                                    format!(
                                                        "`{}` is not a field of the inner type of `{}`",
                                                        field_name, fl.in_variable.1
                                                    ),
                                                    format!(
                                                        "check the object fields of the parameter `{}`",
                                                        fl.in_variable.1
                                                    ),
                                                );
                                            }
                                            body_scope.insert(field_name.as_str(), Type::Unknown);
                                            scope.insert(field_name.as_str(), Type::Unknown);
                                        }
                                        for_variable = ForVariable::ObjectDestructure(
                                            fields
                                                .iter()
                                                .map(|(_, f)| GenRef::Std(f.clone()))
                                                .collect(),
                                        );
                                    }
                                    _ => {
                                        push_query_err(
                                            ctx,
                                            q,
                                            fl.in_variable.0.clone(),
                                            format!(
                                                "the inner type of `{}` is not an object",
                                                fl.in_variable.1
                                            ),
                                            "object destructuring only works with arrays of objects",
                                        );
                                    }
                                },

                                _ => {
                                    push_query_err(
                                        ctx,
                                        q,
                                        fl.in_variable.0.clone(),
                                        format!("`{}` is not an array", fl.in_variable.1),
                                        "object destructuring only works with arrays of objects",
                                    );
                                }
                            }
                        }
                        None => match scope.contains_key(fl.in_variable.1.as_str()) {
                            true => {
                                // TODO: Check fields
                                for_variable = ForVariable::ObjectDestructure(
                                    fields
                                        .iter()
                                        .map(|(_, f)| {
                                            let name = f.as_str();
                                            // adds non-param fields to scope
                                            body_scope.insert(name, Type::Unknown);
                                            scope.insert(name, Type::Unknown);

                                            GenRef::Std(name.to_string())
                                        })
                                        .collect(),
                                );
                            }
                            false => {
                                push_query_err(
                                    ctx,
                                    q,
                                    fl.in_variable.0.clone(),
                                    format!(
                                        "`{}` is not defined in the current scope",
                                        fl.in_variable.1
                                    ),
                                    "add a statement assigning it before the loop",
                                );
                            }
                        },
                    }
                }
            }
            let mut statements = Vec::new();
            for body_stmt in &fl.statements {
                // Recursive walk (but without infinite nesting for now)

                let stmt = validate_statements(ctx, scope, q, query, body_stmt);
                if stmt.is_some() {
                    statements.push(stmt.unwrap());
                }
            }
            // body_scope.iter().for_each(|(k, _)| {
            //     scope.remove(k);
            // });

            let stmt = GeneratedStatement::ForEach(GeneratedForEach {
                for_variables: for_variable,
                in_variable: for_loop_in_variable,
                statements: statements,
            });
            Some(stmt)
        }
    }
}
