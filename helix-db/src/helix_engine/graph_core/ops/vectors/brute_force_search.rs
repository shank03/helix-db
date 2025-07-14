use super::super::tr_val::TraversalVal;
use crate::helix_engine::{
    graph_core::traversal_iter::RoTraversalIterator, types::GraphError,
    vector_core::vector_distance::cosine_similarity,
};
use helix_macros::debug_trace;

pub struct BruteForceSearchV<I: Iterator<Item = Result<TraversalVal, GraphError>>> {
    iter: I,
}

// implementing iterator for OutIterator
impl<I: Iterator<Item = Result<TraversalVal, GraphError>>> Iterator for BruteForceSearchV<I> {
    type Item = Result<TraversalVal, GraphError>;

    #[debug_trace("BRUTE_FORCE_SEARCH_V")]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub trait BruteForceSearchVAdapter<'a>: Iterator<Item = Result<TraversalVal, GraphError>> {
    fn brute_force_search_v(
        self,
        query: &Vec<f64>,
        k: usize,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalVal, GraphError>>>;
}

impl<'a, I: Iterator<Item = Result<TraversalVal, GraphError>> + 'a> BruteForceSearchVAdapter<'a>
    for RoTraversalIterator<'a, I>
{
    fn brute_force_search_v(
        self,
        query: &Vec<f64>,
        k: usize,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalVal, GraphError>>> {
        let mut iter = self.inner.collect::<Vec<_>>();
        println!("iter: {:?}", iter);
        iter = iter
            .into_iter()
            .map(|v| match v {
                Ok(TraversalVal::Vector(mut v)) => {
                    let d = cosine_similarity(&v.get_data(), query).unwrap();
                    v.set_distance(d);
                    Ok(TraversalVal::Vector(v))
                }
                other => {
                    println!("expected vector traversal values, got: {:?}", other);
                    panic!("expected vector traversal values")
                }
            })
            .collect::<Vec<_>>();

        iter.sort_by(|v1, v2| match (v1, v2) {
            (Ok(TraversalVal::Vector(v1)), Ok(TraversalVal::Vector(v2))) => {
                v1.partial_cmp(&v2).unwrap()
            }
            _ => panic!("expected vector traversal values"),
        });

        let iter = iter.into_iter().take(k);

        RoTraversalIterator {
            inner: iter.into_iter(),
            storage: self.storage,
            txn: self.txn,
        }
    }
}
