#[cfg(feature = "rocksdb")]
use std::borrow::Cow;
use std::{collections::HashMap, marker::PhantomData};

use serde::{Deserialize, de::DeserializeOwned};

use crate::helix_engine::types::GraphError;

pub(crate) trait Txn {
    fn commit(self) -> Result<(), GraphError>;
    fn abort(self) -> Result<(), GraphError>;
}

pub(crate) struct HelixIterator<'a, K, V, M> {
    #[cfg(feature = "rocksdb")]
    pub iter: rocksdb::DBIteratorWithThreadMode<
        'a,
        rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    >,
    #[cfg(feature = "lmdb")]
    pub iter: heed3::RoIter<'a, Bytes, heed3::types::LazyDecode<Bytes>, M>,
    #[cfg(feature = "in_memory")]
    pub iter: skipdb::Iter<'a, K, V>,
    _phantom: PhantomData<(K, V, M)>,
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

impl<'a> Txn for RTxn<'a> {
    fn commit(self) -> Result<(), GraphError> {
        #[cfg(feature = "rocksdb")]
        self.txn.commit().map_err(|e| GraphError::from(e))
    }

    #[cfg(feature = "rocksdb")]
    fn abort(self) -> Result<(), GraphError> {
        self.txn.rollback().map_err(|e| GraphError::from(e))
    }

    #[cfg(feature = "lmdb")]
    fn abort(self) -> Result<(), GraphError> {
        self.txn.abort().map_err(|e| GraphError::from(e))
    }

    #[cfg(feature = "in_memory")]
    fn abort(self) -> Result<(), GraphError> {
        Ok(())
    }
}

pub trait Storage<'a> {
    fn get(&self, txn: &'a RTxn<'a>, key: &[u8]) -> Result<Option<Vec<u8>>, GraphError>;

    fn put(&self, txn: &mut WTxn, key: &[u8], value: &[u8]) -> Result<(), GraphError>;
    fn delete(&self, txn: &mut WTxn, key: &[u8]) -> Result<(), GraphError>;
    fn iter<M>(
        &self,
        txn: &'a RTxn<'a>,
    ) -> Result<HelixIterator<&'a [u8], &'a [u8], M>, GraphError>;
}

impl<'a> Storage<'a> for rocksdb::ColumnFamilyRef<'a> {
    fn get(&self, txn: &'a RTxn<'a>, key: &[u8]) -> Result<Option<Vec<u8>>, GraphError> {
        match txn
            .txn
            .get_pinned_cf(self, key) // TODO: Use a generic function to convert to bytes
            .map_err(|e| GraphError::from(e))
        {
            Ok(Some(value)) => Ok(Some(value.to_vec())),
            Ok(None) => Ok(None),
            Err(e) => Err(GraphError::from(e)),
        }
    }

    fn put(&self, txn: &mut WTxn, key: &[u8], value: &[u8]) -> Result<(), GraphError> {
        txn.txn
            .put_cf(self, key, value)
            .map_err(|e| GraphError::from(e))
    }

    fn delete(&self, txn: &mut WTxn, key: &[u8]) -> Result<(), GraphError> {
        txn.txn
            .delete_cf(self, key)
            .map_err(|e| GraphError::from(e))
    }

    fn iter<M>(
        &self,
        txn: &'a RTxn<'a>,
    ) -> Result<HelixIterator<&'a [u8], &'a [u8], M>, GraphError> {
        Ok(HelixIterator {
            iter: txn.txn.iterator_cf(self, rocksdb::IteratorMode::Start),
            _phantom: PhantomData,
        })
    }
}

pub struct Table<'a, K, V> {
    #[cfg(feature = "rocksdb")]
    pub table: rocksdb::ColumnFamilyRef<'a>,
    #[cfg(feature = "lmdb")]
    pub table: heed3::Database<K, V>,
    #[cfg(feature = "in_memory")]
    pub table: skipdb::DB<K, V>,
    _phantom: PhantomData<(K, V)>,
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

