use chrono::{DateTime, Utc};
use get_routes::handler;
use heed3::RoTxn;
use helixdb::helix_engine::vector_core::vector::HVector;
use helixdb::{
    exclude_field, field_remapping, identifier_remapping, traversal_remapping, value_remapping,
};
use helixdb::{
    helix_engine::graph_core::ops::{
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
            dedup::DedupAdapter, drop::Drop, filter_mut::FilterMut, filter_ref::FilterRefAdapter,
            map::MapAdapter, paths::ShortestPathAdapter, props::PropsAdapter, range::RangeAdapter,
            update::UpdateAdapter,
        },
        vectors::{
            brute_force_search::BruteForceSearchVAdapter, insert::InsertVAdapter,
            search::SearchVAdapter,
        },
    },
    helix_engine::types::GraphError,
    helix_gateway::router::router::HandlerInput,
    node_matches, props,
    protocol::count::Count,
    protocol::remapping::{RemappingMap, ResponseRemapping},
    protocol::response::Response,
    protocol::{
        filterable::Filterable, id::ID, remapping::Remapping, return_values::ReturnValue,
        value::Value,
    },
};
use sonic_rs::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

pub struct User {}

pub struct EmbeddingOf {
    pub from: User,
    pub to: Embedding,
    pub category: String,
}

pub struct Embedding {}

#[derive(Serialize, Deserialize)]
pub struct to_vInput {
    pub query: Vec<f64>,
    pub k: i32,
    pub data: String,
}
#[handler]
pub fn to_v(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: to_vInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().unwrap();
    let vectors = G::new(Arc::clone(&db), &txn)
        .n_from_type("User")
        .out_e("EmbeddingOf")
        .filter_ref(|val, txn| {
            if let Ok(val) = val {
                Ok(G::new_from(Arc::clone(&db), &txn, val.clone())
                    .check_property("category")
                    .map_value_or(false, |v| *v == "embedding")?)
            } else {
                Ok(false)
            }
        })
        .to_v()
        .collect_to::<Vec<_>>();
    let filtered = G::new(Arc::clone(&db), &txn)
        .brute_force_search_v(&data.query, data.k as usize)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "filtered".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            filtered.clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}
