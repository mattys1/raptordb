use std::fmt::{self, Display};

use crate::database::{
    id::{ClusterID, StringID},
    store::Store,
};

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

/// Arena of fixed-size clusters. Callers pass a real [`ClusterID`]; an empty chain is the caller's concern.
struct ClusterPool<T, const SIZE: usize>
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
    fn new() -> Self {
        Self {
            clusters: Store::new(),
        }
    }

    /// Allocate one cluster with no filled slots.
    fn alloc_empty(&mut self) -> ClusterID {
        self.clusters.add(Cluster::empty())
    }

    /// Pack `values` into a new chain. Returns `None` and allocates nothing when `values` is empty.
    fn alloc_chain(&mut self, values: impl IntoIterator<Item = T>) -> Option<ClusterID> {
        let mut values = values.into_iter().peekable();
        values.peek()?;

        let mut head: Option<ClusterID> = None;
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
            if head.is_none() {
                head = Some(id);
            }
            if let Some(tail_id) = tail {
                self.clusters.get_mut(tail_id).next = Some(id);
            }
            tail = Some(id);
        }

        head
    }

    /// Free `head` and every cluster linked from it.
    fn free_chain(&mut self, head: ClusterID) {
        let mut current = Some(head);
        while let Some(id) = current {
            let next = self.clusters.get(id).next;
            self.clusters.remove(id);
            current = next;
        }
    }

    /// Number of filled slots in the chain starting at `head`.
    fn len(&self, head: ClusterID) -> usize {
        let mut total = 0;
        let mut current = Some(head);
        while let Some(id) = current {
            let cluster = self.clusters.get(id);
            total += cluster.count;
            current = cluster.next;
        }
        total
    }

    fn locate(&self, head: ClusterID, mut index: usize) -> Option<(ClusterID, usize)> {
        let mut current = Some(head);
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
    fn get(&self, head: ClusterID, index: usize) -> Option<&T> {
        let (id, inner) = self.locate(head, index)?;
        Some(&self.clusters.get(id).contents[inner])
    }

    /// Mutable slot at `index`. `None` when `index` is past the end.
    fn get_mut(&mut self, head: ClusterID, index: usize) -> Option<&mut T> {
        let (id, inner) = self.locate(head, index)?;
        Some(&mut self.clusters.get_mut(id).contents[inner])
    }

    /// Filled slots from `head` to the end of the chain, in order.
    fn iter<'a>(&'a self, head: ClusterID) -> ClusterIter<'a, T, SIZE> {
        ClusterIter {
            pool: self,
            cluster_id: Some(head),
            index_in_cluster: 0,
        }
    }

    /// Append `value` to an existing chain. Allocates a new tail cluster when the last one is full. `head` does not change.
    fn push(&mut self, head: ClusterID, value: T) {
        let mut current = head;
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
    /// [`ChainRemoval::Kept`] leaves `head` valid. [`ChainRemoval::Emptied`] means the chain was freed and `head` must be dropped.
    /// `None` when `index` is past the end.
    fn remove_at(&mut self, head: ClusterID, index: usize) -> Option<ChainRemoval<T>> {
        let len = self.len(head);
        if index >= len {
            return None;
        }

        let removed = *self.get(head, index)?;
        for i in index..len - 1 {
            let next = *self.get(head, i + 1)?;
            let slot = self.get_mut(head, i)?;
            *slot = next;
        }

        if len == 1 {
            self.free_chain(head);
            Some(ChainRemoval::Emptied(removed))
        } else {
            self.truncate_to_len(head, len - 1);
            Some(ChainRemoval::Kept(removed))
        }
    }

    fn truncate_to_len(&mut self, head: ClusterID, new_len: usize) {
        debug_assert!(new_len > 0, "an empty chain is freed by the caller");

        let mut remaining = new_len;
        let mut current = Some(head);

        while let Some(id) = current {
            let next = self.clusters.get(id).next;
            if remaining <= SIZE {
                self.clusters.get_mut(id).count = remaining;
                self.clusters.get_mut(id).next = None;
                if let Some(next_id) = next {
                    self.free_chain(next_id);
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
enum ChainRemoval<T> {
    /// The chain still starts at the same head.
    Kept(T),
    /// The last slot was removed and the chain was freed.
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

/// Incident edges for a node. The node holds the chain head; `None` means the node has no edges.
pub(super) struct AdjacencyStore<E, const SIZE: usize>
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

    /// Number of edges in the chain. `None` is `0`.
    pub fn len(&self, head: Option<ClusterID>) -> usize {
        match head {
            Some(id) => self.pool.len(id),
            None => 0,
        }
    }

    /// Edge at `index`. `None` when `head` is `None` or `index` is past the end.
    pub fn get(&self, head: Option<ClusterID>, index: usize) -> Option<&E> {
        self.pool.get(head?, index)
    }

    /// Mutable edge at `index`. `None` when `head` is `None` or `index` is past the end.
    pub fn get_mut(&mut self, head: Option<ClusterID>, index: usize) -> Option<&mut E> {
        self.pool.get_mut(head?, index)
    }

    /// Edges in chain order. `None` yields nothing.
    pub fn iter<'a>(&'a self, head: Option<ClusterID>) -> AdjacencyIter<'a, E, SIZE> {
        AdjacencyIter {
            inner: head.map(|id| self.pool.iter(id)),
        }
    }

    /// Append `value`. When `head` is `None`, allocates the first cluster and writes its id back.
    pub fn push(&mut self, head: &mut Option<ClusterID>, value: E) {
        match *head {
            Some(id) => self.pool.push(id, value),
            None => *head = self.pool.alloc_chain(std::iter::once(value)),
        }
    }

    /// Remove the edge at `index` and close the gap. Sets `head` to `None` when the last edge is removed.
    /// `None` when `head` is `None` or `index` is past the end.
    pub fn remove_at(&mut self, head: &mut Option<ClusterID>, index: usize) -> Option<E> {
        let id = (*head)?;
        match self.pool.remove_at(id, index)? {
            ChainRemoval::Kept(value) => Some(value),
            ChainRemoval::Emptied(value) => {
                *head = None;
                Some(value)
            }
        }
    }

    /// Free the whole chain and set `head` to `None`. Does nothing when `head` is already `None`.
    pub fn free_chain(&mut self, head: &mut Option<ClusterID>) {
        if let Some(id) = *head {
            self.pool.free_chain(id);
            *head = None;
        }
    }
}

struct AdjacencyIter<'a, E, const SIZE: usize>
where
    E: Copy + Default,
{
    inner: Option<ClusterIter<'a, E, SIZE>>,
}

impl<'a, E, const SIZE: usize> Iterator for AdjacencyIter<'a, E, SIZE>
where
    E: Copy + Default,
{
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut()?.next()
    }
}

/// Strings split across clusters. Callers only see [`StringID`].
pub(super) struct StringStore<const SIZE: usize> {
    pool: ClusterPool<u8, SIZE>,
    heads: Store<ClusterID, StringID>,
}

impl<const SIZE: usize> StringStore<SIZE> {
    /// Empty store.
    pub fn new() -> Self {
        Self {
            pool: ClusterPool::new(),
            heads: Store::new(),
        }
    }

    /// Store `value` and return its id. An empty string still gets an id.
    pub fn add(&mut self, value: &str) -> StringID {
        let bytes = pack_utf8_chunks(value, SIZE).into_iter().flatten();
        let head = self
            .pool
            .alloc_chain(bytes)
            .unwrap_or_else(|| self.pool.alloc_empty());
        self.heads.add(head)
    }

    /// Delete the string and free its clusters.
    pub fn remove(&mut self, id: StringID) {
        let head = *self.heads.get(id);
        self.pool.free_chain(head);
        self.heads.remove(id);
    }

    /// Borrowed view of the string. Does not copy the bytes.
    pub fn get<'a>(&'a self, id: StringID) -> ClusteredStr<'a, SIZE> {
        ClusteredStr {
            pool: &self.pool,
            head: *self.heads.get(id),
        }
    }

    /// Number of stored strings.
    pub fn len(&self) -> usize {
        self.heads.len()
    }
}

