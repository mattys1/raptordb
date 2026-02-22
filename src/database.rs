mod graph;
mod property;
mod store;
mod type_registry;
pub(crate) mod importer;
pub(crate) mod exporter;

use std::path::Path;

use graph::Graph;

use crate::database::{exporter::export_geojson, graph::id::{EdgePropertyID, EdgePropertyTypeID, NodePropertyID, NodePropertyTypeID}, importer::{GraphNode, GraphWay, ImportFormat, import_pbf, import_xml}, property::{PropertyFieldContents, PropertyStore}, type_registry::{FieldDescriptor, TypeRegistry}};

pub struct Database {
    graph: Graph,

    node_type_registry: TypeRegistry<NodePropertyTypeID>,
    edge_type_registry: TypeRegistry<EdgePropertyTypeID>,

    node_properties: PropertyStore<NodePropertyID, NodePropertyTypeID>,
    edge_properties: PropertyStore<EdgePropertyID, EdgePropertyTypeID>,
}

impl Database {
    pub fn new() -> Self {
        Database { 
            graph: Graph::new(),
            node_type_registry: TypeRegistry::new(),
            edge_type_registry: TypeRegistry::new(),
            node_properties: PropertyStore::new(),
            edge_properties: PropertyStore::new(),
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

    pub fn import_type(&mut self) -> NodePropertyTypeID {
        let descriptor = self.node_type_registry.add_type("test_type".into(), vec![
            FieldDescriptor {
                name: "field1".into(),
                field_type: type_registry::FieldType::Integer,
                nullable: false,
            },
            FieldDescriptor {
                name: "field2".into(),
                field_type: type_registry::FieldType::String,
                nullable: false,
            }
        ]);

        self.node_properties.add_type(descriptor)
    }

    pub fn add_node_property(&mut self, id: NodePropertyTypeID) {
        self.node_properties.add_property(id, vec![
            PropertyFieldContents::Integer(1),
            PropertyFieldContents::String("test".into()),
        ]);
    }
}

#[cfg(test)]
mod tests {
    use crate::database::Database;

    #[test]
    fn import_type_test() {
        let mut db = Database::new();
        let id = db.import_type();

        db.add_node_property(id);
    }
}
