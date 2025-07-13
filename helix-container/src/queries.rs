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

pub struct Chapter {
    pub chapter_index: i64,
}

pub struct SubChapter {
    pub title: String,
    pub content: String,
}

pub struct Contains {
    pub from: Chapter,
    pub to: SubChapter,
}

pub struct EmbeddingOf {
    pub from: SubChapter,
    pub to: Embedding,
    pub chunk: String,
}

pub struct Embedding {
    pub chunk: String,
}

#[derive(Serialize, Deserialize)]
pub struct searchdocs_ragInput {
    pub query: Vec<f64>,
    pub k: i32,
}
#[handler]
pub fn searchdocs_rag(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: searchdocs_ragInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().unwrap();
    let vecs = G::new(Arc::clone(&db), &txn)
        .search_v::<fn(&HVector, &RoTxn) -> bool>(&data.query, data.k as usize, None)
        .collect_to::<Vec<_>>();
    let subchapters = G::new_from(Arc::clone(&db), &txn, vecs.clone())
        .in_("EmbeddingOf", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert("subchapters".to_string(), ReturnValue::from_traversal_value_array_with_mixin(G::new_from(Arc::clone(&db), &txn, subchapters.clone())

.map_traversal(|item, txn| { traversal_remapping!(remapping_vals, item.clone(), "title" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("title").collect_to::<Vec<_>>())?;
traversal_remapping!(remapping_vals, item.clone(), "content" => G::new_from(Arc::clone(&db), &txn, vec![item.clone()])

.check_property("content").collect_to::<Vec<_>>())?;
 Ok(item) }).collect_to::<Vec<_>>().clone(), remapping_vals.borrow_mut()));

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct chunksData {
    pub vector: Vec<f64>,
    pub chunk: String,
}
#[derive(Serialize, Deserialize)]
pub struct subchaptersData {
    pub chunks: Vec<chunksData>,
    pub title: String,
    pub content: String,
}
#[derive(Serialize, Deserialize)]
pub struct chaptersData {
    pub id: i64,
    pub subchapters: Vec<subchaptersData>,
}
#[derive(Serialize, Deserialize)]
pub struct loaddocs_ragInput {
    pub chapters: Vec<chaptersData>,
}
#[handler]
pub fn loaddocs_rag(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: loaddocs_ragInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();
    for chaptersData { id, subchapters } in data.chapters {
        let chapter_node = G::new_mut(Arc::clone(&db), &mut txn)
            .add_n("Chapter", Some(props! { "chapter_index" => &id }), None)
            .collect_to_obj();
        for subchaptersData {
            title,
            content,
            chunks,
        } in subchapters
        {
            let subchapter_node = G::new_mut(Arc::clone(&db), &mut txn)
                .add_n(
                    "SubChapter",
                    Some(props! { "title" => &title, "content" => &content }),
                    None,
                )
                .collect_to_obj();
            G::new_mut(Arc::clone(&db), &mut txn)
                .add_e(
                    "Contains",
                    None,
                    chapter_node.id(),
                    subchapter_node.id(),
                    true,
                    EdgeType::Node,
                )
                .collect_to_obj();
            for chunksData { chunk, vector } in chunks {
                let vec = G::new_mut(Arc::clone(&db), &mut txn)
                    .insert_v::<fn(&HVector, &RoTxn) -> bool>(&vector, "Embedding", None)
                    .collect_to_obj();
                G::new_mut(Arc::clone(&db), &mut txn)
                    .add_e(
                        "EmbeddingOf",
                        Some(props! { "chunk" => chunk.clone() }),
                        subchapter_node.id(),
                        vec.id(),
                        true,
                        EdgeType::Node,
                    )
                    .collect_to_obj();
            }
        }
    }
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "Success".to_string(),
        ReturnValue::from(Value::from("Success")),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct edge_nodeInput {
    pub id: ID,
}
#[handler]
pub fn edge_node(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: edge_nodeInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().unwrap();
    let e = G::new(Arc::clone(&db), &txn)
        .n_from_type("Chapter")
        .out_e("Contains")
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "e".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            e.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}
