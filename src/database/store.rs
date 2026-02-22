mod availability_manager;

use std::fmt::Debug;

use crate::database::{graph::IDIntoUSize, store::availability_manager::AvailabilityManager};


// TODO: introduce an ConstantStore struct with a builder that still supports things like pagination and storage just like the normal store, but not deletion and growth
/// TODO: introduce a ClusteredStore struct for use with SoA, so that you don't need to do Vec<Store> and duplicate [AvailabilityManager]
pub(super) struct Store<T, I> {
    items: Vec<T>,
    availability: AvailabilityManager<I>,
}

impl<T, I> Store<T, I> where I: IDIntoUSize + Copy + Debug {
    pub fn new() -> Self {
        Store { items: Vec::new(), availability: AvailabilityManager::new()}
    }

    pub fn all(&self) -> impl Iterator<Item = (I, &T)> {
        self.items.iter()
            .enumerate()
            .filter(|(id, _)| self.availability.is_taken(I::from_usize(*id)))
            .map(|(id, item)| (I::from_usize(id), item))
    }

    pub fn get(&self, id: I) -> &T {
        debug_assert!(self.availability.is_taken(id), "Trying to get not existing element, id: {id:?}");

        &self.items[id.as_usize()]
    }

    pub fn get_mut(&mut self, id: I) -> &mut T {
        debug_assert!(self.availability.is_taken(id), "Trying to get not existing element mutably, id: {id:?}");

        &mut self.items[id.as_usize()]
    }

    pub fn add(&mut self, item: T) -> I {
        let id = self.availability.get_available();

        match self.items.get_mut(id.as_usize()) {
            Some(reference) => { *reference = item; },
            None => { self.items.push(item); },
        }

        id
    }

    pub fn remove(&mut self, id: I) {
        debug_assert!(self.availability.is_taken(id), "Trying to delete not existing element, id: {id:?}");

        self.availability.mark_as_available(id);
    }

    pub(super) fn exists(&self, id: I) -> bool {
        self.availability.is_taken(id)
    }

    pub(super) fn len(&self) -> usize {
        self.availability.taken_count()
    }

    // pub fn replace(&mut self, entry: Entry<T, I>) {
    //     debug_assert!(self.availability.is_taken(entry.id), "Trying to get not existing element, id: {id:?}");
    //
    //     self.items[entry.id.get_inner()].item = entry.item
    // }
}

impl<T, I> Debug for Store<T, I> where T: Debug, I: IDIntoUSize + Copy + Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("items", &self.all().collect::<Vec<_>>())
            .finish()
    }
    
}
