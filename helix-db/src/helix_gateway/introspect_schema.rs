use axum::Json;
use axum::http::StatusCode;

use axum::response::IntoResponse;

use crate::helixc::analyzer::analyzer;

pub async fn introspect_schema_handler() -> axum::response::Response {
    match analyzer::INTROSPECTION_DATA.get() {
        Some(d) => Json(d).into_response(),
        None => (StatusCode::INTERNAL_SERVER_ERROR, "Could not find schema").into_response(),
    }
}
