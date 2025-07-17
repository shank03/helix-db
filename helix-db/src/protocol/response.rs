use std::{collections::HashMap, ops::Deref};
use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

use crate::protocol::{format::Format, return_values::ReturnValue};
#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub value: Option<(Format, HashMap<String, ReturnValue>)>,
}

impl Response {
    /// Create a new response
    pub fn new() -> Response {
        let mut headers = HashMap::new();
        // TODO: Change to use router config for headers and default routes
        headers.insert("Content-Type".to_string(), "text/plain".to_string());

        Response {
            status: 200,
            headers,
            body: Vec::new(),
            value: None,
        }
    }

    /// Send response back via stream
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::Cursor;
    /// use helix_db::protocol::response::Response;
    ///
    /// let mut response = Response::new();
    ///
    /// response.status = 200;
    /// response.body = b"Hello World".to_vec();
    ///
    /// let mut stream = Cursor::new(Vec::new());
    /// response.send(&mut stream).unwrap();
    ///
    /// let data = stream.into_inner();
    /// let data = String::from_utf8(data).unwrap();
    ///
    /// assert!(data.contains("HTTP/1.1 200 OK"));
    /// assert!(data.contains("Content-Length: 11"));
    /// assert!(data.contains("Hello World"));

    pub async fn send<W: AsyncWrite + Unpin>(&mut self, stream: &mut W) -> Result<()> {
        let status_message = match self.status {
            200 => "OK",
            404 => {
                self.body = b"404 - Route Not Found\n".to_vec();
                "Not Found"
            }
            500 => {
                // self.body = b"500 - Internal Server Error\n".to_vec();
                "Internal Server Error"
            }
            _ => "Unknown",
        };
        let mut writer = tokio::io::BufWriter::new(stream);

        // Write status line
        writer
            .write_all(format!("HTTP/1.1 {} {}\r\n", self.status, status_message).as_bytes())
            .await?;

        let serialized = self.value.as_ref().map(|(f, v)| f.serialize(v));

        let body = match serialized.as_ref() {
            Some(s) => {
                assert!(self.body.is_empty());
                s.deref()
            }
            None => &self.body,
        };

        // Write headers
        for (header, value) in &self.headers {
            writer
                .write_all(format!("{}: {}\r\n", header, value).as_bytes())
                .await
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Error writing header: {}", e),
                    )
                })?;
        }

        writer
            .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
            .await?;

        // Write body
        writer.write_all(body).await?;
        writer.flush().await?;
        Ok(())
    }
}
