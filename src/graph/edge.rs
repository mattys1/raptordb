use derive_more::From;
use crate::graph::{ID, node::NodeID};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EdgeKind {
    Directed,
    Undirected
}

#[derive(Debug)]
pub(super) struct Edge<T> {
    pub(super) id: EdgeID,
    pub(super) from: NodeID,
    pub(super) to: NodeID,
    pub(super) kind: EdgeKind,
    pub(super) property: T,
}

impl<E> PartialEq for Edge<E> where E: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.kind == other.kind && self.property == other.property
    }
}

#[derive(From, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct EdgeID(usize);

impl ID for EdgeID {
    fn get_inner(&self) -> usize { self.0 }
}
