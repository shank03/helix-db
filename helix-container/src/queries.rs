// DEFAULT CODE
// use helix_db::helix_engine::graph_core::config::Config;

// pub fn config() -> Option<Config> {
//     None
// }

use chrono::{DateTime, Utc};
use heed3::RoTxn;
use helix_db::{
    embed, embed_async, exclude_field, field_addition_from_old_field, field_addition_from_value,
    field_remapping, field_type_cast,
    helix_engine::{
        traversal_core::{
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
            traversal_value::{Traversable, TraversalValue},
        },
        types::GraphError,
        vector_core::vector::HVector,
    },
    helix_gateway::{
        embedding_providers::embedding_providers::{EmbeddingModel, get_embedding_model},
        mcp::mcp::{MCPHandler, MCPHandlerSubmission, MCPToolInput},
        router::router::{HandlerInput, IoContFn},
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
        db_max_size_gb: Some(10),
        mcp: Some(true),
        bm25: Some(true),
        schema: Some(
            r#"{
  "schema": {
    "nodes": [
      {
        "name": "User",
        "properties": {
          "age": "I32",
          "id": "ID",
          "name": "String"
        }
      }
    ],
    "vectors": [],
    "edges": [
      {
        "name": "Knows",
        "from": "User",
        "to": "User",
        "properties": {}
      }
    ]
  },
  "queries": [
    {
      "name": "object_remapping_from_identfier_property_access_with_spread",
      "parameters": {
        "id": "ID"
      },
      "returns": []
    },
    {
      "name": "object_remapping_from_unnamed_traversal",
      "parameters": {
        "other_id": "ID",
        "id": "ID"
      },
      "returns": []
    },
    {
      "name": "object_remapping_from_anon_traversal_with_spread",
      "parameters": {
        "id": "ID"
      },
      "returns": []
    },
    {
      "name": "create_user",
      "parameters": {
        "name": "String"
      },
      "returns": [
        "user",
        "user2"
      ]
    },
    {
      "name": "object_remapping_with_empty_spread",
      "parameters": {
        "id": "ID"
      },
      "returns": []
    },
    {
      "name": "object_remapping_from_unnamed_traversal_with_spread",
      "parameters": {
        "id": "ID",
        "other_id": "ID"
      },
      "returns": []
    },
    {
      "name": "object_remapping_from_anon_property_access",
      "parameters": {
        "id": "ID"
      },
      "returns": []
    },
    {
      "name": "object_remapping_from_identfier_property_access",
      "parameters": {
        "id": "ID"
      },
      "returns": []
    },
    {
      "name": "object_remapping_from_anon_traversal",
      "parameters": {
        "id": "ID"
      },
      "returns": []
    },
    {
      "name": "object_remapping_from_anon_property_access_with_spread",
      "parameters": {
        "id": "ID"
      },
      "returns": []
    }
  ]
}"#
            .to_string(),
        ),
        embedding_model: Some("text-embedding-ada-002".to_string()),
        graphvis_node_label: Some("".to_string()),
    });
}

pub struct User {
    pub name: String,
    pub age: i32,
}

