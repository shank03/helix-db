use helixdb::helix_engine::{
    storage_core::storage_core::EmbeddingModel,
    types::GraphError,
};
use reqwest::Client;
use serde_json::json;
use std::env;
use async_trait;

pub struct OpenAI {
    api_key: String,
    client: Client,
    model: String,
}

pub struct LocalEmbeddingModel {
    url: String,
    client: Client,
}

impl OpenAI {
    pub fn new(api_key: Option<&str>, model: Option<&str>) -> Self {
        let key = api_key
            .map(String::from)
            .unwrap_or_else(|| {
                env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set")
        });
        OpenAI {
            api_key: key,
            client: Client::new(),
            model: model.map(String::from).unwrap_or("text-embedding-ada-002".into()),
        }
    }
}

#[async_trait::async_trait]
impl EmbeddingModel for OpenAI {
    async fn fetch_embedding(&self, text: &str) -> Result<Vec<f64>, GraphError> {
        let response = self.client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "input": text,
                "model": &self.model,
            }))
            .send()
            .await
            .expect("Failed to send request")
            .json::<serde_json::Value>()
            .await
            .expect("Failed to parse response");

        Ok(response["data"][0]["embedding"]
            .as_array()
            .expect("Invalid embedding format")
            .iter()
            .map(|v| v.as_f64().expect("Invalid float value"))
            .collect()
        )
    }
}

impl LocalEmbeddingModel {
    pub fn new(url: Option<&str>) -> Self {
        LocalEmbeddingModel {
            url: url.map(String::from).unwrap_or("http://localhost:8699/embed".into()),
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl EmbeddingModel for LocalEmbeddingModel {
    async fn fetch_embedding(&self, text: &str) -> Result<Vec<f64>, GraphError> {
        let response = self
            .client
            .post(&self.url)
            .json(&json!({
                "text": text,
                "chunk_style": "recursive",
                "chunk_size": 100
            }))
            .send()
            .await
            .map_err(|e| GraphError::from(format!("Request failed: {}", e)))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| GraphError::from(format!("Failed to parse response: {}", e)))?;

        let embeddings = response["embeddings"]
            .as_array()
            .ok_or_else(|| GraphError::from("Invalid embedding format"))?
            .iter()
            .map(|v| {
                v.as_array()
                    .ok_or_else(|| GraphError::from("Invalid array format"))?
                    .iter()
                    .map(|n| n.as_f64().ok_or_else(|| GraphError::from("Invalid float value")))
                    .collect::<Result<Vec<f64>, _>>()
            })
            .collect::<Result<Vec<Vec<f64>>, _>>()?;

        Ok(embeddings.get(0).cloned().unwrap_or_default())
    }
}

