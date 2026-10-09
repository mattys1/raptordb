use crate::database::{id::ClusterID, store::Store};

/// First cluster of a chain. Only [`ClusterPool::alloc_empty`] and [`ClusterPool::alloc_chain`] construct it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct ChainID(ClusterID);

impl ChainID {
    fn cluster(self) -> ClusterID {
        self.0
    }
}

#[derive(Clone, Copy)]
struct Cluster<T, const SIZE: usize>
where
    T: Copy,
{
    contents: [T; SIZE],
    next: Option<ClusterID>,
    count: usize,
}

impl<T, const SIZE: usize> Cluster<T, SIZE>
where
    T: Copy + Default,
{
    fn empty() -> Self {
        Self {
            contents: [T::default(); SIZE],
            next: None,
            count: 0,
        }
    }

    fn get_filled(&self) -> &[T] {
        &self.contents[..self.count]
    }
}

impl<T, const SIZE: usize> Default for Cluster<T, SIZE>
where
    T: Copy + Default,
{
    fn default() -> Self {
        Self::empty()
    }
}

/// Arena of fixed-size clusters. Chain operations take a [`ChainID`]. Links between clusters stay [`ClusterID`].
pub(super) struct ClusterPool<T, const SIZE: usize>
where
    T: Copy + Default,
{
    clusters: Store<Cluster<T, SIZE>, ClusterID>,
}

impl<T, const SIZE: usize> ClusterPool<T, SIZE>
where
    T: Copy + Default,
{
    /// Empty arena.
    pub(super) fn new() -> Self {
        Self {
            clusters: Store::new(),
        }
    }

    /// Allocate one empty chain.
    pub(super) fn alloc_empty(&mut self) -> ChainID {
        ChainID(self.clusters.add(Cluster::empty()))
    }

    /// Pack `values` into a new chain. Returns `None` and allocates nothing when `values` is empty.
    pub(super) fn alloc_chain(&mut self, values: impl IntoIterator<Item = T>) -> Option<ChainID> {
        let mut values = values.into_iter().peekable();
        values.peek()?;

        let mut first: Option<ClusterID> = None;
        let mut tail: Option<ClusterID> = None;

        while values.peek().is_some() {
            let mut cluster = Cluster::<T, SIZE>::empty();
            let mut count = 0usize;
            while count < SIZE {
                match values.next() {
                    Some(value) => {
                        cluster.contents[count] = value;
                        count += 1;
                    }
                    None => break,
                }
            }
            cluster.count = count;

            let id = self.clusters.add(cluster);
            if first.is_none() {
                first = Some(id);
            }
            if let Some(tail_id) = tail {
                self.clusters.get_mut(tail_id).next = Some(id);
            }
            tail = Some(id);
        }

        first.map(ChainID)
    }

    /// Free every cluster in the chain, starting at its first cluster.
    pub(super) fn free_chain(&mut self, chain: ChainID) {
        self.free_suffix(chain.cluster());
    }

    /// Free this cluster and every cluster linked after it.
    fn free_suffix(&mut self, start: ClusterID) {
        let mut current = Some(start);
        while let Some(id) = current {
            let next = self.clusters.get(id).next;
            self.clusters.remove(id);
            current = next;
        }
    }

    /// Number of filled slots in the chain.
    pub(super) fn len(&self, chain: ChainID) -> usize {
        let mut total = 0;
        let mut current = Some(chain.cluster());
        while let Some(id) = current {
            let cluster = self.clusters.get(id);
            total += cluster.count;
            current = cluster.next;
        }
        total
    }

    fn locate(&self, chain: ChainID, mut index: usize) -> Option<(ClusterID, usize)> {
        let mut current = Some(chain.cluster());
        while let Some(id) = current {
            let cluster = self.clusters.get(id);
            let count = cluster.count;
            if index < count {
                return Some((id, index));
            }
            index -= count;
            current = cluster.next;
        }
        None
    }

    /// Slot at `index`, counting from the start of the chain. `None` when `index` is past the end.
    pub(super) fn get(&self, chain: ChainID, index: usize) -> Option<&T> {
        let (id, inner) = self.locate(chain, index)?;
        Some(&self.clusters.get(id).contents[inner])
    }

    /// Mutable slot at `index`. `None` when `index` is past the end.
    pub(super) fn get_mut(&mut self, chain: ChainID, index: usize) -> Option<&mut T> {
        let (id, inner) = self.locate(chain, index)?;
        Some(&mut self.clusters.get_mut(id).contents[inner])
    }

    /// Filled slots from the start of the chain to its end, in order.
    pub(super) fn iter<'a>(&'a self, chain: ChainID) -> impl Iterator<Item = &'a T> + 'a {
        ClusterIter {
            pool: self,
            cluster_id: Some(chain.cluster()),
            index_in_cluster: 0,
        }
    }

    /// Filled bytes of each cluster, in order. Each slice stops at that cluster's count.
    pub(super) fn filled_slices<'a>(
        &'a self,
        chain: ChainID,
    ) -> impl Iterator<Item = &'a [T]> + 'a {
        FilledSliceIter {
            pool: self,
            cluster_id: Some(chain.cluster()),
        }
    }

    /// The chain's only filled slice, when every value sits in the first cluster.
    pub(super) fn sole_filled(&self, chain: ChainID) -> Option<&[T]> {
        let cluster = self.clusters.get(chain.cluster());
        if cluster.next.is_some() {
            None
        } else {
            Some(cluster.get_filled())
        }
    }

    /// Append `value` to the chain. Allocates a new tail cluster when the last one is full.
    pub(super) fn push(&mut self, chain: ChainID, value: T) {
        let mut current = chain.cluster();
        loop {
            let next = self.clusters.get(current).next;
            match next {
                Some(next_id) => current = next_id,
                None => break,
            }
        }
        let cluster = self.clusters.get_mut(current);
        if cluster.count < SIZE {
            cluster.contents[cluster.count] = value;
            cluster.count += 1;
        } else {
            let mut new_cluster = Cluster::<T, SIZE>::empty();
            new_cluster.contents[0] = value;
            new_cluster.count = 1;
            let new_id = self.clusters.add(new_cluster);
            self.clusters.get_mut(current).next = Some(new_id);
        }
    }

    /// Remove the slot at `index` and close the gap.
    ///
    /// [`ChainRemoval::Kept`] leaves `chain` valid. [`ChainRemoval::Emptied`] means the chain was freed and `chain` must be dropped.
    /// `None` when `index` is past the end.
    pub(super) fn remove_at(&mut self, chain: ChainID, index: usize) -> Option<ChainRemoval<T>> {
        let len = self.len(chain);
        if index >= len {
            return None;
        }

        let removed = *self.get(chain, index)?;
        for i in index..len - 1 {
            let next = *self.get(chain, i + 1)?;
            let slot = self.get_mut(chain, i)?;
            *slot = next;
        }

        if len == 1 {
            self.free_chain(chain);
            Some(ChainRemoval::Emptied(removed))
        } else {
            self.truncate_to_len(chain, len - 1);
            Some(ChainRemoval::Kept(removed))
        }
    }

    fn truncate_to_len(&mut self, chain: ChainID, new_len: usize) {
        debug_assert!(new_len > 0, "an empty chain is freed by the caller");

        let mut remaining = new_len;
        let mut current = Some(chain.cluster());

        while let Some(id) = current {
            let next = self.clusters.get(id).next;
            if remaining <= SIZE {
                self.clusters.get_mut(id).count = remaining;
                self.clusters.get_mut(id).next = None;
                if let Some(next_id) = next {
                    self.free_suffix(next_id);
                }
                break;
            }
            remaining -= SIZE;
            self.clusters.get_mut(id).count = SIZE;
            current = next;
        }
    }
}

