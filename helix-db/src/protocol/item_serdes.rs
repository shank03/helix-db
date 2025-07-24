use std::borrow::Cow;

use serde::{Serialize, de::DeserializeOwned};

use crate::{
    helix_engine::types::GraphError,
    utils::items::{Edge, Node},
};

pub trait ItemSerdes: Sized + Serialize + DeserializeOwned {
    fn encode<'a>(self) -> Result<Cow<'a, [u8]>, GraphError>;
    fn decode(data: &[u8], id: u128) -> Result<Self, GraphError>;
}

impl ItemSerdes for Node {
    fn encode<'a>(self) -> Result<Cow<'a, [u8]>, GraphError> {
        Ok(Cow::Owned(bincode::serialize(&self).map_err(|e| {
            GraphError::ConversionError(format!("Error serializing node: {}", e))
        })?))
    }

    /// Decodes a node from a byte slice.
    ///
    /// Takes ID as the ID is not serialized when stored as it is the key.
    /// Uses the known ID (either from the query or the key in an LMDB iterator) to construct a new node
    fn decode(data: &[u8], id: u128) -> Result<Self, GraphError> {
        match bincode::deserialize::<Node>(data) {
            Ok(node) => Ok(Node {
                id,
                label: node.label,
                properties: node.properties,
            }),
            Err(e) => Err(GraphError::ConversionError(format!(
                "Error deserializing node: {}",
                e
            ))),
        }
    }
}

impl ItemSerdes for Edge {
    fn encode<'a>(self) -> Result<Cow<'a, [u8]>, GraphError> {
        Ok(Cow::Owned(bincode::serialize(&self).map_err(|e| {
            GraphError::ConversionError(format!("Error serializing edge: {}", e))
        })?))
    }

    fn decode(data: &[u8], id: u128) -> Result<Self, GraphError> {
        match bincode::deserialize::<Edge>(data) {
            Ok(edge) => Ok(Edge {
                id,
                label: edge.label,
                from_node: edge.from_node,
                to_node: edge.to_node,
                properties: edge.properties,
            }),
            Err(e) => Err(GraphError::ConversionError(format!(
                "Error deserializing edge: {}",
                e
            ))),
        }
    }
}
