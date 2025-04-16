use std::num::NonZeroUsize;

use derive_more::Display;
// Indexer indexes the array
// ArPointer uses an indexer and an arena reference to have a cleaner api

#[derive(Debug)]
pub struct Arena<T> {
    backing_array: Vec<T>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, Debug)]
pub struct Id(NonZeroUsize);

impl<'a, T> Arena<T> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut arena = Arena {
            backing_array: Vec::<T>::with_capacity(capacity),
        };
        unsafe {
            arena.backing_array.set_len(1);
        }
        arena
    }

    pub fn alloc(&mut self, item: T) -> Id {
        self.backing_array.push(item);

        Id(NonZeroUsize::new(self.backing_array.len() - 1)
            .expect("arena's backing array cant be empty"))
    }

    pub fn get(&'a self, id: Id) -> &'a T {
        self.backing_array
            .get(usize::from(id.0))
            .expect("specified id should have existed")
    }
    pub fn get_mut(&'a mut self, id: Id) -> &'a mut T {
        self.backing_array
            .get_mut(usize::from(id.0))
            .expect("specified id should have existed")
    }
}
