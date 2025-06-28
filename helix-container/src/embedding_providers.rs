use helixdb::{
    helix_engine::{
        storage_core::storage_core::EmbeddingModel,
        types::GraphError,
        graph_core::ops::{
            g::G,
            //vectors::insert,
            vectors::{insert::InsertVAdapter, search::SearchVAdapter, brute_force_search::BruteForceSearchVAdapter},
        },
        vector_core::vector::HVector,
    },
    protocol::{
        return_values::ReturnValue,
        remapping::RemappingMap,
        response::Response,
    },
    helix_gateway::router::router::HandlerInput,
};
use sonic_rs::{Deserialize, Serialize};
use reqwest::blocking::Client;
use serde_json::json;
use async_trait;
use get_routes::handler;
use heed3::RoTxn;
use std::{
    env,
    sync::Arc,
    collections::HashMap,
};

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
            .map(|v| v.as_f64().ok_or_else(|| GraphError::from("Invalid float value")))
            .collect::<Result<Vec<f64>, GraphError>>()?;

        Ok(embedding)
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
            .map(|v| v.as_f64().ok_or_else(|| GraphError::from("Invalid float value")))
            .collect::<Result<Vec<f64>, GraphError>>()?;

        Ok(embedding)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DocInput {
    pub doc: String,
}

#[handler]
pub fn insert_doc(input: &HandlerInput, response: &mut Response) -> Result<(), GraphError> {
    let db = Arc::clone(&input.graph.storage);
    let mut txn = db.graph_env.write_txn().unwrap();

    let data: DocInput = match sonic_rs::from_slice(&input.request.body) {
        Ok(data) => data,
        Err(err) => return Err(GraphError::from(err)),
    };

    let lem: LocalEmbeddingModel = LocalEmbeddingModel::new(None);

    let single_embedding = lem.fetch_embedding(&data.doc)?;

    let vector = G::new_mut(Arc::clone(&db), &mut txn)
        .insert_v::<fn(&HVector, &RoTxn) -> bool>(&single_embedding, "Embedding", None).collect_to::<Vec<_>>();

    println!("vec: {:?}", vector);

    txn.commit().unwrap();
    response.body = sonic_rs::to_vec(&"Success").unwrap();
    Ok(())
}

