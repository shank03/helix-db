

use heed3::RoTxn;
use get_routes::handler;
use helixdb::{field_remapping, identifier_remapping, traversal_remapping, exclude_field, value_remapping};
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
};
use sonic_rs::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use std::cell::RefCell;
use chrono::{DateTime, Utc};
    
pub struct User {
    pub name: String,
}

pub struct EmbeddingOf {
    pub from: User,
    pub to: Embedding,
    pub category: String,
}

pub struct Embedding {
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct addInput {

pub vec: Vec<f64>
}
#[handler]
pub fn add (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
let data: addInput = match sonic_rs::from_slice(&input.request.body) {
    Ok(data) => data,
    Err(err) => return Err(GraphError::from(err)),
};

let mut remapping_vals = RemappingMap::new();
let db = Arc::clone(&input.graph.storage);
let mut txn = db.graph_env.write_txn().unwrap();
    let user = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("User", Some(props! { "name" => "John Doe" }), Some(&["name"])).collect_to::<Vec<_>>();
    let embedding = G::new_mut(Arc::clone(&db), &mut txn)
.insert_v::<fn(&HVector, &RoTxn) -> bool>(&data.vec, "Embedding", Some(props! { "content" => "Hello, world!" })).collect_to::<Vec<_>>();
    G::new_mut(Arc::clone(&db), &mut txn)
.add_e("EmbeddingOf", Some(props! { "category" => "test" }), user.id(), embedding.id(), true, EdgeType::Node).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_array_with_mixin(user.clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct to_vInput {

pub query: Vec<f64>,
pub k: i32,
pub data: String
}
#[handler]
pub fn to_v (input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
let data: to_vInput = match sonic_rs::from_slice(&input.request.body) {
    Ok(data) => data,
    Err(err) => return Err(GraphError::from(err)),
};

let mut remapping_vals = RemappingMap::new();
let db = Arc::clone(&input.graph.storage);
let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
.n_from_index("name", &"John Doe").collect_to::<Vec<_>>();
    let edges = G::new_from(Arc::clone(&db), &txn, user.clone())

.out_e("EmbeddingOf").collect_to::<Vec<_>>();
    let filtered = G::new_from(Arc::clone(&db), &txn, edges.clone())

.filter_ref(|val, txn|{
                if let Ok(val) = val { 
                    Ok(G::new_from(Arc::clone(&db), &txn, val.clone())

.check_property("category")

.map_value_or(false, |v| *v == data.data)?)
                } else {
                    Ok(false)
                }
            }).collect_to::<Vec<_>>();
    let vectors = G::new_from(Arc::clone(&db), &txn, filtered.clone())

.to_v().collect_to::<Vec<_>>();
    let searched = G::new(Arc::clone(&db), &txn)

.brute_force_search_v(&data.query, data.k as usize).collect_to::<Vec<_>>();
let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
        return_vals.insert("user".to_string(), ReturnValue::from_traversal_value_array_with_mixin(user.clone(), remapping_vals.borrow_mut()));

        return_vals.insert("edges".to_string(), ReturnValue::from_traversal_value_array_with_mixin(edges.clone(), remapping_vals.borrow_mut()));

        return_vals.insert("filtered".to_string(), ReturnValue::from_traversal_value_array_with_mixin(filtered.clone(), remapping_vals.borrow_mut()));

        return_vals.insert("vectors".to_string(), ReturnValue::from_traversal_value_array_with_mixin(vectors.clone(), remapping_vals.borrow_mut()));

        return_vals.insert("searched".to_string(), ReturnValue::from_traversal_value_array_with_mixin(searched.clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}
