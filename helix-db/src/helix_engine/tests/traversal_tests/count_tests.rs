use std::sync::Arc;

use crate::{
    helix_engine::{
        storage_core::HelixGraphStorage,
        traversal_core::{
            ops::{
                g::G,
                out::out::OutAdapter,
                source::{
                    add_e::{AddEAdapter, EdgeType},
                    add_n::AddNAdapter,
                    n_from_id::NFromIdAdapter,
                    n_from_type::NFromTypeAdapter,
                },
                util::range::RangeAdapter,
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
fn test_count_single_node() {
    let (storage, _temp_dir) = setup_test_db();
    let mut txn = storage.graph_env.write_txn().unwrap();
    let person = G::new_mut(Arc::clone(&storage), &mut txn)
        .add_n("person", Some(props!()), None, None)
        .collect_to::<Vec<_>>();
    let person = person.first().unwrap();
    txn.commit().unwrap();
    let txn = storage.graph_env.read_txn().unwrap();
    let count = G::new(Arc::clone(&storage), &txn)
        .n_from_id(&person.id())
        .count();

    assert_eq!(count, 1);
}

#[test]
fn test_count_node_array() {
    let (storage, _temp_dir) = setup_test_db();
    let mut txn = storage.graph_env.write_txn().unwrap();
    let _ = G::new_mut(Arc::clone(&storage), &mut txn)
        .add_n("person", Some(props!()), None, None)
        .collect_to::<Vec<_>>();
    let _ = G::new_mut(Arc::clone(&storage), &mut txn)
        .add_n("person", Some(props!()), None, None)
        .collect_to::<Vec<_>>();
    let _ = G::new_mut(Arc::clone(&storage), &mut txn)
        .add_n("person", Some(props!()), None, None)
        .collect_to::<Vec<_>>();

    txn.commit().unwrap();
    let txn = storage.graph_env.read_txn().unwrap();
    let count = G::new(Arc::clone(&storage), &txn)
        .n_from_type("person") // Get all nodes
        .count();
    assert_eq!(count, 3);
}

#[test]
fn test_count_mixed_steps() {
    let (storage, _temp_dir) = setup_test_db();
    let mut txn = storage.graph_env.write_txn().unwrap();

    // Create a graph with multiple paths
    let person1 = G::new_mut(Arc::clone(&storage), &mut txn)
        .add_n("person", Some(props!()), None, None)
        .collect_to::<Vec<_>>();
    let person1 = person1.first().unwrap();
    let person2 = G::new_mut(Arc::clone(&storage), &mut txn)
        .add_n("person", Some(props!()), None, None)
        .collect_to::<Vec<_>>();
    let person2 = person2.first().unwrap();
    let person3 = G::new_mut(Arc::clone(&storage), &mut txn)
        .add_n("person", Some(props!()), None, None)
        .collect_to::<Vec<_>>();
    let person3 = person3.first().unwrap();

    G::new_mut(Arc::clone(&storage), &mut txn)
        .add_e(
            "knows",
            Some(props!()),
            person1.id(),
            person2.id(),
            false,
            EdgeType::Node,
            None,
        )
        .collect_to::<Vec<_>>();
    G::new_mut(Arc::clone(&storage), &mut txn)
        .add_e(
            "knows",
            Some(props!()),
            person1.id(),
            person3.id(),
            false,
            EdgeType::Node,
            None,
        )
        .collect_to::<Vec<_>>();
    txn.commit().unwrap();
    println!("person1: {person1:?},\nperson2: {person2:?},\nperson3: {person3:?}");

    let txn = storage.graph_env.read_txn().unwrap();
    let count = G::new(Arc::clone(&storage), &txn)
        .n_from_id(&person1.id())
        .out("knows", &EdgeType::Node)
        .count();

    assert_eq!(count, 2);
}

#[test]
fn test_count_empty() {
    let (storage, _temp_dir) = setup_test_db();
    let txn = storage.graph_env.read_txn().unwrap();
    let count = G::new(Arc::clone(&storage), &txn)
        .n_from_type("person") // Get all nodes
        .range(0, 0) // Take first 3 nodes
        .count();

    assert_eq!(count, 0);
}
