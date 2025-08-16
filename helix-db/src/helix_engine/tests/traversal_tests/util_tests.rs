use std::sync::Arc;

use crate::{
    helix_engine::{
        storage_core::HelixGraphStorage,
        traversal_core::{
            ops::{
                g::G,
                source::{add_n::AddNAdapter, n_from_type::NFromTypeAdapter},
                util::order::OrderByAdapter,
            },
            traversal_value::Traversable,
        },
    },
    props,
};

use tempfile::TempDir;

fn setup_test_db() -> (Arc<HelixGraphStorage>, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    let storage = HelixGraphStorage::new(
        db_path,
        crate::helix_engine::traversal_core::config::Config::default(),
        Default::default(),
    )
    .unwrap();
    (Arc::new(storage), temp_dir)
}

#[test]
fn test_order_by_asc() {
    let (storage, _temp_dir) = setup_test_db();
    let mut txn = storage.graph_env.write_txn().unwrap();

    let node = G::new_mut(Arc::clone(&storage), &mut txn)
        .add_n("person", Some(props! { "age" => 30 }), None)
        .collect_to_val();

    let node2 = G::new_mut(Arc::clone(&storage), &mut txn)
        .add_n("person", Some(props! { "age" => 20 }), None)
        .collect_to_val();

    let node3 = G::new_mut(Arc::clone(&storage), &mut txn)
        .add_n("person", Some(props! { "age" => 10 }), None)
        .collect_to_val();

    txn.commit().unwrap();

    let txn = storage.graph_env.read_txn().unwrap();
    let traversal = G::new(Arc::clone(&storage), &txn)
        .n_from_type("person")
        .order_by_asc("age")
        .collect_to::<Vec<_>>();

    assert_eq!(traversal.len(), 3);
    assert_eq!(traversal[0].id(), node3.id());
    assert_eq!(traversal[1].id(), node2.id());
    assert_eq!(traversal[2].id(), node.id());
}
