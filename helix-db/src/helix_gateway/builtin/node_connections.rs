use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use sonic_rs::{JsonValueTrait, json};
use tracing::info;

use crate::helix_engine::graph_core::ops::tr_val::TraversalVal;
use crate::helix_engine::storage_core::storage_core::HelixGraphStorage;
use crate::helix_engine::storage_core::storage_methods::StorageMethods;
use crate::helix_engine::types::GraphError;
use crate::helix_gateway::gateway::AppState;
use crate::helix_gateway::router::router::{Handler, HandlerInput, HandlerSubmission};
use crate::protocol::remapping::RemappingMap;
use crate::protocol::return_values::ReturnValue;
use crate::protocol::{self, request::RequestType};

// get all nodes connected to a specific node
// curl "http://localhost:PORT/node-connections?node_id=YOUR_NODE_ID"

#[derive(Deserialize)]
pub struct NodeConnectionsQuery {
    node_id: String,
}

#[derive(Serialize)]
pub struct NodeConnectionsResponse {
    pub connected_nodes: Vec<ReturnValue>,
    pub incoming_edges: Vec<ReturnValue>,
    pub outgoing_edges: Vec<ReturnValue>,
}

pub async fn node_connections_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<NodeConnectionsQuery>,
) -> axum::http::Response<Body> {
    let mut req = protocol::request::Request {
        name: "node_connections".to_string(),
        req_type: RequestType::Query,
        body: axum::body::Bytes::new(),
        in_fmt: protocol::Format::default(),
        out_fmt: protocol::Format::default(),
    };

    if let Ok(params_json) = sonic_rs::to_vec(&json!({
        "node_id": params.node_id
    })) {
        req.body = axum::body::Bytes::from(params_json);
    }

    let res = state.worker_pool.process(req).await;

    match res {
        Ok(r) => r.into_response(),
        Err(e) => {
            info!(?e, "Got error");
            e.into_response()
        }
    }
}

pub fn node_connections_inner(input: &HandlerInput) -> Result<protocol::Response, GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let txn = db.graph_env.read_txn().map_err(GraphError::from)?;

    let node_id_str = if !input.request.body.is_empty() {
        match sonic_rs::from_slice::<sonic_rs::Value>(&input.request.body) {
            Ok(params) => params
                .get("node_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Err(_) => None,
        }
    } else {
        None
    };

    let node_id_str =
        node_id_str.ok_or_else(|| GraphError::New("node_id is required".to_string()))?;

    let node_id = if let Ok(uuid) = uuid::Uuid::parse_str(&node_id_str) {
        uuid.as_u128()
    } else if let Ok(num) = node_id_str.parse::<u128>() {
        num
    } else {
        return Err(GraphError::New(
            "Invalid node_id format - must be UUID or u128".to_string(),
        ));
    };

    let remapping_vals = RemappingMap::new();

    if db.get_node(&txn, &node_id).is_err() {
        return Err(GraphError::New("Node not found".to_string()));
    }

    let mut connected_node_ids = HashSet::new();
    let mut connected_nodes = Vec::new();

    let incoming_edges = db
        .in_edges_db
        .prefix_iter(&txn, &node_id.to_be_bytes())?
        .map(|result| {
            let (_, value) = result?;
            let (edge_id, from_node) = HelixGraphStorage::unpack_adj_edge_data(value)?;

            if connected_node_ids.insert(from_node) {
                let node = db.get_node(&txn, &from_node)?;
                connected_nodes.push(TraversalVal::Node(node));
            }

            let edge = db.get_edge(&txn, &edge_id)?;
            Ok(TraversalVal::Edge(edge))
        })
        .collect::<Result<Vec<_>, GraphError>>()?;

    let outgoing_edges = db
        .out_edges_db
        .prefix_iter(&txn, &node_id.to_be_bytes())?
        .map(|result| {
            let (_, value) = result?;
            let (edge_id, to_node) = HelixGraphStorage::unpack_adj_edge_data(value)?;

            if connected_node_ids.insert(to_node) {
                let node = db.get_node(&txn, &to_node)?;
                connected_nodes.push(TraversalVal::Node(node));
            }

            let edge = db.get_edge(&txn, &edge_id)?;
            Ok(TraversalVal::Edge(edge))
        })
        .collect::<Result<Vec<_>, GraphError>>()?;

    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();

    return_vals.insert(
        "connected_nodes".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            connected_nodes,
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "incoming_edges".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            incoming_edges,
            remapping_vals.borrow_mut(),
        ),
    );

    return_vals.insert(
        "outgoing_edges".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            outgoing_edges,
            remapping_vals.borrow_mut(),
        ),
    );

    Ok(protocol::Response {
        body: sonic_rs::to_vec(&return_vals).map_err(|e| GraphError::New(e.to_string()))?,
        fmt: Default::default(),
    })
}

inventory::submit! {
    HandlerSubmission(
        Handler::new("node_connections", node_connections_inner)
    )
}
