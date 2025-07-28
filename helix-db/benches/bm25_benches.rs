#[cfg(test)]
mod tests {
    use helix_db::{
        helix_engine::{
            bm25::bm25::{
                BM25Flatten, BM25Metadata, HBM25Config, HybridSearch, BM25, METADATA_KEY,
            },
            graph_core::config::Config,
            storage_core::storage_core::HelixGraphStorage,
            vector_core::{hnsw::HNSW, vector::HVector},
        },
        protocol::value::Value,
        debug_println,
    };

    use heed3::{Env, EnvOpenOptions, RoTxn};
    use std::collections::HashMap;
    use tempfile::tempdir;
    use rand::seq::SliceRandom;

    fn setup_test_env() -> (Env, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(4 * 1024 * 1024 * 1024) // 4GB
                .max_dbs(20)
                .open(path)
                .unwrap()
        };

        (env, temp_dir)
    }

    fn setup_bm25_config() -> (HBM25Config, tempfile::TempDir) {
        let (env, temp_dir) = setup_test_env();
        let mut wtxn = env.write_txn().unwrap();
        let config = HBM25Config::new(&env, &mut wtxn).unwrap();
        wtxn.commit().unwrap();
        (config, temp_dir)
    }

    /// Tests the precision (number of docs returned) of the implemented
    /// bm25 search algorithm
    #[test]
    fn test_bm25_precision() {
        let (bm25, _temp_dir) = setup_bm25_config();
        let mut wtxn = bm25.graph_env.write_txn().unwrap();

        let mut rng = rand::rng();
        let mut docs = vec![];
        let relevant_count = 4000;
        let total_docs = 1_000_000;

        for i in 0..relevant_count {
            let doc = format!("queryterm document {}", i);
            docs.push((i as u128, doc));
        }

        debug_println!("inserted relevant docs");

        for i in relevant_count..total_docs {
            let doc = format!("document {} other words", i);
            docs.push((i as u128, doc));
        }

        docs.shuffle(&mut rng);
        for (doc_id, doc) in &docs {
            bm25.insert_doc(&mut wtxn, *doc_id, doc).unwrap();
        }

        debug_println!("inserted irrelevant docs");

        wtxn.commit().unwrap();

        let rtxn = bm25.graph_env.read_txn().unwrap();
        let results = bm25.search(&rtxn, "queryterm", relevant_count+1).unwrap();

        debug_println!("searched");

        let relevant_retrieved = results
            .iter()
            .filter(|(id, _)| *id < relevant_count as u128)
            .count();
        let precision = relevant_retrieved as f64 / results.len() as f64;

        assert!(precision >= 0.9, "Precision {} below threshold 0.9", precision);
        assert_eq!(relevant_retrieved, relevant_count, "Not all relevant docs retrieved");
    }
}

