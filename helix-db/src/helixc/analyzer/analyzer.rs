//! Semantic analyzer for Helix‑QL.
use crate::helixc::{
    analyzer::{
        diagnostic::Diagnostic,
        methods::{
            query_validation::check_query,
            schema_methods::{build_field_lookups, check_schema},
        },
    },
    generator::generator_types::Source as GeneratedSource,
    parser::helix_parser::{EdgeSchema, Field, Source},
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

pub fn analyze(src: &Source) -> (Vec<Diagnostic>, GeneratedSource) {
    let mut ctx = Ctx::new(src);
    ctx.check_schema();
    ctx.check_queries();
    (ctx.diagnostics, ctx.output)
}

/// Internal working context shared by all passes.
pub(crate) struct Ctx<'a> {
    pub(super) src: &'a Source,

    /// Quick look‑ups
    pub(super) node_set: HashSet<&'a str>,
    pub(super) vector_set: HashSet<&'a str>,
    pub(super) edge_map: HashMap<&'a str, &'a EdgeSchema>,
    pub(super) node_fields: HashMap<&'a str, HashMap<&'a str, Cow<'a, Field>>>,
    pub(super) edge_fields: HashMap<&'a str, HashMap<&'a str, Cow<'a, Field>>>,
    pub(super) vector_fields: HashMap<&'a str, HashMap<&'a str, Cow<'a, Field>>>,
    pub(super) diagnostics: Vec<Diagnostic>,
    pub(super) output: GeneratedSource,
}

impl<'a> Ctx<'a> {
    pub(super) fn new(src: &'a Source) -> Self {
        // Build field look‑ups once
        let (node_fields, edge_fields, vector_fields) = build_field_lookups(src);

        let mut output = GeneratedSource::default();
        output.src = src.source.clone();

        Self {
            node_set: src.node_schemas.iter().map(|n| n.name.1.as_str()).collect(),
            vector_set: src.vector_schemas.iter().map(|v| v.name.as_str()).collect(),
            edge_map: src
                .edge_schemas
                .iter()
                .map(|e| (e.name.1.as_str(), e))
                .collect(),
            node_fields,
            edge_fields,
            vector_fields,
            src,
            diagnostics: Vec::new(),
            output,
        }
    }

    // ---------- Pass #1: schema --------------------------
    /// Validate that every edge references declared node types.
    pub(super) fn check_schema(&mut self) {
        check_schema(self);
    }

    // ---------- Pass #2: queries -------------------------
    pub(super) fn check_queries(&mut self) {
        for q in &self.src.queries {
            check_query(self, q);
        }
    }
}
