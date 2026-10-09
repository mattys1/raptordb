use derive_more::Into;

use super::pool::{ChainID, ChainRemoval, ClusterPool};

/// Id of one node's incident edges. Wraps the chain that stores them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Into)]
pub(in crate::database) struct AdjacencyID(ChainID);

/// Incident edges for a node. Each chain is addressed by an [`AdjacencyID`].
pub(in crate::database) struct AdjacencyStore<E, const SIZE: usize>
where
    E: Copy + Default,
{
    pool: ClusterPool<E, SIZE>,
}

impl<E, const SIZE: usize> AdjacencyStore<E, SIZE>
where
    E: Copy + Default,
{
    /// Empty store.
    pub fn new() -> Self {
        Self {
            pool: ClusterPool::new(),
        }
    }

    /// Start a chain containing `value`.
    pub fn add(&mut self, value: E) -> AdjacencyID {
        let chain = self
            .pool
            .alloc_chain(std::iter::once(value))
            .expect("one value produces a chain");
        AdjacencyID(chain)
    }

    /// Number of edges in the chain.
    pub fn len(&self, id: AdjacencyID) -> usize {
        self.pool.len(id.into())
    }

    /// Edge at `index`. `None` when `index` is past the end.
    pub fn get(&self, id: AdjacencyID, index: usize) -> Option<&E> {
        self.pool.get(id.into(), index)
    }

    /// Mutable edge at `index`. `None` when `index` is past the end.
    pub fn get_mut(&mut self, id: AdjacencyID, index: usize) -> Option<&mut E> {
        self.pool.get_mut(id.into(), index)
    }

    /// Edges in chain order.
    pub fn iter(&self, id: AdjacencyID) -> impl Iterator<Item = &E> + '_ {
        self.pool.iter(id.into())
    }

    /// Append `value` to the chain.
    pub fn push(&mut self, id: AdjacencyID, value: E) {
        self.pool.push(id.into(), value);
    }

    /// Remove the edge at `index` and close the gap.
    ///
    /// [`ChainRemoval::Emptied`] means `id` is no longer valid. `None` when `index` is past the end.
    pub fn remove_at(&mut self, id: AdjacencyID, index: usize) -> Option<ChainRemoval<E>> {
        self.pool.remove_at(id.into(), index)
    }

    /// Free the chain. `id` is no longer valid.
    pub fn free(&mut self, id: AdjacencyID) {
        self.pool.free_chain(id.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::id::EdgeID;

    #[test]
    fn adjacency_push_and_iterate() {
        let mut store = AdjacencyStore::<EdgeID, 4>::new();
        let id = store.add(EdgeID::from(0));
        for i in 1..6 {
            store.push(id, EdgeID::from(i));
        }
        assert_eq!(store.len(id), 6);
        let ids: Vec<_> = store.iter(id).copied().collect();
        assert_eq!(ids.len(), 6);
        assert_eq!(usize::from(ids[0]), 0);
        assert_eq!(usize::from(ids[5]), 5);
    }

    #[test]
    fn adjacency_remove_at_compacts() {
        let mut store = AdjacencyStore::<EdgeID, 4>::new();
        let id = store.add(EdgeID::from(0));
        for i in 1..5 {
            store.push(id, EdgeID::from(i));
        }
        let ChainRemoval::Kept(removed) = store.remove_at(id, 2).unwrap() else {
            panic!("chain should remain");
        };
        assert_eq!(usize::from(removed), 2);
        assert_eq!(store.len(id), 4);
        let ids: Vec<usize> = store.iter(id).map(|edge| (*edge).into()).collect();
        assert_eq!(ids, vec![0, 1, 3, 4]);
    }
}
