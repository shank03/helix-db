use crate::helix_engine::{
    traversal_core::{traversal_value::TraversalValue, traversal_iter::RoTraversalIterator},
    storage_core::HelixGraphStorage,
    types::GraphError,
};
use heed3::RoTxn;
use std::sync::Arc;
use helix_macros::debug_trace;

pub struct ToVIterator<'a, I, T> {
    iter: I,
    storage: Arc<HelixGraphStorage>,
    txn: &'a T,
}

// implementing iterator for OutIterator
impl<'a, I> Iterator for ToVIterator<'a, I, RoTxn<'a>>
where
    I: Iterator<Item = Result<TraversalValue, GraphError>>,
{
    type Item = Result<TraversalValue, GraphError>;

    #[debug_trace("TO_V")]
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(item) => match item {
                Ok(TraversalValue::Edge(item)) => Some(Ok(TraversalValue::Vector(
                    match self.storage.get_vector(self.txn, &item.to_node) {
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
pub trait ToVAdapter<'a, T>: Iterator<Item = Result<TraversalValue, GraphError>> {
    fn to_v(
        self,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>>;
}

impl<'a, I: Iterator<Item = Result<TraversalValue, GraphError>>> ToVAdapter<'a, RoTxn<'a>>
    for RoTraversalIterator<'a, I>
{
    #[inline(always)]
    fn to_v(
        self,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>> {
        let iter = ToVIterator {
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