/// Result of removing one slot from a chain.
pub(in crate::database) enum ChainRemoval<T> {
    /// The chain id is still valid.
    Kept(T),
    /// The last slot was removed and the chain was freed. The id is no longer valid.
    Emptied(T),
}

struct ClusterIter<'a, T, const SIZE: usize>
where
    T: Copy + Default,
{
    pool: &'a ClusterPool<T, SIZE>,
    cluster_id: Option<ClusterID>,
    index_in_cluster: usize,
}

impl<'a, T, const SIZE: usize> Iterator for ClusterIter<'a, T, SIZE>
where
    T: Copy + Default,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.cluster_id?;
        let cluster = self.pool.clusters.get(id);
        if self.index_in_cluster >= cluster.count {
            self.cluster_id = cluster.next;
            self.index_in_cluster = 0;
            return self.next();
        }
        let value = &cluster.contents[self.index_in_cluster];
        self.index_in_cluster += 1;
        Some(value)
    }
}

struct FilledSliceIter<'a, T, const SIZE: usize>
where
    T: Copy + Default,
{
    pool: &'a ClusterPool<T, SIZE>,
    cluster_id: Option<ClusterID>,
}

impl<'a, T, const SIZE: usize> Iterator for FilledSliceIter<'a, T, SIZE>
where
    T: Copy + Default,
{
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.cluster_id?;
        let cluster = self.pool.clusters.get(id);
        let slice = cluster.get_filled();
        self.cluster_id = cluster.next;
        Some(slice)
    }
}
