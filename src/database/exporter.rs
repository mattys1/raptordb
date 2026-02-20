use std::path::{self, Path};

use geojson::{Feature, GeoJson, Geometry, JsonObject, JsonValue};
use osm_xml::Id;
use serde_json::Map;

use crate::database::{graph::Graph, importer::{GraphNode, GraphWay}};

pub(super) fn export_geojson(graph: &Graph, export_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // let mut features = Vec::with_capacity(graph.nodes().count() + graph.edges().count());
    //
    // for id in graph.nodes() {
    //     let node = graph.get_node(id); 
    //     let feature = Feature { 
    //         bbox: None,
    //         geometry: Some(Geometry { bbox: None, value: geojson::Value::Point(vec![node.lon.into(), node.lat.into()]), foreign_members: None }),
    //         id: Some(id.into()),
    //         properties: None,
    //         foreign_members: None 
    //     };
    //
    //     features.push(feature);
    // }
    //
    // for id in graph.edges() {
    //     let nodes = graph.get_connected_nodes(id);
    //     let from = graph.get_node(nodes.from);
    //     let to = graph.get_node(nodes.to);
    //
    //     let line_geo = Geometry::new(geojson::Value::LineString(
    //         vec![
    //             vec![from.lon.into(), from.lat.into()],
    //             vec![to.lon.into(), to.lat.into()],
    //         ]
    //     ));
    //
    //     let mut props = Map::new();
    //     props.insert("from".to_string(), JsonValue::String(nodes.from.to_string()));
    //     props.insert("to".to_string(), JsonValue::String(nodes.to.to_string()));
    //
    //     let line_feature = Feature {
    //         bbox: None,
    //         geometry: Some(line_geo),
    //         id: None,
    //         properties: Some(props),
    //         foreign_members: None,
    //     };
    //
    //     features.push(line_feature);
    // }
    //
    // let collection = geojson::FeatureCollection {
    //     features,
    //     bbox: None,
    //     foreign_members: None
    // };
    //
    // std::fs::write(export_path, collection.to_string())?;
    // Ok(())

    todo!()
}
