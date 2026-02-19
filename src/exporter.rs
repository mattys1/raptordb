use std::path::{self, Path};

use geojson::{Feature, GeoJson, Geometry, JsonObject};
use osm_xml::Id;
use serde_json::Map;

use crate::{graph::Graph, importer::{GraphNode, GraphWay}};

pub fn export_geojson(graph: &Graph<GraphNode, GraphWay>, export_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // let json = JsonObject::new();
    let mut features = Vec::with_capacity(graph.nodes().count());
    
    for id in graph.nodes() {
        let node = graph.get_node(id); 
        let feature = Feature { 
            bbox: None,
            geometry: Some(Geometry { bbox: None, value: geojson::Value::Point(vec![node.lon.into(), node.lat.into()]), foreign_members: None }),
            id: Some(id.into()),
            properties: None,
            foreign_members: None 
        };

        features.push(feature);
    }

    let collection = geojson::FeatureCollection {
        features,
        bbox: None,
        foreign_members: None
    };

    std::fs::write(export_path, collection.to_string())?;
    Ok(())
}
