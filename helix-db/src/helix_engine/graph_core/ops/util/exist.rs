use crate::helix_engine::{graph_core::ops::tr_val::TraversalVal, types::GraphError};

pub struct Exist<I> {
    pub iter: I,
}

impl<'a, I> Exist<I>
where
    I: Iterator<Item = Result<TraversalVal, GraphError>>,
{
    pub fn exists(iter: &mut I) -> bool {
        while let Some(item) = iter.next() {
            match item {
                Ok(_) => return true,
                Err(_) => continue,
            }
        }
        false
    }
}
