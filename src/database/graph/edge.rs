use std::hash::Hash;

use derive_more::{Display, Eq, From};
use ordered_float::OrderedFloat;
use serde_json::Number;

use crate::database::{graph::{EdgeID, IDIntoUSize, NodeID, id::{EdgePropertyID, EdgePropertyTypeID}}, property_manager::PropertyIdentifier};

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum EdgeKind {
    Directed,
    Undirected
}

#[derive(Debug, Clone, Copy, Eq)]
pub(crate) struct Edge {
    pub(super) from: NodeID,
    pub(super) to: NodeID,
    // pub(super) cost: EdgeCost,
    // pub(super) kind: EdgeKind,
    // pub(super) property: EdgeProperty
    pub(super) data: EdgeData,
}

pub(super) type EdgeProperty = PropertyIdentifier<EdgePropertyID, EdgePropertyTypeID>; 

// TODO: move edge and node comparision into the graph itself, so that elements dependant on id can also be compared
impl PartialEq for Edge where {
    fn eq(&self, other: &Self) -> bool {
        self.data.kind == other.data.kind && self.data.property == other.data.property && self.data.cost == other.data.cost //&& match self.kind {
        //     EdgeKind::Directed => self.from == other.from && self.to == other.to,
        //     EdgeKind::Undirected => (self.from == other.from && self.to == other.to) || (self.from == other.to && self.to == other.from)
        // }
    }
}

// TODO: for now graph comparision relies on this but it would be better to hash everything probably
impl Hash for Edge where {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.kind.hash(state);
        self.data.property.hash(state);
        self.data.cost.hash(state);
    }
}

impl From<EdgeID> for geojson::feature::Id {
    fn from(value: EdgeID) -> Self {
        geojson::feature::Id::Number(Number::from(value.as_usize()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct EdgeCost(OrderedFloat<f64>);

impl From<f64> for EdgeCost {
    fn from(value: f64) -> Self {
        EdgeCost(OrderedFloat(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct EdgeData {
    pub(super) cost: EdgeCost,
    pub(super) kind: EdgeKind,
    pub(super) property: EdgeProperty
}
