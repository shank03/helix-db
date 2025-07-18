//! Semantic analyzer for Helix‑QL.

use crate::helixc::{
    analyzer::{
        analyzer::Ctx,
        errors::{push_query_err, push_query_warn},
        methods::{infer_expr_type::infer_expr_type, statement_validation::walk_statements},
        types::Type,
        utils::{gen_identifier_or_param, is_valid_identifier},
    },
    generator::{
        generator_types::{
            Parameter as GeneratedParameter, Query as GeneratedQuery, ReturnValue, ReturnValueExpr,
            Statement as GeneratedStatement,
        },
        source_steps::SourceStep,
        traversal_steps::ShouldCollect,
        utils::{GenRef, GeneratedValue},
    },
    parser::{helix_parser::*, location::Loc},
};
use std::collections::HashMap;

pub(crate) fn check_query<'a>(ctx: &mut Ctx<'a>, q: &'a Query) {
    let mut query = GeneratedQuery::default();
    query.name = q.name.clone();
    // -------------------------------------------------
    // Parameter validation
    // -------------------------------------------------
    for param in &q.parameters {
        if let FieldType::Identifier(ref id) = param.param_type.1 {
            if is_valid_identifier(ctx, q, param.param_type.0.clone(), id.as_str()) {
                // TODO: add support for edges
                if !ctx.node_set.contains(id.as_str()) {
                    push_query_err(
                        ctx,
                        q,
                        param.param_type.0.clone(),
                        format!("unknown type `{}` for parameter `{}`", id, param.name.1),
                        "declare or use a matching schema object or use a primitive type",
                    );
                }
            }
        }
        // constructs parameters and sub‑parameters for generator
        GeneratedParameter::unwrap_param(
            param.clone(),
            &mut query.parameters,
            &mut query.sub_parameters,
        );
    }

    // -------------------------------------------------
    // Statement‑by‑statement walk
    // -------------------------------------------------
    let mut scope: HashMap<&str, Type> = HashMap::new();
    for param in &q.parameters {
        scope.insert(
            param.name.1.as_str(),
            Type::from(param.param_type.1.clone()),
        );
    }
    for stmt in &q.statements {
        let statement = walk_statements(ctx, &mut scope, q, &mut query, stmt);
        if statement.is_some() {
            query.statements.push(statement.unwrap());
        } else {
            // given all erroneous statements are caught by the analyzer, this should never happen
            unreachable!()
        }
    }

    // -------------------------------------------------
    // Validate RETURN expressions
    // -------------------------------------------------
    if q.return_values.is_empty() {
        let end = q.loc.end.clone();
        push_query_warn(
            ctx,
            q,
            Loc::new(q.loc.filepath.clone(), end.clone(), end, q.loc.span.clone()),
            "query has no RETURN clause".to_string(),
            "add `RETURN <expr>` at the end",
            None,
        );
    }
    for ret in &q.return_values {
        let (_, stmt) = infer_expr_type(ctx, ret, &mut scope, q, None, &mut query);

        assert!(stmt.is_some(), "RETURN value should be a valid expression");
        match stmt.unwrap() {
            GeneratedStatement::Traversal(traversal) => {
                match &traversal.source_step.inner() {
                    SourceStep::Identifier(v) => {
                        is_valid_identifier(ctx, q, ret.loc.clone(), v.inner().as_str());

                        // if is single object, need to handle it as a single object
                        // if is array, need to handle it as an array
                        match traversal.should_collect {
                            ShouldCollect::ToVec => {
                                query.return_values.push(ReturnValue::new_named(
                                    GeneratedValue::Literal(GenRef::Literal(v.inner().clone())),
                                    ReturnValueExpr::Traversal(traversal.clone()),
                                ));
                            }
                            ShouldCollect::ToVal => {
                                query.return_values.push(ReturnValue::new_single_named(
                                    GeneratedValue::Literal(GenRef::Literal(v.inner().clone())),
                                    ReturnValueExpr::Traversal(traversal.clone()),
                                ));
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    _ => {
                        query.return_values.push(ReturnValue::new_unnamed(
                            ReturnValueExpr::Traversal(traversal.clone()),
                        ));
                    }
                }
            }
            GeneratedStatement::Identifier(id) => {
                is_valid_identifier(ctx, q, ret.loc.clone(), id.inner().as_str());
                let identifier_end_type = match scope.get(id.inner().as_str()) {
                    Some(t) => t.clone(),
                    None => {
                        push_query_err(
                            ctx,
                            q,
                            ret.loc.clone(),
                            format!("variable named `{}` is not in scope", id),
                            "declare it earlier or fix the typo",
                        );
                        Type::Unknown
                    }
                };
                let value = gen_identifier_or_param(q, id.inner().as_str(), false, true);
                match identifier_end_type {
                    Type::Scalar(_) => {
                        query.return_values.push(ReturnValue::new_named_literal(
                            GeneratedValue::Literal(GenRef::Literal(id.inner().clone())),
                            value,
                        ));
                    }
                    Type::Node(_) | Type::Vector(_) | Type::Edge(_) => {
                        query.return_values.push(ReturnValue::new_single_named(
                            GeneratedValue::Literal(GenRef::Literal(id.inner().clone())),
                            ReturnValueExpr::Identifier(value),
                        ));
                    }
                    _ => {
                        query.return_values.push(ReturnValue::new_named(
                            GeneratedValue::Literal(GenRef::Literal(id.inner().clone())),
                            ReturnValueExpr::Identifier(value),
                        ));
                    }
                }
            }
            GeneratedStatement::Literal(l) => {
                query.return_values.push(ReturnValue::new_literal(
                    GeneratedValue::Literal(l.clone()),
                    GeneratedValue::Literal(l.clone()),
                ));
            }
            GeneratedStatement::Empty => query.return_values = vec![],

            // given all erroneous statements are caught by the analyzer, this should never happen
            // all malformed statements (not gramatically correct) should be caught by the parser
            _ => unreachable!(),
        }
    }
    if q.is_mcp {
        if query.return_values.len() != 1 {
            push_query_err(ctx,
                q,
                q.loc.clone(),
                "MCP queries can only return a single value as LLM needs to be able to traverse from the result".to_string(),
                "add a single return value that is a node, edge, or vector",
            );
        } else {
            // match query.return_values.first().unwrap().return_type {

            // }
        }
        let return_name = query.return_values.first().unwrap().get_name();
        query.mcp_handler = Some(return_name);
    }
    ctx.output.queries.push(query);
}
