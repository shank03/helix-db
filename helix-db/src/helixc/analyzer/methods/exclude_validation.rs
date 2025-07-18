//! Semantic analyzer for Helix‑QL.
use crate::helixc::{
    analyzer::{
        analyzer::Ctx,
        errors::{push_query_err, push_query_err_with_fix},
        fix::Fix,
        types::Type,
    },
    parser::{helix_parser::*, location::Loc},
};
use std::{borrow::Cow, collections::HashMap};

pub(crate) fn validate_exclude_fields<'a>(
    ctx: &mut Ctx<'a>,
    ex: &Exclude,
    field_set: &HashMap<&str, Cow<'a, Field>>,
    excluded: &HashMap<&str, Loc>,
    q: &'a Query,
    type_name: &str,
    type_kind: &str,
    span: Option<Loc>,
) {
    for (loc, key) in &ex.fields {
        if let Some(loc) = excluded.get(key.as_str()) {
            push_query_err_with_fix(
                ctx,
                q,
                loc.clone(),
                format!("field `{}` was previously excluded in this traversal", key),
                format!("remove the exclusion of `{}`", key),
                Fix::new(span.clone(), Some(loc.clone()), None),
            );
        } else if !field_set.contains_key(key.as_str()) {
            push_query_err(
                ctx,
                q,
                loc.clone(),
                format!("`{}` is not a field of {} `{}`", key, type_kind, type_name),
                "check the schema field names",
            );
        }
    }
}

pub(crate) fn validate_exclude<'a>(
    ctx: &mut Ctx<'a>,
    cur_ty: &Type,
    tr: &Traversal,
    ex: &Exclude,
    excluded: &HashMap<&str, Loc>,
    q: &'a Query,
) {
    match &cur_ty {
        Type::Nodes(Some(node_ty)) | Type::Node(Some(node_ty)) => {
            if let Some(field_set) = ctx.node_fields.get(node_ty.as_str()).cloned() {
                validate_exclude_fields(
                    ctx,
                    ex,
                    &field_set,
                    &excluded,
                    q,
                    node_ty,
                    "node",
                    Some(tr.loc.clone()),
                );
            }
        }
        Type::Edges(Some(edge_ty)) | Type::Edge(Some(edge_ty)) => {
            // for (key, val) in &obj.fields {
            if let Some(field_set) = ctx.edge_fields.get(edge_ty.as_str()).cloned() {
                validate_exclude_fields(
                    ctx,
                    ex,
                    &field_set,
                    &excluded,
                    q,
                    edge_ty,
                    "edge",
                    Some(tr.loc.clone()),
                );
            }
        }
        Type::Vectors(Some(vector_ty)) | Type::Vector(Some(vector_ty)) => {
            // Vectors only have 'id' and 'embedding' fields
            if let Some(fields) = ctx.vector_fields.get(vector_ty.as_str()).cloned() {
                validate_exclude_fields(
                    ctx,
                    ex,
                    &fields,
                    &excluded,
                    q,
                    vector_ty,
                    "vector",
                    Some(tr.loc.clone()),
                );
            }
        }
        Type::Anonymous(ty) => {
            validate_exclude(ctx, ty, tr, ex, excluded, q);
        }
        _ => {
            push_query_err(
                ctx,
                q,
                ex.fields[0].0.clone(),
                "cannot access properties on this type".to_string(),
                "exclude is only valid on nodes, edges and vectors",
            );
        }
    }
}
