

use heed3::RoTxn;
use proc_macros::handler;
use helixdb::{
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
        router::router::HandlerInput,
    },
    node_matches, props, embed,
    field_remapping, identifier_remapping, 
    traversal_remapping, exclude_field, value_remapping, 
    protocol::{
        remapping::{Remapping, RemappingMap, ResponseRemapping},
        response::Response,
        return_values::ReturnValue,
        value::Value,
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
    
pub struct Entity {
    pub entity_name: String,
}

pub struct Relationship {
    pub from: Entity,
    pub to: Entity,
    pub edge_name: String,
}

pub struct Embedding {
    pub vector_name: String,
    pub vec: Vec<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct get_entityInput {

pub entity_name_in: String
}
#[handler]
pub fn get_entity (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
let data: get_entityInput = match sonic_rs::from_slice(&input.request.body) {
    Ok(data) => data,
    Err(err) => return Err(GraphError::from(err)),
};

let mut remapping_vals = RemappingMap::new();
let db = Arc::clone(&input.graph.storage);
let txn = db.graph_env.read_txn().unwrap();
    let node = G::new(Arc::clone(&db), &txn)
.n_from_index("entity_name", &data.entity_name_in).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("node".to_string(), ReturnValue::from_traversal_value_with_mixin(node.clone().clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct insert_relationshipInput {

pub from_entity_label: String,
pub to_entity_label: String,
pub edge_name_in: String
}
#[handler]
pub fn insert_relationship (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
let data: insert_relationshipInput = match sonic_rs::from_slice(&input.request.body) {
    Ok(data) => data,
    Err(err) => return Err(GraphError::from(err)),
};

let mut remapping_vals = RemappingMap::new();
let db = Arc::clone(&input.graph.storage);
let mut txn = db.graph_env.write_txn().unwrap();
    let from_entity = G::new(Arc::clone(&db), &txn)
.n_from_index("entity_name", &data.from_entity_label).collect_to_obj();
    let to_entity = G::new(Arc::clone(&db), &txn)
.n_from_index("entity_name", &data.to_entity_label).collect_to_obj();
    let e = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("Relationship", Some(props! { "edge_name" => data.edge_name_in.clone() }), from_entity.id(), to_entity.id(), true, EdgeType::Node).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("e".to_string(), ReturnValue::from_traversal_value_with_mixin(e.clone().clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct insert_entityInput {

pub entity_name_in: String
}
#[handler]
pub fn insert_entity (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
let data: insert_entityInput = match sonic_rs::from_slice(&input.request.body) {
    Ok(data) => data,
    Err(err) => return Err(GraphError::from(err)),
};

let mut remapping_vals = RemappingMap::new();
let db = Arc::clone(&input.graph.storage);
let mut txn = db.graph_env.write_txn().unwrap();
    let node = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Entity", Some(props! { "entity_name" => &data.entity_name_in }), Some(&["entity_name"])).collect_to_obj();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("node".to_string(), ReturnValue::from_traversal_value_with_mixin(node.clone().clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}
