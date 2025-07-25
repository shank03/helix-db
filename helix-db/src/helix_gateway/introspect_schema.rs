use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;

use axum::response::IntoResponse;

use crate::helix_engine::graph_core::config::Config;
use crate::helix_engine::graph_core::graph_core::HelixGraphEngineOpts;

pub async fn introspect_schema_handler(
    State(opts): State<Arc<Option<HelixGraphEngineOpts>>>,
) -> axum::response::Response {
    match &*opts {
        Some(HelixGraphEngineOpts {
            config: Config {
                schema: Some(data), ..
            },
            ..
        }) => axum::response::Response::builder()
            .header("Content-Type", "application/json")
            .body(Body::from(data.clone().into_bytes()))
            .expect("should be able to make response from string"),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Could not find schema").into_response(),
    }
}
