use axum::response::IntoResponse;
use reqwest::header::CONTENT_TYPE;

use crate::protocol::Format;
#[derive(Debug)]
pub struct Response {
    pub body: Vec<u8>,
    pub fmt: Format,
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .header(CONTENT_TYPE, self.fmt.to_string())
            .body(axum::body::Body::from(self.body))
            .expect("Should be able to construct response")
    }
}
