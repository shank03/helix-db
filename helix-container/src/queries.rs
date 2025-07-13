use chrono::{DateTime, Utc};
use heed3::RoTxn;
use helixdb::{
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
        router::router::HandlerInput,
    },
    identifier_remapping, node_matches, props,
    protocol::{
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
use proc_macros::handler;
use sonic_rs::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

pub struct File15 {
    pub name: String,
    pub age: i32,
}

pub struct Follows {
    pub from: File15,
    pub to: File15,
}

#[derive(Serialize, Deserialize)]
pub struct file15_3Input {
    pub userID: ID,
}
#[handler]
pub fn file15_3(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: file15_3Input = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.userID)
            .collect_to_obj(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[handler]
pub fn file15(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_type("File15")
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "success".to_string(),
        ReturnValue::from(Value::from("success")),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct file15_2Input {
    pub userID: ID,
}
#[handler]
pub fn file15_2(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: file15_2Input = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();
    Drop::<Vec<_>>::drop_traversal(
        G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.userID)
            .out("Follows", &EdgeType::Node)
            .collect_to::<Vec<_>>(),
        Arc::clone(&db),
        &mut txn,
    )?;
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}
