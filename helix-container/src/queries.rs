// DEFAULT CODE
// use helix_db::helix_engine::graph_core::config::Config;

// pub fn config() -> Option<Config> {
//     None
// }

use chrono::{DateTime, Utc};
use heed3::RoTxn;
use helix_db::{
    embed, exclude_field, field_remapping,
    helix_engine::{
        graph_core::{
            config::{Config, GraphConfig, VectorConfig},
            ops::{
                bm25::search_bm25::SearchBM25Adapter,
                g::G,
                in_::{in_::InAdapter, in_e::InEdgesAdapter, to_n::ToNAdapter, to_v::ToVAdapter},
                out::{
                    from_n::FromNAdapter, from_v::FromVAdapter, out::OutAdapter,
                    out_e::OutEdgesAdapter,
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

pub fn config() -> Option<Config> {
    return Some(Config {
        vector_config: Some(VectorConfig {
            m: Some(16),
            ef_construction: Some(128),
            ef_search: Some(768),
        }),
        graph_config: Some(GraphConfig {
            secondary_indices: Some(vec![]),
        }),
        db_max_size_gb: Some(20),
        mcp: Some(true),
        bm25: Some(true),
        schema: None,
        embedding_model: None,
        graphvis_node_label: None,
    });
}

pub struct User {
    pub name: String,
    pub age: i32,
}

#[derive(Serialize, Deserialize)]
pub struct dropUserInput {
    pub query: String,
}
#[handler(with_write)]
pub fn dropUser(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    {
        Drop::<Vec<_>>::drop_traversal(
            G::new(Arc::clone(&db), &txn)
                .search_v::<fn(&HVector, &RoTxn) -> bool>(&embed!(db, &data.query), 10, None)
                .collect_to::<Vec<_>>(),
            Arc::clone(&db),
            &mut txn,
        )?;
        let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert(
            "success".to_string(),
            ReturnValue::from(Value::from("success")),
        );
    }
    Ok(())
}
