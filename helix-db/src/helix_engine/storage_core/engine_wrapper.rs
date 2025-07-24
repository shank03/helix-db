use std::ops::Deref;
use std::{borrow::Cow, collections::HashMap, marker::PhantomData, path::Path};

#[cfg(feature = "lmdb")]
use heed3::EnvOpenOptions;
use heed3::WithoutTls;
#[cfg(feature = "lmdb")]
use heed3::{AnyTls, WithTls};
use serde::{Deserialize, de::DeserializeOwned};

use crate::helix_engine::bm25::bm25::HBM25Config;
#[cfg(feature = "rocksdb")]
use crate::helix_engine::storage_core::engine_wrappers::rocksdb_wrapper::{Bytes, U128};

#[cfg(feature = "lmdb")]
use crate::helix_engine::storage_core::engine_wrappers::lmdb_wrapper::{Bytes, U128};

use crate::helix_engine::vector_core::vector_core::VectorCore;
use crate::protocol::item_serdes::ItemSerdes;
use crate::{
    helix_engine::{storage_core::storage_core::StorageConfig, types::GraphError},
    utils::{
        items::{Edge, Node},
        label_hash::hash_label,
    },
};

#[cfg(feature = "lmdb")]
use heed3::{Database, DatabaseFlags, RoTxn, RwTxn, byteorder::BE, types::*};

pub const DB_NODES: &str = "nodes"; // for node data (n:)
pub const DB_EDGES: &str = "edges"; // for edge data (e:)
pub const DB_OUT_EDGES: &str = "out_edges"; // for outgoing edge indices (o:)
pub const DB_IN_EDGES: &str = "in_edges"; // for incoming edge indices (i:)

pub trait Txn<'a>: AsRef<Self::TxnType> {
    type TxnType;
    fn get_txn(&'a self) -> &'a Self::TxnType;

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
impl<'a, I: Iterator> Iterator for HelixIterator<'a, I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
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
    pub txn: rocksdb::Transaction<'env, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    #[cfg(feature = "lmdb")]
    pub txn: heed3::RwTxn<'env>,
    #[cfg(feature = "in_memory")]
    pub txn: skipdb::optimistic::OptimisticTransaction<&'a [u8], &'a [u8]>,
}

pub trait Storage<'a, Ro, Rw> {
    type Key;
    type Value;
    type BasicIter: Iterator;
    type PrefixIter: Iterator;
    type DuplicateIter: Iterator;

    fn get_data<'tx, T>(
        &self,
        txn: &'tx T,
        key: Self::Key,
    ) -> Result<Option<Cow<'a, [u8]>>, GraphError>
    where
        T: AsRef<Ro>;

    fn get_and_decode_data<'tx, T, D: ItemSerdes>(
        &self,
        txn: &'tx T,
        key: Self::Key,
    ) -> Result<Option<D>, GraphError>
    where
        T: AsRef<Ro>;

    fn put_data<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError>;
    fn put_data_with_duplicate<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError>;
    fn put_data_in_order<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError>;
    fn delete_data<'tx>(&self, txn: &'a mut WTxn<'tx>, key: Self::Key) -> Result<(), GraphError>;
    fn delete_duplicate<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError>;
    fn iter_data<'tx, T: AsRef<heed3::RoTxn<'tx>>>(
        &self,
        txn: &'tx T,
    ) -> Result<HelixIterator<'a, Self::BasicIter>, GraphError>
    where
        'tx: 'a;
    fn prefix_iter_data<'tx, T: AsRef<heed3::RoTxn<'tx>>>(
        &self,
        txn: &'tx T,
        prefix: Self::Key,
    ) -> Result<HelixIterator<'a, Self::PrefixIter>, GraphError>
    where
        T: AsRef<heed3::RoTxn<'tx>>,
        'tx: 'a;
    fn get_duplicate_data<'tx, T: AsRef<heed3::RoTxn<'tx>>>(
        &self,
        txn: &'tx T,
        key: Self::Key,
    ) -> Result<HelixIterator<'a, Self::DuplicateIter>, GraphError>
    where
        T: AsRef<heed3::RoTxn<'tx>>,
        'tx: 'a;
}

pub struct Table<K, V> {
    #[cfg(feature = "rocksdb")]
    pub table: rocksdb::ColumnFamilyRef<'static>,
    #[cfg(feature = "lmdb")]
    pub table: heed3::Database<K, V>,
    #[cfg(feature = "in_memory")]
    pub table: skipdb::DB<K>,
    pub _phantom: PhantomData<(K, V)>,
}

impl<K, V> Table<K, V> {
    #[cfg(feature = "lmdb")]
    pub fn new_lmdb(table: heed3::Database<K, V>) -> Table<K, V> {
        Table {
            table,
            _phantom: PhantomData,
        }
    }

    #[cfg(feature = "rocksdb")]
    pub fn new_rocksdb(table: rocksdb::ColumnFamilyRef<'t>) -> Table<'t, K, V> {
        Table {
            table,
            _phantom: PhantomData,
        }
    }

    #[cfg(feature = "in_memory")]
    pub fn new_in_memory(table: skipdb::DB<K, V>) -> Table<'a, K, V> {
        Table {
            table,
            _phantom: PhantomData,
        }
    }
}

pub struct HelixEnv {
    #[cfg(feature = "lmdb")]
    pub env: heed3::Env<WithTls>,
    #[cfg(feature = "rocksdb")]
    pub env: rocksdb::TransactionDB<rocksdb::SingleThreaded>,
    #[cfg(feature = "in_memory")]
    pub env: skipdb::DB<K, V>,
}

impl HelixEnv {
    #[cfg(feature = "lmdb")]
    pub fn new_lmdb(env: heed3::Env<WithTls>) -> HelixEnv {
        HelixEnv { env }
    }

    #[cfg(feature = "rocksdb")]
    pub fn new_rocksdb(env: rocksdb::TransactionDB<rocksdb::SingleThreaded>) -> HelixEnv {
        HelixEnv { env }
    }
}

pub struct HelixDB {
    pub storage_config: StorageConfig,
    pub nodes_db: Table<U128, Bytes>,
    pub edges_db: Table<U128, Bytes>,
    pub out_edges_db: Table<Bytes, Bytes>,
    pub in_edges_db: Table<Bytes, Bytes>,
    pub secondary_indices: HashMap<String, Table<Bytes, U128>>,
    pub vectors: VectorCore,
    pub bm25: Option<HBM25Config>,
    pub env: HelixEnv,
}

pub trait HelixDBMethods: Sized {
    // #[cfg(feature = "rocksdb")]
    // fn config() -> rocksdb::Options;
    // #[cfg(feature = "rocksdb")]
    // fn new(path: &str, opts: rocksdb::Options) -> Result<Self, GraphError>;

    fn read_txn(&self) -> Result<RTxn, GraphError>;
    fn write_txn(&self) -> Result<WTxn, GraphError>;

    fn nodes_db(&self) -> &Table<U128, Bytes>;
    fn edges_db(&self) -> &Table<U128, Bytes>;
    fn out_edges_db(&self) -> &Table<Bytes, Bytes>;
    fn in_edges_db(&self) -> &Table<Bytes, Bytes>;
    // fn secondary_indices(&self) -> HashMap<String, HelixTable<Bytes, U128>>;
}

impl HelixDBMethods for HelixDB {
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