const NODES_DB: &str = "nodes";
const EDGES_DB: &str = "edges";
const INDICES_DB: &str = "indices";
const OUT_EDGES_DB: &str = "out_edges";
const IN_EDGES_DB: &str = "in_edges";

impl Database for rocksdb::TransactionDB<rocksdb::SingleThreaded> {
    fn read_txn(&self) -> RTxn {
        RTxn {
            txn: self.transaction(),
        }
    }

    fn write_txn(&self) -> WTxn {
        WTxn {
            txn: self.transaction(),
        }
    }

    fn nodes_db(&self) -> Result<Table<U128, Bytes>, GraphError> {
        let cf = self
            .cf_handle(NODES_DB)
            .ok_or(GraphError::TableNotFound(NODES_DB))?;
        Ok(Table {
            table: cf,
            _phantom: PhantomData,
        })
    }

    fn edges_db(&self) -> Result<Table<U128, Bytes>, GraphError> {
        let cf = self
            .cf_handle(EDGES_DB)
            .ok_or(GraphError::TableNotFound(EDGES_DB))?;
        Ok(Table {
            table: cf,
            _phantom: PhantomData,
        })
    }

    fn indices_db(&self) -> Result<Table<Bytes, U128>, GraphError> {
        let cf = self
            .cf_handle(INDICES_DB)
            .ok_or(GraphError::TableNotFound(INDICES_DB))?;
        Ok(Table {
            table: cf,
            _phantom: PhantomData,
        })
    }

    fn out_edges_db(&self) -> Result<Table<Bytes, Bytes>, GraphError> {
        let cf = self
            .cf_handle(OUT_EDGES_DB)
            .ok_or(GraphError::TableNotFound(OUT_EDGES_DB))?;
        Ok(Table {
            table: cf,
            _phantom: PhantomData,
        })
    }

    fn in_edges_db(&self) -> Result<Table<Bytes, Bytes>, GraphError> {
        let cf = self
            .cf_handle(IN_EDGES_DB)
            .ok_or(GraphError::TableNotFound(IN_EDGES_DB))?;
        Ok(Table {
            table: cf,
            _phantom: PhantomData,
        })
    }

    fn config() -> rocksdb::Options {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);

        opts.create_missing_column_families(true);
        opts.increase_parallelism(num_cpus::get() as i32);
        opts.set_max_background_jobs(8);
        // opts.set_compaction_style(DBCompactionStyle::);

        // Write path optimizations
        opts.set_write_buffer_size(256 * 1024 * 1024); // 256MB write buffer
        opts.set_max_write_buffer_number(4);
        opts.set_min_write_buffer_number_to_merge(2);
        opts.set_level_zero_file_num_compaction_trigger(4);
        opts.set_level_zero_slowdown_writes_trigger(20);
        opts.set_level_zero_stop_writes_trigger(36);

        // Configure compaction
        opts.set_disable_auto_compactions(false);
        opts.set_target_file_size_base(64 * 1024 * 1024); // 64MB
        opts.set_target_file_size_multiplier(1);
        opts.set_max_bytes_for_level_base(512 * 1024 * 1024); // 512MB
        opts.set_max_bytes_for_level_multiplier(8.0);

        opts.set_compaction_style(rocksdb::DBCompactionStyle::Level);

        // Optimize level-based compaction
        opts.set_level_compaction_dynamic_level_bytes(true);

        // Increase read performance at cost of space
        opts.set_optimize_filters_for_hits(true);
        opts.set_prefix_extractor(rocksdb::SliceTransform::create_fixed_prefix(8));

