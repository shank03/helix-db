use crate::helix_engine::storage_core::storage_core::HelixGraphStorage;
use crate::helix_engine::types::GraphError;
use crate::helix_gateway::mcp::mcp::{McpBackend, McpConnections};
use std::sync::{Arc, Mutex};
use crate::helix_engine::graph_core::config::Config;

#[derive(Debug)]
pub enum QueryInput {
    StringValue { value: String },
    IntegerValue { value: i32 },
    FloatValue { value: f64 },
    BooleanValue { value: bool },
}

pub struct HelixGraphEngine {
    pub storage: Arc<HelixGraphStorage>,
    pub mcp_backend: Option<Arc<McpBackend>>,
    pub mcp_connections: Option<Arc<Mutex<McpConnections>>>,
}

pub struct HelixGraphEngineOpts {
    pub path: String,
    pub config: Config,
}

impl HelixGraphEngineOpts {
    pub fn default() -> Self {
        Self {
            path: String::new(),
            config: Config::default(),
        }
    }
}

impl HelixGraphEngine {
    pub fn new(opts: HelixGraphEngineOpts) -> Result<HelixGraphEngine, GraphError> {
        let should_use_mcp = opts.config.mcp;
        let storage = match HelixGraphStorage::new(opts.path.as_str(), opts.config) {
            Ok(db) => Arc::new(db),
            Err(err) => return Err(err),
        };

        let (mcp_backend, mcp_connections) = if should_use_mcp.unwrap_or(false) {
            let mcp_backend = Arc::new(McpBackend::new(storage.clone()));
            let mcp_connections = Arc::new(Mutex::new(McpConnections::new()));
            (Some(mcp_backend), Some(mcp_connections))
        } else {
            (None, None)
        };

        Ok(Self {
            storage,
            mcp_backend,
            mcp_connections,
        })
    }
}

