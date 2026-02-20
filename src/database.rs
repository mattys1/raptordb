mod graph;
mod property;
mod store;
pub(crate) mod importer;
pub(crate) mod exporter;

use std::path::Path;

use graph::Graph;

use crate::{database::{exporter::export_geojson, importer::{GraphNode, GraphWay, ImportFormat, import_pbf, import_xml}}};

pub struct Database {
    graph: Graph
}

impl Database {

    pub fn new() -> Self {
        Database { graph: Graph::new() }
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
