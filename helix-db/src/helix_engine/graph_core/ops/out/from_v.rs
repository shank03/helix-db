use crate::helix_engine::{
    graph_core::{traversal_value::TraversalValue, traversal_iter::RoTraversalIterator},
    storage_core::storage_core::HelixGraphStorage,
    types::GraphError,
};
use helix_macros::debug_trace;
use heed3::RoTxn;
use std::sync::Arc;

pub struct FromVIterator<'a, I, T> {
    iter: I,
    storage: Arc<HelixGraphStorage>,
    txn: &'a T,
}

// implementing iterator for OutIterator
impl<'a, I> Iterator for FromVIterator<'a, I, RoTxn<'a>>
where
    I: Iterator<Item = Result<TraversalValue, GraphError>>,
{
    type Item = Result<TraversalValue, GraphError>;

    #[debug_trace("FROM_V")]
    fn next(&mut self) -> Option<Self::Item> {

        match self.iter.next() {
            Some(item) => match item {
                Ok(TraversalValue::Edge(item)) => Some(Ok(TraversalValue::Vector(
                    match self.storage.get_vector(self.txn, &item.from_node) {
                        Ok(vector) => vector,
                        Err(e) => {
                            println!("Error getting vector: {e:?}");
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
pub trait FromVAdapter<'a, T>: Iterator<Item = Result<TraversalValue, GraphError>> {
    fn from_v(
        self,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>>;
}

impl<'a, I: Iterator<Item = Result<TraversalValue, GraphError>>> FromVAdapter<'a, RoTxn<'a>>
    for RoTraversalIterator<'a, I>
{
    #[inline(always)]
    fn from_v(
        self,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>> {
        let iter = FromVIterator {
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
