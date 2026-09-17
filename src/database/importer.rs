// TODO: the whole thing should eventually become multithreaded
use std::{collections::HashMap, error::Error, fmt::Debug, fs::File, path::Path};

use derive_more::From;
use log::warn;
use ordered_float::OrderedFloat;
use osm_xml::OSM;
use osmpbf::{Element, ElementReader, TagIter};

use crate::database::{EdgePropertyID, EdgePropertyTypeID, NodePropertyID, NodePropertyTypeID, PropertyManager, graph::{IDIntoUSize, id::NodeID}, property_manager::{self, PropertyField, type_registry::{FieldDescriptor, FieldType, TypeDescriptor}}};

use super::graph::{EdgeKind, Graph};

pub(crate) enum ImportFormat {
    OSM,
    PBF
}

#[derive(Clone, Copy, From, Debug, PartialEq, Hash, Eq)]
pub struct Lattitude(OrderedFloat<f64>);
impl From<Lattitude> for f64 {
    fn from(value: Lattitude) -> Self {
        value.0.into()
    }
}

#[derive(Clone, Copy, From, Debug, PartialEq, Hash, Eq)]
pub struct Longitude(OrderedFloat<f64>);

impl From<Longitude> for f64 {
    fn from(value: Longitude) -> Self {
        value.0.into()
    }
}

#[derive(Copy, Debug, Clone)]
struct ImportedNode {
    lat: Lattitude,
    lon: Longitude,
}


#[derive(Copy, Debug, Clone, PartialEq, Hash, Eq)]
pub struct GraphNode {
    pub lat: Lattitude,
    pub lon: Longitude,
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

pub(super) struct Importer<'a> {
    node_properties: &'a mut PropertyManager<NodePropertyID, NodePropertyTypeID>,
    edge_properties: &'a mut PropertyManager<EdgePropertyID, EdgePropertyTypeID>,

    node_types: Vec<NodePropertyTypeID>,
    edge_types: Vec<EdgePropertyTypeID>,
}

impl<'a> Importer<'a> {
    pub fn new(
            node_properties: &'a mut PropertyManager::<NodePropertyID, NodePropertyTypeID>,
            edge_properties: &'a mut PropertyManager::<EdgePropertyID, EdgePropertyTypeID>,
    ) -> Self {
            let node_types = vec![
                TypeDescriptor::new(
                    "Amenity".into(),
                    vec![
                        FieldDescriptor::new(String::from("name").into(), FieldType::String, false),
                    ]
                )
            ];

            let edge_types = vec![
                TypeDescriptor::new(
                    "Highway".into(),
                    vec![
                        FieldDescriptor::new(String::from("type").into(), FieldType::String, false),
                    ]
                ),
            ];

        let mut node_type_ids = Vec::<NodePropertyTypeID>::new();
        for node_type in &node_types {
            let id = node_properties.register_type(node_type.clone());
            node_type_ids.push(id);
        }

        let mut edge_type_ids = Vec::<EdgePropertyTypeID>::new();
        for edge_type in &edge_types {
            let id = edge_properties.register_type(edge_type.clone());
            edge_type_ids.push(id);
        }

        Importer { node_properties, edge_properties, node_types: node_type_ids, edge_types: edge_type_ids }
    }

    fn haversine_distance(start: &GraphNode, end: &GraphNode) -> f64 {
        const EARTH_RADIUS_M: f64 = 6371e3;

        let lat1 = start.lat.0.into_inner();
        let lon1 = start.lon.0.into_inner();
        let lat2 = end.lat.0.into_inner();
        let lon2 = end.lon.0.into_inner();

        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lon = (lon2 - lon1).to_radians();

        let a = (delta_lat / 2.0).sin() * (delta_lat / 2.0).sin()
        + lat1_rad.cos() * lat2_rad.cos()
        * (delta_lon / 2.0).sin() * (delta_lon / 2.0).sin();
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_M * c
    }

    // TODO: hacky but technically solves floating point precision issues, should base node identity on labelling in the source instead of coordinates
    fn quantize_coord(v: f64) -> f64 {
        const COORD_QUANT: f64 = 1e7;
        (v * COORD_QUANT).round() / COORD_QUANT
    }

    fn add_property_to(&mut self, id: NodePropertyTypeID, field_contents: &[PropertyField]) -> Option<NodePropertyID> {
         self.node_properties.add_property(id, field_contents).ok()
    }

