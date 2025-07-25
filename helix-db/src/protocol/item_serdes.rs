use std::borrow::Cow;

use serde::{Serialize, de::DeserializeOwned};

use crate::{
    helix_engine::types::GraphError,
    utils::items::{DirectedEdge, Edge, Node},
};

pub trait ItemSerdes: Sized + Serialize + DeserializeOwned {
    fn encode<'a>(&self) -> Result<impl AsRef<[u8]>, GraphError>;
    fn decode(data: &[u8]) -> Result<Self, GraphError>;
    fn decode_with_id(data: &[u8], id: u128) -> Result<Self, GraphError>;
}

impl ItemSerdes for Node {
    #[allow(refining_impl_trait)]
    fn encode<'a>(&self) -> Result<Cow<'a, [u8]>, GraphError> {
        Ok(Cow::Owned(bincode::serialize(self).map_err(|e| {
            GraphError::ConversionError(format!("Error serializing node: {}", e))
        })?))
    }

    /// Decodes a node from a byte slice.
    ///
    /// Takes ID as the ID is not serialized when stored as it is the key.
    /// Uses the known ID (either from the query or the key in an LMDB iterator) to construct a new node
    fn decode_with_id(data: &[u8], id: u128) -> Result<Self, GraphError> {
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

    /// ## NOTE:
    ///
    /// **DO NOT USE THIS FOR EDGES, VECTORS OR NODES - will panic**
    fn decode(_: &[u8]) -> Result<Self, GraphError> {
        unimplemented!()
    }
}

impl ItemSerdes for Edge {
    #[allow(refining_impl_trait)]
    fn encode<'a>(&self) -> Result<Cow<'a, [u8]>, GraphError> {
        Ok(Cow::Owned(bincode::serialize(self).map_err(|e| {
            GraphError::ConversionError(format!("Error serializing edge: {}", e))
        })?))
    }

    fn decode_with_id(data: &[u8], id: u128) -> Result<Self, GraphError> {
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

    fn decode(_: &[u8]) -> Result<Self, GraphError> {
        unimplemented!()
    }
}

impl ItemSerdes for DirectedEdge {
    fn encode<'a>(&self) -> Result<impl AsRef<[u8]>, GraphError> {
        let mut buf = [0u8; 32];
        buf[..16].copy_from_slice(&self.edge_id);
        buf[16..32].copy_from_slice(&self.other_node_id);
        Ok(buf)
    }

    fn decode_with_id(_: &[u8], _: u128) -> Result<Self, GraphError> {
        unimplemented!()
    }

    fn decode(bytes: &[u8]) -> Result<Self, GraphError> {
        let edge_id = bytes[..16].try_into().unwrap();
        let other_node_id = bytes[16..32].try_into().unwrap();
        Ok(DirectedEdge {
            edge_id,
            other_node_id,
        })
    }
}

/// Decodes some bytes into a type.
///
/// If given with a `u128` ID, it will will call the `decode_with_id` method, otherwise it will call the `decode` method.
///
/// # Example
///
/// ```rust
/// use helix_db::decode;
/// use helix_db::utils::items::Node;
/// use helix_db::utils::items::DirectedEdge;
///
/// let node = Node {
///     id: 1,
///     label: "test".to_string(),
///     properties: None,
/// };
///
/// let data = encode!(node).unwrap();
/// // for node with id
/// let node: Node = decode!(data, node.id).unwrap();
///
/// let directed_edge = DirectedEdge {
///     edge_id: [0; 16],
///     other_node_id: [1; 16],
/// };
///
/// let data = encode!(directed_edge).unwrap();
/// // for directed edge
/// let directed_edge: DirectedEdge = decode!(data).unwrap();
/// ```
#[macro_export]
macro_rules! decode {
    ($data:expr, $id:expr) => {
        $crate::protocol::item_serdes::ItemSerdes::decode_with_id($data, $id)
    };
    ($data:expr) => {
        $crate::protocol::item_serdes::ItemSerdes::decode($data)
    };
}

#[macro_export]
macro_rules! encode {
    ($data:expr) => {
        $crate::protocol::item_serdes::ItemSerdes::encode($data)
    };
}