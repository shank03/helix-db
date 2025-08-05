// DEFAULT CODE
// use helix_db::helix_engine::graph_core::config::Config;

// pub fn config() -> Option<Config> {
//     None
// }

use chrono::{DateTime, Utc};
use heed3::RoTxn;
use helix_db::{
    embed, exclude_field, field_addition_from_old_field, field_addition_from_value,
    field_remapping, field_type_cast,
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
                    filter_ref::FilterRefAdapter, map::MapAdapter, order::OrderByAdapter,
                    paths::ShortestPathAdapter, props::PropsAdapter, range::RangeAdapter,
                    update::UpdateAdapter,
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
        value::{
            Value,
            casting::{CastType, cast},
        },
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
use helix_macros::{handler, mcp_handler, migration, tool_call};
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
        schema: Some(
            r#"{
  "schema": {
    "nodes": [],
    "vectors": [
      {
        "name": "UserEmbedding",
        "properties": {
          "id": "ID",
          "userId": "String",
          "dataType": "String",
          "createdAt": "Date",
          "metadata": "String",
          "lastUpdated": "String"
        }
      }
    ],
    "edges": []
  },
  "queries": [
    {
      "name": "CreateUserBioEmbedding",
      "parameters": {
        "userId": "String",
        "bioText": "String",
        "lastUpdated": "String"
      },
      "returns": [
        "embedding"
      ]
    },
    {
      "name": "SearchSimilarUsers",
      "parameters": {
        "k": "I64",
        "queryText": "String",
        "dataType": "String"
      },
      "returns": [
        "search_results"
      ]
    }
  ]
}"#
            .to_string(),
        ),
        embedding_model: None,
        graphvis_node_label: None,
    });
}

pub struct UserEmbedding {
    pub userId: String,
    pub dataType: String,
    pub metadata: String,
    pub lastUpdated: String,
    pub createdAt: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserBioEmbeddingInput {
    pub userId: String,
    pub bioText: String,
    pub lastUpdated: String,
}
#[handler(with_write)]
pub fn CreateUserBioEmbedding(input: &HandlerInput) -> Result<Response, GraphError> {
    {
        let embedding = G::new_mut(Arc::clone(&db), &mut txn)
.insert_v::<fn(&HVector, &RoTxn) -> bool>(&embed!(db, &data.bioText), "UserEmbedding", Some(props! { "metadata" => "{}", "userId" => data.userId.clone(), "dataType" => "bio", "lastUpdated" => data.lastUpdated.clone() })).collect_to_obj();
        let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert(
            "embedding".to_string(),
            ReturnValue::from_traversal_value_with_mixin(
                embedding.clone().clone(),
                remapping_vals.borrow_mut(),
            ),
        );
    }
}

#[derive(Serialize, Deserialize)]
pub struct SearchSimilarUsersInput {
    pub queryText: Option<String>,
    pub k: i64,
    pub dataType: String,
}
#[handler(with_read)]
pub fn SearchSimilarUsers(input: &HandlerInput) -> Result<Response, GraphError> {
    {
        let search_results = G::new(Arc::clone(&db), &txn)
            .search_v::<fn(&HVector, &RoTxn) -> bool, _>(
                &embed!(
                    db,
                    data.queryText
                        .as_ref()
                        .ok_or_else(|| GraphError::ParamNotFound("queryText"))?
                ),
                data.k.clone(),
                "UserEmbedding",
                None,
            )
            .collect_to::<Vec<_>>();
        let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert(
            "search_results".to_string(),
            ReturnValue::from_traversal_value_array_with_mixin(
                search_results.clone().clone(),
                remapping_vals.borrow_mut(),
            ),
        );
    }
}
