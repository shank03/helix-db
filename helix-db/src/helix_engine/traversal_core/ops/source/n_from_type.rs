use crate::{
    helix_engine::{
        traversal_core::{traversal_value::TraversalValue, traversal_iter::RoTraversalIterator},
        types::GraphError,
    },
    utils::items::Node,
};
use helix_macros::debug_trace;
use heed3::{
    byteorder::BE,
    types::{Bytes, U128},
};

pub struct NFromType<'a> {
    pub iter: heed3::RoIter<'a, U128<BE>, heed3::types::LazyDecode<Bytes>>,
    pub label: &'a str,
}

impl<'a> Iterator for NFromType<'a> {
    type Item = Result<TraversalValue, GraphError>;

    #[debug_trace("N_FROM_TYPE")]
    fn next(&mut self) -> Option<Self::Item> {
        for value in self.iter.by_ref() {
            let (key_, value) = value.unwrap();
            match value.decode() {
                Ok(value) => match Node::decode_node(value, key_) {
                    Ok(node) => match &node.label {
                        label if label == self.label => return Some(Ok(TraversalValue::Node(node))),
                        _ => continue,
                    },
                    Err(e) => {
                        println!("{} Error decoding node: {:?}", line!(), e);
                        return Some(Err(GraphError::ConversionError(e.to_string())));
                    }
                },
                Err(e) => return Some(Err(GraphError::ConversionError(e.to_string()))),
            }
        }
        None
    }
}
pub trait NFromTypeAdapter<'a>: Iterator<Item = Result<TraversalValue, GraphError>> {
    /// Returns an iterator containing the nodes with the given label.
    ///
    /// Note that the `label` cannot be empty and must be a valid, existing node label.
    fn n_from_type(
        self,
        label: &'a str,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>>;
}
impl<'a, I: Iterator<Item = Result<TraversalValue, GraphError>>> NFromTypeAdapter<'a>
    for RoTraversalIterator<'a, I>
{
    #[inline]
    fn n_from_type(
        self,
        label: &'a str,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>> {
        let iter = self
            .storage
            .nodes_db
            .lazily_decode_data()
            .iter(self.txn)
            .unwrap();
        RoTraversalIterator {
            inner: NFromType { iter, label },
            storage: self.storage,
            txn: self.txn,
        }
    }
}
