#[cfg(feature = "rocksdb")]
use crate::helix_engine::graph_core::config::Config;
#[cfg(feature = "rocksdb")]
use crate::helix_engine::storage_core::engine_wrapper::{HelixDB, HelixEnv};
#[cfg(feature = "rocks")]
use crate::helix_engine::storage_core::engine_wrapper::{HelixDBMethods, Table};
use crate::helix_engine::storage_core::engine_wrapper::{HelixIterator, RTxn, Storage, WTxn};
use crate::helix_engine::types::GraphError;
#[cfg(feature = "rocks")]
use num_cpus;
#[cfg(feature = "rocks")]
use std::borrow::Cow;
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

#[cfg(feature = "rocksdb")]
impl HelixEnv {
    pub fn read_txn(&self) -> Result<RTxn, GraphError> {
        Ok(RTxn {
            txn: self.env.transaction(),
        })
    }

    pub fn write_txn(&self) -> Result<WTxn, GraphError> {
        Ok(WTxn {
            txn: self.env.transaction(),
        })
    }
}

#[cfg(feature = "rocks")]
impl<'a, 't> Storage<'a> for Table<'t, U128, Bytes> {
    type Key = &'a u128;
    type Value = &'a [u8];
    type BasicIter = rocksdb::DBIteratorWithThreadMode<
        'a,
        rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    >;
    type PrefixIter = rocksdb::DBIteratorWithThreadMode<
        'a,
        rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    >;
    type DuplicateIter = rocksdb::DBIteratorWithThreadMode<
        'a,
        rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    >;

    fn get_data<'tx>(
        &self,
        txn: &'a RTxn<'tx>,
        key: Self::Key,
    ) -> Result<Option<Cow<'a, [u8]>>, GraphError> {
        match txn
            .txn
            .get_pinned_cf(&self.table, key.to_be_bytes())
            .map_err(|e| GraphError::from(e))
        {
            Ok(Some(value)) => Ok(Some(Cow::Owned(value.to_vec()))),
            Ok(None) => Ok(None),
            Err(e) => Err(GraphError::from(e)),
        }
    }

    fn put_data<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError> {
        txn.txn
            .put_cf(&self.table, key.to_be_bytes(), value)
            .map_err(|e| GraphError::from(e))
    }

    fn put_data_with_duplicate<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError> {
        txn.txn
            .put_cf(&self.table, key.to_be_bytes(), value)
            .map_err(|e| GraphError::from(e))
    }

    fn put_data_in_order<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError> {
        txn.txn
            .put_cf(&self.table, key.to_be_bytes(), value)
            .map_err(|e| GraphError::from(e))
    }

    fn delete_data<'tx>(&self, txn: &'a mut WTxn<'tx>, key: Self::Key) -> Result<(), GraphError> {
        txn.txn
            .delete_cf(&self.table, key.to_be_bytes())
            .map_err(|e| GraphError::from(e))
    }

    fn delete_duplicate<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        _value: Self::Value,
    ) -> Result<(), GraphError> {
        txn.txn
            .delete_cf(&self.table, key.to_be_bytes())
            .map_err(|e| GraphError::from(e))
    }

    fn iter_data<'tx>(
        &self,
        txn: &'a RTxn<'tx>,
    ) -> Result<HelixIterator<'a, Self::BasicIter>, GraphError> {
        Ok(HelixIterator {
            iter: txn
                .txn
                .iterator_cf(&self.table, rocksdb::IteratorMode::Start),
            _phantom: PhantomData,
        })
    }

    fn prefix_iter_data<'tx>(
        &self,
        txn: &'a RTxn<'tx>,
        prefix: Self::Key,
    ) -> Result<HelixIterator<'a, Self::PrefixIter>, GraphError> {
        Ok(HelixIterator {
            iter: txn
                .txn
                .prefix_iterator_cf(&self.table, prefix.to_be_bytes()),
            _phantom: PhantomData,
        })
    }

    fn get_duplicate_data<'tx>(
        &self,
        txn: &'a RTxn<'tx>,
        key: Self::Key,
    ) -> Result<HelixIterator<'a, Self::DuplicateIter>, GraphError> {
        Ok(HelixIterator {
            iter: txn.txn.prefix_iterator_cf(&self.table, key.to_be_bytes()),
            _phantom: PhantomData,
        })
    }
}