    // fn add_property_to<EdgePropertyID>(&mut self, id: NodePropertyTypeID, field_contents: &[PropertyField]) -> EdgePropertyID {
    //      self.edge_properties.add_property(id, field_contents).unwrap()
    // }

    // fn try_adding_based_on_tags<TypeId>(
    //     tags: &TagIter,
    //     requirements: Vec<(&str, Box<dyn Fn(&str, &str) -> bool>)>,
    //     // keys_type: &[(String, FieldType)],
    //     transformations: Vec<(&str, Box<dyn Fn()>)>
    // ) -> Option<TypeId> where TypeId: Copy + IDIntoUSize + Debug {
    //     let mut requirements_passed = Vec::<bool>::new();
    //     requirements_passed.resize(requirements.len(), false);
    //
    //     for (req, passed) in requirements.iter().zip(requirements_passed.iter_mut()) {
    //         for (key, value) in tags.clone() {
    //             if req((key, value)) {
    //                 *passed = true;
    //             }
    //         }
    //     }
    //
    //     if requirements_passed.iter().any(|p| *p == false) {
    //         return None
    //     }
    //
    //     let fields = Vec::<FieldDescriptor>::new();
    //     for (key, value) in tags.clone() {
    //         if let Some(key_type) = keys_type.iter().find(|kt| kt.0 == key) {
    //             let field_type = key_type.1;
    //             let property_name = &key_type.0;
    //         }
    //     }
    //
    //     todo!()
    // }

pub fn try_adding_based_on_tags(
    &mut self,
    tags: &osmpbf::TagIter,
    mappings: Vec<(&str, Vec<(&str, FieldDescriptor)>)>,
) -> Option<NodePropertyTypeID> {
    let tag_map: HashMap<&str, &str> = tags.clone().map(|(k, v)| (k, v)).collect();

    for (type_name, fields) in mappings {
        if let Some(type_id) = self.node_properties.get_type_by_name(type_name) {
            let type_desc = self.node_properties.describe(type_id);
            let mut property_fields = Vec::new();
            let mut all_found = true;

            for field in &type_desc.fields {
                // Find the mapping for this field name
                if let Some((key, field_desc)) = fields.iter().find(|(k, _)| field.name == String::from(*k).into()) {
                    if let Some(tag_val) = tag_map.get(*key) {
                        // Convert tag_val (string) to PropertyFieldContents based on field_type
                        let value = match field_desc.field_type {
                            property_manager::type_registry::FieldType::Integer => {
                                tag_val.parse::<i64>().ok().map(property_manager::property::PropertyFieldContents::Integer)
                            }
                            property_manager::type_registry::FieldType::Float => {
                                tag_val.parse::<f64>().ok().map(property_manager::property::PropertyFieldContents::Float)
                            }
                            property_manager::type_registry::FieldType::String => {
                                Some(property_manager::property::PropertyFieldContents::String(tag_val.to_string()))
                            }
                            property_manager::type_registry::FieldType::Boolean => {
                                match *tag_val {
                                    "true" | "yes" | "1" => Some(property_manager::property::PropertyFieldContents::Boolean(true)),
                                    "false" | "no" | "0" => Some(property_manager::property::PropertyFieldContents::Boolean(false)),
                                    _ => None,
                                }
                            }
                        };

                        if let Some(value) = value {
                            property_fields.push(property_manager::PropertyField::new(field.name.clone(), value));
                        } else {
                            all_found = false;
                            break;
                        }
                    } else {
                        all_found = false;
                        break;
                    }
                } else {
                    all_found = false;
                    break;
                }
            }

            if all_found {
                self.add_property_to(type_id, &property_fields);
                return Some(type_id);
            }
        }
    }
    None
}

