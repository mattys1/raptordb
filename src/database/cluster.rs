use std::marker::PhantomData;

use crate::database::{id::{ClusterID, Id}, store::Store};

struct Cluster<T, const Size: usize>
where
    T: Copy,
{
    contents: [T; Size],
    next: Option<ClusterID>,
    count: u8,
}

struct ClusteredStore<T, ExtId, const Size: usize>
where
    T: Copy,
    ExtId: Id,
{
    clusters: Store<Cluster<T, Size>, ClusterID>,
    _external_id: PhantomData<ExtId>,
}
