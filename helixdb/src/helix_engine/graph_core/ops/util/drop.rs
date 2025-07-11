use crate::helix_engine::{
    graph_core::ops::tr_val::TraversalVal,
    storage_core::{storage_core::HelixGraphStorage, storage_methods::StorageMethods},
    types::GraphError,
};
use heed3::RwTxn;
use std::sync::Arc;

pub struct Drop<I> {
    pub iter: I,
}

impl<'a> Drop<Vec<Result<TraversalVal, GraphError>>> {
    pub fn drop_traversal(
        iter: Vec<TraversalVal>,
        storage: Arc<HelixGraphStorage>,
        txn: &mut RwTxn,
    ) -> Result<(), GraphError> {
        println!("Dropping traversal {:?}", iter);
        iter.into_iter()
            .try_for_each(|item| -> Result<(), GraphError> {
                match item {
                    TraversalVal::Node(node) => match storage.drop_node(txn, &node.id) {
                        Ok(_) => Ok(println!("Dropped node: {:?}", node.id)),
                        Err(e) => return Err(e),
                    },
                    TraversalVal::Edge(edge) => match storage.drop_edge(txn, &edge.id) {
                        Ok(_) => Ok(()),
                        Err(e) => return Err(e),
                    },
                    TraversalVal::Vector(_) => Ok(()),
                    _ => {
                        return Err(GraphError::ConversionError(format!(
                            "Incorrect Type: {:?}",
                            item
                        )));
                    }
                }
            })
    }
}
