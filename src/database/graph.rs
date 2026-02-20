mod tests;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

mod node;
mod edge;
mod availability_manager;
mod store;

use log::trace;
use log::warn;

use crate::database::graph::{edge::Edge, node::Node, store::Store};

pub use crate::database::graph::node::NodeID;
pub use crate::database::graph::edge::EdgeID;
pub use crate::database::graph::edge::EdgeKind;

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
        StoreIterable::new(&self.node_store)
    }

    pub fn edges(&self) -> impl Iterator<Item = EdgeID> {
        StoreIterable::new(&self.edge_store)
    }

    pub fn add_node(&mut self, property: N) -> NodeID {
        self.node_store.add(Node {edges: Vec::new(), property})
    }

    pub fn add_edge(&mut self, from: NodeID, to: NodeID, property: E, kind: EdgeKind,) -> EdgeID {
        debug_assert!(self.node_store.exists(from), "invalid 'from' NodeID: {from:?}");
        debug_assert!(self.node_store.exists(to), "invalid 'to' NodeID: {to:?}");
        debug_assert!(from != to, "cyclical edges are not supported: from and to are the same NodeID: {from:?}");
        // debug_assert!(self.get_edges_between(from, to).is_empty(), "parallel edge detected: there is already an edge between {from:?} and {to:?}");
        if !self.get_edges_between(from, to).is_empty() {
            warn!("parallel edge detected: there is already an edge between {from:?} and {to:?}");
        } 

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

    pub fn get_connected_nodes(&self, id: EdgeID) -> ConnectedNodes {
        let edge = self.edge_store.get(id);
        ConnectedNodes { from: edge.from, to: edge.to }
    }

    // not sure if this should count undirected edges
    pub fn get_outgoing_edges(&self, id: NodeID) -> Vec<EdgeID> {
        debug_assert!(self.node_store.exists(id), "invalid NodeID: {id:?}");

        self.node_store.get(id).edges.iter()
            .filter(|edge_id| {
                let edge = self.edge_store.get(**edge_id);
                edge.from == id || edge.kind == EdgeKind::Undirected
            }).copied().collect()
    }

    // not sure if this should count undirected edges
    pub fn get_incoming_edges(&self, id: NodeID) -> Vec<EdgeID> {
        debug_assert!(self.node_store.exists(id), "invalid NodeID: {id:?}");

        self.node_store.get(id).edges.iter()
            .filter(|edge_id| {
                let edge = self.edge_store.get(**edge_id);
                edge.to == id || edge.kind == EdgeKind::Undirected
            }).copied().collect()
    }

    pub fn get_edges_between(&self, from: NodeID, to: NodeID) -> Vec<EdgeID> {
        debug_assert!(self.node_store.exists(from), "invalid 'from' NodeID: {from:?}");
        debug_assert!(self.node_store.exists(to), "invalid 'to' NodeID: {to:?}");

        self.node_store.get(from).edges.iter()
            .filter_map(|edge_id| {
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
            }).collect()
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

        let to_node = self.node_store.get_mut(self.edge_store.get(id).to);
        to_node.edges.retain(|&e| e != id);

        let from_node = self.node_store.get_mut(self.edge_store.get(id).from);
        from_node.edges.retain(|&e| e != id);

        self.edge_store.remove(id);
    }
}

