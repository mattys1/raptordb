use std::hash::Hash;

use derive_more::{Eq, From};
use crate::graph::{IDIntoUSize, node::NodeID};

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum EdgeKind {
    Directed,
    Undirected
}

#[derive(Debug, Clone, Copy, std::cmp::Eq)]
pub(super) struct Edge<T> {
    pub(super) from: NodeID,
    pub(super) to: NodeID,
    pub(super) kind: EdgeKind,
    pub(super) property: T,
}

// TODO: move edge and node comparision into the graph itself, so that elements dependant on id can also be compared
impl<E> PartialEq for Edge<E> where E: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.property == other.property //&& match self.kind {
        //     EdgeKind::Directed => self.from == other.from && self.to == other.to,
        //     EdgeKind::Undirected => (self.from == other.from && self.to == other.to) || (self.from == other.to && self.to == other.from)
        // }
    }
}

// TODO: for now graph comparision relies on this but it would be better to hash everything probably
impl<E> Hash for Edge<E> where E: Hash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.property.hash(state) 
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct EdgeID(usize);

// impl<T> HasID for Edge<T> {
//     fn get_id(&self) -> EdgeID {
//         self.id
//     }
//
//     type ID = EdgeID;
// }

impl IDIntoUSize for EdgeID {
    fn as_usize(&self) -> usize { self.0 }
    fn from_usize(id: usize) -> Self { EdgeID(id) }
}
