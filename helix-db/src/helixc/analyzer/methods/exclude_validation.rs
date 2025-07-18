//! Semantic analyzer for Helix‑QL.
use crate::{
    helix_engine::graph_core::ops::source::add_e::EdgeType,
    helixc::{
        analyzer::{
            analyzer::Ctx,
            diagnostic::Diagnostic,
            errors::{push_query_err, push_query_err_with_fix, push_query_warn, push_schema_err},
            fix::Fix,
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
