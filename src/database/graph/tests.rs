#[cfg(all(test, not(feature = "disable_graph_unit_tests")))]
#[cfg(test)]
mod test { 
    use ordered_float::OrderedFloat;

    use crate::database::graph::{EdgeKind, Graph, IDIntoUSize};
    use crate::database::graph::id::{NodePropertyID, NodePropertyTypeID, EdgePropertyID, EdgePropertyTypeID};
    use crate::database::importer::{Lattitude, Longitude};
    use crate::database::property_manager::PropertyIdentifier;
    use crate::database::graph::node::NodeData;
    use crate::database::graph::edge::EdgeData;

    fn node_prop(id: usize) -> PropertyIdentifier<NodePropertyID, NodePropertyTypeID> {
        PropertyIdentifier { id: NodePropertyID::from_usize(id), type_id: NodePropertyTypeID::from_usize(1) }
    }
    fn edge_prop(id: usize) -> PropertyIdentifier<EdgePropertyID, EdgePropertyTypeID> {
        PropertyIdentifier { id: EdgePropertyID::from_usize(id), type_id: EdgePropertyTypeID::from_usize(1) }
    }

    fn lat(id: usize) -> Lattitude {
        Lattitude::from(OrderedFloat(id as f64))
    }

    fn lon(id: usize) -> Longitude {
        Longitude::from(OrderedFloat(id as f64))
    }


    #[test]
    fn test_add_multiple_nodes() {
        let mut graph = Graph::new();
        let id1 = graph.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let id2 = graph.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        let id3 = graph.add_node(NodeData { property: node_prop(3), lat: lat(3), lon: lon(3) });

        assert_eq!(graph.get_node(id1).property, node_prop(1));
        assert_eq!(graph.get_node(id2).property, node_prop(2));
        assert_eq!(graph.get_node(id3).property, node_prop(3));


        assert_eq!(graph.get_node(id1).lon, lon(1));
        assert_eq!(graph.get_node(id2).lon, lon(2));
        assert_eq!(graph.get_node(id3).lon, lon(3));

        assert_eq!(graph.get_node(id1).lat, lat(1));
        assert_eq!(graph.get_node(id2).lat, lat(2));
        assert_eq!(graph.get_node(id3).lat, lat(3));
    }

    #[test]
    fn test_add_directed_edge() {
        let mut graph = Graph::new();
        let n1 = graph.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = graph.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        let edge_id = graph.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        assert_eq!(graph.get_edge(edge_id).property, edge_prop(10));
    }

    #[test]
    fn test_add_undirected_edge() {
        let mut graph = Graph::new();
        let n1 = graph.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = graph.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        let edge_id = graph.add_edge(n1, n2, EdgeData { property: edge_prop(100), kind: EdgeKind::Undirected, cost: 0.0.into() });

        assert_eq!(graph.get_edge(edge_id).property, edge_prop(100));
    }

    #[test]
    fn test_multiple_edges() {
        let mut graph = Graph::new();
        let n1 = graph.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = graph.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        let n3 = graph.add_node(NodeData { property: node_prop(3), lat: lat(3), lon: lon(3) });

        let e1 = graph.add_edge(n1, n2, EdgeData { property: edge_prop(15), kind: EdgeKind::Directed, cost: 0.0.into() });
        let e2 = graph.add_edge(n2, n3, EdgeData { property: edge_prop(25), kind: EdgeKind::Directed, cost: 0.0.into() });
        let e3 = graph.add_edge(n1, n3, EdgeData { property: edge_prop(35), kind: EdgeKind::Undirected, cost: 0.0.into() });

        assert_eq!(graph.get_edge(e1).property, edge_prop(15));
        assert_eq!(graph.get_edge(e2).property, edge_prop(25));
        assert_eq!(graph.get_edge(e3).property, edge_prop(35));
    }

    #[test]
    fn test_add_edge_registers_on_nodes() {
        let mut graph = Graph::new();
        let n1 = graph.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = graph.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        let e1 = graph.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        assert!(graph.node_store.get(n1).edges.contains(&e1));
        assert!(graph.node_store.get(n2).edges.contains(&e1));
    }

    #[test]
    fn test_delete_existing_edge() {
        let mut graph = Graph::new();
        let n1 = graph.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = graph.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        let e1 = graph.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        graph.delete_edge(e1);

        assert!(!graph.edge_store.exists(e1) && !graph.edges().any(|e| e == e1));
    }

