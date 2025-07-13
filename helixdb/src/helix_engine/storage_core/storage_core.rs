use super::storage_methods::DBMethods;
use crate::{
    debug_println,
    helix_engine::{
        bm25::bm25::HBM25Config,
        graph_core::config::Config,
        storage_core::storage_methods::StorageMethods,
        types::GraphError,
        vector_core::{
            hnsw::HNSW,
            vector::HVector,
            vector_core::{HNSWConfig, VectorCore},
        },
    },
    utils::{
        items::{Edge, Node},
        label_hash::hash_label,
    },
};
use heed3::{
    types::*,
    Database, DatabaseFlags,
    Env, EnvOpenOptions,
    RoTxn, RwTxn,
    byteorder::BE,
    RoIter,

};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    fs,
    path::Path,
    sync::Arc,
};
use sonic_rs::{
    json,
    Value as JsonValue,
    JsonValueMutTrait
};

// database names for different stores
const DB_NODES: &str = "nodes"; // for node data (n:)
const DB_EDGES: &str = "edges"; // for edge data (e:)
const DB_OUT_EDGES: &str = "out_edges"; // for outgoing edge indices (o:)
const DB_IN_EDGES: &str = "in_edges"; // for incoming edge indices (i:)

pub type NodeId = u128;
pub type EdgeId = u128;

pub struct HelixGraphStorage {
    // TODO: maybe make not public?
    pub graph_env: Env,
    pub nodes_db: Database<U128<BE>, Bytes>,
    pub edges_db: Database<U128<BE>, Bytes>,
    pub out_edges_db: Database<Bytes, Bytes>,
    pub in_edges_db: Database<Bytes, Bytes>,
    pub secondary_indices: HashMap<String, Database<Bytes, U128<BE>>>,
    pub vectors: VectorCore,
    pub bm25: HBM25Config,
    pub schema: String,
    pub graphvis_node_label: Option<String>,
    pub embedding_model: Option<String>,
}

impl HelixGraphStorage {
    pub fn new(path: &str, config: Config) -> Result<HelixGraphStorage, GraphError> {
        fs::create_dir_all(path)?;

        let db_size = if config.db_max_size_gb.unwrap_or(100) >= 9999 {
            9998
        } else {
            config.db_max_size_gb.unwrap_or(100)
        };

        let graph_env = unsafe {
            EnvOpenOptions::new()
                .map_size(db_size * 1024 * 1024 * 1024) // Sets max size of the database in GB
                .max_dbs(20) // Sets max number of databases
                .max_readers(200) // Sets max number of readers
                .open(Path::new(path))?
        };

        let mut wtxn = graph_env.write_txn()?;

        // creates the lmdb databases (tables)
        // Table: [key]->[value]
        //        [size]->[size]

        // Nodes: [node_id]->[bytes array of node data]
        //        [16 bytes]->[dynamic]
        let nodes_db = graph_env
            .database_options()
            .types::<U128<BE>, Bytes>()
            .name(DB_NODES)
            .create(&mut wtxn)?;

        // Edges: [edge_id]->[bytes array of edge data]
        //        [16 bytes]->[dynamic]
        let edges_db = graph_env
            .database_options()
            .types::<U128<BE>, Bytes>()
            .name(DB_EDGES)
            .create(&mut wtxn)?;

        // Out edges: [from_node_id + label]->[edge_id + to_node_id]  (edge first because value is ordered by byte size)
        //                    [20 + 4 bytes]->[16 + 16 bytes]
        //
        // DUP_SORT used to store all values of duplicated keys under a single key. Saves on space and requires a single read to get all values.
        // DUP_FIXED used to ensure all values are the same size meaning 8 byte length header is discarded.
        let out_edges_db: Database<Bytes, Bytes> = graph_env
            .database_options()
            .types::<Bytes, Bytes>()
            .flags(DatabaseFlags::DUP_SORT | DatabaseFlags::DUP_FIXED)
            .name(DB_OUT_EDGES)
            .create(&mut wtxn)?;

        // In edges: [to_node_id + label]->[edge_id + from_node_id]  (edge first because value is ordered by byte size)
        //                 [20 + 4 bytes]->[16 + 16 bytes]
        //
        // DUP_SORT used to store all values of duplicated keys under a single key. Saves on space and requires a single read to get all values.
        // DUP_FIXED used to ensure all values are the same size meaning 8 byte length header is discarded.
        let in_edges_db: Database<Bytes, Bytes> = graph_env
            .database_options()
            .types::<Bytes, Bytes>()
            .flags(DatabaseFlags::DUP_SORT | DatabaseFlags::DUP_FIXED)
            .name(DB_IN_EDGES)
            .create(&mut wtxn)?;

        // Creates the secondary indices databases if there are any
        let mut secondary_indices = HashMap::new();
        if let Some(indexes) = config.graph_config.secondary_indices {
            for index in indexes {
                secondary_indices.insert(
                    index.clone(),
                    graph_env
                        .database_options()
                        .types::<Bytes, U128<BE>>()
                        .flags(DatabaseFlags::DUP_SORT) // DUP_SORT used to store all duplicated node keys under a single key. Saves on space and requires a single read to get all values.
                        .name(&index)
                        .create(&mut wtxn)?,
                );
            }
        }

        // Creates the vector database
        let vectors = VectorCore::new(
            &graph_env,
            &mut wtxn,
            HNSWConfig::new(
                config.vector_config.m,
                config.vector_config.ef_construction,
                config.vector_config.ef_search,
            ),
        )?;

        let bm25 = HBM25Config::new(&graph_env, &mut wtxn)?;
        let schema = config.schema.unwrap_or("".to_string());
        let graphvis_node_label = config.graphvis_node_label;
        let embedding_model = config.embedding_model;

        wtxn.commit()?;
        Ok(Self {
            graph_env,
            nodes_db,
            edges_db,
            out_edges_db,
            in_edges_db,
            secondary_indices,
            vectors,
            bm25,
            schema,
            graphvis_node_label,
            embedding_model,
        })
    }

