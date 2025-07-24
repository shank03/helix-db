#[test]
fn test_lmdb_wrapper() {
    use crate::helix_engine::graph_core::config::Config;
    use crate::helix_engine::storage_core::engine_wrapper::{
        HelixDB, HelixDBMethods, Storage, Txn,
    };
    let temp_dir = tempfile::tempdir().unwrap();
    let db = HelixDB::new(temp_dir.path().to_str().unwrap(), Config::default()).unwrap();
    let mut txn = db.write_txn().unwrap();
    let value = b"test_value".to_vec();
    assert!(db.nodes_db.put_data(&mut txn, &2, &value).is_ok());
    txn.commit_txn().unwrap();

    let txn = db.read_txn().unwrap();
    let value = db.nodes_db.get_data(&txn, &2).unwrap().unwrap();
    assert_eq!(*value, *b"test_value");
    txn.abort_txn().unwrap();
}
