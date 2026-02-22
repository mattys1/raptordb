mod graph;
mod store;
mod property_manager;
pub(crate) mod importer;
pub(crate) mod exporter;

use std::path::Path;

use graph::Graph;

use crate::database::{exporter::export_geojson, graph::id::{EdgePropertyID, EdgePropertyTypeID, NodePropertyID, NodePropertyTypeID}, importer::{GraphNode, GraphWay, ImportFormat, import_pbf, import_xml}, property_manager::PropertyManager};

pub struct Database {
    graph: Graph,

    node_properties: PropertyManager<NodePropertyID, NodePropertyTypeID>,
    edge_properties: PropertyManager<EdgePropertyID, EdgePropertyTypeID>
}

impl Database {
    pub fn new() -> Self {
        Database { 
            graph: Graph::new(),
            node_properties: PropertyManager::new(),
            edge_properties: PropertyManager::new(),
        }
    }

    //TODO: once actual db operations are implemented, revisit this so that it doesnt use the graph directly
    pub fn import_graph(&mut self, path: &Path, format: &ImportFormat) -> Result<(), Box<dyn std::error::Error>> {
        match format {
            ImportFormat::OSM => self.graph = import_xml(path)?,
            ImportFormat::PBF => self.graph = import_pbf(path)?,
        }

        Ok(())
    }

    pub fn export_graph(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        export_geojson(&self.graph, path)
    }
}
