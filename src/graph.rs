mod tests;

use std::fmt::Debug;

mod node;
mod edge;
mod id_manager;
use id_manager::IDManager;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};

use crate::graph::{edge::{Edge}, node::{Node}};

pub use crate::graph::node::NodeID;
pub use crate::graph::edge::EdgeID;
pub use crate::graph::edge::EdgeKind;

pub struct Graph<V: Copy, E: Copy> {
    nodes: Vec<Node<V>>,
    edges: Vec<Edge<E>>, 

    node_id_manager: IDManager<NodeID>,
    edge_id_manager: IDManager<EdgeID>,
}

impl<T, E> Graph<T, E> where T: Copy + PartialEq + Send + Sync, E: Copy + PartialEq + Send + Sync {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),

            node_id_manager: IDManager::new(),
            edge_id_manager: IDManager::new(),
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = NodeID> {
        self.nodes.iter().filter_map(|n| if self.node_id_manager.is_taken(n.id) { Some(n.id) } else { None })
    }

    pub fn edges(&self) -> impl Iterator<Item = EdgeID> {
        self.edges.iter().filter_map(|e| if self.edge_id_manager.is_taken(e.id) { Some(e.id) } else { None })
    }

    pub fn add_node(&mut self, property: T) -> NodeID {
        let id = self.node_id_manager.get_available();
        self.nodes.push(Node { id, property, edges: Vec::new() });
        id
    }

    pub fn add_edge(&mut self, from: NodeID, to: NodeID, property: E, kind: EdgeKind,) -> EdgeID {
        debug_assert!(self.node_id_manager.is_taken(from), "invalid 'from' NodeID: {from:?}");
        debug_assert!(self.node_id_manager.is_taken(to), "invalid 'to' NodeID: {to:?}");

        let id = self.edge_id_manager.get_available();
        self.edges.push(Edge {
            id, from, to, kind, property 
        });

        self.nodes[from.get_inner()].edges.push(id);
        self.nodes[to.get_inner()].edges.push(id);

        id
    }

    pub fn get_node(&self, id: NodeID) -> T {
        debug_assert!(self.node_id_manager.is_taken(id), "invalid NodeID: {id:?}");
        self.nodes[id.get_inner()].property
    }

    pub fn get_edge(&self, id: EdgeID) -> E {
        debug_assert!(self.edge_id_manager.is_taken(id), "invalid EdgeID: {id:?}");
        self.edges[id.get_inner()].property
    }

    pub fn delete_node(&mut self, id: NodeID) {
        debug_assert!(self.node_id_manager.is_taken(id), "invalid NodeID: {id:?}");
        let edge_ids: Vec<EdgeID> = self.nodes[id.get_inner()]
            .edges.clone();
        
        for edge_id in edge_ids { self.delete_edge(edge_id); }

        self.node_id_manager.mark_as_taken(id);
    }

    pub fn delete_edge(&mut self, id: EdgeID) {
        debug_assert!(self.edge_id_manager.is_taken(id), "invalid EdgeID: {id:?}");
        let edge = &self.edges[id.get_inner()];

        debug_assert!(edge.from != edge.to, "NOT IMPLEMENTED: cyclical edge deletion - from edge: {:?}, to edge: {:?}", edge.from, edge.to);
        unsafe {
            let [from_node, to_node] = self.nodes.get_disjoint_unchecked_mut([edge.from.get_inner(), edge.to.get_inner()]);

            from_node.edges.retain(|&e| e != id);
            to_node.edges.retain(|&e| e != id);
        }

        self.edge_id_manager.mark_as_taken(id);
    }

    pub fn intersection(graph1: &Self, graph2: &Self) -> Self {
        // add node in g1 and g2 ->
        // for every edge in g1, check:
        // is edge in g2? => is `to` in `new_nodes` && is from in `new_nodes` => add_edge


        let mut result = Self::new();
        let new_nodes: Vec<NodeID> = graph1.nodes().par_bridge()
            .filter_map(|node1| { 
                if graph2.nodes().par_bridge()
                    .find_any(|node2| { 
                        graph2.get_node(*node2) == graph1.get_node(node1) 
                    }).is_some() {
                    Some(node1)
                } else {
                    None
                }
            }).collect();

        let new_edges: Vec<&Edge<E>> = graph1.edges.par_iter()
            .filter_map(|edge1| {
                if graph2.edges.par_iter().any(|edge2| {
                    edge1 == edge2
                }) &&
                (new_nodes.par_iter().any(|nn| edge1.from == *nn) &&
                new_nodes.par_iter().any(|nn| edge1.to == *nn)) {
                    Some(edge1)
                } else {
                    None
                }
            }).collect();

        let mut result_ids = Vec::new();
        #[cfg(debug_assertions)] {
            result_ids.reserve(new_nodes.len());
        }

        for nn in new_nodes {
            let id = result.add_node(graph1.get_node(nn));

            #[cfg(debug_assertions)] {
                result_ids.push(id);
            }
        }

        debug_assert_eq!(result_ids, result.nodes.iter().map(|n| n.id).collect::<Vec<NodeID>>(), "Mismatched graph1 node ids and result node ids, id computing logic should be deterministic and the ids should be ordered the same way");

        for ne in new_edges {
            result.add_edge(ne.from, ne.to, ne.property, ne.kind);
        }

       result
    }

}

impl <N, E> PartialEq for Graph<N, E> where N: Debug + Copy + PartialEq, E: Debug + Copy + PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes && self.edges == other.edges
    }
}

impl<N: Debug + Copy, E: Debug + Copy> Debug for Graph<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Graph")
            .field("nodes", &self.nodes)
            .field("edges", &self.edges)
            .finish()
    }
}

pub(super) trait ID {
    fn get_inner(&self) -> usize;
}
