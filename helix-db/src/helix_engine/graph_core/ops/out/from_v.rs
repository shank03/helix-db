use crate::helix_engine::{
    graph_core::{ops::tr_val::TraversalVal, traversal_iter::RoTraversalIterator},
    storage_core::storage_core::HelixGraphStorage,
    types::GraphError,
};
use heed3::RoTxn;
use std::sync::Arc;
use tracing::instrument;

pub struct FromVIterator<'a, I, T> {
    iter: I,
    storage: Arc<HelixGraphStorage>,
    txn: &'a T,
}

// implementing iterator for OutIterator
impl<'a, I> Iterator for FromVIterator<'a, I, RoTxn<'a>>
where
    I: Iterator<Item = Result<TraversalVal, GraphError>>,
{
    type Item = Result<TraversalVal, GraphError>;

    #[instrument(skip(self), fields(result), name = "FROM_V")]
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(item) => match item {
                Ok(TraversalVal::Edge(item)) => Some(Ok(TraversalVal::Vector(
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
pub trait FromVAdapter<'a, T>: Iterator<Item = Result<TraversalVal, GraphError>> {
    fn from_v(
        self,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalVal, GraphError>>>;
}

impl<'a, I: Iterator<Item = Result<TraversalVal, GraphError>>> FromVAdapter<'a, RoTxn<'a>>
    for RoTraversalIterator<'a, I>
{
    #[inline(always)]
    fn from_v(
        self,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalVal, GraphError>>> {
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
