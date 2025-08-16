use crate::{
    helix_engine::{
        graph_core::{traversal_value::TraversalValue, traversal_iter::RoTraversalIterator},
        types::GraphError,
    },
    utils::items::Edge,
};
use heed3::{
    byteorder::BE,
    types::{Bytes, U128},
};
use helix_macros::debug_trace;

pub struct EFromType<'a> {
    pub iter: heed3::RoIter<'a, U128<BE>, heed3::types::LazyDecode<Bytes>>,
    pub label: &'a str,
}

impl<'a> Iterator for EFromType<'a> {
    type Item = Result<TraversalValue, GraphError>;

    #[debug_trace("E_FROM_TYPE")]
    fn next(&mut self) -> Option<Self::Item> {
        for value in self.iter.by_ref() {
            let (key, value) = value.unwrap();
            match value.decode() {
                Ok(value) => match Edge::decode_edge(value, key) {
                    Ok(edge) => match &edge.label {
                        label if label == self.label => return Some(Ok(TraversalValue::Edge(edge))),
                        _ => continue,
                    },
                    Err(e) => return Some(Err(GraphError::ConversionError(e.to_string()))),
                },
                Err(e) => return Some(Err(GraphError::ConversionError(e.to_string()))),
            }
        }
        None
    }
}
pub trait EFromTypeAdapter<'a>: Iterator<Item = Result<TraversalValue, GraphError>> {
    fn e_from_type(
        self,
        label: &'a str,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>>;
}
impl<'a, I: Iterator<Item = Result<TraversalValue, GraphError>>> EFromTypeAdapter<'a>
    for RoTraversalIterator<'a, I>
{
    #[inline]
    fn e_from_type(
        self,
        label: &'a str,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>> {
        let iter = self
            .storage
            .edges_db
            .lazily_decode_data()
            .iter(self.txn)
            .unwrap();
        RoTraversalIterator {
            inner: EFromType { iter, label },
            storage: self.storage,
            txn: self.txn,
        }
    }
}
