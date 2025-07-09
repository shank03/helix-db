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

pub struct Cluster {
    pub region: String,
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
            instances.clone(),
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
.add_n("User", Some(props! { "gh_id" => data.gh_id.clone(), "gh_login" => data.gh_login.clone(), "email" => data.email.clone(), "created_at" => chrono::Utc::now().to_rfc3339(), "updated_at" => chrono::Utc::now().to_rfc3339(), "name" => data.name.clone() }), Some(&["gh_id"])).collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            user.clone(),
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
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "user".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            user.clone(),
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
        .collect_to::<Vec<_>>();
    let new_cluster = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Cluster", Some(props! { "created_at" => chrono::Utc::now().to_rfc3339(), "region" => data.region.clone(), "updated_at" => chrono::Utc::now().to_rfc3339() }), None).collect_to::<Vec<_>>();
    let new_instance = G::new_mut(Arc::clone(&db), &mut txn)
.add_n("Instance", Some(props! { "region" => data.region.clone(), "ram_gb" => data.ram_gb.clone(), "created_at" => chrono::Utc::now().to_rfc3339(), "storage_gb" => data.storage_gb.clone(), "updated_at" => chrono::Utc::now().to_rfc3339(), "instance_type" => data.instance_type.clone() }), None).collect_to::<Vec<_>>();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "CreatedCluster",
            None,
            user.id(),
            new_cluster.id(),
            true,
            EdgeType::Node,
        )
        .collect_to::<Vec<_>>();
    G::new_mut(Arc::clone(&db), &mut txn)
        .add_e(
            "CreatedInstance",
            None,
            new_cluster.id(),
            new_instance.id(),
            true,
            EdgeType::Node,
        )
        .collect_to::<Vec<_>>();
    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "new_cluster".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            new_cluster.clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&return_vals).unwrap();
    Ok(())
}
