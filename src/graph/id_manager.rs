use std::marker::PhantomData;

use bitvec::prelude::*;
use crate::graph::ID;

const TAKEN: bool = true;
const AVAILABLE: bool = false;

#[derive(Debug)]
pub struct IDManager<T> {
    ids: BitVec,
    _marker: PhantomData<T>,
}


impl<T: From<usize> + Copy + ID> IDManager<T> {
    pub fn new() -> Self {
        IDManager { 
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

                idx.into()
            },
            None => {
                self.ids.push(TAKEN);
                (self.ids.len() - 1).into()
            }
        }
    }

    pub fn mark_as_taken(&mut self, id: T) {
        assert!(self.ids.len() > id.get_inner(), "tried to add id bigger than the graph");

        unsafe {
            let mut bit = self.ids.get_unchecked_mut(id.get_inner());
            *bit = TAKEN;
        }
    }

    pub fn is_taken(&self, id: T) -> bool {
        assert!(self.ids.len() > id.get_inner(), "tried to check for id bigger than the graph");
        // if id.get_inner() >= self.ids.len() {
        //     return AVAILABLE;
        // }

        self.ids[id.get_inner()]
    }
}
