// TODO: the whole thing should eventually become multithreaded
use std::{collections::HashMap, error::Error, fs::File, path::Path};

use derive_more::From;
use log::warn;
use ordered_float::OrderedFloat;
use osm_xml::OSM;
use osmpbf::{Element, ElementReader};

use crate::graph::{EdgeKind, Graph, NodeID};

#[derive(Clone, Copy, From, Debug, PartialEq, Hash, Eq)]
struct Lattitude(OrderedFloat<f64>);
#[derive(Clone, Copy, From, Debug, PartialEq, Hash, Eq)]
struct Longitude(OrderedFloat<f64>);

#[derive(Copy, Debug, Clone)]
struct ImportedNode {
    lat: Lattitude,
    lon: Longitude,
}


#[derive(Copy, Debug, Clone, PartialEq, Hash, Eq)]
pub struct GraphNode {
    lat: Lattitude,
    lon: Longitude,
}

#[derive(Debug, Clone)]
struct ImportedWay {
    node_refs: Vec<i64>,
    tags: HashMap<String, String>,
    for_graph: GraphWay,
}

#[derive(Copy, Debug, Clone, PartialEq, Hash, Eq)]
pub struct GraphWay {
    distance: OrderedFloat<f64> //TODO: newtype this probably
}


// TODO: hacky but technically solves floating point precision issues, should base node identity on labelling in the source instead of coordinates
fn quantize_coord(v: f64) -> f64 {
    const COORD_QUANT: f64 = 1e7;
    (v * COORD_QUANT).round() / COORD_QUANT
}

pub fn import_pbf(path: &Path) -> Result<Graph<GraphNode, GraphWay>, Box<dyn Error>> {
    let reader = ElementReader::from_path(path)?;
    let mut graph = Graph::<GraphNode, GraphWay>::new();

    let mut graph_id_by_import_id = HashMap::<i64, NodeID>::new();
    let mut imported_ways = Vec::<ImportedWay>::new();

    reader.for_each(|element| {
        match element {
            Element::Node(node) => {
                let lat = quantize_coord(node.lat());
                let lon = quantize_coord(node.lon());
                let graph_id = graph.add_node(GraphNode { lat: OrderedFloat(lat).into(), lon: OrderedFloat(lon).into() });
                graph_id_by_import_id.insert(node.id(), graph_id);
            },
            Element::DenseNode(dense_node) => {
                let lat = quantize_coord(dense_node.lat());
                let lon = quantize_coord(dense_node.lon());
                let graph_id = graph.add_node(GraphNode { lat: OrderedFloat(lat).into(), lon: OrderedFloat(lon).into() });
                graph_id_by_import_id.insert(dense_node.id(), graph_id);
            },
            Element::Way(way) => imported_ways.push(ImportedWay { 
                node_refs: Iterator::collect(way.refs()),
                tags: way.tags().map(|(key, value)| { (key.into(), value.into()) } ).collect(),
                for_graph: GraphWay { distance: OrderedFloat(1.) } // TODO: distance should probably be computed when adding to graph and base it on nodes 
            }),
            Element::Relation(relation) => {
                warn!("Encountered relation with id {}, skipping", relation.id());
            },
        }

    })?;

    for way in imported_ways {
        way.node_refs.windows(2).for_each(|window| {
            // assuming nodes are ordered
            let start_node = window[0];
            let end_node = window[1];
            let kind = if way.tags.iter().any(|(key, value)| {
                key == "oneway" && value != "no"
            }) {
                EdgeKind::Directed
            } else {
                EdgeKind::Undirected
            };


            let Some(&start_node_graph) = graph_id_by_import_id.get(&start_node) else {
                warn!("Encountered way with dangling node id: {way:#?}");
                return; 
            };
            let Some(&end_node_graph) = graph_id_by_import_id.get(&end_node) else {
                warn!("Encountered way with dangling node id: {way:#?}");
                return;
            };

            graph.add_edge(start_node_graph, end_node_graph, GraphWay { distance: way.for_graph.distance }, kind); // TODO: FIXME: actually compute distance
        });
    }

    Ok(graph)

}

