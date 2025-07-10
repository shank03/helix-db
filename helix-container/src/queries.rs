

use heed3::RoTxn;
use proc_macros::handler;
use helixdb::{field_remapping, identifier_remapping, traversal_remapping, exclude_field, value_remapping, embed};
use helixdb::helix_engine::vector_core::vector::HVector;
use helixdb::{
    helix_engine::graph_core::ops::{
        g::G,
        in_::{in_::InAdapter, in_e::InEdgesAdapter, to_n::ToNAdapter, to_v::ToVAdapter},
        out::{from_n::FromNAdapter, from_v::FromVAdapter, out::OutAdapter, out_e::OutEdgesAdapter},
        source::{
            add_e::{AddEAdapter, EdgeType},
            add_n::AddNAdapter,
            e_from_id::EFromIdAdapter,
            e_from_type::EFromTypeAdapter,
            n_from_id::NFromIdAdapter,
            n_from_type::NFromTypeAdapter,
            n_from_index::NFromIndexAdapter,
        },
        tr_val::{Traversable, TraversalVal},
        util::{
            dedup::DedupAdapter, filter_mut::FilterMut,
            filter_ref::FilterRefAdapter, range::RangeAdapter, update::UpdateAdapter,
            map::MapAdapter, paths::ShortestPathAdapter, props::PropsAdapter, drop::Drop,
        },
        vectors::{insert::InsertVAdapter, search::SearchVAdapter, brute_force_search::BruteForceSearchVAdapter},
        bm25::search_bm25::SearchBM25Adapter,

    },
    helix_engine::types::GraphError,
    helix_gateway::router::router::HandlerInput,
    node_matches, props,
    protocol::count::Count,
    protocol::remapping::{RemappingMap, ResponseRemapping},
    protocol::response::Response,
    protocol::{
        filterable::Filterable, remapping::Remapping, return_values::ReturnValue, value::Value, id::ID,
    },
    providers::embedding_providers::get_embedding_model,
};
use sonic_rs::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use std::cell::RefCell;
use chrono::{DateTime, Utc};
    
pub struct User {
    pub user_num: u64,
    pub age: u32,
    pub gender: String,
    pub r_status: String,
    pub interests: Vec<String>,
    pub numoffriends: u64,
}

pub struct Friends {
    pub from: User,
    pub to: User,
}


#[derive(Serialize, Deserialize)]
pub struct insertUserInput {

pub in_user_num: u64,
pub in_age: u32,
pub in_gender: String,
pub in_r_status: String,
pub in_interests: Vec<String>,
pub in_numoffriends: u64
}
#[handler]
pub fn insertUser (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
let data: insertUserInput = match sonic_rs::from_slice(&input.request.body) {
    Ok(data) => data,
    Err(err) => return Err(GraphError::from(err)),
};

let mut remapping_vals = RemappingMap::new();
let db = Arc::clone(&input.graph.storage);
let mut txn = db.graph_env.write_txn().unwrap();
    let n = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("User", Some(props! { "interests" => data.in_interests.clone(), "age" => data.in_age.clone(), "gender" => data.in_gender.clone(), "user_num" => data.in_user_num.clone(), "r_status" => data.in_r_status.clone(), "numoffriends" => data.in_numoffriends.clone() }), Some(&["user_num"])).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("Success".to_string(), ReturnValue::from(Value::from("Success")));

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct insertFriendRelationshipInput {

pub from: u64,
pub to: u64
}
#[handler]
pub fn insertFriendRelationship (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
let data: insertFriendRelationshipInput = match sonic_rs::from_slice(&input.request.body) {
    Ok(data) => data,
    Err(err) => return Err(GraphError::from(err)),
};

let mut remapping_vals = RemappingMap::new();
let db = Arc::clone(&input.graph.storage);
let mut txn = db.graph_env.write_txn().unwrap();
    let from_user = G::new(Arc::clone(&db), &txn)
.n_from_index("user_num", &data.from).collect_to::<Vec<_>>();
    let to_user = G::new(Arc::clone(&db), &txn)
.n_from_index("user_num", &data.to).collect_to::<Vec<_>>();
    let e = G::new_mut(Arc::clone(&db), &mut txn)
.add_e("Friends", None, from_user.id(), to_user.id(), true, EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("Success".to_string(), ReturnValue::from(Value::from("Success")));

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct getUserInput {

pub in_user_num: u64
}
#[handler]
pub fn getUser (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
let data: getUserInput = match sonic_rs::from_slice(&input.request.body) {
    Ok(data) => data,
    Err(err) => return Err(GraphError::from(err)),
};

let mut remapping_vals = RemappingMap::new();
let db = Arc::clone(&input.graph.storage);
let txn = db.graph_env.read_txn().unwrap();
    let u = G::new(Arc::clone(&db), &txn)
.n_from_index("user_num", &data.in_user_num).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("u".to_string(), ReturnValue::from_traversal_value_array_with_mixin(u.clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}