pub struct Knows {
    pub from: User,
    pub to: User,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct object_remapping_from_identfier_property_access_with_spreadInput {
    pub id: ID,
}
#[handler]
pub fn object_remapping_from_identfier_property_access_with_spread(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<object_remapping_from_identfier_property_access_with_spreadInput>(
        &input.request.body,
    )?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.id)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_array_with_mixin(G::new_from(Arc::clone(&db), &txn, user.clone())

.map_traversal(|item, txn| { identifier_remapping!(remapping_vals, item.clone(), true, "username" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("name").collect_to_obj())?;
 Ok(item) }).collect_to::<Vec<_>>().clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct object_remapping_from_unnamed_traversalInput {
    pub id: ID,
    pub other_id: ID,
}
#[handler]
pub fn object_remapping_from_unnamed_traversal(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<object_remapping_from_unnamed_traversalInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.id)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_array_with_mixin(G::new_from(Arc::clone(&db), &txn, user.clone())

.map_traversal(|item, txn| { traversal_remapping!(remapping_vals, item.clone(), false, "knows" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.in_("Knows",&EdgeType::Node).collect_to::<Vec<_>>())?;
 Ok(item) }).collect_to::<Vec<_>>().clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct object_remapping_from_anon_traversal_with_spreadInput {
    pub id: ID,
}
#[handler]
pub fn object_remapping_from_anon_traversal_with_spread(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<object_remapping_from_anon_traversal_with_spreadInput>(
            &input.request.body,
        )?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.id)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_array_with_mixin(G::new_from(Arc::clone(&db), &txn, user.clone())

.map_traversal(|item, txn| { traversal_remapping!(remapping_vals, item.clone(), true, "knows" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.out("Knows",&EdgeType::Node).collect_to::<Vec<_>>())?;
 Ok(item) }).collect_to::<Vec<_>>().clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct create_userInput {
    pub name: String,
}
#[handler]
pub fn create_user(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<create_userInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let mut txn = db.graph_env.write_txn().unwrap();
    let user = G::new_mut(Arc::clone(&db), &mut txn)
        .add_n(
            "User",
            Some(props! { "name" => &data.name, "age" => 50 }),
            None,
        )
        .collect_to_obj();
    let user2 = G::new_mut(Arc::clone(&db), &mut txn)
        .add_n("User", Some(props! { "age" => 20, "name" => "John" }), None)
        .collect_to_obj();
    let edge = G::new_mut(Arc::clone(&db), &mut txn)
        .add_e("Knows", None, user.id(), user2.id(), true, EdgeType::Node)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            user.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "user2".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            user2.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct object_remapping_with_empty_spreadInput {
    pub id: ID,
}
#[handler]
pub fn object_remapping_with_empty_spread(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<object_remapping_with_empty_spreadInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.id)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            G::new_from(Arc::clone(&db), &txn, user.clone())
                .map_traversal(|item, txn| {
                    value_remapping!(remapping_vals, item.clone(), true, "name" => "test")?;
                    Ok(item)
                })
                .collect_to::<Vec<_>>()
                .clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct object_remapping_from_unnamed_traversal_with_spreadInput {
    pub id: ID,
    pub other_id: ID,
}
#[handler]
pub fn object_remapping_from_unnamed_traversal_with_spread(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<object_remapping_from_unnamed_traversal_with_spreadInput>(
            &input.request.body,
        )?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.id)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_array_with_mixin(G::new_from(Arc::clone(&db), &txn, user.clone())

.map_traversal(|item, txn| { traversal_remapping!(remapping_vals, item.clone(), true, "knows" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.in_("Knows",&EdgeType::Node).collect_to::<Vec<_>>())?;
 Ok(item) }).collect_to::<Vec<_>>().clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct object_remapping_from_anon_property_accessInput {
    pub id: ID,
}
#[handler]
pub fn object_remapping_from_anon_property_access(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<object_remapping_from_anon_property_accessInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.id)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_array_with_mixin(G::new_from(Arc::clone(&db), &txn, user.clone())

.map_traversal(|item, txn| { identifier_remapping!(remapping_vals, item.clone(), false, "username" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("name").collect_to_obj())?;
identifier_remapping!(remapping_vals, item.clone(), false, "age" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("age").collect_to_obj())?;
 Ok(item) }).collect_to::<Vec<_>>().clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct object_remapping_from_identfier_property_accessInput {
    pub id: ID,
}
#[handler]
pub fn object_remapping_from_identfier_property_access(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<object_remapping_from_identfier_property_accessInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.id)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_array_with_mixin(G::new_from(Arc::clone(&db), &txn, user.clone())

.map_traversal(|item, txn| { identifier_remapping!(remapping_vals, item.clone(), false, "username" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("name").collect_to_obj())?;
identifier_remapping!(remapping_vals, item.clone(), false, "age" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("age").collect_to_obj())?;
 Ok(item) }).collect_to::<Vec<_>>().clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct object_remapping_from_anon_traversalInput {
    pub id: ID,
}
#[handler]
pub fn object_remapping_from_anon_traversal(input: HandlerInput) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<object_remapping_from_anon_traversalInput>(&input.request.body)?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.id)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_array_with_mixin(G::new_from(Arc::clone(&db), &txn, user.clone())

.map_traversal(|item, txn| { traversal_remapping!(remapping_vals, item.clone(), false, "knows" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.out("Knows",&EdgeType::Node).collect_to::<Vec<_>>())?;
 Ok(item) }).collect_to::<Vec<_>>().clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct object_remapping_from_anon_property_access_with_spreadInput {
    pub id: ID,
}
#[handler]
pub fn object_remapping_from_anon_property_access_with_spread(
    input: HandlerInput,
) -> Result<Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let data = input
        .request
        .in_fmt
        .deserialize::<object_remapping_from_anon_property_access_with_spreadInput>(
            &input.request.body,
        )?;
    let mut remapping_vals = RemappingMap::new();
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.id)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_array_with_mixin(G::new_from(Arc::clone(&db), &txn, user.clone())

.map_traversal(|item, txn| { identifier_remapping!(remapping_vals, item.clone(), true, "username" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("name").collect_to_obj())?;
 Ok(item) }).collect_to::<Vec<_>>().clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    Ok(input.request.out_fmt.create_response(&return_vals))
}