/// A string borrowed from a [`StringStore`]. A value split across clusters is not one `&str`.
pub(super) struct ClusteredStr<'a, const SIZE: usize> {
    pool: &'a ClusterPool<u8, SIZE>,
    head: ClusterID,
}

impl<'a, const SIZE: usize> ClusteredStr<'a, SIZE> {
    /// Bytes in order, without copying them into a `String`.
    pub fn bytes(&self) -> impl Iterator<Item = u8> + 'a {
        self.pool.iter(self.head).copied()
    }

    /// Unicode scalar values in order.
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.cluster_slices().flat_map(|chunk| {
            std::str::from_utf8(chunk)
                .expect("stored string must be valid UTF-8")
                .chars()
        })
    }

    /// Length in bytes.
    pub fn len(&self) -> usize {
        self.pool.len(self.head)
    }

    /// `true` when the string has no bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `&str` when the whole value sits in one cluster. `None` when it spans more than one.
    pub fn as_str(&self) -> Option<&str> {
        let cluster = self.pool.clusters.get(self.head);
        if cluster.next.is_some() {
            return None;
        }
        std::str::from_utf8(cluster.get_filled()).ok()
    }

    /// Compare with `other` without building a `String`.
    pub fn eq_str(&self, other: &str) -> bool {
        self.bytes().eq(other.bytes())
    }

    /// Copy the clusters into an owned `String`.
    pub fn into_string(&self) -> String {
        self.cluster_slices()
            .map(|chunk| std::str::from_utf8(chunk).expect("stored string must be valid UTF-8"))
            .collect()
    }

    fn cluster_slices(&self) -> ClusterSliceIter<'_, SIZE> {
        ClusterSliceIter {
            pool: self.pool,
            cluster_id: Some(self.head),
        }
    }
}

