use derive_more::{Add, Display, Sub};
// Indexer indexes the array
// ArPointer uses an indexer and an arena reference to have a cleaner api

#[derive(Debug)]
pub struct Arena<T> {
    backing_array: Vec<T>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Add, Sub, Display, Debug)]
pub struct Id(pub usize);

impl<'a, T> Arena<T> {
    pub fn new() -> Self {
        Arena {
            backing_array: Vec::<T>::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Arena {
            backing_array: Vec::with_capacity(capacity),
        }
    }

    pub fn alloc(&mut self, item: T) -> Id {
        self.backing_array.push(item);

        Id(self.backing_array.len() - 1)
    }

    pub fn get(&'a self, id: Id) -> &'a T {
        self.backing_array
            .get(id.0)
            .expect("specified id should have existed")
    }
    pub fn get_mut(&'a mut self, id: Id) -> &'a mut T {
        self.backing_array
            .get_mut(id.0)
            .expect("specified id should have existed")
    }
}