    /// Used because in the case the key changes in the future.
    /// Believed to not introduce any overhead being inline and using a reference.
    #[must_use]
    #[inline(always)]
    pub fn node_key(id: &u128) -> &u128 {
        id
    }

    /// Used because in the case the key changes in the future.
    /// Believed to not introduce any overhead being inline and using a reference.
    #[must_use]
    #[inline(always)]
    pub fn edge_key(id: &u128) -> &u128 {
        id
    }

    /// Out edge key generator. Creates a 20 byte array and copies in the node id and 4 byte label.
    ///
    /// key = `from-node(16)` | `label-id(4)`                 ← 20 B
    ///
    /// The generated out edge key will remain the same for the same from_node_id and label.
    /// To save space, the key is only stored once,
    /// with the values being stored in a sorted sub-tree, with this key being the root.
    #[inline(always)]
    pub fn out_edge_key(from_node_id: &u128, label: &[u8; 4]) -> [u8; 20] {
        let mut key = [0u8; 20];
        key[0..16].copy_from_slice(&from_node_id.to_be_bytes());
        key[16..20].copy_from_slice(label);
        key
    }

    /// In edge key generator. Creates a 20 byte array and copies in the node id and 4 byte label.
    ///
    /// key = `to-node(16)` | `label-id(4)`                 ← 20 B
    ///
    /// The generated in edge key will remain the same for the same to_node_id and label.
    /// To save space, the key is only stored once,
    /// with the values being stored in a sorted sub-tree, with this key being the root.
    #[inline(always)]
    pub fn in_edge_key(to_node_id: &u128, label: &[u8; 4]) -> [u8; 20] {
        let mut key = [0u8; 20];
        key[0..16].copy_from_slice(&to_node_id.to_be_bytes());
        key[16..20].copy_from_slice(label);
        key
    }

    /// Packs the edge data into a 32 byte array.
    ///
    /// data = `edge-id(16)` | `node-id(16)`                 ← 32 B (DUPFIXED)
    #[inline(always)]
    pub fn pack_edge_data(edge_id: &u128, node_id: &u128) -> [u8; 32] {
        let mut key = [0u8; 32];
        key[0..16].copy_from_slice(&edge_id.to_be_bytes());
        key[16..32].copy_from_slice(&node_id.to_be_bytes());
        key
    }