#[cfg(feature = "rocks")]
impl<'a, 't> Storage<'a> for Table<'t, Bytes, Bytes> {
    type Key = &'a [u8];
    type Value = &'a [u8];
    type BasicIter = rocksdb::DBIteratorWithThreadMode<
        'a,
        rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    >;
    type PrefixIter = rocksdb::DBIteratorWithThreadMode<
        'a,
        rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    >;
    type DuplicateIter = rocksdb::DBIteratorWithThreadMode<
        'a,
        rocksdb::Transaction<'a, rocksdb::TransactionDB<rocksdb::SingleThreaded>>,
    >;

    fn get_data<'tx>(
        &self,
        txn: &'a RTxn<'tx>,
        key: Self::Key,
    ) -> Result<Option<Cow<'a, [u8]>>, GraphError> {
        match txn
            .txn
            .get_pinned_cf(&self.table, key) // TODO: Use a generic function to convert to bytes
            .map_err(|e| GraphError::from(e))
        {
            Ok(Some(value)) => Ok(Some(Cow::Owned(value.to_vec()))),
            Ok(None) => Ok(None),
            Err(e) => Err(GraphError::from(e)),
        }
    }

    fn put_data<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError> {
        txn.txn
            .put_cf(&self.table, key, value)
            .map_err(|e| GraphError::from(e))
    }

    fn put_data_with_duplicate<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError> {
        txn.txn
            .put_cf(&self.table, key, value)
            .map_err(|e| GraphError::from(e))
    }

    fn put_data_in_order<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        value: Self::Value,
    ) -> Result<(), GraphError> {
        txn.txn
            .put_cf(&self.table, key, value)
            .map_err(|e| GraphError::from(e))
    }

    fn delete_data<'tx>(&self, txn: &'a mut WTxn<'tx>, key: Self::Key) -> Result<(), GraphError> {
        txn.txn
            .delete_cf(&self.table, key)
            .map_err(|e| GraphError::from(e))
    }

    fn delete_duplicate<'tx>(
        &self,
        txn: &'a mut WTxn<'tx>,
        key: Self::Key,
        _value: Self::Value,
    ) -> Result<(), GraphError> {
        txn.txn
            .delete_cf(&self.table, key)
            .map_err(|e| GraphError::from(e))
    }

    fn iter_data<'tx>(
        &self,
        txn: &'a RTxn<'tx>,
    ) -> Result<HelixIterator<'a, Self::BasicIter>, GraphError> {
        Ok(HelixIterator {
            iter: txn
                .txn
                .iterator_cf(&self.table, rocksdb::IteratorMode::Start),
            _phantom: PhantomData,
        })
    }

    fn prefix_iter_data<'tx>(
        &self,
        txn: &'a RTxn<'tx>,
        prefix: Self::Key,
    ) -> Result<HelixIterator<'a, Self::PrefixIter>, GraphError> {
        Ok(HelixIterator {
            iter: txn.txn.prefix_iterator_cf(&self.table, prefix),
            _phantom: PhantomData,
        })
    }

    fn get_duplicate_data<'tx>(
        &self,
        txn: &'a RTxn<'tx>,
        key: Self::Key,
    ) -> Result<HelixIterator<'a, Self::DuplicateIter>, GraphError> {
        Ok(HelixIterator {
            iter: txn.txn.prefix_iterator_cf(&self.table, key),
            _phantom: PhantomData,
        })
    }
}

