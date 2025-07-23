#[cfg(feature = "rocksdb")]
use std::borrow::Cow;
use std::{collections::HashMap, marker::PhantomData};

use serde::{Deserialize, de::DeserializeOwned};

#[cfg(feature = "rocksdb")]
use crate::helix_engine::storage_core::engine_wrappers::rocksdb_wrapper::{Bytes, U128};

#[cfg(feature = "lmdb")]
use crate::helix_engine::storage_core::engine_wrappers::lmdb_wrapper::{Bytes, U128};

use crate::helix_engine::types::GraphError;

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
    pub txn: heed3::RoTxn<'a>,
    #[cfg(feature = "in_memory")]
    pub txn: skipdb::ReadTransaction<
        &'a [u8],
        &'a [u8],
        OptimisticDb<&'a [u8], &'a [u8]>,
        txn_core::sync::HashCm<&'a [u8]>,
    >,
}

pub struct WTxn<'a> {
    #[cfg(feature = "rocksdb")]
    pub txn: rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    #[cfg(feature = "lmdb")]
    pub txn: heed3::RwTxn<'a>,
    #[cfg(feature = "in_memory")]
    pub txn: skipdb::optimistic::OptimisticTransaction<&'a [u8], &'a [u8]>,
}

impl<'a> WTxn<'a> {
    #[cfg(feature = "lmdb")]
    pub fn get_txn(&'a mut self) -> &'a mut heed3::RwTxn<'a> {
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

pub trait Storage {
    type Key<'a>;
    type Value<'a>;
    type BasicIter<'a>: Iterator
    where
        Self: 'a;
    type PrefixIter<'a>: Iterator
    where
        Self: 'a;
    type DuplicateIter<'a>: Iterator
    where
        Self: 'a;

    fn get_data<'a>(&self, txn: &'a RTxn<'a>, key: Self::Key<'a>) -> Result<Option<Vec<u8>>, GraphError>;
    fn put_data<'a>(&self, txn: &'a mut WTxn<'a>, key: Self::Key<'a>, value: Self::Value<'a>)
    -> Result<(), GraphError>;
    fn delete_data<'a>(&self, txn: &'a mut WTxn<'a>, key: Self::Key<'a>) -> Result<(), GraphError>;
    fn iter_data<'a>(
        &'a self,
        txn: &'a RTxn<'a>,
    ) -> Result<HelixIterator<'a, Self::BasicIter<'a>>, GraphError>;
    fn prefix_iter_data<'a>(
        &'a self,
        txn: &'a RTxn<'a>,
        prefix: Self::Key<'a>,
    ) -> Result<HelixIterator<'a, Self::PrefixIter<'a>>, GraphError>;
    fn get_duplicate_data<'a>(
        &'a self,
        txn: &'a RTxn<'a>,
        key: Self::Key<'a>,
    ) -> Result<HelixIterator<'a, Self::DuplicateIter<'a>>, GraphError>;
}

pub struct Table<'a, K, V> {
    #[cfg(feature = "rocksdb")]
    pub table: rocksdb::ColumnFamilyRef<'a>,
    #[cfg(feature = "lmdb")]
    pub table: heed3::Database<K, V>,
    #[cfg(feature = "in_memory")]
    pub table: skipdb::DB<K, V>,
    pub _phantom: PhantomData<(&'a K, &'a V)>,
}

pub struct HelixDB<'a> {
    nodes_db: Table<'a, U128, Bytes>,
    edges_db: Table<'a, U128, Bytes>,
    indices_db: Table<'a, Bytes, U128>,
    out_edges_db: Table<'a, Bytes, Bytes>,
    in_edges_db: Table<'a, Bytes, Bytes>,
    secondary_indices: HashMap<String, Table<'a, Bytes, U128>>,
}

pub trait Database: Sized {
    #[cfg(feature = "rocksdb")]
    fn config() -> rocksdb::Options;
    #[cfg(feature = "rocksdb")]
    fn new(path: &str, opts: rocksdb::Options) -> Result<Self, GraphError>;

    fn read_txn(&self) -> RTxn;
    fn write_txn(&self) -> WTxn;

    fn nodes_db(&self) -> Result<Table<U128, Bytes>, GraphError>;
    fn edges_db(&self) -> Result<Table<U128, Bytes>, GraphError>;
    fn indices_db(&self) -> Result<Table<Bytes, U128>, GraphError>;
    fn out_edges_db(&self) -> Result<Table<Bytes, Bytes>, GraphError>;
    fn in_edges_db(&self) -> Result<Table<Bytes, Bytes>, GraphError>;
    // fn secondary_indices(&self) -> HashMap<String, HelixTable<Bytes, U128>>;
}

pub const NODES_DB: &str = "nodes";
pub const EDGES_DB: &str = "edges";
pub const INDICES_DB: &str = "indices";
pub const OUT_EDGES_DB: &str = "out_edges";
pub const IN_EDGES_DB: &str = "in_edges";

#[cfg(feature = "in_memory")]
pub enum U128 {}

#[cfg(feature = "in_memory")]
pub enum Bytes {}
