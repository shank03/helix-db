use crate::helix_engine::types::GraphError;
use std::{
    path::PathBuf,
    fmt,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VectorConfig {
    pub m: Option<usize>,
    pub ef_construction: Option<usize>,
    pub ef_search: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GraphConfig {
    pub secondary_indices: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub vector_config: VectorConfig,
    pub graph_config: GraphConfig,
    pub db_max_size_gb: Option<usize>,
    pub mcp: bool,
    pub bm25: bool,
    pub schema: Option<String>,
    pub embedding_model: Option<String>,
    pub graphvis_node_label: Option<String>,
}

impl Config {
    pub fn new(
        m: usize,
        ef_construction: usize,
        ef_search: usize,
        db_max_size_gb: usize,
        mcp: bool,
        bm25: bool,
        schema: Option<String>,
        embedding_model: Option<String>,
        graphvis_node_label: Option<String>,
    ) -> Self {
        Self {
            vector_config: VectorConfig {
                m: Some(m),
                ef_construction: Some(ef_construction),
                ef_search: Some(ef_search),
            },
            graph_config: GraphConfig {
                secondary_indices: None,
            },
            db_max_size_gb: Some(db_max_size_gb),
            mcp,
            bm25,
            schema,
            embedding_model,
            graphvis_node_label,
        }
    }

    pub fn from_files(config_path: PathBuf, schema_path:PathBuf) -> Result<Self, GraphError> {
        if !config_path.exists() {
            println!("no config path!");
            return Err(GraphError::ConfigFileNotFound);
        }

        let config = std::fs::read_to_string(config_path)?;
        let mut config = sonic_rs::from_str::<Config>(&config)?;

        if schema_path.exists() {
            let schema_string = std::fs::read_to_string(schema_path)?;
            config.schema = Some(schema_string);
        } else {
            config.schema = None;
        }

        Ok(config)
    }

    pub fn init_config() -> String {
    r#"
    {
        "vector_config": {
            "m": 16,
            "ef_construction": 128,
            "ef_search": 768
        },
        "graph_config": {
            "secondary_indices": []
        },
        "db_max_size_gb": 10,
        "mcp": true,
        "bm25": true,
        "embedding_model": "text-embedding-ada-002",
        "graphvis_node_label": ""
    }
    "#
    .to_string()
    }

    pub fn to_json(&self) -> String {
        sonic_rs::to_string_pretty(self).unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vector_config: VectorConfig {
                m: Some(16),
                ef_construction: Some(128),
                ef_search: Some(768),
            },
            graph_config: GraphConfig {
                secondary_indices: None,
            },
            db_max_size_gb: Some(10),
            mcp: true,
            bm25: true,
            schema: None,
            embedding_model: Some("text-embedding-ada-002".to_string()),
            graphvis_node_label: None,
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "pub fn config() -> Option<Config> {{"
        )?;
        write!(f, "return Some(Config {{")?;
        write!(f, "vector_config: VectorConfig {{")?;
        write!(f, "m: Some({}),", self.vector_config.m.unwrap_or(16))?;
        write!(f, "ef_construction: Some({}),", self.vector_config.ef_construction.unwrap_or(128))?;
        write!(f, "ef_search: Some({}),", self.vector_config.ef_search.unwrap_or(768))?;
        write!(f, "}},")?;
        write!(f, "graph_config: GraphConfig {{")?;
        write!(f, "secondary_indices: {},", match &self.graph_config.secondary_indices {
            Some(indices) => format!("Some(vec![{}])", indices.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", ")),
            None => "None".to_string(),
        })?;
        write!(f, "}},")?;
        write!(f, "db_max_size_gb: Some({}),", self.db_max_size_gb.unwrap_or(10))?;
        write!(f, "mcp: {},", self.mcp)?;
        write!(f, "bm25: {},", self.bm25)?;
        write!(f, "schema: None,")?;
        write!(f, "embedding_model: {},", match &self.embedding_model {
            Some(model) => format!("Some(\"{}\".to_string())", model),
            None => "None".to_string(),
        })?;
        write!(f, "graphvis_node_label: {},", match &self.graphvis_node_label {
            Some(label) => format!("Some(\"{}\".to_string())", label),
            None => "None".to_string(),
        })?;
        write!(f, "}})")?;
        write!(f, "}}")?;
        Ok(())
    }
}

