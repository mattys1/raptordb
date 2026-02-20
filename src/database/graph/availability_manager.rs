use std::marker::PhantomData;

use bitvec::prelude::*;
use crate::database::graph::IDIntoUSize;

const TAKEN: bool = true;
const AVAILABLE: bool = false;

#[derive(Debug)]
pub struct AvailabilityManager<T> {
    ids: BitVec,
    _marker: PhantomData<T>,
}


impl<T: IDIntoUSize + Copy> AvailabilityManager<T> {
    pub fn new() -> Self {
        AvailabilityManager { 
            ids: BitVec::new(),
            _marker: PhantomData
        } 
    }

    pub fn get_available(&mut self) -> T {
        match self.ids.first_zero() {
            Some(idx) => {
                unsafe {
                    let mut bit = self.ids.get_unchecked_mut(idx);
                    *bit = TAKEN;
                }

                debug_assert!(!self.is_taken(T::from_usize(idx)), "tried to get unabailable id, idx: {idx}");
                T::from_usize(idx)
            },
            None => {
                self.ids.push(TAKEN);
                T::from_usize(self.ids.len() - 1)
            }
        }
    }
    //
    // pub fn mark_as_taken(&mut self, id: T) {
    //     assert!(self.ids.len() > id.get_inner(), "tried to add id bigger than the graph");
    //
    //     unsafe {
    //         let mut bit = self.ids.get_unchecked_mut(id.into());
    //         *bit = TAKEN;
    //     }
    // }

    pub fn mark_as_available(&mut self, id: T) {
        debug_assert!(self.ids.len() > id.as_usize(), "tried to mark id bigger than the graph");

        unsafe {
            let mut bit = self.ids.get_unchecked_mut(id.as_usize());
            *bit = AVAILABLE;
        }
    }

    pub fn is_taken(&self, id: T) -> bool {
        debug_assert!(self.ids.len() > id.as_usize(), "tried to check for id bigger than the graph");
        // if id.get_inner() >= self.ids.len() {
        //     return AVAILABLE;
        // }

        self.ids[id.as_usize()]
    }

    pub fn taken_count(&self) -> usize {
        self.ids.count_ones()
    }
}
