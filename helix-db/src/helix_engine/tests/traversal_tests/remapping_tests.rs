use std::{collections::HashMap, sync::Arc};

use crate::{
    exclude_field,
    helix_engine::{
        storage_core::HelixGraphStorage,
        traversal_core::{
            ops::{
                g::G,
                source::{add_n::AddNAdapter, n_from_type::NFromTypeAdapter},
                util::map::MapAdapter,
            },
            traversal_value::{Traversable, TraversalValue},
        },
        types::GraphError,
    },
    props,
    protocol::{
        remapping::{Remapping, RemappingMap, ResponseRemapping},
        return_values::ReturnValue,
    },
};

use serde::{Deserialize, Serialize};
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
fn test_exclude_field_remapping() {
    let (storage, _temp_dir) = setup_test_db();
    let mut txn = storage.graph_env.write_txn().unwrap();

    let _node = G::new_mut(Arc::clone(&storage), &mut txn)
        .add_n(
            "person",
            Some(props! { "text" => "test", "other" => "other" }),
            None,
        )
        .collect_to_val();

    let traversal = G::new(Arc::clone(&storage), &txn)
        .n_from_type("person")
        .collect_to::<Vec<_>>();

    let remapping_vals = RemappingMap::new();

    let mut return_vals: HashMap<String, ReturnValue> = HashMap::new();
    return_vals.insert(
        "files".to_string(),
        ReturnValue::from_traversal_value_array_with_mixin(
            G::new_from(Arc::clone(&storage), &txn, traversal.clone())
                .map_traversal(|item, _txn| {
                    println!("item: {item:?}");
                    exclude_field!(remapping_vals, item.clone(), "text")?;
                    Ok(item)
                })
                .collect_to::<Vec<_>>()
                .clone(),
            remapping_vals.borrow_mut(),
        ),
    );

    assert_eq!(return_vals.len(), 1);

    #[derive(Serialize, Deserialize)]
    struct Test {
        text: Option<String>,
        other: Option<String>,
    }
    #[derive(Serialize, Deserialize)]
    struct Response {
        files: Vec<Test>,
    }
    let value = sonic_rs::to_vec(&return_vals).unwrap();
    let value: Response = sonic_rs::from_slice(&value).unwrap();
    let value = value.files.first().unwrap();

    let expected = Test {
        text: None,
        other: Some("other".to_string()),
    };

    assert_eq!(value.text, expected.text);
    assert_eq!(value.other, expected.other);
}