    pub(super) fn import_pbf(&self, path: &Path) -> Result<Graph, Box<dyn Error>> {
        let reader = ElementReader::from_path(path)?;
        let mut graph = Graph::new();

        let mut graph_id_by_import_id = HashMap::<i64, NodeID>::new();
        let mut imported_ways = Vec::<ImportedWay>::new();

        reader.for_each(|element| {
            match element {
                Element::Node(node) => {
                    let lat = Importer::quantize_coord(node.lat());
                    let lon = Importer::quantize_coord(node.lon());
                    let graph_id = graph.add_node(GraphNode { lat: OrderedFloat(lat).into(), lon: OrderedFloat(lon).into() });
                    graph_id_by_import_id.insert(node.id(), graph_id);
                },
                Element::DenseNode(dense_node) => {
                    let lat = Importer::quantize_coord(dense_node.lat());
                    let lon = Importer::quantize_coord(dense_node.lon());
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
        
                graph.add_edge(start_node_graph, end_node_graph, GraphWay { distance: OrderedFloat(haversine_distance(graph.get_node(start_node_graph), graph.get_node(end_node_graph))) }, kind);
            });
        }
        
        Ok(graph)
        
    }

    pub(super) fn import_xml(path: &Path) -> Result<Graph, Box<dyn Error>> {
        todo!()
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
        
                graph.add_edge(start_node_graph, end_node_graph, GraphWay { distance: OrderedFloat(haversine_distance(graph.get_node(start_node_graph), graph.get_node(end_node_graph))) }, kind);
            });  
        }
        
        Ok(graph)
    }

    pub fn import(&self, format: ImportFormat, path: &Path) -> Result<Graph, Box<dyn Error>> {
        self.register_types(); 

        match format {
            ImportFormat::OSM => Self::import_xml(path),
            ImportFormat::PBF => Self::import_pbf(path)
        }
    }
}

#[cfg(all(test, not(feature = "disable_graph_import_tests")))]
mod tests {
    // use std::{cell::OnceCell, sync::{Once, OnceLock}};
    //
    // use log::trace;
    //
    // use super::*;
    //
    // struct TestData {
    //     xml_graph: Graph<GraphNode, GraphWay>,
    //     pbf_graph: Graph<GraphNode, GraphWay>,
    // }
    //
    // static TEST_LOGGER: std::sync::Once = Once::new();
    //
    // static TEST_DATA: OnceLock<TestData> = OnceLock::new();
    //
    // fn init_test_logger() {
    //     TEST_LOGGER.call_once(|| {
    //         // ignore "already set" errors
    //         let _ = simple_logger::init();
    //     });
    // }
    //
    // fn load_test_data() {
    //     let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    //     let xml_path = workspace_root.join("./maps/sacz_mniejszy.osm");
    //     let pbf_path = workspace_root.join("./maps/sacz_mniejszy.osm.pbf");
    //
    //     TEST_DATA.get_or_init(|| TestData {
    //         xml_graph: import_xml(&xml_path).expect("failed to import xml"),
    //         pbf_graph: import_pbf(&pbf_path).expect("failed to import pbf"),
    //     });
    // } 
    //
    // macro_rules! test_with_data {
    //     ($test_name:ident, |$xml:ident, $pbf:ident| $body:block) => {
    //         #[test]
    //         fn $test_name() {
    //             init_test_logger();
    //             load_test_data();
    //
    //             let $xml = &TEST_DATA.get().unwrap().xml_graph;
    //             let $pbf = &TEST_DATA.get().unwrap().pbf_graph;
    //
    //             $body
    //         }
    //     }
    // }
    //
    //
    // #[test]
    // fn test_distance_computation() {
    //     let node1 = GraphNode { lat: Lattitude(OrderedFloat(0.0)), lon: Longitude(OrderedFloat(0.0)) };
    //     let node2 = GraphNode { lat: Lattitude(OrderedFloat(1.0)), lon: Longitude(OrderedFloat(1.0)) };
    //     let dist = haversine_distance(&node1, &node2);
    //
    //     let expected = 157_249.381_271_943_97; 
    //     assert!((dist - expected).abs() < 1.0, "Expected {expected}, got {dist}");
    // }
    //
    // test_with_data!(check_if_xml_pbf_are_same, |xml, pbf| {
    //     assert_eq!(xml, pbf);
    // });
    //
    // // #[test]
    // // fn check_if_node_refs_correspond_to_node_ids() {
    // //     todo!();
    // // }
    //
    // test_with_data!(check_if_node_refs_correspond_to_node_ids, |xml, pbf| {
    //     for graph in [xml, pbf] {
    //         for id in graph.edges() {
    //             let nodes = graph.get_connected_nodes(id);
    //
    //             assert!(graph.nodes().any(|n| nodes.from == n), "edge {id:?} has a from node id that doesn't correspond to any node in the graph: {nodes:#?}");
    //             assert!(graph.get_outgoing_edges(nodes.from).contains(&id), "edge {id:?} has a from node id that doesn't have this edge in its outgoing edges: {nodes:#?}");
    //             assert!(graph.get_incoming_edges(nodes.to).contains(&id), "edge {id:?} has a from node id that doesn't have this edge in its outgoing edges: {nodes:#?}");
    //         }
    //     }
    // });
}
