// TODO: the whole thing should eventually become multithreaded

use std::{collections::HashMap, error::Error, path::Path};

use derive_more::From;
use log::warn;
use osmpbf::{DenseTagIter, Element, ElementReader, TagIter};

use crate::graph::{EdgeKind, Graph, NodeID};


#[derive(Clone, Copy, From, Debug)]
struct Lattitude(f64);
#[derive(Clone, Copy, From, Debug)]
struct Longitude(f64);

#[derive(Copy, Debug, Clone)]
struct ImportedNode {
    lat: Lattitude,
    lon: Longitude,
}


#[derive(Copy, Debug, Clone)]
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

#[derive(Copy, Debug, Clone)]
pub struct GraphWay {
    distance: f64 //TODO: newtype this probably
}

enum TagMerger<'a> {
    DenseTagIter(DenseTagIter<'a>),
    TagIter(TagIter<'a>),
}

pub fn import_pbf(path: &Path) -> Result<Graph<GraphNode, GraphWay>, Box<dyn Error>> {
    let reader = ElementReader::from_path(path)?;
    let mut graph = Graph::<GraphNode, GraphWay>::new();

    let mut graph_id_by_import_id = HashMap::<i64, NodeID>::new();
    let mut imported_ways = Vec::<ImportedWay>::new();

    reader.for_each(|element| {
        // elements.push(match element {
        //     Element::Node(node) => node.id(),
        //     Element::DenseNode(dense_node) => dense_node.id(),
        //     Element::Way(way) => way.id(),
        //     Element::Relation(relation) => relation.id(),
        // });
        match element {
            // Element::Node(node) => nodes.push(ImportedNode { lat: node.lat().into(), lon: node.lon().into() }),
            Element::Node(node) => {
                let graph_id = graph.add_node(GraphNode { lat: node.lat().into(), lon: node.lon().into() });
                graph_id_by_import_id.insert(node.id(), graph_id);
            },
            Element::DenseNode(dense_node) => {
                let graph_id = graph.add_node(GraphNode { lat: dense_node.lat().into(), lon: dense_node.lon().into() });
                graph_id_by_import_id.insert(dense_node.id(), graph_id);
            },
            Element::Way(way) => imported_ways.push(ImportedWay { 
                node_refs: Iterator::collect(way.refs()),
                tags: way.tags().map(|(key, value)| { (key.into(), value.into()) } ).collect(),
                for_graph: GraphWay { distance: 1. } // TODO: distance should probably be computed when adding to graph and base it on nodes 
            }),
            Element::Relation(relation) => {
                warn!("Encountered relation with id {}, skipping", relation.id());
            },
        }

    })?;

    for way in imported_ways {
        way.node_refs.windows(2).for_each(|window| {
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


// this works for pbf files but the xml importer is broken
// though this is insanely slow
pub fn import_xml(path: &Path) -> Result<Graph<i64, i64>, Box<dyn Error>> {
    todo!()
}
