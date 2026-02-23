use serde_json::Number;

use crate::database::{graph::{IDIntoUSize, NodeID, id::{EdgeID, NodePropertyID, NodePropertyTypeID}}, importer::{Lattitude, Longitude}, property_manager::PropertyIdentifier};

#[derive(Debug, Eq)]
pub(super) struct Node {
    // pub(super) id: NodeID,
    // pub(super) lat: Lattitude,
    // pub(super) lon: Longitude,
    pub(super) edges: Vec<EdgeID>,

    // pub(super) property: NodeProperty

    pub(super) data: NodeData
}

pub(super) type NodeProperty = PropertyIdentifier<NodePropertyID, NodePropertyTypeID>; 

// TODO: move edge and node comparision into the graph itself, so that elements dependant on id can also be compared
// FIXME: this especially affects `Node`
impl PartialEq for Node where {
    fn eq(&self, other: &Self) -> bool {
        self.data.lat == other.data.lat && self.data.lon == other.data.lon && self.data.property == other.data.property && self.edges.len() == other.edges.len()
    } 
}

impl From<NodeID> for geojson::feature::Id {
    fn from(value: NodeID) -> Self {
        geojson::feature::Id::Number(Number::from(value.as_usize()))
    }
}

#[derive(Debug, Eq)]
pub(crate) struct NodeData {
    pub lat: Lattitude,
    pub lon: Longitude,

    pub(super) property: NodeProperty
}

impl PartialEq for NodeData {
    fn eq(&self, other: &Self) -> bool {
        self.lat == other.lat && self.lon == other.lon && self.property == other.property
    }
}