impl<const SIZE: usize> PartialEq<str> for ClusteredStr<'_, SIZE> {
    fn eq(&self, other: &str) -> bool {
        self.eq_str(other)
    }
}

impl<const SIZE: usize> Display for ClusteredStr<'_, SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for chunk in self.cluster_slices() {
            let s = std::str::from_utf8(chunk).map_err(|_| fmt::Error)?;
            f.write_str(s)?;
        }
        Ok(())
    }
}

struct ClusterSliceIter<'a, const SIZE: usize> {
    pool: &'a ClusterPool<u8, SIZE>,
    cluster_id: Option<ClusterID>,
}

impl<'a, const SIZE: usize> Iterator for ClusterSliceIter<'a, SIZE> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.cluster_id?;
        let cluster = self.pool.clusters.get(id);
        let slice = cluster.get_filled();
        self.cluster_id = cluster.next;
        Some(slice)
    }
}

fn pack_utf8_chunks(value: &str, max_bytes: usize) -> Vec<Vec<u8>> {
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < value.len() {
        let mut end = (start + max_bytes).min(value.len());
        while end > start && !value.is_char_boundary(end) {
            end -= 1;
        }
        chunks.push(value[start..end].as_bytes().to_vec());
        start = end;
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::id::EdgeID;

    #[test]
    fn adjacency_push_and_iterate() {
        let mut store = AdjacencyStore::<EdgeID, 4>::new();
        let mut head = None;
        for i in 0..6 {
            store.push(&mut head, EdgeID::from(i));
        }
        assert_eq!(store.len(head), 6);
        let ids: Vec<_> = store.iter(head).copied().collect();
        assert_eq!(ids.len(), 6);
        assert_eq!(usize::from(ids[0]), 0);
        assert_eq!(usize::from(ids[5]), 5);
    }

    #[test]
    fn adjacency_remove_at_compacts() {
        let mut store = AdjacencyStore::<EdgeID, 4>::new();
        let mut head = None;
        for i in 0..5 {
            store.push(&mut head, EdgeID::from(i));
        }
        let removed = store.remove_at(&mut head, 2).unwrap();
        assert_eq!(usize::from(removed), 2);
        assert_eq!(store.len(head), 4);
        let ids: Vec<usize> = store.iter(head).map(|id| (*id).into()).collect();
        assert_eq!(ids, vec![0, 1, 3, 4]);
    }

    #[test]
    fn string_split_and_reassemble() {
        let mut store = StringStore::<4>::new();
        let id = store.add("quick brown fox");
        let view = store.get(id);
        assert!(view.as_str().is_none());
        assert!(view.eq_str("quick brown fox"));
        assert_eq!(view.into_string(), "quick brown fox");
        store.remove(id);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn string_single_cluster_as_str() {
        let mut store = StringStore::<32>::new();
        let id = store.add("hi");
        let view = store.get(id);
        assert_eq!(view.as_str(), Some("hi"));
    }
}
