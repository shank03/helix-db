use crate::helix_engine::storage_core::engine_wrapper::{HelixDB, HelixDBMethods, Storage, Txn};
use crate::helix_engine::graph_core::config::Config;

#[test]
fn test_lmdb_wrapper() {
    let db = HelixDB::new("test_db", Config::default()).unwrap();
    let mut txn = db.write_txn().unwrap();
    let value = b"test_value".to_vec();
    assert!(db.nodes_db.put_data(&mut txn, &2, &value).is_ok());
    txn.commit_txn().unwrap();
}