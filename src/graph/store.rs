use std::fmt::Debug;

use crate::graph::{IDIntoUSize, availability_manager::AvailabilityManager};

#[derive(Debug)]
pub(super) struct Entry<T, I> {
    pub(super) id: I,
    pub(super) item: T,
}

#[derive(Debug)]
pub(super) struct Store<T, I> {
    items: Vec<Entry<T, I>>,
    availability: AvailabilityManager<I>,
    // _marker: PhantomData<P>
}

impl<T, I> Store<T, I> where I: IDIntoUSize + Copy + Debug {
    pub fn new() -> Self {
        Store { items: Vec::new(), availability: AvailabilityManager::new()}
    }

    pub fn all(&self) -> impl Iterator<Item = &Entry<T, I>> {
        self.items.iter().filter_map(|en| if self.availability.is_taken(en.id) { Some(en) } else { None })
    }

    pub fn get(&self, id: I) -> &T {
        debug_assert!(self.availability.is_taken(id), "Trying to get not existing element, id: {id:?}");

        &self.items[id.into_usize()].item
    }

    pub fn get_mut(&mut self, id: I) -> &mut T {
        debug_assert!(self.availability.is_taken(id), "Trying to get not existing element mutably, id: {id:?}");

        &mut self.items[id.into_usize()].item
    }

    pub fn add(&mut self, item: T) -> I {
        let id = self.availability.get_available();
        self.items.push(Entry { id, item });
        id
    }

    pub fn remove(&mut self, id: I) {
        debug_assert!(self.availability.is_taken(id), "Trying to delete not existing element, id: {id:?}");

        self.availability.mark_as_available(id);
    }

    pub(super) fn exists(&self, id: I) -> bool {
        self.availability.is_taken(id)
    }

    // pub fn replace(&mut self, entry: Entry<T, I>) {
    //     debug_assert!(self.availability.is_taken(entry.id), "Trying to get not existing element, id: {id:?}");
    //
    //     self.items[entry.id.get_inner()].item = entry.item
    // }
}
