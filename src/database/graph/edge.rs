use std::hash::Hash;

use derive_more::{Display, Eq, From};
use serde_json::Number;

use crate::database::graph::{NodeID, id::{EdgePropertyID, EdgePropertyTypeID}};

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum EdgeKind {
    Directed,
    Undirected
}

#[derive(Debug, Clone, Copy, std::cmp::Eq)]
pub(crate) struct Edge {
    pub(super) from: NodeID,
    pub(super) to: NodeID,
    pub(super) kind: EdgeKind,
    pub(super) property_id: EdgePropertyID,
    pub(super) property_type_id: EdgePropertyTypeID,
}

// TODO: move edge and node comparision into the graph itself, so that elements dependant on id can also be compared
impl PartialEq for Edge where {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.property == other.property //&& match self.kind {
        //     EdgeKind::Directed => self.from == other.from && self.to == other.to,
        //     EdgeKind::Undirected => (self.from == other.from && self.to == other.to) || (self.from == other.to && self.to == other.from)
        // }
    }
}

// TODO: for now graph comparision relies on this but it would be better to hash everything probably
impl Hash for Edge where {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.property.hash(state);
    }
}

impl From<EdgeID> for geojson::feature::Id {
    fn from(value: EdgeID) -> Self {
        geojson::feature::Id::Number(Number::from(value.0))
    }
}

// #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Display)]
// pub struct PropertyEdgeID(usize);
//
// #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Display)]
// pub struct PropertyEdgeTypeID(usize);

