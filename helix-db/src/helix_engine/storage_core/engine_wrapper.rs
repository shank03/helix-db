#[cfg(feature = "rocksdb")]
use std::borrow::Cow;
use std::{borrow::Cow, collections::HashMap, marker::PhantomData, path::Path};

use heed3::EnvOpenOptions;
#[cfg(feature = "lmdb")]
use heed3::{AnyTls, WithTls};
use serde::{Deserialize, de::DeserializeOwned};

#[cfg(feature = "rocksdb")]
use crate::helix_engine::storage_core::engine_wrappers::rocksdb_wrapper::{Bytes, U128};

#[cfg(feature = "lmdb")]
use crate::helix_engine::storage_core::engine_wrappers::lmdb_wrapper::{Bytes, U128};

use super::storage_methods::DBMethods;
use crate::{
    helix_engine::{
        bm25::bm25::HBM25Config,
        graph_core::config::{Config, GraphConfig},
        storage_core::{storage_core::StorageConfig, storage_methods::StorageMethods},
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
use heed3::{Database, DatabaseFlags, RoTxn, RwTxn, byteorder::BE, types::*};

pub const DB_NODES: &str = "nodes"; // for node data (n:)
pub const DB_EDGES: &str = "edges"; // for edge data (e:)
pub const DB_OUT_EDGES: &str = "out_edges"; // for outgoing edge indices (o:)
pub const DB_IN_EDGES: &str = "in_edges"; // for incoming edge indices (i:)


pub trait Txn<'a>: Sized {
    fn commit_txn(self) -> Result<(), GraphError>;
    fn abort_txn(self) -> Result<(), GraphError>;
}

pub struct HelixIterator<'a, I: Iterator> {
    #[cfg(feature = "rocksdb")]
    pub iter: rocksdb::DBIteratorWithThreadMode<
        'a,
        rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    >,
    #[cfg(feature = "lmdb")]
    pub iter: I,
    #[cfg(feature = "in_memory")]
    pub iter: skipdb::Iter<'a, K, V>,
    pub(super) _phantom: PhantomData<(&'a I)>,
}

pub struct RTxn<'a> {
    #[cfg(feature = "rocksdb")]
    pub txn: rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    #[cfg(feature = "lmdb")]
    pub txn: heed3::RoTxn<'a, WithTls>,
    #[cfg(feature = "in_memory")]
    pub txn: skipdb::ReadTransaction<
        &'a [u8],
        &'a [u8],
        OptimisticDb<&'a [u8], &'a [u8]>,
        txn_core::sync::HashCm<&'a [u8]>,
    >,
}

pub struct WTxn<'env> {
    #[cfg(feature = "rocksdb")]
    pub txn: rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    #[cfg(feature = "lmdb")]
    pub txn: heed3::RwTxn<'env>,
    #[cfg(feature = "in_memory")]
    pub txn: skipdb::optimistic::OptimisticTransaction<&'a [u8], &'a [u8]>,
}

impl<'env> WTxn<'env> {
    #[cfg(feature = "lmdb")]
    pub fn get_txn<'tx>(&'tx mut self) -> &'tx mut heed3::RwTxn<'env> {
        return &mut self.txn;
    }

    #[cfg(feature = "rocksdb")]
    pub fn get_txn(
        &'a self,
    ) -> &'a rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>> {
        return &self.txn;
    }

    #[cfg(feature = "in_memory")]
    pub fn get_txn(
        &'a self,
    ) -> &'a skipdb::ReadTransaction<
        &'a [u8],
        &'a [u8],
        OptimisticDb<&'a [u8], &'a [u8]>,
        txn_core::sync::HashCm<&'a [u8]>,
    > {
        return &self.txn;
    }
}

impl<'a> Txn<'a> for RTxn<'a> {
    fn commit_txn(self) -> Result<(), GraphError> {
        #[cfg(feature = "rocksdb")]
        self.txn.commit().map_err(|e| GraphError::from(e))?;
        #[cfg(feature = "lmdb")]
        self.txn.commit().map_err(|e| GraphError::from(e))?;
        Ok(())
    }

    fn abort_txn(self) -> Result<(), GraphError> {
        #[cfg(feature = "rocksdb")]
        self.txn.rollback().map_err(|e| GraphError::from(e))?;
        #[cfg(feature = "lmdb")]
        self.txn.commit().map_err(|e| GraphError::from(e))?;
        Ok(())
    }

    #[cfg(feature = "in_memory")]
    fn abort_txn(self) -> Result<(), GraphError> {
        Ok(())
    }
}

impl<'env> Txn<'env> for WTxn<'env> {
    fn commit_txn(self) -> Result<(), GraphError> {
        #[cfg(feature = "rocksdb")]
        self.txn.commit().map_err(|e| GraphError::from(e))?;
        #[cfg(feature = "lmdb")]
        self.txn.commit().map_err(|e| GraphError::from(e))?;
        Ok(())
    }

    fn abort_txn(self) -> Result<(), GraphError> {
        #[cfg(feature = "rocksdb")]
        self.txn.rollback().map_err(|e| GraphError::from(e))?;
        #[cfg(feature = "lmdb")]
        self.txn.commit().map_err(|e| GraphError::from(e))?;
        Ok(())
    }

    #[cfg(feature = "in_memory")]
    fn abort_txn(self) -> Result<(), GraphError> {
        Ok(())
    }
}

