mod tests;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

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
pub struct Graph<N: Copy, E: Copy> {
    node_store: Store<Node<N>, NodeID>,
    edge_store: Store<Edge<E>, EdgeID>, 
}

impl<N, E> Graph<N, E> where N: Copy + PartialEq, E: Copy + PartialEq {
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

    pub fn add_node(&mut self, property: N) -> NodeID {
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

    pub fn get_node(&self, id: NodeID) -> &N {
        debug_assert!(self.node_store.exists(id), "invalid NodeID: {id:?}");
        &self.node_store.get(id).property
    }

    pub fn get_edge(&self, id: EdgeID) -> &E {
        debug_assert!(self.edge_store.exists(id), "invalid EdgeID: {id:?}");
        &self.edge_store.get(id).property
    }

    // TODO: maybe make this unchecked by default to match get_edge and get_node?
    pub fn get_edge_between(&self, from: NodeID, to: NodeID) -> Option<EdgeID> {
        debug_assert!(self.node_store.exists(from), "invalid 'from' NodeID: {from:?}");
        debug_assert!(self.node_store.exists(to), "invalid 'to' NodeID: {to:?}");

        self.node_store.get(from).edges.iter()
            .find_map(|edge_id| {
                let edge = self.edge_store.get(*edge_id);
                match edge.kind {
                    EdgeKind::Directed => {
                        if edge.to == to {
                            Some(*edge_id)
                        } else {
                            None
                        }
                    },
                    EdgeKind::Undirected => {
                        if (edge.to == to) || (edge.from == to) {
                            Some(*edge_id)
                        } else {
                            None
                        }
                    },
                }
            })
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
}

// impl<N, E> Graph<N, E> where N: Copy + Eq + Hash + PartialEq, E: Copy + PartialEq + Hash + Eq {
//     pub fn intersection(graph1: &Self, graph2: &Self) -> Self {
//         // add node in g1 and g2 ->
//         // for every edge in g1, check:
//         // is edge in g2? => is `to` in `new_nodes` && is from in `new_nodes` => add_edge
//         let nodes_set: HashSet<&Node<N>> = graph2.node_store.all().map(|n| &n.item).collect();
//         let edges_set: HashSet<&Edge<E>> = graph2.edge_store.all().map(|e| &e.item).collect();
//
//         let mut result = Self::new();
//         let common_nodes = graph1.nodes()
//             .filter_map(|node1| { 
//                 if nodes_set.contains(graph1.node_store.get(node1)) {
//                     Some(node1)
//                 } else {
//                     None
//                 }
//             });
//
//         let common_edges = graph1.edges()
//             .filter_map(|edge1| {
//                 if edges_set.contains(graph1.edge_store.get(edge1)) {
//                     Some(edge1)    
//                 } else {
//                     None
//                 }
//             });
//         //
//         // let new_edges: Vec<&Edge<E>> = graph1.edge_store.par_iter()
//         //     .filter_map(|edge1| {
//         //         if graph2.edge_store.par_iter().any(|edge2| {
//         //             edge1 == edge2
//         //         }) &&
//         //         (new_nodes.par_iter().any(|nn| edge1.from == *nn) &&
//         //         new_nodes.par_iter().any(|nn| edge1.to == *nn)) {
//         //             Some(edge1)
//         //         } else {
//         //             None
//         //         }
//         //     }).collect();
//         //
//         // let mut result_ids = Vec::new();
//         // #[cfg(debug_assertions)] {
//         //     result_ids.reserve(new_nodes.len());
//         // }
//         //
//         // for nn in new_nodes {
//         //     let id = result.add_node(graph1.get_node(nn));
//         //
//         //     #[cfg(debug_assertions)] {
//         //         result_ids.push(id);
//         //     }
//         // }
//         //
//         // debug_assert_eq!(result_ids, result.node_store.iter().map(|n| n.id).collect::<Vec<NodeID>>(), "Mismatched graph1 node ids and result node ids, id computing logic should be deterministic and the ids should be ordered the same way");
//         //
//         // for ne in new_edges {
//         //     result.add_edge(ne.from, ne.to, ne.property, ne.kind);
//         // }
//         //
//         // result
//     }
// }


// slow, but should work for testing
impl <N, E> PartialEq for Graph<N, E> where N: Debug + Copy + PartialEq + Hash + Eq + Sync + Send, E: Debug + Copy + PartialEq + Hash + Eq + Sync + Send {
    fn eq(&self, other: &Self) -> bool {
        if(self.node_store.len()) != other.node_store.len() || self.edge_store.len() != other.edge_store.len() {
            return false;
        }

        let (self_sets, other_sets) = rayon::join(
            || {
                let nodes = self.node_store.all()
                    .map(|n| &n.item.property)
                    .fold(HashMap::new(), |mut acc, prop| {
                        *acc.entry(prop).or_insert(0) += 1;
                        acc
                    });
                let edges = self.edge_store.all()
                    .map(|n| &n.item.property)
                    .fold(HashMap::new(), |mut acc, prop| {
                        *acc.entry(prop).or_insert(0) += 1;
                        acc
                    });
                (nodes, edges)
            },
            || {
                let nodes = self.node_store.all()
                    .map(|n| &n.item.property)
                    .fold(HashMap::new(), |mut acc, prop| {
                        *acc.entry(prop).or_insert(0) += 1;
                        acc
                    });
                let edges = self.edge_store.all()
                    .map(|n| &n.item.property)
                    .fold(HashMap::new(), |mut acc, prop| {
                        *acc.entry(prop).or_insert(0) += 1;
                        acc
                    });
                (nodes, edges)
            },
        );

        let (self_nodes_set, self_edges_set) = self_sets;
        let (other_nodes_set, other_edges_set) = other_sets;

        if self_nodes_set != other_nodes_set || self_edges_set != other_edges_set {
            return false;
        }

        let mut node_mappings = HashMap::<NodeID, NodeID>::new();
        let mut used = HashSet::<NodeID>::new();

        self.backtrack(other, &mut node_mappings, &mut used)
    }
}

impl <N, E> Graph<N, E> where N: Debug + Copy + PartialEq + Hash + Eq + Sync + Send, E: Debug + Copy + PartialEq + Hash + Eq + Sync + Send {
    fn backtrack(
        &self,
        other: &Self,
        node_mappings: &mut HashMap<NodeID, NodeID>,
        used: &mut HashSet<NodeID>,
    ) -> bool {
        if node_mappings.len() == self.node_store.len() {
            return true;
        }

        let unmapped = self.nodes().find(|node| !node_mappings.contains_key(node)).expect("there should be an unmapped node since we haven't mapped all nodes yet");

        for node in other.nodes() {
            if used.contains(&node) {
                continue;
            }

            if self.get_node(unmapped) != other.get_node(node) {
                continue;
            }

            if !self.are_adjacencies_consistent(other, node, unmapped, node_mappings) {
                continue;
            }

            node_mappings.insert(unmapped, node);
            used.insert(node);

            if self.backtrack(other, node_mappings, used) {
                return true;
            }

            node_mappings.remove(&unmapped);
            used.remove(&node);
        }

        false
    }

    fn are_adjacencies_consistent(&self, other: &Self, other_node: NodeID, self_node: NodeID, node_mappings: &HashMap<NodeID, NodeID>) -> bool {
        for (self_prime_node, other_prime_node) in node_mappings {
            let self_edge = self.get_edge_between(*self_prime_node, self_node);
            let other_edge = other.get_edge_between(*other_prime_node, other_node);

            if self_edge.is_some() != other_edge.is_some() {
                return false;
            }

            let (Some(self_edge), Some(other_edge)) = (self_edge, other_edge) else { continue };
            if self.get_edge(self_edge) != other.get_edge(other_edge) {
                return false;
            }
        }

        true
    }
}

trait IDIntoUSize {
    fn as_usize(&self) -> usize;
    fn from_usize(id: usize) -> Self;
}