    /// Unpacks the 32 byte array into an (edge_id, node_id) tuple of u128s.
    ///
    /// Returns (edge_id, node_id)
    #[inline(always)]
    // Uses Type Aliases for clarity
    pub fn unpack_adj_edge_data(data: &[u8]) -> Result<(EdgeId, NodeId), GraphError> {
        let edge_id = u128::from_be_bytes(
            data[0..16]
                .try_into()
                .map_err(|_| GraphError::SliceLengthError)?,
        );
        let node_id = u128::from_be_bytes(
            data[16..32]
                .try_into()
                .map_err(|_| GraphError::SliceLengthError)?,
        );
        Ok((edge_id, node_id))
    }

    /// Gets a vector from level 0 of HNSW index (because that's where all are stored)
    pub fn get_vector(&self, txn: &RoTxn, id: &u128) -> Result<HVector, GraphError> {
        Ok(self.vectors.get_vector(txn, *id, 0, true)?)
    }

        /// Converts graph nodes and edges to a JSON string for visualization.
        ///
        /// # Arguments
        /// * `txn` - Read-only transaction for database access.
        /// * `k` - Optional number of top nodes to include (default: 200, max: 300).
        /// * `node_prop` - Optional node property to include in the output.
        ///
        /// # Returns
        /// A `Result` containing the JSON string or a `GraphError` if:
        /// - More than 300 nodes are requested.
        /// - Nodes or edges database is empty.
        /// - JSON serialization fails.
    pub fn nodes_edges_to_json(
        &self,
        txn: &RoTxn,
        k: Option<usize>,
        node_prop: Option<String>
    ) -> Result<String, GraphError> {
        let k = k.unwrap_or(200);
        if k > 300 {
            return Err(GraphError::New("cannot not visualize more than 300 nodes!".to_string()));
        }

        if self.nodes_db.is_empty(&txn)? || self.edges_db.is_empty(&txn)? {
            return Err(GraphError::New("edges or nodes db is empty!".to_string()));
        }

        let top_nodes = self.get_nodes_by_cardinality(&txn, k)?;

        let ret_json = self.cards_to_json(&txn, k, top_nodes, node_prop)?;
        sonic_rs::to_string(&ret_json).map_err(|e| GraphError::New(e.to_string()))
    }

