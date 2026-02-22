use serde_json::Number;

use crate::database::{graph::{IDIntoUSize, NodeID, id::{EdgeID, NodePropertyID, NodePropertyTypeID}}, property_manager::PropertyIdentifier};

#[derive(Debug, Eq)]
pub(in crate) struct Node {
    // pub(super) id: NodeID,
    pub(super) edges: Vec<EdgeID>,
    pub(super) property: NodeProperty
}

pub(super) type NodeProperty = PropertyIdentifier<NodePropertyID, NodePropertyTypeID>; 

// TODO: move edge and node comparision into the graph itself, so that elements dependant on id can also be compared
// FIXME: this especially affects `Node`
impl PartialEq for Node where {
    fn eq(&self, other: &Self) -> bool {
        self.property == other.property && self.edges.len() == other.edges.len()
    } 
}

impl From<NodeID> for geojson::feature::Id {
    fn from(value: NodeID) -> Self {
        geojson::feature::Id::Number(Number::from(value.as_usize()))
    }
}
