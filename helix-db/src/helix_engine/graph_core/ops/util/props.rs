use crate::{helix_engine::{
    graph_core::{
        ops::tr_val::TraversalVal,
        traversal_iter::{RoTraversalIterator, RwTraversalIterator},
    },
    types::GraphError,
}, utils::filterable::Filterable};

pub struct PropsIterator<'a, I> {
    iter: I,
    prop: &'a str,
}

// TODO: get rid of clones in return values
impl<'a, I> Iterator for PropsIterator<'a, I>
where
    I: Iterator<Item = Result<TraversalVal, GraphError>>,
{
    type Item = Result<TraversalVal, GraphError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok(TraversalVal::Node(node))) => match node.check_property(self.prop) {
                Ok(prop) => Some(Ok(TraversalVal::Value(prop.into_owned()))),
                Err(e) => Some(Err(e)),
            },
            Some(Ok(TraversalVal::Edge(edge))) => match edge.check_property(self.prop) {
                Ok(prop) => Some(Ok(TraversalVal::Value(prop.into_owned()))),
                Err(e) => Some(Err(e)),
            },
            Some(Ok(TraversalVal::Vector(vec))) => match vec.check_property(self.prop) {
                Ok(prop) => Some(Ok(TraversalVal::Value(prop.into_owned()))),
                Err(e) => Some(Err(e)),
            },
            _ => None,
        }
    }
}
pub trait PropsAdapter<'a, I>: Iterator<Item = Result<TraversalVal, GraphError>> {
    /// Returns a new iterator which yeilds the value of the property if it exists
    /// 
    /// Given the type checking of the schema there should be no need to return an empty traversal.
    fn check_property(
        self,
        prop: &'a str,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalVal, GraphError>>>;
}

impl<'a, I> PropsAdapter<'a, I> for RoTraversalIterator<'a, I>
where
    I: Iterator<Item = Result<TraversalVal, GraphError>>,
{
    #[inline]
    fn check_property(
        self,
        prop: &'a str,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalVal, GraphError>>> {
        RoTraversalIterator {
            inner: PropsIterator {
                iter: self.inner,
                prop,
            },
            storage: self.storage,
            txn: self.txn,
        }
    }
}

impl<'a, 'b, I> PropsAdapter<'a, I> for RwTraversalIterator<'a, 'b, I>
where
    I: Iterator<Item = Result<TraversalVal, GraphError>>,
    'b: 'a,
{
    #[inline]
    fn check_property(
        self,
        prop: &'a str,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalVal, GraphError>>> {
        RoTraversalIterator {
            inner: PropsIterator {
                iter: self.inner,
                prop,
            },
            storage: self.storage,
            txn: self.txn,
        }
    }
}