pub fn import_xml(path: &Path) -> Result<Graph<GraphNode, GraphWay>, Box<dyn Error>> {
    let file = File::open(path)?;
    let doc = OSM::parse(file).unwrap();

    let mut graph = Graph::<GraphNode, GraphWay>::new();

    let mut graph_id_by_import_id = HashMap::<i64, NodeID>::new();
    // let mut imported_ways = Vec::<ImportedWay>::new();

    for node in doc.nodes.values() {
        let lat = quantize_coord(node.lat);
        let lon = quantize_coord(node.lon);
        let graph_id = graph.add_node(GraphNode { lat: OrderedFloat(lat).into(), lon: OrderedFloat(lon).into() });
        graph_id_by_import_id.insert(node.id, graph_id); 
    }

    for way in doc.ways.values() {
        way.nodes.windows(2).for_each(|window| {
            // assuming nodes are ordered
            let start_node = match doc.resolve_reference(&window[0]) {
                osm_xml::Reference::Node(node) => node.id,
                _ => {
                    warn!("Way with id {} has a non node reference in node references for some reason, skipping", way.id);
                    return;
                }
            };

            let end_node = match doc.resolve_reference(&window[1]) {
                osm_xml::Reference::Node(node) => node.id,
                _ => {
                    warn!("Way with id {} has a non node reference in node references for some reason, skipping", way.id);
                    return;
                }
            };

            let kind = if way.tags.iter().any(|tag| {
                tag.key == "oneway" && tag.val != "no"
            }) {
                EdgeKind::Directed
            } else {
                EdgeKind::Undirected
            };

            let Some(&start_node_graph) = graph_id_by_import_id.get(&start_node) else {
                warn!("Encountered way with dangling node id: {way:#?}");
                return; 
            };
            let Some(&end_node_graph) = graph_id_by_import_id.get(&end_node) else {
                warn!("Encountered way with dangling node id: {way:#?}");
                return;
            };

            graph.add_edge(start_node_graph, end_node_graph, GraphWay { distance: OrderedFloat(1.) }, kind); // TODO: FIXME: actually compute distance
        });  
    }

    Ok(graph)
}

#[cfg(all(test, not(feature = "disable_graph_import_tests")))]
mod tests {
    use std::{cell::OnceCell, sync::{Once, OnceLock}};

    use log::trace;

    use super::*;

    struct TestData {
        xml_graph: Graph<GraphNode, GraphWay>,
        pbf_graph: Graph<GraphNode, GraphWay>,
    }

    static TEST_LOGGER: std::sync::Once = Once::new();

    static TEST_DATA: OnceLock<TestData> = OnceLock::new();

    fn init_test_logger() {
        TEST_LOGGER.call_once(|| {
            // ignore "already set" errors
            let _ = simple_logger::init();
        });
    }

    fn load_test_data() {
        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let xml_path = workspace_root.join("./maps/sacz_mniejszy.osm");
        let pbf_path = workspace_root.join("./maps/sacz_mniejszy.osm.pbf");

        TEST_DATA.get_or_init(|| TestData {
            xml_graph: import_xml(&xml_path).expect("failed to import xml"),
            pbf_graph: import_pbf(&pbf_path).expect("failed to import pbf"),
        });
    } 

    macro_rules! test_with_data {
        ($test_name:ident, |$xml:ident, $pbf:ident| $body:block) => {
            #[test]
            fn $test_name() {
                init_test_logger();
                load_test_data();

                let $xml = &TEST_DATA.get().unwrap().xml_graph;
                let $pbf = &TEST_DATA.get().unwrap().pbf_graph;

                $body
            }
        }
    }

    test_with_data!(check_if_xml_pbf_are_same, |xml, pbf| {
        assert_eq!(xml, pbf);
    });

    // #[test]
    // fn check_if_node_refs_correspond_to_node_ids() {
    //     todo!();
    // }

    test_with_data!(check_if_node_refs_correspond_to_node_ids, |xml, pbf| {
        for graph in [xml, pbf] {
            for id in graph.edges() {
                let nodes = graph.get_connected_nodes(id);

                assert!(graph.nodes().any(|n| nodes.from == n), "edge {id:?} has a from node id that doesn't correspond to any node in the graph: {nodes:#?}");
                assert!(graph.get_outgoing_edges(nodes.from).contains(&id), "edge {id:?} has a from node id that doesn't have this edge in its outgoing edges: {nodes:#?}");
                assert!(graph.get_incoming_edges(nodes.to).contains(&id), "edge {id:?} has a from node id that doesn't have this edge in its outgoing edges: {nodes:#?}");
            }
        }
    });
}