pub trait Storage<'a> {
    type Key;
    type Value;
    type BasicIter: Iterator
    where
        Self: 'a;
    type PrefixIter: Iterator
    where
        Self: 'a;
    type DuplicateIter: Iterator
    where
        Self: 'a;

    fn get_data(&self, txn: &'a RTxn<'a>, key: Self::Key) -> Result<Option<Self::Value>, GraphError>;
    fn put_data(
        &self,
        txn: &'a mut WTxn<'a>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError>;
    fn delete_data(&self, txn: &'a mut WTxn<'a>, key: Self::Key) -> Result<(), GraphError>;
    fn delete_duplicate(
        &self,
        txn: &'a mut WTxn<'a>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError>;
    fn iter_data(
        &'a self,
        txn: &'a RTxn<'a>,
    ) -> Result<HelixIterator<'a, Self::BasicIter>, GraphError>;
    fn prefix_iter_data(
        &'a self,
        txn: &'a RTxn<'a>,
        prefix: Self::Key,
    ) -> Result<HelixIterator<'a, Self::PrefixIter>, GraphError>;
    fn get_duplicate_data(
        &'a self,
        txn: &'a RTxn<'a>,
        key: Self::Key,
    ) -> Result<HelixIterator<'a, Self::DuplicateIter>, GraphError>;
}

pub struct Table<'a, K, V> {
    #[cfg(feature = "rocksdb")]
    pub table: rocksdb::ColumnFamilyRef<'a>,
    #[cfg(feature = "lmdb")]
    pub table: heed3::Database<K, V>,
    #[cfg(feature = "in_memory")]
    pub table: skipdb::DB<K>,
    pub _phantom: PhantomData<(&'a K, &'a V)>,
}

impl<'a, K, V> Table<'a, K, V> {
    #[cfg(feature = "lmdb")]
    pub fn new_lmdb(table: heed3::Database<K, V>) -> Table<'a, K, V> {
        Table { table, _phantom: PhantomData }
    }

    #[cfg(feature = "rocksdb")]
    pub fn new_rocksdb(table: rocksdb::ColumnFamilyRef<'a>) -> Table<'a, K, V> {
        Table { table, _phantom: PhantomData }
    }

    #[cfg(feature = "in_memory")]
    pub fn new_in_memory(table: skipdb::DB<K, V>) -> Table<'a, K, V> {
        Table { table, _phantom: PhantomData }
    }
}


pub struct HelixEnv<'a> {
    #[cfg(feature = "lmdb")]
    pub env: heed3::Env<WithTls>,
    #[cfg(feature = "rocksdb")]
    pub env: rocksdb::DB,
    #[cfg(feature = "in_memory")]
    pub env: skipdb::DB<K, V>,

    _phantom: PhantomData<&'a ()>,
}

impl<'a> HelixEnv<'a> {
    #[cfg(feature = "lmdb")]
    pub fn new_lmdb(env: heed3::Env<WithTls>) -> HelixEnv<'a> {
        HelixEnv { env, _phantom: PhantomData }
    }

    #[cfg(feature = "rocksdb")]
    pub fn new_rocksdb(env: rocksdb::DB) -> HelixEnv<'a> {
        HelixEnv { env, _phantom: PhantomData }
    }
}

pub struct HelixDB<'a> {
    pub env: HelixEnv<'a>,
    pub storage_config: StorageConfig,
    pub nodes_db: Table<'a, U128, Bytes>,
    pub edges_db: Table<'a, U128, Bytes>,
    pub out_edges_db: Table<'a, Bytes, Bytes>,
    pub in_edges_db: Table<'a, Bytes, Bytes>,
    pub secondary_indices: HashMap<String, Table<'a, Bytes, U128>>,
}

pub trait HelixDBMethods: Sized {
    #[cfg(feature = "rocksdb")]
    fn config() -> rocksdb::Options;
    #[cfg(feature = "rocksdb")]
    fn new(path: &str, opts: rocksdb::Options) -> Result<Self, GraphError>;

    fn read_txn(&self) -> Result<RTxn, GraphError>;
    fn write_txn(&self) -> Result<WTxn, GraphError>;

    fn nodes_db(&self) -> &Table<U128, Bytes>;
    fn edges_db(&self) -> &Table<U128, Bytes>;
    fn out_edges_db(&self) -> &Table<Bytes, Bytes>;
    fn in_edges_db(&self) -> &Table<Bytes, Bytes>;
    // fn secondary_indices(&self) -> HashMap<String, HelixTable<Bytes, U128>>;
}

impl<'a> HelixDBMethods for HelixDB<'a> {
    fn read_txn(&self) -> Result<RTxn, GraphError> {
        self.env.read_txn()
    }

    fn write_txn(&self) -> Result<WTxn, GraphError> {
        self.env.write_txn()
    }

    fn nodes_db(&self) -> &Table<U128, Bytes> {
        &self.nodes_db
    }

    fn edges_db(&self) -> &Table<U128, Bytes> {
        &self.edges_db
    }

    fn out_edges_db(&self) -> &Table<Bytes, Bytes> {
        &self.out_edges_db
    }

    fn in_edges_db(&self) -> &Table<Bytes, Bytes> {
        &self.in_edges_db
    }
}

#[cfg(feature = "in_memory")]
pub enum U128 {}

#[cfg(feature = "in_memory")]
pub enum Bytes {}