    /// Get the top k nodes and all of the edges associated with them by checking their
    /// cardinalities (total number of in and out edges)
    ///
    /// Output:
    /// Vec [
    ///     node_id: u128,
    ///     out_edges: Vec<(EdgeID, FromNodeId, ToNodeId)>,
    ///     in_edges: Vec<(EdgeID, FromNodeId, ToNodeId)>,
    /// ]
    fn get_nodes_by_cardinality(
        &self,
        txn: &RoTxn,
        k: usize,
    ) -> Result<Vec<(u128, Vec<(u128, u128, u128)>, Vec<(u128, u128, u128)>)>, GraphError> {
        let node_count = self.nodes_db.len(&txn)?;

        type EdgeID = u128;
        type ToNodeId = u128;
        type FromNodeId = u128;

        struct EdgeCount {
            node_id: u128,
            edges_count: usize,
            out_edges: Vec<(EdgeID, FromNodeId, ToNodeId)>,
            in_edges: Vec<(EdgeID, FromNodeId, ToNodeId)>,
        }

        impl PartialEq for EdgeCount {
            fn eq(&self, other: &Self) -> bool {
                self.edges_count == other.edges_count
            }
        }
        impl PartialOrd for EdgeCount {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.edges_count.cmp(&other.edges_count))
            }
        }
        impl Eq for EdgeCount {}
        impl Ord for EdgeCount {
            fn cmp(&self, other: &Self) -> Ordering {
                self.edges_count.cmp(&other.edges_count)
            }
        }

        let db = Arc::new(self);
        let out_db = Arc::clone(&db);
        let in_db = Arc::clone(&db);

        struct Edges<'a> {
            edge_count: usize,
            out_edges: Option<
                RoIter<
                    'a,
                    Bytes,
                    LazyDecode<Bytes>,
                    heed3::iteration_method::MoveOnCurrentKeyDuplicates,
                >,
            >,
            in_edges: Option<
                RoIter<
                    'a,
                    Bytes,
                    LazyDecode<Bytes>,
                    heed3::iteration_method::MoveOnCurrentKeyDuplicates,
                >,
            >,
        }

        impl<'a> Default for Edges<'a> {
            fn default() -> Self {
                Self {
                    edge_count: 0,
                    out_edges: None,
                    in_edges: None,
                }
            }
        }

        let mut edge_counts: HashMap<u128, Edges> = HashMap::with_capacity(node_count as usize);
        let mut ordered_edge_counts: BinaryHeap<EdgeCount> =
            BinaryHeap::with_capacity(node_count as usize);

        // out edges
        // this gets each node ID from the out edges db
        // by using the out_edges_db it pulls data into os cache
        let out_node_key_iter = out_db.out_edges_db.lazily_decode_data().iter(&txn).unwrap();
        for data in out_node_key_iter {
            match data {
                Ok((key, _)) => {
                    let node_id = &key[0..16];
                    // for each node id, it gets the edges which are all stored in the same key
                    // so it gets all the edges for a node at once
                    // without decoding anything. so you only ever decode the key from LMDB once
                    let edges = out_db
                        .out_edges_db
                        .lazily_decode_data()
                        .get_duplicates(&txn, key)
                        .unwrap();

                    let edges_count = edges.iter().count();

                    let edge_count = edge_counts
                        .entry(u128::from_be_bytes(node_id.try_into().unwrap()))
                        .or_insert(Edges::default());
                    edge_count.edge_count += edges_count;
                    edge_count.out_edges = edges;
                }
                Err(_e) => {
                    debug_println!("Error in out_node_key_iter: {:?}", _e);
                }
            }
        }

        // in edges
        // this gets each node ID from the in edges db
        // by using the in_edges_db it pulls data into os cache
        let in_node_key_iter = in_db.in_edges_db.lazily_decode_data().iter(&txn).unwrap();
        for data in in_node_key_iter {
            match data {
                Ok((key, _)) => {
                    let node_id = &key[0..16];
                    // for each node id, it gets the edges which are all stored in the same key
                    // so it gets all the edges for a node at once
                    // without decoding anything. so you only ever decode the key from LMDB once
                    let edges = in_db
                        .in_edges_db
                        .lazily_decode_data()
                        .get_duplicates(&txn, key)
                        .unwrap();
                    let edges_count = edges.iter().count();

                    let edge_count = edge_counts
                        .entry(u128::from_be_bytes(node_id.try_into().unwrap()))
                        .or_insert(Edges::default());
                    edge_count.edge_count += edges_count;
                    edge_count.in_edges = edges;
                }
                Err(_e) => {
                    debug_println!("Error in in_node_key_iter: {:?}", _e);
                }
            }
        }

        // for each node, get the decode the edges and extract the edge id and other node id
        // and add to the ordered_edge_counts heap
        for (node_id, edges_count) in edge_counts.into_iter() {
            let out_edges = match edges_count.out_edges {
                Some(out_edges_iter) => {
                    out_edges_iter
                        .map(|result| {
                            let (key, value) = result.unwrap();
                            let from_node = u128::from_be_bytes(key[0..16].try_into().unwrap());
                            let (edge_id, to_node) =
                                Self::unpack_adj_edge_data(value.decode().unwrap()).unwrap();
                            (edge_id, from_node, to_node)
                        })
                    .collect::<Vec<(EdgeID, FromNodeId, ToNodeId)>>()
                }
                None => vec![]
            };
            let in_edges = match edges_count.in_edges {
                Some(in_edges_iter) => {
                    in_edges_iter
                        .map(|result| {
                            let (key, value) = result.unwrap();
                            let to_node = u128::from_be_bytes(key[0..16].try_into().unwrap());
                            let (edge_id, from_node) =
                                Self::unpack_adj_edge_data(value.decode().unwrap()).unwrap();
                            (edge_id, from_node, to_node)
                        })
                    .collect::<Vec<(EdgeID, FromNodeId, ToNodeId)>>()
                }
                None => vec![]
            };

            ordered_edge_counts.push(EdgeCount {
                node_id,
                edges_count: edges_count.edge_count,
                out_edges,
                in_edges,
            });
        }

        let mut top_nodes = Vec::with_capacity(k);
        while let Some(edges_count) = ordered_edge_counts.pop() {
            top_nodes.push((
                edges_count.node_id,
                edges_count.out_edges,
                edges_count.in_edges,
            ));
            if top_nodes.len() >= k {
                break;
            }
        }

        Ok(top_nodes)
    }

    /// Output:
    /// {
    ///     "nodes": [{"id": uuid_id_node, "label": "optional_property", "title": "uuid"}],
    ///     "edges": [{"from": uuid, "to": uuid, "title": "uuid"}]
    /// }
    fn cards_to_json(
        &self,
        txn: &RoTxn,
        k: usize,
        top_nodes: Vec<(u128, Vec<(u128, u128, u128)>, Vec<(u128, u128, u128)>)>,
        node_prop: Option<String>,
    ) -> Result<JsonValue, GraphError> {
        let mut nodes = Vec::with_capacity(k);
        let mut edges = Vec::new();

        top_nodes.iter().try_for_each(|(id, out_edges, in_edges)| {
            let mut json_node = json!({ "id": id.to_string(), "title": id.to_string() });
            if let Some(prop) = &node_prop {
                let mut node = self.nodes_db
                    .lazily_decode_data()
                    .prefix_iter(&txn, id)
                    .unwrap();
                if let Some((_, data)) = node.next().transpose().unwrap() {
                    let node = Node::decode_node(data.decode().unwrap(), *id)?;
                    let props = node.properties.as_ref().ok_or_else(|| {
                        GraphError::New(format!("no properties for node {}", id))
                    })?;
                    let prop_value = props.get(prop).ok_or_else(|| {
                        GraphError::New(format!("property {} not found for node {}", prop, id))
                    })?;
                    json_node
                        .as_object_mut()
                        .ok_or_else(|| GraphError::New("invalid JSON object".to_string()))?
                        .insert("label", json!(prop_value));
                }
            }

            nodes.push(json_node);
            out_edges.iter().for_each(|(edge_id, from_node_id, to_node_id)| {
                edges.push(json!({
                    "from": from_node_id.to_string(),
                    "to": to_node_id.to_string(),
                    "title": edge_id.to_string(),
                }));
            });

            // TODO: still having 1 error this is basically duplicated
            /*
            in_edges.iter().for_each(|(edge_id, from_node_id, to_node_id)| {
                edges.push(json!({
                    "from": from_node_id.to_string(),
                    "to": to_node_id.to_string(),
                    "title": edge_id.to_string(),
                }));
            });
            */

            Ok::<(), GraphError>(())
        })?;

        let result = json!({
            "nodes": nodes,
            "edges": edges,
        });

        Ok(result)
    }

    /// Get number of nodes, edges, and vectors from lmdb
    pub fn get_db_stats_json(&self, txn: &RoTxn) -> Result<String, GraphError> {
        let result = json!({
            "num_nodes":   self.nodes_db.len(&txn).unwrap_or(0),
            "num_edges":   self.edges_db.len(&txn).unwrap_or(0),
            "num_vectors": self.vectors.vectors_db.len(&txn).unwrap_or(0),
        });
        debug_println!("db stats json: {:?}", result);

        serde_json::to_string(&result).map_err(|e| GraphError::New(e.to_string()))
    }
}

