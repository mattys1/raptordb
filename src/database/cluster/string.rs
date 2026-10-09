use std::fmt::{self, Display};

use derive_more::Into;

use super::pool::{ChainID, ClusterPool};

/// Id of one string. Wraps the chain that stores its bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Into)]
pub(in crate::database) struct StringID(ChainID);

pub(in crate::database) struct StringStore<const SIZE: usize> {
    pool: ClusterPool<u8, SIZE>,
    count: usize,
}

impl<const SIZE: usize> StringStore<SIZE> {
    /// Empty store.
    pub fn new() -> Self {
        Self {
            pool: ClusterPool::new(),
            count: 0,
        }
    }

    /// Store `value` and return its id. An empty string still gets an id.
    pub fn add(&mut self, value: &str) -> StringID {
        let bytes = Self::pack_utf8_chunks(value).into_iter().flatten();
        let chain = self
            .pool
            .alloc_chain(bytes)
            .unwrap_or_else(|| self.pool.alloc_empty());
        self.count += 1;
        StringID(chain)
    }

    /// Delete the string and free its clusters. `id` is no longer valid.
    pub fn remove(&mut self, id: StringID) {
        self.pool.free_chain(id.into());
        self.count -= 1;
    }

    /// Borrowed view of the string. Does not copy the bytes.
    pub fn get<'a>(&'a self, id: StringID) -> ClusteredStr<'a, SIZE> {
        ClusteredStr {
            pool: &self.pool,
            chain: id.into(),
        }
    }

    /// Number of stored strings.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Split `value` into chunks of at most `SIZE` bytes, each ending on a char boundary.
    fn pack_utf8_chunks(value: &str) -> Vec<Vec<u8>> {
        let mut chunks = Vec::new();
        let mut start = 0;
        while start < value.len() {
            let mut end = (start + SIZE).min(value.len());
            while end > start && !value.is_char_boundary(end) {
                end -= 1;
            }
            chunks.push(value.as_bytes()[start..end].to_owned());
            start = end;
        }
        chunks
    }
}

/// A string borrowed from a [`StringStore`]. A value split across clusters is not one `&str`.
pub(in crate::database) struct ClusteredStr<'a, const SIZE: usize> {
    pool: &'a ClusterPool<u8, SIZE>,
    chain: ChainID,
}

impl<'a, const SIZE: usize> ClusteredStr<'a, SIZE> {
    /// Bytes in order, without copying them into a `String`.
    pub fn bytes(&self) -> impl Iterator<Item = u8> + 'a {
        self.pool.iter(self.chain).copied()
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
        self.pool.len(self.chain)
    }

    /// `true` when the string has no bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `&str` when the whole value sits in one cluster. `None` when it spans more than one.
    pub fn as_str_if_small(&self) -> Option<&str> {
        self.pool
            .sole_filled(self.chain)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
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

    fn cluster_slices(&self) -> impl Iterator<Item = &[u8]> {
        self.pool.filled_slices(self.chain)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_split_and_reassemble() {
        let mut store = StringStore::<4>::new();
        let id = store.add("quick brown fox");
        let view = store.get(id);
        assert!(view.as_str_if_small().is_none());
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
        assert_eq!(view.as_str_if_small(), Some("hi"));
    }
}
