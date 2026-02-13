use crate::graph::EdgeID;
use crate::graph::IDIntoUSize;

#[derive(Debug)]
pub struct Node<T> {
    // pub(super) id: NodeID,
    pub(super) edges: Vec<EdgeID>,
    pub(super) property: T,
}


// TODO: move edge and node comparision into the graph itself, so that elements dependant on id can also be compared
// FIXME: this especially affects `Node`
impl<T> PartialEq for Node<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.property == other.property && self.edges.len() == other.edges.len()
    } 
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct NodeID(usize);

// impl<T> HasID for Node<T> {
//     fn get_id(&self) -> NodeID {
//         self.id
//     }
//
//     fn set_id(&mut self, id: Self::ID) {
//         self.id = id;
//     }
//
//     type ID = NodeID;
// }

impl IDIntoUSize for NodeID {
    fn into_usize(&self) -> usize { self.0 }
    fn from_usize(id: usize) -> Self { NodeID(id) }
}            