impl DBMethods for HelixGraphStorage {
    /// Creates a secondary index lmdb db (table) for a given index name
    fn create_secondary_index(&mut self, name: &str) -> Result<(), GraphError> {
        let mut wtxn = self.graph_env.write_txn()?;
        let db = self.graph_env.create_database(&mut wtxn, Some(name))?;
        wtxn.commit()?;
        self.secondary_indices.insert(name.to_string(), db);
        Ok(())
    }

    /// Drops a secondary index lmdb db (table) for a given index name
    fn drop_secondary_index(&mut self, name: &str) -> Result<(), GraphError> {
        let mut wtxn = self.graph_env.write_txn()?;
        let db = self
            .secondary_indices
            .get(name)
            .ok_or(GraphError::New(format!(
                "Secondary Index {} not found",
                name
            )))?;
        db.clear(&mut wtxn)?;
        wtxn.commit()?;
        self.secondary_indices.remove(name);
        Ok(())
    }
}

impl StorageMethods for HelixGraphStorage {
    #[inline(always)]
    fn check_exists(&self, txn: &RoTxn, id: &u128) -> Result<bool, GraphError> {
        Ok(self.nodes_db.get(txn, Self::node_key(id))?.is_some())
    }

    #[inline(always)]
    fn get_node(&self, txn: &RoTxn, id: &u128) -> Result<Node, GraphError> {
        let node = match self.nodes_db.get(txn, Self::node_key(id))? {
            Some(data) => data,
            None => return Err(GraphError::NodeNotFound),
        };
        let node: Node = match Node::decode_node(&node, *id) {
            Ok(node) => node,
            Err(e) => return Err(e),
        };
        Ok(node)
    }

