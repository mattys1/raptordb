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
        self.nodes.push(Node { id, property, edges: Vec::new() });
        id
    }

    pub fn add_edge(&mut self, from: NodeID, to: NodeID, property: T, kind: EdgeKind,) -> Option<EdgeID> {
        if !self.node_id_manager.is_taken(from) || !self.node_id_manager.is_taken(to) {
            return None
        }

        let id = self.edge_id_manager.get_available();
        self.edges.push(Edge {
            id, from, to, kind, property 
        });

        self.nodes[from.get_inner()].edges.push(id);
        self.nodes[to.get_inner()].edges.push(id);

        Some(id)
    }

    pub fn get_node_unchecked(&self, id: NodeID) -> T {
        self.nodes[id.get_inner()].property
    }

    pub fn get_node(&self, id: NodeID) -> Option<T> {
        self.node_id_manager.is_taken(id).then(|| self.get_node_unchecked(id))

    }

    pub fn get_edge_unchecked(&self, id: EdgeID) -> T {
        self.edges[id.get_inner()].property
    }

    pub fn get_edge(&self, id: EdgeID) -> Option<T> {
        self.edge_id_manager.is_taken(id).then(|| self.get_edge_unchecked(id))
    }

    pub fn delete_node_unchecked(&mut self, id: NodeID) {
        let edge_ids: Vec<EdgeID> = self.nodes[id.get_inner()]
            .edges.clone();
        
        for edge_id in edge_ids { self.delete_edge_unchecked(edge_id); }

        self.node_id_manager.mark_as_taken(id);
    }

    pub fn delete_node(&mut self, id: NodeID) -> Option<()> {
        self.node_id_manager.is_taken(id).then(|| self.delete_node_unchecked(id))
    }

    pub fn delete_edge_unchecked(&mut self, id: EdgeID) {
        let edge = &self.edges[id.get_inner()];

        assert!(edge.from != edge.to, "NOT IMPLEMENTED: cyclical edge deletion - from edge: {:?}, to edge: {:?}", edge.from, edge.to);
        unsafe {
            let [from_node, to_node] = self.nodes.get_disjoint_unchecked_mut([edge.from.get_inner(), edge.to.get_inner()]);

            from_node.edges.retain(|&e| e != id);
            to_node.edges.retain(|&e| e != id);
        }

        self.edge_id_manager.mark_as_taken(id);
    }

    pub fn delete_edge(&mut self, id: EdgeID) -> Option<()> {
        self.edge_id_manager.is_taken(id).then(|| self.delete_edge_unchecked(id))
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
    edges: Vec<EdgeID>,
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

#[derive(From, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct NodeID(usize);

#[derive(From, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
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