        opts
    }
    fn new(path: &str, opts: rocksdb::Options) -> Result<Self, GraphError> {
        // Setup column families with specific options
        let mut node_opts = rocksdb::Options::default();
        let mut edge_opts = rocksdb::Options::default();
        let mut index_opts = rocksdb::Options::default();

        // Node CF optimizations
        let node_cache = rocksdb::Cache::new_lru_cache(1 * 1024 * 1024 * 1024); // 4GB cache
        let mut node_block_opts = rocksdb::BlockBasedOptions::default();
        node_block_opts.set_block_cache(&node_cache);
        node_block_opts.set_block_size(32 * 1024); // 32KB blocks
        node_block_opts.set_cache_index_and_filter_blocks(true);
        node_block_opts.set_bloom_filter(10.0, false);
        node_opts.set_block_based_table_factory(&node_block_opts);

        // Edge CF optimizations
        let edge_cache = rocksdb::Cache::new_lru_cache(2 * 1024 * 1024 * 1024); // 8GB cache
        let mut edge_block_opts = rocksdb::BlockBasedOptions::default();
        edge_block_opts.set_block_cache(&edge_cache);
        edge_block_opts.set_block_size(64 * 1024); // 64KB blocks
        edge_block_opts.set_cache_index_and_filter_blocks(true);
        edge_block_opts.set_bloom_filter(10.0, false);
        edge_opts.set_block_based_table_factory(&edge_block_opts);

        // Index CF optimizations (for edge indices)
        let index_cache = rocksdb::Cache::new_lru_cache(1 * 1024 * 1024 * 1024); // 2GB cache
        let mut index_block_opts = rocksdb::BlockBasedOptions::default();
        index_block_opts.set_block_cache(&index_cache);
        index_block_opts.set_block_size(16 * 1024); // 16KB blocks
        index_block_opts.set_cache_index_and_filter_blocks(true);
        index_block_opts.set_bloom_filter(10.0, false);
        index_opts.set_block_based_table_factory(&index_block_opts);

        let cf_descriptors: Vec<rocksdb::ColumnFamilyDescriptor> = vec![
            // rocksdb::ColumnFamilyDescriptor::new(CF_NODES, node_opts),
            // rocksdb::ColumnFamilyDescriptor::new(CF_EDGES, edge_opts),
            // rocksdb::ColumnFamilyDescriptor::new(CF_INDICES, index_opts),
        ];

        let txn_opts = rocksdb::TransactionDBOptions::default();
        let db: rocksdb::TransactionDB<rocksdb::SingleThreaded> =
            match rocksdb::TransactionDB::open_cf_descriptors(
                &opts,
                &txn_opts,
                path,
                cf_descriptors,
            ) {
                Ok(db) => db,
                Err(err) => return Err(GraphError::from(err)),
            };

        // TODO: Set options for each CF (can't do with TransactionDB)
        // let cf_edges = db
        //     .cf_handle("edges") // TODO: Change to const
        //     .ok_or_else(|| GraphError::from("Column Family not found"))?;
        // db.set_options_cf(
        //     &cf_edges,
        //     &[
        //         ("level0_file_num_compaction_trigger", "2"),
        //         ("level0_slowdown_writes_trigger", "20"),
        //         ("level0_stop_writes_trigger", "36"),
        //         ("target_file_size_base", "67108864"), // 64MB
        //         ("max_bytes_for_level_base", "536870912"), // 512MB
        //         ("write_buffer_size", "67108864"),     // 64MB
        //         ("max_write_buffer_number", "2"),
        //     ],
        // )?;

        // drop(cf_edges);

        Ok(db)
    }
}

#[cfg(feature = "rocksdb")]
pub enum U128 {}
#[cfg(feature = "lmdb")]
pub type U128 = heed3::types::U128<heed3::byteorder::BE>;
#[cfg(feature = "in_memory")]
pub enum U128 {}

#[cfg(feature = "rocksdb")]
pub enum Bytes {}
#[cfg(feature = "lmdb")]
pub type Bytes = heed3::types::Bytes;
#[cfg(feature = "in_memory")]
pub enum Bytes {}
