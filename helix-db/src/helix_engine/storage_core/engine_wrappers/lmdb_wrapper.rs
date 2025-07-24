#[cfg(feature = "lmdb")]
use heed3::iteration_method::MoveOnCurrentKeyDuplicates;
#[cfg(feature = "lmdb")]
use heed3::{Database, DatabaseFlags, EnvOpenOptions};

use crate::helix_engine::bm25::bm25::HBM25Config;
use crate::helix_engine::graph_core::config::Config;
use crate::helix_engine::storage_core::engine_wrapper::{
    DB_EDGES, DB_IN_EDGES, DB_NODES, DB_OUT_EDGES, HelixDB, HelixIterator, RTxn, Storage, WTxn,
};
#[cfg(feature = "rocksdb")]
use crate::helix_engine::storage_core::engine_wrapper::{Database, Table};
#[cfg(feature = "lmdb")]
use crate::helix_engine::storage_core::engine_wrapper::{HelixEnv, Table};
use crate::helix_engine::storage_core::storage_core::StorageConfig;
use crate::helix_engine::types::GraphError;
use crate::helix_engine::vector_core::vector_core::{HNSWConfig, VectorCore};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::Path;

#[cfg(feature = "lmdb")]
pub type U128 = heed3::types::U128<heed3::byteorder::BE>;

#[cfg(feature = "lmdb")]
pub type Bytes = heed3::types::Bytes;

#[cfg(feature = "lmdb")]
impl<'a> RTxn<'a> {
    pub fn get_txn(&'a self) -> &'a heed3::RoTxn<'a> {
        return &self.txn;
    }
}

#[cfg(feature = "lmdb")]
impl HelixEnv {
    pub fn read_txn(&self) -> Result<RTxn, GraphError> {
        self.env
            .read_txn()
            .map(|txn| RTxn { txn })
            .map_err(|e| GraphError::from(e))
    }

    pub fn write_txn(&self) -> Result<WTxn, GraphError> {
        self.env
            .write_txn()
            .map(|txn| WTxn { txn })
            .map_err(|e| GraphError::from(e))
    }
}

#[cfg(feature = "lmdb")]
impl<'a> Storage<'a> for Table<U128, Bytes> {
    type Key = &'a u128;
    type Value = &'a [u8];
    type BasicIter
        = heed3::RoIter<'a, U128, heed3::types::LazyDecode<Bytes>>
    where
        Self: 'a;
    type PrefixIter
        = heed3::RoPrefix<'a, U128, heed3::types::LazyDecode<Bytes>>
    where
        Self: 'a;
    type DuplicateIter
        = heed3::RoIter<'a, U128, heed3::types::LazyDecode<Bytes>, MoveOnCurrentKeyDuplicates>
    where
        Self: 'a;

    fn get_data<'tx>(
        &self,
        txn: &'a RTxn<'tx>,
        key: Self::Key,
    ) -> Result<Option<&'a [u8]>, GraphError> {
        Ok(self.table.get(txn.get_txn(), key)?)
    }

    fn put_data<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError> {
        Ok(self.table.put(txn.get_txn(), key, value)?)
    }

    fn put_data_with_duplicate<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError> {
        use heed3::PutFlags;

        Ok(self
            .table
            .put_with_flags(txn.get_txn(), PutFlags::APPEND_DUP, key, value)?)
    }

    fn put_data_in_order<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError> {
        use heed3::PutFlags;

        Ok(self
            .table
            .put_with_flags(txn.get_txn(), PutFlags::APPEND, key, value)?)
    }
    fn delete_data<'tx>(&self, txn: &'a mut WTxn<'tx>, key: Self::Key) -> Result<(), GraphError> {
        self.table.delete(txn.get_txn(), key)?;
        Ok(())
    }

    fn delete_duplicate<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError> {
        self.table.delete_one_duplicate(txn.get_txn(), key, value)?;
        Ok(())
    }

    fn iter_data<'tx>(
        &self,
        txn: &'a RTxn<'tx>,
    ) -> Result<HelixIterator<'a, Self::BasicIter>, GraphError> {
        Ok(HelixIterator {
            iter: self
                .table
                .lazily_decode_data()
                .iter(txn.get_txn())
                .map_err(|e| GraphError::from(e))?,
            _phantom: PhantomData,
        })
    }

    fn prefix_iter_data<'tx>(
        &self,
        txn: &'a RTxn<'tx>,
        prefix: Self::Key,
    ) -> Result<HelixIterator<'a, Self::PrefixIter>, GraphError> {
        Ok(HelixIterator {
            iter: self
                .table
                .lazily_decode_data()
                .prefix_iter(txn.get_txn(), prefix)?,
            _phantom: PhantomData,
        })
    }

    fn get_duplicate_data<'tx>(
        &self,
        txn: &'a RTxn<'tx>,
        key: Self::Key,
    ) -> Result<HelixIterator<'a, Self::DuplicateIter>, GraphError> {
        let duplicate_iter = match self
            .table
            .lazily_decode_data()
            .get_duplicates(txn.get_txn(), key)?
        {
            Some(iter) => iter,
            None => return Err(GraphError::from("No duplicates found")),
        };

        Ok(HelixIterator {
            iter: duplicate_iter,
            _phantom: PhantomData,
        })
    }
}

