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

pub struct Cluster {
    pub region: String,
    pub api_url: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Instance {
    pub region: String,
    pub instance_type: String,
    pub storage_gb: i64,
    pub ram_gb: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct User {
    pub gh_id: u64,
    pub gh_login: String,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CreatedCluster {
    pub from: User,
    pub to: Cluster,
}

pub struct CreatedInstance {
    pub from: Cluster,
    pub to: Instance,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateClusterStatusInput {
    pub cluster_id: ID,
    pub status: String,
}
#[handler]
pub fn UpdateClusterStatus(
    input: &HandlerInput,
    response: &mut Response,
) -> Result<(), GraphError> {
    let data: UpdateClusterStatusInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().unwrap();
    let clusters = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.cluster_id)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "clusters".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            clusters.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserInput {
    pub gh_id: u64,
    pub gh_login: String,
    pub name: String,
    pub email: String,
}
#[handler]
pub fn CreateUser(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: CreateUserInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();
    let user = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("User", Some(props! { "name" => &data.name, "updated_at" => chrono::Utc::now().to_rfc3339(), "gh_id" => &data.gh_id, "gh_login" => &data.gh_login, "email" => &data.email, "created_at" => chrono::Utc::now().to_rfc3339() }), Some(&["gh_id"])).collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            user.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct GetInstancesForUserInput {
    pub user_id: ID,
}
#[handler]
pub fn GetInstancesForUser(
    input: &HandlerInput,
    response: &mut Response,
) -> Result<(), GraphError> {
    let data: GetInstancesForUserInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().unwrap();
    let instances = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .out("CreatedCluster", &EdgeType::Node)
        .out("CreatedInstance", &EdgeType::Node)
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "instances".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            instances.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct UpdateClusterApiUrlInput {
    pub cluster_id: ID,
    pub api_url: String,
}
#[handler]
pub fn UpdateClusterApiUrl(
    input: &HandlerInput,
    response: &mut Response,
) -> Result<(), GraphError> {
    let data: UpdateClusterApiUrlInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();
    let clusters = {
        let update_tr = G::new(Arc::clone(&db), &txn)
            .n_from_id(&data.cluster_id)
            .collect_to::<Vec<_>>();
        G::new_mut_from(Arc::clone(&db), &mut txn, update_tr)
            .update(Some(props! { "api_url" => &data.api_url }))
            .collect_to_obj()
    };
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "clusters".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            clusters.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct CreateClusterInput {
    pub user_id: ID,
    pub region: String,
    pub instance_type: String,
    pub storage_gb: i64,
    pub ram_gb: i64,
}
#[handler]
pub fn CreateCluster(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: CreateClusterInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_id(&data.user_id)
        .collect_to_obj();
    let new_cluster = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Cluster", Some(props! { "updated_at" => chrono::Utc::now().to_rfc3339(), "status" => "pending", "region" => &data.region, "api_url" => "", "created_at" => chrono::Utc::now().to_rfc3339() }), None).collect_to_obj();
    let new_instance = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Instance", Some(props! { "created_at" => chrono::Utc::now().to_rfc3339(), "updated_at" => chrono::Utc::now().to_rfc3339(), "instance_type" => &data.instance_type, "ram_gb" => &data.ram_gb, "region" => &data.region, "storage_gb" => &data.storage_gb }), None).collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "CreatedCluster",
            None,
            user.id(),
            new_cluster.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "CreatedInstance",
            None,
            new_cluster.id(),
            new_instance.id(),
            true,
            EdgeType::Node,
        )
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "new_cluster".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            new_cluster.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct LookupUserInput {
    pub gh_id: u64,
}
#[handler]
pub fn LookupUser(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let data: LookupUserInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let mut remapping_vals = RemappingMap::new();
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().unwrap();
    let user = G::new(Arc::clone(&db), &txn)
        .n_from_index("gh_id", &data.gh_id)
        .collect_to_obj();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user".to_string(),
        ReturnValue::from_traversal_value_with_mixin(
            user.clone().clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}
