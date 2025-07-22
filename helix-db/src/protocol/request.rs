use axum::{body::Bytes, extract::FromRequest};
use reqwest::{
    StatusCode,
    header::{ACCEPT, CONTENT_TYPE},
};
use tokio::sync::oneshot;
use tracing::info;

use crate::protocol::{Format, HelixError, Response};

pub type ReqMsg = (Request, oneshot::Sender<Result<Response, HelixError>>);

#[derive(Debug)]
pub struct Request {
    pub name: String,
    pub req_type: RequestType,
    /// This contains the input parameters serialized with in_fmt
    pub body: Bytes,
    pub in_fmt: Format,
    pub out_fmt: Format,
}

#[derive(Debug)]
pub enum RequestType {
    Query,
    MCP,
}

impl<S> FromRequest<S> for Request
where
    S: Send + Sync,
{
    #[doc = " If the extractor fails it\'ll use this \"rejection\" type. A rejection is"]
    #[doc = " a kind of error that can be converted into a response."]
    type Rejection = StatusCode;

    #[doc = " Perform the extraction."]
    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let path = req.uri().path();

        let (name, req_type) = match path.strip_prefix("/mcp/") {
            Some(n) => (n.to_string(), RequestType::MCP),
            None => (
                path.strip_prefix('/')
                    .expect("paths should start with a '/'")
                    .to_string(),
                RequestType::Query,
            ),
        };

        if name.contains('/') || name.is_empty() {
            // TODO: improve errors
            return Err(StatusCode::BAD_REQUEST);
        }

        let headers = req.headers();
        let in_fmt = match headers.get(CONTENT_TYPE) {
            Some(v) => match v.to_str() {
                Ok(s) => s.parse().map_err(|_| StatusCode::UNSUPPORTED_MEDIA_TYPE)?,
                Err(_) => return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE),
            },
            None => Format::default(),
        };

        let out_fmt = match headers.get(ACCEPT) {
            Some(v) => match v.to_str() {
                Ok(s) => s.parse().unwrap_or_default(),
                Err(_) => return Err(StatusCode::BAD_REQUEST),
            },
            None => Format::default(),
        };

        let body = Bytes::from_request(req, state).await.expect("todo");
        let out = Request {
            name,
            req_type,
            body,
            in_fmt,
            out_fmt,
        };

        Ok(out)
    }
}

// impl Request {
//     /// Parse a request from a stream
//     ///
//     /// # Example
//     ///
//     /// ```rust
//     /// use std::io::Cursor;
//     /// use helix_db::protocol::request::Request;
//     ///
//     /// let request = Request::from_stream(Cursor::new("GET /test HTTP/1.1\r\n\r\n")).unwrap();
//     /// assert_eq!(request.method, "GET");
//     /// assert_eq!(request.path, "/test");
//     /// ```
//     pub async fn from_stream<R: AsyncRead + Unpin>(stream: &mut R) -> Result<Request> {
//         let mut reader = BufReader::new(stream);
//         let mut first_line = String::new();
//         reader.read_line(&mut first_line).await?;

//         // Get method and path
//         let mut parts = first_line.trim().split_whitespace();
//         let method = parts
//             .next()
//             .ok_or_else(|| {
//                 std::io::Error::new(
//                     std::io::ErrorKind::InvalidData,
//                     format!("Missing HTTP method: {}", first_line),
//                 )
//             })?
//             .to_string();
//         let path = parts
//             .next()
//             .ok_or_else(|| {
//                 std::io::Error::new(
//                     std::io::ErrorKind::InvalidData,
//                     format!("Missing path: {}", first_line),
//                 )
//             })?
//             .to_string();

//         // Parse headers
//         let mut headers = HashMap::new();
//         let mut line = String::new();
//         loop {
//             line.clear();
//             let bytes_read = reader.read_line(&mut line).await?;
//             if bytes_read == 0 || line.eq("\r\n") || line.eq("\n") {
//                 break;
//             }
//             if let Some((key, value)) = line.trim().split_once(':') {
//                 headers.insert(key.trim().to_lowercase(), value.trim().to_string());
//             }
//         }

//         // Read body
//         let mut body = Vec::new();
//         if let Some(length) = headers.get("content-length") {
//             if let Ok(length) = length.parse::<usize>() {
//                 let mut buffer = vec![0; length];
//                 match tokio::time::timeout(
//                     std::time::Duration::from_secs(5),
//                     reader.read_exact(&mut buffer),
//                 )
//                 .await
//                 {
//                     Ok(Ok(_)) => body = buffer,
//                     Ok(Err(e)) => {
//                         eprintln!("Error reading body: {}", e);
//                         return Err(std::io::Error::new(
//                             std::io::ErrorKind::Other,
//                             "Error reading body",
//                         ));
//                     }
//                     Err(_) => {
//                         eprintln!("Timeout reading body");
//                         return Err(std::io::Error::new(
//                             std::io::ErrorKind::TimedOut,
//                             "Timeout reading body",
//                         ));
//                     }
//                 }
//             }
//         }

//         Ok(Request {
//             method,
//             headers,
//             path,
//             body,
//         })
//     }
// }
