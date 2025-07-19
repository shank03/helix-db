//! Semantic analyzer for Helix‑QL.
use crate::helixc::{
    analyzer::{analyzer::Ctx, errors::push_query_err, types::Type},
    generator::utils::{GenRef, GeneratedValue},
    parser::{helix_parser::*, location::Loc},
};
use std::collections::HashMap;

pub(super) fn is_valid_identifier(
    ctx: &mut Ctx,
    original_query: &Query,
    loc: Loc,
    name: &str,
) -> bool {
    match name {
        "true" | "false" | "NONE" | "String" | "Boolean" | "F32" | "F64" | "I8" | "I16" | "I32"
        | "I64" | "U8" | "U16" | "U32" | "U64" | "U128" | "Uuid" | "Date" => {
            push_query_err(
                ctx,
                original_query,
                loc.clone(),
                format!("`{}` is not a valid identifier", name),
                "use a valid identifier",
            );
            false
        }
        _ => true,
    }
}

pub(super) fn is_param(q: &Query, name: &str) -> bool {
    q.parameters.iter().find(|p| p.name.1 == *name).is_some()
}

pub(super) fn gen_identifier_or_param(
    original_query: &Query,
    name: &str,
    should_ref: bool,
    _should_clone: bool,
) -> GeneratedValue {
    if is_param(original_query, name) {
        if should_ref {
            GeneratedValue::Parameter(GenRef::Ref(format!("data.{}", name)))
        } else {
            GeneratedValue::Parameter(GenRef::Std(format!("data.{}.clone()", name)))
        }
    } else {
        if should_ref {
            GeneratedValue::Identifier(GenRef::Ref(name.to_string()))
        } else {
            GeneratedValue::Identifier(GenRef::Std(format!("{}.clone()", name.to_string())))
        }
    }
}

pub(super) fn gen_id_access_or_param(original_query: &Query, name: &str) -> GeneratedValue {
    if is_param(original_query, name) {
        GeneratedValue::Parameter(GenRef::DeRef(format!("data.{}", name)))
    } else {
        GeneratedValue::Identifier(GenRef::Std(format!("{}.id()", name.to_string())))
    }
}

pub(super) fn type_in_scope(
    ctx: &mut Ctx,
    original_query: &Query,
    loc: Loc,
    scope: &HashMap<&str, Type>,
    name: &str,
) -> Option<Type> {
    match scope.get(name) {
        Some(ty) => Some(ty.clone()),
        None => {
            push_query_err(
                ctx,
                original_query,
                loc.clone(),
                format!("variable named `{}` is not in scope", name),
                "declare {} in the current scope or fix the typo".to_string(),
            );
            None
        }
    }
}

pub(super) fn field_exists_on_item_type(
    ctx: &mut Ctx,
    original_query: &Query,
    item_type: Type,
    fields: Vec<(&str, &Loc)>,
) {
    match item_type {
        Type::Node(Some(node_type)) => {
            for (key, loc) in fields {
                if !ctx
                    .node_fields
                    .get(node_type.as_str())
                    .map(|fields| fields.contains_key(key))
                    .unwrap_or(true)
                {
                    push_query_err(
                        ctx,
                        original_query,
                        loc.clone(),
                        format!(
                            "`{}` is not a field of node `{}` {}",
                            key,
                            node_type,
                            line!()
                        ),
                        "check the schema field names",
                    );
                }
            }
        }
        Type::Edge(Some(edge_type)) => {
            for (key, loc) in fields {
                if !ctx
                    .edge_fields
                    .get(edge_type.as_str())
                    .map(|fields| fields.contains_key(key))
                    .unwrap_or(true)
                {
                    push_query_err(
                        ctx,
                        original_query,
                        loc.clone(),
                        format!(
                            "`{}` is not a field of edge `{}` {}",
                            key,
                            edge_type,
                            line!()
                        ),
                        "check the schema field names",
                    );
                }
            }
        }
        Type::Vector(_) => unreachable!("Updating vectors is not supported yet"),
        _ => unreachable!("shouldve been caught eariler"),
    }
}
