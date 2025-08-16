use crate::helix_engine::{
    traversal_core::{traversal_value::TraversalValue, traversal_iter::RoTraversalIterator},
    storage_core::{HelixGraphStorage, storage_methods::StorageMethods},
    types::GraphError,
};
use helix_macros::debug_trace;
use heed3::RoTxn;
use std::sync::Arc;

pub struct FromNIterator<'a, I, T> {
    iter: I,
    storage: Arc<HelixGraphStorage>,
    txn: &'a T,
}

impl<'a, I> Iterator for FromNIterator<'a, I, RoTxn<'a>>
where
    I: Iterator<Item = Result<TraversalValue, GraphError>>,
{
    type Item = Result<TraversalValue, GraphError>;

    #[debug_trace("FROM_N")]
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(item) => match item {
                Ok(TraversalValue::Edge(item)) => Some(Ok(TraversalValue::Node(
                    match self.storage.get_node(self.txn, &item.from_node) {
                        Ok(node) => node,
                        Err(e) => {
                            println!("Error getting node: {e:?}");
                            return Some(Err(e));
                        }
                    },
                ))),
                _ => return None,
            },
            None => None,
        }
    }
}

pub trait FromNAdapter<'a, T>: Iterator<Item = Result<TraversalValue, GraphError>> {
    /// Returns an iterator containing the nodes that the edges in `self.inner` originate from.
    fn from_n(
        self,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>>;
}

impl<'a, I: Iterator<Item = Result<TraversalValue, GraphError>> + 'a> FromNAdapter<'a, RoTxn<'a>>
    for RoTraversalIterator<'a, I>
{
    #[inline(always)]
    fn from_n(
        self,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>> {
        let iter = FromNIterator {
            iter: self.inner,
            storage: Arc::clone(&self.storage),
            txn: self.txn,
        };
        RoTraversalIterator {
            inner: iter,
            storage: self.storage,
            txn: self.txn,
        }
    }
}
