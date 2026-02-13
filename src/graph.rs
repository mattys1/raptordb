mod tests;

use std::fmt::Debug;

mod node;
mod edge;
mod availability_manager;
mod store;

use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};

use crate::graph::{edge::Edge, node::Node, store::Store};

pub use crate::graph::node::NodeID;
pub use crate::graph::edge::EdgeID;
pub use crate::graph::edge::EdgeKind;

#[derive(Debug)]
pub struct Graph<V: Copy, E: Copy> {
    node_store: Store<Node<V>, NodeID>,
    edge_store: Store<Edge<E>, EdgeID>, 
}

impl<T, E> Graph<T, E> where T: Copy + PartialEq + Send + Sync, E: Copy + PartialEq + Send + Sync {
    pub fn new() -> Self {
        Self {
            node_store: Store::new(),
            edge_store: Store::new(),
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = NodeID> {
        // self.nodes.iter().filter_map(|n| if self.node_id_manager.is_taken(n.id) { Some(n.id) } else { None })
        self.node_store.all().map(|n| n.id)
    }

    pub fn edges(&self) -> impl Iterator<Item = EdgeID> {
        self.edge_store.all().map(|e| e.id)
    }

    pub fn add_node(&mut self, property: T) -> NodeID {
        self.node_store.add(Node {edges: Vec::new(), property})
    }

    pub fn add_edge(&mut self, from: NodeID, to: NodeID, property: E, kind: EdgeKind,) -> EdgeID {
        debug_assert!(self.node_store.exists(from), "invalid 'from' NodeID: {from:?}");
        debug_assert!(self.node_store.exists(to), "invalid 'to' NodeID: {to:?}");

        let id = self.edge_store.add(Edge { from, to, kind, property });

        let to_node = self.node_store.get_mut(self.edge_store.get(id).to);
        to_node.edges.push(id);

        let from_node = self.node_store.get_mut(self.edge_store.get(id).from);
        from_node.edges.push(id);

        id
    }

    pub fn get_node(&self, id: NodeID) -> T {
        debug_assert!(self.node_store.exists(id), "invalid NodeID: {id:?}");
        self.node_store.get(id).property
    }

    pub fn get_edge(&self, id: EdgeID) -> E {
        debug_assert!(self.edge_store.exists(id), "invalid EdgeID: {id:?}");
        self.edge_store.get(id).property
    }

    pub fn delete_node(&mut self, id: NodeID) {
        debug_assert!(self.node_store.exists(id), "invalid NodeID: {id:?}");
        self.delete_node_impl(id);
    }

    fn delete_node_impl(&mut self, id: NodeID) {
        if !self.node_store.exists(id) {
            return;
        }

        let edge_ids: Vec<EdgeID> = self.node_store.get(id)
            .edges.clone();

        for edge_id in edge_ids { self.delete_edge_impl(edge_id); }

        self.node_store.remove(id);
    }

    pub fn delete_edge(&mut self, id: EdgeID) {
        debug_assert!(self.edge_store.exists(id), "invalid EdgeID: {id:?}");
        self.delete_edge_impl(id);
    }

    fn delete_edge_impl(&mut self, id: EdgeID) {
        if !self.edge_store.exists(id) {
            return;
        }

        let edge = &self.edge_store.get(id);

        debug_assert!(edge.from != edge.to, "NOT IMPLEMENTED: cyclical edge deletion - from edge: {:?}, to edge: {:?}", edge.from, edge.to);
        let to_node = self.node_store.get_mut(self.edge_store.get(id).to);
        to_node.edges.retain(|&e| e != id);

        let from_node = self.node_store.get_mut(self.edge_store.get(id).from);
        from_node.edges.retain(|&e| e != id);


        self.edge_store.remove(id);
    }

    pub fn intersection(graph1: &Self, graph2: &Self) -> Self {
        // add node in g1 and g2 ->
        // for every edge in g1, check:
        // is edge in g2? => is `to` in `new_nodes` && is from in `new_nodes` => add_edge
        todo!()


        // let mut result = Self::new();
        // let new_nodes: Vec<NodeID> = graph1.nodes().par_bridge()
        //     .filter_map(|node1| { 
        //         if graph2.nodes().par_bridge()
        //             .find_any(|node2| { 
        //                 graph2.get_node(*node2) == graph1.get_node(node1) 
        //             }).is_some() {
        //             Some(node1)
        //         } else {
        //             None
        //         }
        //     }).collect();
        //
        // let new_edges: Vec<&Edge<E>> = graph1.edge_store.par_iter()
        //     .filter_map(|edge1| {
        //         if graph2.edge_store.par_iter().any(|edge2| {
        //             edge1 == edge2
        //         }) &&
        //         (new_nodes.par_iter().any(|nn| edge1.from == *nn) &&
        //         new_nodes.par_iter().any(|nn| edge1.to == *nn)) {
        //             Some(edge1)
        //         } else {
        //             None
        //         }
        //     }).collect();
        //
        // let mut result_ids = Vec::new();
        // #[cfg(debug_assertions)] {
        //     result_ids.reserve(new_nodes.len());
        // }
        //
        // for nn in new_nodes {
        //     let id = result.add_node(graph1.get_node(nn));
        //
        //     #[cfg(debug_assertions)] {
        //         result_ids.push(id);
        //     }
        // }
        //
        // debug_assert_eq!(result_ids, result.node_store.iter().map(|n| n.id).collect::<Vec<NodeID>>(), "Mismatched graph1 node ids and result node ids, id computing logic should be deterministic and the ids should be ordered the same way");
        //
        // for ne in new_edges {
        //     result.add_edge(ne.from, ne.to, ne.property, ne.kind);
        // }
        //
        // result
    }

}

impl <N, E> PartialEq for Graph<N, E> where N: Debug + Copy + PartialEq, E: Debug + Copy + PartialEq {
    fn eq(&self, other: &Self) -> bool {
        if(self.node_store.len()) != other.node_store.len() || self.edge_store.len() != other.edge_store.len() {
            return false;
        }

        self.node_store.all().all(|n| other.node_store.all().any(|on| on.item == n.item)) &&
        self.edge_store.all().all(|e| other.edge_store.all().any(|oe| oe.item == e.item))
    }
}

trait IDIntoUSize {
    fn into_usize(&self) -> usize;
    fn from_usize(id: usize) -> Self;
}
//
// trait HasID {
//     type ID: IDIntoUSize + Copy;
//     fn get_id(&self) -> Self::ID;
//     fn set_id(&mut self, id: Self::ID);
// }

