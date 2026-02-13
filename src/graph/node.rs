use derive_more::From;
use crate::graph::EdgeID;
use crate::graph::ID;

#[derive(Debug)]
pub struct Node<T> {
    pub(super) id: NodeID,
    pub(super) edges: Vec<EdgeID>,
    pub(super) property: T,
}

impl<T> PartialEq for Node<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.edges == other.edges && self.property == other.property
    } 
}

#[derive(From, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct NodeID(usize);


impl ID for NodeID {
    fn get_inner(&self) -> usize { self.0 }
}            
             
