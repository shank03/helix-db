use crate::helix_engine::types::GraphError;
use reqwest::blocking::Client;
use serde_json::json;
#[cfg(feature = "embed_local")]
use url::Url;
#[cfg(feature = "embed_openai")]
use std::env;

// TODO: add support for rust native embedding model libs as well so it runs fully built in
//      in case we have a gpu or something on the server we're running it on

/// Trait for embedding models to fetch text embeddings.
pub trait EmbeddingModel {
    fn fetch_embedding(&self, text: &str) -> Result<Vec<f64>, GraphError>;
}

#[cfg(feature = "embed_openai")]
struct EmbeddingModelImpl {
    api_key: String,
    client: Client,
    model: String,
}

/// Embed func via OpenAI, need OPENAI_API_KEY
#[cfg(feature = "embed_openai")]
impl EmbeddingModelImpl {
    fn new(api_key: Option<&str>, model: Option<&str>) -> Result<Self, GraphError> {
        let key = api_key
            .map(String::from)
            .unwrap_or_else(|| env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"));
        Ok(EmbeddingModelImpl {
            api_key: key,
            client: Client::new(),
            model: model
                .map(String::from)
                .unwrap_or("text-embedding-ada-002".into()),
        })
    }
}

#[cfg(feature = "embed_openai")]
impl EmbeddingModel for EmbeddingModelImpl {
    fn fetch_embedding(&self, text: &str) -> Result<Vec<f64>, GraphError> {
        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "input": text,
                "model": &self.model,
            }))
            .send()
            .map_err(|e| GraphError::from(format!("Failed to send request: {}", e)))?
            .json::<serde_json::Value>()
            .map_err(|e| GraphError::from(format!("Failed to parse response: {}", e)))?;

        let embedding = response["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| GraphError::from("Invalid embedding format"))?
            .iter()
            .map(|v| {
                v.as_f64()
                    .ok_or_else(|| GraphError::from("Invalid float value"))
            })
            .collect::<Result<Vec<f64>, GraphError>>()?;

        Ok(embedding)
    }
}

#[cfg(feature = "embed_local")]
struct EmbeddingModelImpl {
    url: String,
    client: Client,
}

#[cfg(feature = "embed_local")]
impl EmbeddingModelImpl {
    fn new(url: Option<&str>) -> Result<Self, GraphError> {
        let url_str = url
            .map(String::from)
            .unwrap_or("http://localhost:8699/embed".into());
        Url::parse(&url_str)
            .map_err(|e| GraphError::from(format!("Invalid URL: {}", e)))?;
        Ok(EmbeddingModelImpl {
            url: url_str,
            client: Client::new(),
        })
    }
}

/// Embed local is meant to be used with `helix-py/apps/texttovec.py`
#[cfg(feature = "embed_local")]
impl EmbeddingModel for EmbeddingModelImpl {
    fn fetch_embedding(&self, text: &str) -> Result<Vec<f64>, GraphError> {
        let response = self
            .client
            .post(&self.url)
            .json(&json!({
                "text": text,
                "chunk_style": "recursive",
                "chunk_size": 100
            }))
            .send()
            .map_err(|e| GraphError::from(format!("Request failed: {}", e)))?
            .json::<serde_json::Value>()
            .map_err(|e| GraphError::from(format!("Failed to parse response: {}", e)))?;

        let embedding = response["embedding"]
            .as_array()
            .ok_or_else(|| GraphError::from("Invalid embedding format"))?
            .iter()
            .map(|v| {
                v.as_f64()
                    .ok_or_else(|| GraphError::from("Invalid float value"))
            })
            .collect::<Result<Vec<f64>, GraphError>>()?;

        Ok(embedding)
    }
}

/// Creates embedding based on provider.
pub fn get_embedding_model(
    api_key: Option<&str>,
    model: Option<&str>,
    url: Option<&str>,
) -> Result<Box<dyn EmbeddingModel>, GraphError> {
    #[cfg(feature = "embed_openai")]
    return Ok(Box::new(EmbeddingModelImpl::new(api_key, model)?));

    #[cfg(feature = "embed_local")]
    return Ok(Box::new(EmbeddingModelImpl::new(url)?));

    #[cfg(not(any(feature = "embed_openai", feature = "embed_local")))]
    panic!("No embedding model feature enabled. Enable either 'openai' or 'local'.");
}

#[macro_export]
/// Fetches an embedding from the embedding model.
///
/// If no model or url is provided, it will use the default model and url.
///
/// ## Example Use
/// ```rust
/// let query = embed!("Hello, world!");
/// let embedding = embed!("Hello, world!", "text-embedding-ada-002");
/// let embedding = embed!("Hello, world!", "text-embedding-ada-002", "http://localhost:8699/embed");
/// ```
macro_rules! embed {
    ($query:expr) => {{
        let embedding_model = get_embedding_model(None, None, None);
        embedding_model.fetch_embedding($query)?
    }};
    ($query:expr, $model:expr) => {{
        let embedding_model = get_embedding_model(None, Some($model), None);
        embedding_model.fetch_embedding($query)?
    }};
    ($query:expr, $model:expr, $url:expr) => {{
        let embedding_model = get_embedding_model(None, Some($model), Some($url));
        embedding_model.fetch_embedding($query)?
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "embed_openai")]
    #[test]
    fn test_openai_embedding_success() {
        let model = get_embedding_model(None, None, None).unwrap();
        let result = model.fetch_embedding("test text");
        assert!(result.is_ok());
        let embedding = result.unwrap();
        println!("embedding: {:?}", embedding);
    }

    #[cfg(feature = "embed_local")]
    #[test]
    fn test_local_embedding_success() {
        let model = get_embedding_model(None, None, None).unwrap();
        let result = model.fetch_embedding("test text");
        assert!(result.is_ok());
        let embedding = result.unwrap();
        println!("embedding: {:?}", embedding);
    }

    #[cfg(feature = "embed_local")]
    #[test]
    fn test_local_embedding_invalid_url() {
        let model = get_embedding_model(None, None, Some("invalid_url"));
        assert!(model.is_err());
    }
}

