use derive_more::{From};

mod id_manager;
use id_manager::IDManager;

mod tests;

#[derive(Debug)]
pub struct Graph<T: Copy> {
    nodes: Vec<Node<T>>,
    edges: Vec<Edge<T>>, 

    node_id_manager: IDManager<NodeID>,
    edge_id_manager: IDManager<EdgeID>,
}

impl<T: Copy> Graph<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),

            node_id_manager: IDManager::new(),
            edge_id_manager: IDManager::new(),
        }
    }

    pub fn add_node(&mut self, property: T) -> NodeID {
        let id = self.node_id_manager.get_available();
        self.nodes.push(Node { id, property });
        id
    }

    pub fn add_edge(&mut self, from: NodeID, to: NodeID, property: T, kind: EdgeKind,) -> EdgeID {
        let id = self.edge_id_manager.get_available();
        self.edges.push(Edge {
            id, from, to, kind, property 
        });

        id
    }

    pub fn get_node(&self, id: NodeID) -> T {
        self.nodes[id.get_inner()].property
    }

    pub fn get_edge(&self, id: EdgeID) -> T {
        self.edges[id.get_inner()].property
    }
}

// impl<T> Display for Graph<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Graph with nodes:\n{}\nand edges:\n{}", self.nodes.len(), self.edges.len())
//     }
// }

#[derive(Debug)]
pub enum EdgeKind {
    Directed,
    Undirected
}

#[derive(Debug)]
struct Node<T> {
    id: NodeID,
    property: T,
}

#[derive(Debug)]
struct Edge<T> {
    id: EdgeID,
    from: NodeID,
    to: NodeID,
    kind: EdgeKind,
    property: T,
}

#[derive(From, Copy, Clone, Debug)]
pub struct NodeID(usize);

#[derive(From, Copy, Clone, Debug)]
pub struct EdgeID(usize);

pub(super) trait ID {
    fn get_inner(&self) -> usize;
}

impl ID for NodeID {
    fn get_inner(&self) -> usize { self.0 }
}            
             
impl ID for EdgeID {
    fn get_inner(&self) -> usize { self.0 }
}
