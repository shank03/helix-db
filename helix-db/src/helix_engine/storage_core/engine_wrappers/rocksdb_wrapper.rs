#[cfg(feature = "rocks")]
use crate::helix_engine::storage_core::engine_wrapper::{Database, Table};
use crate::helix_engine::storage_core::engine_wrapper::{HelixIterator, RTxn, Storage, WTxn};
use crate::helix_engine::types::GraphError;
#[cfg(feature = "rocks")]
use num_cpus;
use std::marker::PhantomData;

#[cfg(feature = "rocks")]
pub enum U128 {}

#[cfg(feature = "rocks")]
pub enum Bytes {}

#[cfg(feature = "rocksdb")]
impl<'a> RTxn<'a> {
    pub fn get_txn(
        &'a self,
    ) -> &'a rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>> {
        return &self.txn;
    }
}

#[cfg(feature = "rocks")]
impl<'env> Storage for rocksdb::ColumnFamilyRef<'env> {
    type Key<'a> = &'a [u8];
    type Value<'a> = &'a [u8];
    type BasicIter<'a>
        = rocksdb::DBIteratorWithThreadMode<
        'a,
        rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    >
    where
        Self: 'a;
    type PrefixIter<'a>
        = rocksdb::DBIteratorWithThreadMode<
        'a,
        rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    >
    where
        Self: 'a;
    type DuplicateIter<'a>
        = rocksdb::DBIteratorWithThreadMode<
        'a,
        rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    >
    where
        Self: 'a;

    fn get_data<'a>(&self, txn: &'a RTxn<'a>, key: Self::Key<'a>) -> Result<Option<Vec<u8>>, GraphError> {
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

    fn put_data<'a>(
        &self,
        txn: &'a mut WTxn<'a>,
        key: Self::Key<'a>,
        value: Self::Value<'a>,
    ) -> Result<(), GraphError> {
        txn.txn
            .put_cf(self, key, value)
            .map_err(|e| GraphError::from(e))
    }

    fn delete_data<'a>(&self, txn: &'a mut WTxn<'a>, key: &[u8]) -> Result<(), GraphError> {
        txn.txn
            .delete_cf(self, key)
            .map_err(|e| GraphError::from(e))
    }

    fn iter_data<'a>(
        &'a self,
        txn: &'a RTxn<'a>,
    ) -> Result<HelixIterator<'a, Self::BasicIter<'a>>, GraphError> {
        Ok(HelixIterator {
            iter: txn.txn.iterator_cf(self, rocksdb::IteratorMode::Start),
            _phantom: PhantomData,
        })
    }

    fn prefix_iter_data<'a>(
        &'a self,
        txn: &'a RTxn<'a>,
        prefix: Self::Key<'a>,
    ) -> Result<HelixIterator<'a, Self::PrefixIter<'a>>, GraphError> {
        Ok(HelixIterator {
            iter: txn.txn.prefix_iterator_cf(self, prefix),
            _phantom: PhantomData,
        })
    }

    fn get_duplicate_data<'a>(
        &'a self,
        txn: &'a RTxn<'a>,
        key: Self::Key<'a>,
    ) -> Result<HelixIterator<'a, Self::DuplicateIter<'a>>, GraphError> {
        Ok(HelixIterator {
            iter: txn.txn.prefix_iterator_cf(self, key),
            _phantom: PhantomData,
        })
    }
}

#[cfg(feature = "rocksdb")]
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
        use crate::helix_engine::storage_core::engine_wrapper::NODES_DB;

        let cf = self
            .cf_handle(NODES_DB)
            .ok_or(GraphError::TableNotFound(NODES_DB))?;
        Ok(Table {
            table: cf,
            _phantom: PhantomData,
        })
    }

    fn edges_db(&self) -> Result<Table<U128, Bytes>, GraphError> {
        use crate::helix_engine::storage_core::engine_wrapper::EDGES_DB;

        let cf = self
            .cf_handle(EDGES_DB)
            .ok_or(GraphError::TableNotFound(EDGES_DB))?;
        Ok(Table {
            table: cf,
            _phantom: PhantomData,
        })
    }

    fn indices_db(&self) -> Result<Table<Bytes, U128>, GraphError> {
        use crate::helix_engine::storage_core::engine_wrapper::INDICES_DB;

        let cf = self
            .cf_handle(INDICES_DB)
            .ok_or(GraphError::TableNotFound(INDICES_DB))?;
        Ok(Table {
            table: cf,
            _phantom: PhantomData,
        })
    }

    fn out_edges_db(&self) -> Result<Table<Bytes, Bytes>, GraphError> {
        use crate::helix_engine::storage_core::engine_wrapper::OUT_EDGES_DB;

        let cf = self
            .cf_handle(OUT_EDGES_DB)
            .ok_or(GraphError::TableNotFound(OUT_EDGES_DB))?;
        Ok(Table {
            table: cf,
            _phantom: PhantomData,
        })
    }

    fn in_edges_db(&self) -> Result<Table<Bytes, Bytes>, GraphError> {
        use crate::helix_engine::storage_core::engine_wrapper::IN_EDGES_DB;

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
