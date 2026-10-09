use std::marker::PhantomData;

use bitvec::prelude::*;

use crate::database::id::Id;

const TAKEN: bool = true;
const AVAILABLE: bool = false;

#[derive(Debug)]
pub struct AvailabilityManager<T> {
    ids: BitVec,
    _marker: PhantomData<T>,
}

impl<T: Id> AvailabilityManager<T> {
    pub fn new() -> Self {
        AvailabilityManager {
            ids: BitVec::new(),
            _marker: PhantomData,
        }
    }

    pub fn get_available(&mut self) -> T {
        match self.ids.first_zero() {
            Some(idx) => {
                unsafe {
                    let mut bit = self.ids.get_unchecked_mut(idx);
                    *bit = TAKEN;
                }

                debug_assert!(
                    !self.is_taken(T::from(idx)),
                    "tried to get unabailable id, idx: {idx}"
                );
                T::from(idx)
            }
            None => {
                self.ids.push(TAKEN);
                T::from(self.ids.len() - 1)
            }
        }
    }

    pub fn mark_as_available(&mut self, id: T) {
        let idx: usize = id.into();
        debug_assert!(
            self.ids.len() > idx,
            "tried to mark id bigger than the graph"
        );

        unsafe {
            let mut bit = self.ids.get_unchecked_mut(idx);
            *bit = AVAILABLE;
        }
    }

    pub fn is_taken(&self, id: T) -> bool {
        let idx: usize = id.into();
        debug_assert!(
            self.ids.len() > idx,
            "tried to check for id bigger than the graph"
        );

        self.ids[idx]
    }

    pub fn taken_count(&self) -> usize {
        self.ids.count_ones()
    }
}
