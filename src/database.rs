mod graph;
mod store;
mod property_manager;
pub(crate) mod importer;
pub(crate) mod exporter;

use std::path::Path;

use graph::Graph;

use crate::database::{exporter::export_geojson, graph::id::{EdgePropertyID, EdgePropertyTypeID, NodePropertyID, NodePropertyTypeID}, importer::{GraphNode, GraphWay, ImportFormat, Importer, }, property_manager::PropertyManager};

pub struct Database {
    graph: Graph,

    node_properties: PropertyManager<NodePropertyID, NodePropertyTypeID>,
    edge_properties: PropertyManager<EdgePropertyID, EdgePropertyTypeID>,

    importer: Importer,
}

impl Database {
    pub fn new() -> Self {
        Database { 
            graph: Graph::new(),
            node_properties: PropertyManager::new(),
            edge_properties: PropertyManager::new(),
            importer: Importer::new(),
        }
    }

    //TODO: once actual db operations are implemented, revisit this so that it doesnt use the graph directly
    pub fn import_graph(&mut self, path: &Path, format: ImportFormat) -> Result<(), Box<dyn std::error::Error>> {
        self.importer.import(format, path)?;
        Ok(())
    }

    pub fn export_graph(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        export_geojson(&self.graph, path)
    }
}
