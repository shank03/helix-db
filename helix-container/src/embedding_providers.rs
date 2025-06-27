use helixdb::helix_engine::storage_core::storage_core::EmbeddingModel;
use reqwest::Client;
use serde_json::json;
use std::env;
use async_trait;

pub struct OpenAI {
    api_key: String,
    client: Client,
}

impl OpenAI {
    pub fn new(api_key: Option<String>) -> Self {
        let key = api_key.unwrap_or_else(|| {
            env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set")
        });
        OpenAI {
            api_key: key,
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl EmbeddingModel for OpenAI {
    async fn fetch_embedding(&self, text: String) -> Vec<f64> {
        let response = self.client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "input": text,
                "model": "text-embedding-ada-002"
            }))
            .send()
            .await
            .expect("Failed to send request")
            .json::<serde_json::Value>()
            .await
            .expect("Failed to parse response");

        response["data"][0]["embedding"]
            .as_array()
            .expect("Invalid embedding format")
            .iter()
            .map(|v| v.as_f64().expect("Invalid float value"))
            .collect()
    }
}

