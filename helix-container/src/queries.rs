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

pub struct Continent {
    pub name: String,
}

pub struct Country {
    pub name: String,
    pub currency: String,
    pub population: i64,
    pub gdp: f64,
}

pub struct City {
    pub name: String,
    pub description: String,
    pub zip_codes: Vec<String>,
}

pub struct Continent_to_Country {
    pub from: Continent,
    pub to: Country,
}

pub struct Country_to_City {
    pub from: Country,
    pub to: City,
}

pub struct Country_to_Capital {
    pub from: Country,
    pub to: City,
}

#[derive(Serialize, Deserialize)]
pub struct countCapitalsInput {}
#[handler(with_read)]
pub fn countCapitals(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    {
        let num_capital = G::new(Arc::clone(&db), &txn)
            .n_from_type("City")
            .filter_ref(|val, txn| {
                if let Ok(val) = val {
                    Ok(Exist::exists(
                        &mut G::new_from(Arc::clone(&db), &txn, vec![val.clone()])
                            .in_("Country_to_Capital", &EdgeType::Node),
                    ))
                } else {
                    Ok(false)
                }
            })
            .count_to_val();
        let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert(
            "num_capital".to_string(),
            ReturnValue::from(Value::from(num_capital.clone())),
        );
    }
    Ok(())
}