    #[inline(always)]
    fn get_edge(&self, txn: &RoTxn, id: &u128) -> Result<Edge, GraphError> {
        let edge = match self.edges_db.get(txn, Self::edge_key(id))? {
            Some(data) => data,
            None => return Err(GraphError::EdgeNotFound),
        };
        let edge: Edge = match Edge::decode_edge(&edge, *id) {
            Ok(edge) => edge,
            Err(e) => return Err(e),
        };
        Ok(edge)
    }

    fn drop_node(&self, txn: &mut RwTxn, id: &u128) -> Result<(), GraphError> {
        // Get node to get its label
        //let node = self.get_node(txn, id)?;

        // Delete outgoing edges
        let out_edges = {
            let iter = self.out_edges_db.prefix_iter(&txn, &id.to_be_bytes())?;
            let capacity = match iter.size_hint() {
                (_, Some(upper)) => upper,
                (lower, None) => lower,
            };
            let mut out_edges = Vec::with_capacity(capacity);

            for result in iter {
                let (key, value) = result?;
                assert_eq!(key.len(), 20);
                let mut label = [0u8; 4];
                label.copy_from_slice(&key[16..20]);
                let (edge_id, _) = Self::unpack_adj_edge_data(&value)?;
                out_edges.push((edge_id, label));
            }
            out_edges
        };

        // Delete incoming edges

        let in_edges = {
            let iter = self.in_edges_db.prefix_iter(&txn, &id.to_be_bytes())?;
            let capacity = match iter.size_hint() {
                (_, Some(c)) => c,
                (c, None) => c,
            };
            let mut in_edges = Vec::with_capacity(capacity);

            for result in iter {
                let (key, value) = result?;
                assert_eq!(key.len(), 20);
                let mut label = [0u8; 4];
                label.copy_from_slice(&key[16..20]);
                let (edge_id, node_id) = Self::unpack_adj_edge_data(&value)?;
                in_edges.push((edge_id, label, node_id));
            }

            in_edges
        };

        // Delete all related data
        for (out_edge_id, label_bytes) in out_edges.iter() {
            // Delete edge data
            self.edges_db.delete(txn, &Self::edge_key(out_edge_id))?;
            self.out_edges_db
                .delete(txn, &Self::out_edge_key(id, label_bytes))?;
        }
        for (in_edge_id, label_bytes, other_id) in in_edges.iter() {
            self.edges_db.delete(txn, &Self::edge_key(in_edge_id))?;
            self.in_edges_db
                .delete(txn, &Self::in_edge_key(other_id, label_bytes))?;
        }

        // Delete node data and label
        self.nodes_db.delete(txn, Self::node_key(id))?;

        Ok(())
    }

    fn drop_edge(&self, txn: &mut RwTxn, edge_id: &u128) -> Result<(), GraphError> {
        // Get edge data first
        let edge_data = match self.edges_db.get(&txn, &Self::edge_key(edge_id))? {
            Some(data) => data,
            None => return Err(GraphError::EdgeNotFound),
        };
        let edge: Edge = bincode::deserialize(edge_data)?;
        let label_hash = hash_label(&edge.label, None);
        // Delete all edge-related data
        self.edges_db.delete(txn, &Self::edge_key(edge_id))?;
        self.out_edges_db
            .delete(txn, &Self::out_edge_key(&edge.from_node, &label_hash))?;
        self.in_edges_db
            .delete(txn, &Self::in_edge_key(&edge.to_node, &label_hash))?;

        Ok(())
    }
}
