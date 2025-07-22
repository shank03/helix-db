
// DEFAULT CODE
// use helix_db::helix_engine::graph_core::config::Config;

// pub fn config() -> Option<Config> {
//     None
// }



use heed3::RoTxn;
use helix_macros::{handler, tool_call, mcp_handler};
use helix_db::{
    helix_engine::{
        graph_core::{
            config::{Config, GraphConfig, VectorConfig},
            ops::{
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
            }
        },
        types::GraphError,
        vector_core::vector::HVector,
    },
    helix_gateway::{
        embedding_providers::embedding_providers::{EmbeddingModel, get_embedding_model},
        router::router::HandlerInput,
        mcp::mcp::{MCPHandlerSubmission, MCPToolInput, MCPHandler}
    },
    node_matches, props, embed,
    field_remapping, identifier_remapping, 
    traversal_remapping, exclude_field, value_remapping, 
    protocol::{
        remapping::{Remapping, RemappingMap, ResponseRemapping},
        response::Response,
        return_values::ReturnValue,
        value::Value,
        format::Format,
    },
    utils::{
        count::Count,
        filterable::Filterable,
        id::ID,
        items::{Edge, Node},
    },
};
use sonic_rs::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use chrono::{DateTime, Utc};
    
pub fn config() -> Option<Config> {return Some(Config {vector_config: Some(VectorConfig {m: Some(16),ef_construction: Some(128),ef_search: Some(768),}),graph_config: Some(GraphConfig {secondary_indices: Some(vec![]),}),db_max_size_gb: Some(20),mcp: Some(true),bm25: Some(true),schema: None,embedding_model: None,graphvis_node_label: None,})}
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
pub struct countCapitalsInput {


}
#[handler(with_read)]
pub fn countCapitals (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
{
    let num_capital = G::new(Arc::clone(&db), &txn)
.n_from_type("City")

.filter_ref(|val, txn|{
                if let Ok(val) = val { 
                    Ok(Exist::exists(&mut G::new_from(Arc::clone(&db), &txn, vec![val.clone()])

.in_("Country_to_Capital",&EdgeType::Node)))
                } else {
                    Ok(false)
                }
            })

.count_to_val();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("num_capital".to_string(), ReturnValue::from(Value::from(num_capital.clone())));

}
    Ok(())
}