#[cfg(feature = "rocksdb")]
impl<'t> HelixDB<'t> {
    pub fn new(path: &str, config: Config) -> Result<Self, GraphError> {
        use std::collections::HashMap;

        use crate::helix_engine::storage_core::{
            engine_wrapper::{DB_EDGES, DB_IN_EDGES, DB_NODES, DB_OUT_EDGES, HelixEnv},
            storage_core::StorageConfig,
        };

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
        // Setup column families with specific options
        let mut node_opts = rocksdb::Options::default();
        let mut edge_opts = rocksdb::Options::default();
        let mut out_edges_opts = rocksdb::Options::default();
        let mut in_edges_opts = rocksdb::Options::default();

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

        // Out Edges CF optimizations (for out_edges indices)
        let out_edges_cache = rocksdb::Cache::new_lru_cache(1 * 1024 * 1024 * 1024); // 2GB cache
        let mut out_edges_block_opts = rocksdb::BlockBasedOptions::default();
        out_edges_block_opts.set_block_cache(&out_edges_cache);
        out_edges_block_opts.set_block_size(16 * 1024); // 16KB blocks
        out_edges_block_opts.set_cache_index_and_filter_blocks(true);
        out_edges_block_opts.set_bloom_filter(10.0, false);
        out_edges_opts.set_block_based_table_factory(&out_edges_block_opts);

        // In Edges CF optimizations (for in_edges indices)
        let in_edges_cache = rocksdb::Cache::new_lru_cache(1 * 1024 * 1024 * 1024); // 2GB cache
        let mut in_edges_block_opts = rocksdb::BlockBasedOptions::default();
        in_edges_block_opts.set_block_cache(&in_edges_cache);
        in_edges_block_opts.set_block_size(16 * 1024); // 16KB blocks
        in_edges_block_opts.set_cache_index_and_filter_blocks(true);
        in_edges_block_opts.set_bloom_filter(10.0, false);
        in_edges_opts.set_block_based_table_factory(&in_edges_block_opts);

        let mut secondary_indices = HashMap::new();
        if let Some(indices) = config.get_graph_config().secondary_indices {
            for index in indices {
                let mut index_opts = rocksdb::Options::default();
                let index_cache = rocksdb::Cache::new_lru_cache(1 * 1024 * 1024 * 1024); // 2GB cache
                let mut index_block_opts = rocksdb::BlockBasedOptions::default();
                index_block_opts.set_block_cache(&index_cache);
                index_block_opts.set_block_size(16 * 1024); // 16KB blocks
                index_block_opts.set_cache_index_and_filter_blocks(true);
                index_block_opts.set_bloom_filter(10.0, false);
                index_opts.set_block_based_table_factory(&index_block_opts);
                secondary_indices.insert(
                    index,
                    rocksdb::ColumnFamilyDescriptor::new(index, index_opts),
                );
            }
        }

        let mut cf_descriptors: Vec<rocksdb::ColumnFamilyDescriptor> = vec![
            rocksdb::ColumnFamilyDescriptor::new(DB_NODES, node_opts),
            rocksdb::ColumnFamilyDescriptor::new(DB_EDGES, edge_opts),
            rocksdb::ColumnFamilyDescriptor::new(DB_OUT_EDGES, out_edges_opts),
            rocksdb::ColumnFamilyDescriptor::new(DB_IN_EDGES, in_edges_opts),
        ];

        cf_descriptors.extend(secondary_indices.into_iter().map(|(_, cf)| cf));

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
        let storage_config = StorageConfig::new(
            config.schema.unwrap_or("".to_string()),
            config.graphvis_node_label,
            config.embedding_model,
        );

        Ok(HelixDB {
            storage_config,
            nodes_db: Table::new_rocksdb(db.cf_handle("nodes").unwrap()),
            edges_db: Table::new_rocksdb(db.cf_handle("edges").unwrap()),
            out_edges_db: Table::new_rocksdb(db.cf_handle("out_edges").unwrap()),
            in_edges_db: Table::new_rocksdb(db.cf_handle("in_edges").unwrap()),
            secondary_indices: secondary_indices
                .into_iter()
                .map(|(_, cf)| {
                    (
                        cf.name().to_string(),
                        Table::new_rocksdb(db.cf_handle(cf.name()).unwrap()),
                    )
                })
                .collect(),
            env: HelixEnv::new_rocksdb(db),
        })
    }
}