impl HelixDB {
    pub fn new(path: &str, config: Config) -> Result<HelixDB, GraphError> {
        std::fs::create_dir_all(path)?;

        let db_size = if config.db_max_size_gb.unwrap_or(100) >= 9999 {
            9998
        } else {
            config.db_max_size_gb.unwrap_or(100)
        };

        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(db_size * 1024 * 1024 * 1024)
                .max_dbs(20)
                .max_readers(200)
                .open(Path::new(path))?
        };

        let mut wtxn = env.write_txn()?;

        let nodes_db = env
            .database_options()
            .types::<U128, Bytes>()
            .name(DB_NODES)
            .create(&mut wtxn)?;

        // Edges: [edge_id]->[bytes array of edge data]
        //        [16 bytes]->[dynamic]
        let edges_db = env
            .database_options()
            .types::<U128, Bytes>()
            .name(DB_EDGES)
            .create(&mut wtxn)?;

        // Out edges: [from_node_id + label]->[edge_id + to_node_id]  (edge first because value is ordered by byte size)
        //                    [20 + 4 bytes]->[16 + 16 bytes]
        //
        // DUP_SORT used to store all values of duplicated keys under a single key. Saves on space and requires a single read to get all values.
        // DUP_FIXED used to ensure all values are the same size meaning 8 byte length header is discarded.
        let out_edges_db: Database<Bytes, Bytes> = env
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
        let in_edges_db: Database<Bytes, Bytes> = env
            .database_options()
            .types::<Bytes, Bytes>()
            .flags(DatabaseFlags::DUP_SORT | DatabaseFlags::DUP_FIXED)
            .name(DB_IN_EDGES)
            .create(&mut wtxn)?;

        let mut secondary_indices = HashMap::new();
        if let Some(indexes) = config.get_graph_config().secondary_indices {
            for index in indexes {
                secondary_indices.insert(
                    index.clone(),
                    env.database_options()
                        .types::<Bytes, U128>()
                        .flags(DatabaseFlags::DUP_SORT) // DUP_SORT used to store all duplicated node keys under a single key. Saves on space and requires a single read to get all values.
                        .name(&index)
                        .create(&mut wtxn)?,
                );
            }
        }

        let vector_config = config.get_vector_config();
        let vectors = VectorCore::new(
            &env,
            &mut wtxn,
            HNSWConfig::new(
                vector_config.m,
                vector_config.ef_construction,
                vector_config.ef_search,
            ),
        )?;

        let bm25 = config
            .get_bm25()
            .then(|| HBM25Config::new(&env, &mut wtxn))
            .transpose()?;

        let storage_config = StorageConfig::new(
            config.schema.unwrap_or("".to_string()),
            config.graphvis_node_label,
            config.embedding_model,
        );

        wtxn.commit()?;

        Ok(HelixDB {
            env: HelixEnv::new_lmdb(env),
            storage_config,
            nodes_db: Table::new_lmdb(nodes_db),
            edges_db: Table::new_lmdb(edges_db),
            out_edges_db: Table::new_lmdb(out_edges_db),
            in_edges_db: Table::new_lmdb(in_edges_db),
            secondary_indices: secondary_indices
                .into_iter()
                .map(|(k, v)| (k, Table::new_lmdb(v)))
                .collect(),
            // vectors,
            // bm25,
            // storage_config,
        })
    }
}