// slow, but should work for testing
impl <N, E> PartialEq for Graph<N, E> where N: Debug + Copy + PartialEq + Hash + Eq + Sync + Send, E: Debug + Copy + PartialEq + Hash + Eq + Sync + Send {
    fn eq(&self, other: &Self) -> bool {
        if(self.node_store.len()) != other.node_store.len() || self.edge_store.len() != other.edge_store.len() {
            trace!("graph size mismatch: self has {} nodes and {} edges but other has {} nodes and {} edges", self.node_store.len(), self.edge_store.len(), other.node_store.len(), other.edge_store.len());
            return false;
        }

        let (self_sets, other_sets) = rayon::join(
            || {
                let nodes = self.node_store.all()
                    .map(|(_, n)| &n.property)
                    .fold(HashMap::new(), |mut acc, prop| {
                        *acc.entry(prop).or_insert(0) += 1;
                        acc
                    });
                let edges = self.edge_store.all()
                    .map(|(_, e)| &e.property)
                    .fold(HashMap::new(), |mut acc, prop| {
                        *acc.entry(prop).or_insert(0) += 1;
                        acc
                    });
                (nodes, edges)
            },
            || {
                let nodes = other.node_store.all()
                    .map(|(_, n)| &n.property)
                    .fold(HashMap::new(), |mut acc, prop| {
                        *acc.entry(prop).or_insert(0) += 1;
                        acc
                    });
                let edges = other.edge_store.all()
                    .map(|(_, e)| &e.property)
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
            #[cfg(debug_assertions)] {
                let nodes_in_self_not_other = self_nodes_set.iter().filter(|(prop, _)| !other_nodes_set.contains_key(*prop)).collect::<Vec<_>>();
                let nodes_in_other_not_self = other_nodes_set.iter().filter(|(prop, _)| !self_nodes_set.contains_key(*prop)).collect::<Vec<_>>();

                let edges_in_self_not_other = self_edges_set.iter().filter(|(prop, _)| !other_edges_set.contains_key(*prop)).collect::<Vec<_>>();
                let edges_in_other_not_self = other_edges_set.iter().filter(|(prop, _)| !self_edges_set.contains_key(*prop)).collect::<Vec<_>>();

                trace!("graph property mismatch detected, node property differences: in self but not in other: {nodes_in_self_not_other:?}, in other but not in self: {nodes_in_other_not_self:?}\n edge property differences: in self but not in other: {edges_in_self_not_other:?}, in other but not in self: {edges_in_other_not_self:?}");
            }
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

        trace!("backtracking on node {unmapped:?} failed, current mapping: {node_mappings:?}");
        false
    }

    fn are_adjacencies_consistent(
        &self,
        other: &Self,
        other_node: NodeID,
        self_node: NodeID,
        node_mappings: &HashMap<NodeID, NodeID>,
    ) -> bool {
        for (self_prime_node, other_prime_node) in node_mappings {
            let self_edges = self.get_edges_between(*self_prime_node, self_node);
            let other_edges = other.get_edges_between(*other_prime_node, other_node);

            if self_edges.len() != other_edges.len() {
                trace!("adjacency inconsistency detected: number of edges between {:?} and {:?} in self is {} but number of edges between {:?} and {:?} in other is {}", self_prime_node, self_node, self_edges.len(), other_prime_node, other_node, other_edges.len());
                return false;
            }

            let mut counts = HashMap::new();

            for e in &self_edges {
                let prop = self.get_edge(*e);
                *counts.entry(prop).or_insert(0usize) += 1;
            }

            for e in &other_edges {
                let prop = other.get_edge(*e);
                match counts.get_mut(&prop) {
                    Some(count) => {
                        *count -= 1;
                        if *count == 0 {
                            counts.remove(&prop);
                        }
                    }
                    None => return false,
                }
            }

            if !counts.is_empty() {
                trace!("adjacency inconsistency detected: edge properties between {self_prime_node:?} and {self_node:?} in self do not match edge properties between {other_prime_node:?} and {other_node:?} in other, remaining counts: {counts:?}");
                return false;
            }}

        true
    }
}

trait IDIntoUSize {
    fn as_usize(&self) -> usize;
    fn from_usize(id: usize) -> Self;
}

#[derive(Debug)]
pub struct ConnectedNodes {
    pub from: NodeID,
    pub to: NodeID,
}


//TODO: make this statically polymorphic
struct StoreIterable<'a, T, I> {
    store: &'a Store<T, I>,
    inner: Box<dyn Iterator<Item = (I, &'a T)> + 'a>,
    // inner: It,
}

impl<'a, T, I> StoreIterable<'a, T, I> where 
    I: IDIntoUSize + Copy + Debug {
    pub fn new(store: &'a Store<T, I>) -> Self {
        // let inner = Box::new(graph.node_store.all().map(|entry| entry.id));
        let inner = store.all();
        Self { store, inner: Box::new(inner) }
    }
}

impl<T, I> Iterator for StoreIterable<'_, T, I> where
    I: IDIntoUSize + Copy + Debug {
    type Item = I;

    fn next(&mut self) -> Option<I> {
        self.inner.next().map(|(id, _)| id)
    }

    fn count(self) -> usize {
        self.store.len()
    }
}
