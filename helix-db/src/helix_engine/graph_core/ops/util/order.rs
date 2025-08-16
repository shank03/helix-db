use std::cmp::Ordering;

use itertools::Itertools;

use crate::{
    helix_engine::{
        graph_core::{traversal_value::TraversalValue, traversal_iter::RoTraversalIterator},
        types::GraphError,
    },
    utils::filterable::Filterable,
};

pub struct OrderByAsc<I> {
    iter: I,
}

impl<I> Iterator for OrderByAsc<I>
where
    I: Iterator<Item = Result<TraversalValue, GraphError>>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct OrderByDesc<I> {
    iter: I,
}

impl<I> Iterator for OrderByDesc<I>
where
    I: Iterator<Item = Result<TraversalValue, GraphError>>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub trait OrderByAdapter<'a>: Iterator {
    fn order_by_asc(
        self,
        property: &str,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>>;

    fn order_by_desc(
        self,
        property: &str,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>>;
}

impl<'a, I: Iterator<Item = Result<TraversalValue, GraphError>>> OrderByAdapter<'a>
    for RoTraversalIterator<'a, I>
{
    fn order_by_asc(
        self,
        property: &str,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>> {
        RoTraversalIterator {
            inner: OrderByAsc {
                iter: self.inner.sorted_by(|a, b| match (a, b) {
                    (Ok(a), Ok(b)) => match (a, b) {
                        (TraversalValue::Node(a), TraversalValue::Node(b)) => {
                            match (a.check_property(property), b.check_property(property)) {
                                (Ok(val_a), Ok(val_b)) => val_a.cmp(&val_b),
                                (Ok(_), Err(_)) => Ordering::Less,
                                (Err(_), Ok(_)) => Ordering::Greater,
                                (Err(_), Err(_)) => Ordering::Equal,
                            }
                        }
                        (TraversalValue::Edge(a), TraversalValue::Edge(b)) => {
                            match (a.check_property(property), b.check_property(property)) {
                                (Ok(val_a), Ok(val_b)) => val_a.cmp(&val_b),
                                (Ok(_), Err(_)) => Ordering::Less,
                                (Err(_), Ok(_)) => Ordering::Greater,
                                (Err(_), Err(_)) => Ordering::Equal,
                            }
                        }
                        (TraversalValue::Vector(a), TraversalValue::Vector(b)) => {
                            match (a.check_property(property), b.check_property(property)) {
                                (Ok(val_a), Ok(val_b)) => val_a.cmp(&val_b),
                                (Ok(_), Err(_)) => Ordering::Less,
                                (Err(_), Ok(_)) => Ordering::Greater,
                                (Err(_), Err(_)) => Ordering::Equal,
                            }
                        }
                        (TraversalValue::Count(val_a), TraversalValue::Count(val_b)) => {
                            val_a.cmp(val_b)
                        }
                        (TraversalValue::Value(val_a), TraversalValue::Value(val_b)) => {
                            val_a.cmp(val_b)
                        }
                        _ => Ordering::Equal,
                    },
                    (Err(_), _) => Ordering::Equal,
                    (_, Err(_)) => Ordering::Equal,
                }),
            },
            storage: self.storage,
            txn: self.txn,
        }
    }

    fn order_by_desc(
        self,
        property: &str,
    ) -> RoTraversalIterator<'a, impl Iterator<Item = Result<TraversalValue, GraphError>>> {
        RoTraversalIterator {
            inner: OrderByAsc {
                iter: self.inner.sorted_by(|a, b| match (a, b) {
                    (Ok(a), Ok(b)) => match (a, b) {
                        (TraversalValue::Node(a), TraversalValue::Node(b)) => {
                            match (a.check_property(property), b.check_property(property)) {
                                (Ok(val_a), Ok(val_b)) => val_b.cmp(&val_a),
                                (Ok(_), Err(_)) => Ordering::Less,
                                (Err(_), Ok(_)) => Ordering::Greater,
                                (Err(_), Err(_)) => Ordering::Equal,
                            }
                        }
                        (TraversalValue::Edge(a), TraversalValue::Edge(b)) => {
                            match (a.check_property(property), b.check_property(property)) {
                                (Ok(val_a), Ok(val_b)) => val_b.cmp(&val_a),
                                (Ok(_), Err(_)) => Ordering::Less,
                                (Err(_), Ok(_)) => Ordering::Greater,
                                (Err(_), Err(_)) => Ordering::Equal,
                            }
                        }
                        (TraversalValue::Vector(a), TraversalValue::Vector(b)) => {
                            match (a.check_property(property), b.check_property(property)) {
                                (Ok(val_a), Ok(val_b)) => val_b.cmp(&val_a),
                                (Ok(_), Err(_)) => Ordering::Less,
                                (Err(_), Ok(_)) => Ordering::Greater,
                                (Err(_), Err(_)) => Ordering::Equal,
                            }
                        }
                        (TraversalValue::Count(val_a), TraversalValue::Count(val_b)) => {
                            val_b.cmp(val_a)
                        }
                        (TraversalValue::Value(val_a), TraversalValue::Value(val_b)) => {
                            val_b.cmp(val_a)
                        }
                        _ => Ordering::Equal,
                    },
                    (Err(_), _) => Ordering::Equal,
                    (_, Err(_)) => Ordering::Equal,
                }),
            },
            storage: self.storage,
            txn: self.txn,
        }
    }
}