    #[test]
    fn test_delete_edge_preserves_other_edges() {
        let mut graph = Graph::new();
        let n1 = graph.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = graph.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        let n3 = graph.add_node(NodeData { property: node_prop(3), lat: lat(3), lon: lon(3) });

        let e1 = graph.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });
        let e2 = graph.add_edge(n2, n3, EdgeData { property: edge_prop(20), kind: EdgeKind::Directed, cost: 0.0.into() });

        graph.delete_edge(e1);

        assert_eq!(graph.get_edge(e2).property, edge_prop(20));
    }

    #[test]
    fn test_delete_edge_preserves_nodes() {
        let mut graph = Graph::new();
        let n1 = graph.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = graph.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        let e1 = graph.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        graph.delete_edge(e1);

        assert_eq!(graph.get_node(n1).property, node_prop(1));
        assert_eq!(graph.get_node(n2).property, node_prop(2));
    }

    #[test]
    fn test_delete_all_edges() {
        let mut graph = Graph::new();
        let n1 = graph.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = graph.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });

        let e1 = graph.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });
        let e2 = graph.add_edge(n1, n2, EdgeData { property: edge_prop(20), kind: EdgeKind::Directed, cost: 0.0.into() });
        let e3 = graph.add_edge(n1, n2, EdgeData { property: edge_prop(30), kind: EdgeKind::Undirected, cost: 0.0.into() });

        graph.delete_edge(e1);
        graph.delete_edge(e2);
        graph.delete_edge(e3);

        assert!(graph.edges().next().is_none());
    }

    #[test]
    fn test_empty_graphs_equality() {
        let g1 = Graph::new();
        let g2 = Graph::new();
        assert_eq!(g1, g2);
    }

    #[test]
    fn test_same_graphs_equality() {
        let mut g1 = Graph::new();
        let mut g2 = Graph::new();

        let n1 = g1.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = g1.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g1.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        let m1 = g2.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let m2 = g2.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g2.add_edge(m1, m2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        assert_eq!(g1, g2);
    }

    #[test]
    fn test_different_graphs_inequality_different_nodes() {
        let mut g1 = Graph::new();
        let mut g2 = Graph::new();

        let n1 = g1.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = g1.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g1.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        let m1 = g2.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let m2 = g2.add_node(NodeData { property: node_prop(3), lat: lat(3), lon: lon(3) });
        g2.add_edge(m1, m2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        assert_ne!(g1, g2);
    }

    #[test]
    fn test_different_graphs_inequality_different_edges() {
        let mut g1 = Graph::new();
        let mut g2 = Graph::new();

        let n1 = g1.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = g1.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g1.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        let m1 = g2.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let m2 = g2.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g2.add_edge(m1, m2, EdgeData { property: edge_prop(20), kind: EdgeKind::Directed, cost: 0.0.into() });

        assert_ne!(g1, g2);
    }

    #[test]
    fn test_different_graphs_equality_different_node_order_on_edge_undirected() {
        let mut g1 = Graph::new();
        let mut g2 = Graph::new();

        let n1 = g1.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = g1.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g1.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Undirected, cost: 0.0.into() });

        let m1 = g2.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let m2 = g2.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g2.add_edge(m2, m1, EdgeData { property: edge_prop(10), kind: EdgeKind::Undirected, cost: 0.0.into() });

        assert_eq!(g1, g2);
    }

    #[test]
    fn test_different_graphs_inequality_different_node_order_on_edge_directed() {
        let mut g1 = Graph::new();
        let mut g2 = Graph::new();

        let n1 = g1.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = g1.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g1.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        let m1 = g2.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let m2 = g2.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g2.add_edge(m2, m1, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        assert_ne!(g1, g2);
    }

    #[test]
    fn test_different_graphs_equality_edge_undirected_different_node_order_different_construction() {
        let mut g1 = Graph::new();
        let mut g2 = Graph::new();

        let n1 = g1.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = g1.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g1.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Undirected, cost: 0.0.into() });

        let m1 = g2.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let dummy_node = g2.add_node(NodeData { property: node_prop(10), lat: lat(10), lon: lon(10) });
        let m2 = g2.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });

        g2.delete_node(dummy_node);
        g2.add_edge(m2, m1, EdgeData { property: edge_prop(10), kind: EdgeKind::Undirected, cost: 0.0.into() });

        assert_eq!(g1, g2);
    }

    #[test]
    fn test_different_graphs_inequality_different_node_order_on_edge_directed_different_construction() {
        let mut g1 = Graph::new();
        let mut g2 = Graph::new();

        let n1 = g1.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = g1.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g1.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        let m1 = g2.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let dummy_node = g2.add_node(NodeData { property: node_prop(10), lat: lat(10), lon: lon(10) });
        let m2 = g2.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });

        g2.delete_node(dummy_node);
        g2.add_edge(m2, m1, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        assert_ne!(g1, g2);
    }

    #[test]
    fn test_different_graphs_same_node_order_on_edge_directed_different_construction() {
        let mut g1 = Graph::new();
        let mut g2 = Graph::new();

        let n1 = g1.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = g1.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g1.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        let m1 = g2.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let dummy_node = g2.add_node(NodeData { property: node_prop(10), lat: lat(10), lon: lon(10) });
        let m2 = g2.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });

        g2.delete_node(dummy_node);
        g2.add_edge(m1, m2, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() });

        assert_eq!(g1, g2);
    }

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn test_directed_incoming_outgoing_mismatch() {
        let mut g1 = Graph::new();
        let mut g2 = Graph::new();

        let a = g1.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let b = g1.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        let c = g1.add_node(NodeData { property: node_prop(3), lat: lat(3), lon: lon(3) });

        g1.add_edge(a, b, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() }); // 1 → 2
        g1.add_edge(c, a, EdgeData { property: edge_prop(20), kind: EdgeKind::Directed, cost: 0.0.into() }); // 3 → 1

        let x = g2.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let y = g2.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        let z = g2.add_node(NodeData { property: node_prop(3), lat: lat(3), lon: lon(3) });

        g2.add_edge(y, x, EdgeData { property: edge_prop(10), kind: EdgeKind::Directed, cost: 0.0.into() }); // 2 → 1
        g2.add_edge(x, z, EdgeData { property: edge_prop(20), kind: EdgeKind::Directed, cost: 0.0.into() }); // 1 → 3

        assert_ne!(g1, g2);
    }

    #[test]
    fn parallel_edges_between_nodes_are_allowed_and_counted() {
        let mut g = Graph::new();
        let n1 = g.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = g.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });

        let e1 = g.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Undirected, cost: 0.0.into() });
        let e2 = g.add_edge(n1, n2, EdgeData { property: edge_prop(10), kind: EdgeKind::Undirected, cost: 0.0.into() });

        let edges = g.get_edges_between(n1, n2);
        assert_eq!(edges.len(), 2, "there should be two parallel edges between n1 and n2");
        assert!(edges.contains(&e1) && edges.contains(&e2));
        assert_eq!(g.get_edge(e1).property, edge_prop(10));
        assert_eq!(g.get_edge(e2).property, edge_prop(10));
    }

    #[test]
    fn deleting_one_parallel_edge_leaves_the_other() {
        let mut g = Graph::new();
        let a = g.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let b = g.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });

        let e1 = g.add_edge(a, b, EdgeData { property: edge_prop(7), kind: EdgeKind::Undirected, cost: 0.0.into() });
        let e2 = g.add_edge(a, b, EdgeData { property: edge_prop(7), kind: EdgeKind::Undirected, cost: 0.0.into() });

        g.delete_edge(e1);

        let edges = g.get_edges_between(a, b);
        assert_eq!(edges.len(), 1);
        assert!(edges.contains(&e2));
        assert!(!edges.contains(&e1));
    }

    #[test]
    fn delete_node_removes_incident_edges() {
        let mut g = Graph::new();
        let a = g.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let b = g.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        let c = g.add_node(NodeData { property: node_prop(3), lat: lat(3), lon: lon(3) });

        let e1 = g.add_edge(a, b, EdgeData { property: edge_prop(100), kind: EdgeKind::Undirected, cost: 0.0.into() });
        let e2 = g.add_edge(b, c, EdgeData { property: edge_prop(200), kind: EdgeKind::Undirected, cost: 0.0.into() });

        g.delete_node(b);

        // b should no longer be present
        assert!(!g.nodes().any(|n| n == b));
        // both incident edges must be removed
        assert!(g.edges().all(|id| id != e1 && id != e2));
        // remaining nodes are a and c
        let remaining: Vec<_> = g.nodes().collect();
        assert_eq!(remaining.len(), 2);
        assert!(remaining.contains(&a) && remaining.contains(&c));
    }

    #[test]
    fn directed_edges_respect_direction() {
        let mut g = Graph::new();
        let n1 = g.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let n2 = g.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });

        g.add_edge(n1, n2, EdgeData { property: edge_prop(1), kind: EdgeKind::Directed, cost: 0.0.into() });

        assert_eq!(g.get_edges_between(n1, n2).len(), 1);
        assert_eq!(g.get_edges_between(n2, n1).len(), 0, "directed edge should not be found in reverse direction");
    }

    #[test]
    fn partial_eq_considers_edge_multiplicity() {
        let mut g1 = Graph::new();
        let a1 = g1.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let b1 = g1.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g1.add_edge(a1, b1, EdgeData { property: edge_prop(5), kind: EdgeKind::Undirected, cost: 0.0.into() });
        g1.add_edge(a1, b1, EdgeData { property: edge_prop(5), kind: EdgeKind::Undirected, cost: 0.0.into() });

        let mut g2 = Graph::new();
        let a2 = g2.add_node(NodeData { property: node_prop(1), lat: lat(1), lon: lon(1) });
        let b2 = g2.add_node(NodeData { property: node_prop(2), lat: lat(2), lon: lon(2) });
        g2.add_edge(a2, b2, EdgeData { property: edge_prop(5), kind: EdgeKind::Undirected, cost: 0.0.into() });
        g2.add_edge(a2, b2, EdgeData { property: edge_prop(5), kind: EdgeKind::Undirected, cost: 0.0.into() });

        assert_eq!(g1, g2);

        let e = g2.get_edges_between(a2, b2)[0];
        g2.delete_edge(e);
        assert_ne!(g1, g2);
    }
}
