//! Semantic analyzer for Helix‑QL.
use super::{fix::Fix, pretty};
use crate::{
    helix_engine::graph_core::ops::source::add_e::EdgeType,
    helixc::{
        analyzer::{
            analyzer::Ctx,
            diagnostic::Diagnostic,
            errors::{push_query_err, push_query_err_with_fix, push_query_warn, push_schema_err},
            methods::schema_methods::{build_field_lookups, check_schema},
            types::Type,
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

pub(super) fn is_valid_identifier(ctx: &mut Ctx, q: &Query, loc: Loc, name: &str) -> bool {
    match name {
        "true" | "false" | "NONE" | "String" | "Boolean" | "F32" | "F64" | "I8" | "I16" | "I32"
        | "I64" | "U8" | "U16" | "U32" | "U64" | "U128" | "Uuid" | "Date" => {
            push_query_err(
                ctx,
                q,
                loc.clone(),
                format!("`{}` is not a valid identifier", name),
                "use a valid identifier",
            );
            false
        }
        _ => true,
    }
}

pub(super) fn is_param(ctx: &mut Ctx, q: &Query, name: &str) -> bool {
    q.parameters.iter().find(|p| p.name.1 == *name).is_some()
}

pub(super) fn gen_identifier_or_param(
    ctx: &mut Ctx,
    q: &Query,
    name: &str,
    should_ref: bool,
    should_clone: bool,
) -> GeneratedValue {
    if is_param(ctx, q, name) {
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

pub(super) fn gen_id_access_or_param(ctx: &mut Ctx, q: &Query, name: &str) -> GeneratedValue {
    if is_param(ctx, q, name) {
        GeneratedValue::Parameter(GenRef::DeRef(format!("data.{}", name)))
    } else {
        GeneratedValue::Identifier(GenRef::Std(format!("{}.id()", name.to_string())))
    }
}

pub(super) fn type_in_scope(
    ctx: &mut Ctx,
    q: &Query,
    loc: Loc,
    scope: &HashMap<&str, Type>,
    name: &str,
) -> Option<Type> {
    match scope.get(name) {
        Some(ty) => Some(ty.clone()),
        None => {
            push_query_err(
                ctx,
                q,
                loc.clone(),
                format!("variable named `{}` is not in scope", name),
                "declare {} in the current scope or fix the typo".to_string(),
            );
            None
        }
    }
}
