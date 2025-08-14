use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EventType {
    #[serde(rename = "cli_install")]
    CliInstall,
    #[serde(rename = "compile")]
    Compile,
    #[serde(rename = "deploy")]
    Deploy,
    #[serde(rename = "query_success")]
    QuerySuccess,
    #[serde(rename = "query_error")]
    QueryError,
    #[serde(rename = "write_error")]
    WriteError,
    #[serde(rename = "read_error")]
    ReadError,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::CliInstall => "cli_install",
            EventType::Compile => "compile",
            EventType::Deploy => "deploy",
            EventType::QuerySuccess => "query_success",
            EventType::QueryError => "query_error",
            EventType::WriteError => "write_error",
            EventType::ReadError => "read_error",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawEvent {
    pub ip_hash: Option<String>,
    pub os: String,
    pub event_type: EventType,
    pub event_data: EventData,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventData {
    CliInstall,
    Compile(CompileEvent),
    Deploy(DeployEvent),
    QuerySuccess(QuerySuccessEvent),
    QueryError(QueryErrorEvent),
    WriteError(WriteErrorEvent),
    ReadError(ReadErrorEvent),
}



#[derive(Debug, Serialize, Deserialize)]
pub struct CompileEvent {
    cluster_id: String,
    queries_string: String,
    num_of_queries: u32,
    time_taken_seconds: u32,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_messages: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployEvent {
    cluster_id: String,
    queries_string: String,
    num_of_queries: u32,
    time_taken_sec: u32,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_messages: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct WriteErrorEvent {
    cluster_id: String,
    key: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_string: Option<String>,
    value: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_string: Option<String>,
    time_taken_usec: u32,
    error_messages: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadErrorEvent {
    cluster_id: String,
    key: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_string: Option<String>,
    value: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_string: Option<String>,
    time_taken_usec: u32,
    error_messages: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryErrorEvent {
    cluster_id: String,
    query_string: String,
    input_json: String,
    output_json: String,
    time_taken_usec: u32,
    uses_embeddings: bool,
    uses_bm25: bool,
    uses_vectors: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuerySuccessEvent {
    cluster_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_json: Option<String>,
    time_taken_usec: u32,
    uses_embeddings: bool,
    uses_bm25: bool,
    uses_vectors: bool,
}
