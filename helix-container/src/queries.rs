use chrono::{DateTime, Utc};
use heed3::RoTxn;
use helix_db::{
    embed, exclude_field, field_remapping,
    helix_engine::{
        graph_core::ops::{
            bm25::search_bm25::SearchBM25Adapter,
            g::G,
            in_::{in_::InAdapter, in_e::InEdgesAdapter, to_n::ToNAdapter, to_v::ToVAdapter},
            out::{
                from_n::FromNAdapter, from_v::FromVAdapter, out::OutAdapter, out_e::OutEdgesAdapter,
            },
            source::{
                add_e::{AddEAdapter, EdgeType},
                add_n::AddNAdapter,
                e_from_id::EFromIdAdapter,
                e_from_type::EFromTypeAdapter,
                n_from_id::NFromIdAdapter,
                n_from_index::NFromIndexAdapter,
                n_from_type::NFromTypeAdapter,
            },
            tr_val::{Traversable, TraversalVal},
            util::{
                dedup::DedupAdapter, drop::Drop, exist::Exist, filter_mut::FilterMut,
                filter_ref::FilterRefAdapter, map::MapAdapter, paths::ShortestPathAdapter,
                props::PropsAdapter, range::RangeAdapter, update::UpdateAdapter,
            },
            vectors::{
                brute_force_search::BruteForceSearchVAdapter, insert::InsertVAdapter,
                search::SearchVAdapter,
            },
        },
        types::GraphError,
        vector_core::vector::HVector,
    },
    helix_gateway::{
        embedding_providers::embedding_providers::{EmbeddingModel, get_embedding_model},
        mcp::mcp::{MCPHandler, MCPHandlerSubmission, MCPToolInput},
        router::router::HandlerInput,
    },
    identifier_remapping, node_matches, props,
    protocol::{
        format::Format,
        remapping::{Remapping, RemappingMap, ResponseRemapping},
        response::Response,
        return_values::ReturnValue,
        value::Value,
    },
    traversal_remapping,
    utils::{
        count::Count,
        filterable::Filterable,
        id::ID,
        items::{Edge, Node},
    },
    value_remapping,
};
use helix_macros::{handler, mcp_handler, tool_call};
use sonic_rs::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

pub struct Country {
    pub name: String,
    pub currency: String,
    pub population: u64,
    pub gdp: f64,
}

#[derive(Serialize, Deserialize)]
pub struct getCountriesByPopulationInput {
    pub max_population: u64,
}
#[handler(with_read)]
pub fn getCountriesByPopulation(
    input: &HandlerInput,
    response: &mut Response,
) -> Result<(), GraphError> {
    {
        let countries = G::new(Arc::clone(&db), &txn)
            .n_from_type("Country")
            .filter_ref(|val, txn| {
                if let Ok(val) = val {
                    Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                        .check_property("population")
                        .map_value_or(false, |v| *v < data.max_population.clone())?)
                } else {
                    Ok(false)
                }
            })
            .collect_to::<Vec<_>>();
        let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert(
            "countries".to_string(),
            ReturnValue::from_traversal_value_array_with_mixin(
                countries.clone().clone(),
                remapping_vals.borrow_mut(),
            ),
        );
    }
    Ok(())
}
