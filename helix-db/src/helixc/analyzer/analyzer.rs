//! Semantic analyzer for Helix‑QL.
use crate::helixc::{
    analyzer::{
        diagnostic::Diagnostic,
        methods::{
            query_validation::validate_query,
            schema_methods::{build_field_lookups, check_schema},
        },
        types::Type,
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

        let output = GeneratedSource {
            src: src.source.clone(),
            ..Default::default()
        };

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

    #[allow(unused)]
    pub(super) fn get_item_fields(&self, item_type: &Type) -> Option<&HashMap<&str, Cow<Field>>> {
        match item_type {
            Type::Node(Some(node_type)) | Type::Nodes(Some(node_type)) => {
                self.node_fields.get(node_type.as_str())
            }
            Type::Edge(Some(edge_type)) | Type::Edges(Some(edge_type)) => {
                self.edge_fields.get(edge_type.as_str())
            }
            Type::Vector(Some(vector_type)) | Type::Vectors(Some(vector_type)) => {
                self.vector_fields.get(vector_type.as_str())
            }
            _ => None,
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
            validate_query(self, q);
        }
    }
}
