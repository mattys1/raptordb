#[cfg(test)]
mod test {
    use super::*;
    use crate::graph::{EdgeKind, Graph};

    #[test]
    fn test_create_graph() {
        let _graph: Graph<i32, i32> = Graph::new();
    }

    #[test]
    fn test_add_single_node() {
        let mut graph: Graph<i32, i32> = Graph::new();
        let node_id = graph.add_node(42);
        let property = graph.get_node(node_id);
        assert_eq!(property, 42);
    }

    #[test]
    fn test_add_multiple_nodes() {
        let mut graph: Graph<i32, i32> = Graph::new();
        let id1 = graph.add_node(10);
        let id2 = graph.add_node(20);
        let id3 = graph.add_node(30);

        assert_eq!(graph.get_node(id1), 10);
        assert_eq!(graph.get_node(id2), 20);
        assert_eq!(graph.get_node(id3), 30);
    }

    #[test]
    fn test_add_directed_edge() {
        let mut graph: Graph<&str, &str> = Graph::new();
        let n1 = graph.add_node("A");
        let n2 = graph.add_node("B");
        let edge_id = graph.add_edge(n1, n2, "edge_prop", EdgeKind::Directed);

        assert_eq!(graph.get_edge(edge_id), "edge_prop");
    }

    #[test]
    fn test_add_undirected_edge() {
        let mut graph: Graph<i32, i32> = Graph::new();
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);
        let edge_id = graph.add_edge(n1, n2, 100, EdgeKind::Undirected);

        assert_eq!(graph.get_edge(edge_id), 100);
    }

    #[test]
    fn test_multiple_edges() {
        let mut graph: Graph<f64, f64> = Graph::new();
        let n1 = graph.add_node(1.0);
        let n2 = graph.add_node(2.0);
        let n3 = graph.add_node(3.0);

        let e1 = graph.add_edge(n1, n2, 1.5, EdgeKind::Directed);
        let e2 = graph.add_edge(n2, n3, 2.5, EdgeKind::Directed);
        let e3 = graph.add_edge(n1, n3, 3.5, EdgeKind::Undirected);

        assert_eq!(graph.get_edge(e1), 1.5);
        assert_eq!(graph.get_edge(e2), 2.5);
        assert_eq!(graph.get_edge(e3), 3.5);
    }

    #[test]
    fn test_add_edge_registers_on_nodes() {
        let mut graph: Graph<i32, i32> = Graph::new();
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);
        let e1 = graph.add_edge(n1, n2, 10, EdgeKind::Directed);

        assert!(graph.node_store.get(n1).edges.contains(&e1));
        assert!(graph.node_store.get(n2).edges.contains(&e1));
    }

    #[test]
    fn test_delete_existing_edge() {
        let mut graph: Graph<i32, i32> = Graph::new();
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);
        let e1 = graph.add_edge(n1, n2, 10, EdgeKind::Directed);

        graph.delete_edge(e1);

        assert!(!graph.edge_store.exists(e1) && !graph.edges().any(|e| e == e1));
    }

    #[test]
    fn test_delete_edge_preserves_other_edges() {
        let mut graph: Graph<i32, i32> = Graph::new();
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);
        let n3 = graph.add_node(3);

        let e1 = graph.add_edge(n1, n2, 10, EdgeKind::Directed);
        let e2 = graph.add_edge(n2, n3, 20, EdgeKind::Directed);

        graph.delete_edge(e1);

        assert_eq!(graph.get_edge(e2), 20);
    }

    #[test]
    fn test_delete_edge_preserves_nodes() {
        let mut graph: Graph<i32, i32> = Graph::new();
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);
        let e1 = graph.add_edge(n1, n2, 10, EdgeKind::Directed);

        graph.delete_edge(e1);

        assert_eq!(graph.get_node(n1), 1);
        assert_eq!(graph.get_node(n2), 2);
    }

    #[test]
    fn test_delete_all_edges() {
        let mut graph: Graph<i32, i32> = Graph::new();
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);

        let e1 = graph.add_edge(n1, n2, 10, EdgeKind::Directed);
        let e2 = graph.add_edge(n1, n2, 20, EdgeKind::Directed);
        let e3 = graph.add_edge(n1, n2, 30, EdgeKind::Undirected);

        graph.delete_edge(e1);
        graph.delete_edge(e2);
        graph.delete_edge(e3);

        assert!(graph.edges().next().is_none());
    }
}
