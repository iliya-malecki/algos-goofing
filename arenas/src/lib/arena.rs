use std::{
    num::NonZeroUsize,
    ops::{Add, Div, Mul, Sub},
};

use derive_more::Display;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display, Debug)]
pub struct Id(NonZeroUsize);

impl Id {
    pub fn from_array_index(index: usize) -> Self {
        Self(NonZeroUsize::new(index + 1).unwrap()) // since were secretly a 1-based arena
    }
}

impl Add<usize> for Id {
    type Output = Id;

    fn add(self, rhs: usize) -> Self::Output {
        Self(NonZeroUsize::new(self.0.get() + rhs).unwrap())
    }
}
impl Mul<usize> for Id {
    type Output = Id;

    fn mul(self, rhs: usize) -> Self::Output {
        Self(NonZeroUsize::new(self.0.get() * rhs).unwrap())
    }
}
impl Div<usize> for Id {
    type Output = Id;

    fn div(self, rhs: usize) -> Self::Output {
        Self(NonZeroUsize::new(self.0.get() / rhs).unwrap())
    }
}
impl Sub<Id> for usize {
    type Output = Id;

    fn sub(self, rhs: Id) -> Self::Output {
        Id(NonZeroUsize::new(self - rhs.0.get()).unwrap())
    }
}
impl Sub<usize> for Id {
    type Output = Id;

    fn sub(self, rhs: usize) -> Self::Output {
        Id(NonZeroUsize::new(self.0.get() - rhs).unwrap())
    }
}

#[derive(Debug)]
pub struct Arena<T> {
    backing_array: Vec<T>,
}

impl<'a, T> Arena<T> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut arena = Self {
            backing_array: Vec::<T>::with_capacity(capacity + 1),
        };
        unsafe {
            arena.backing_array.set_len(1);
        }
        arena
    }
    pub fn len(&self) -> usize {
        self.backing_array.len() - 1
    }

    pub fn alloc(&mut self, item: T) -> Id {
        self.backing_array.push(item);

        Id(NonZeroUsize::new(self.backing_array.len() - 1)
            .expect("arena's backing array cant be empty"))
    }

    pub fn get(&'a self, id: &Id) -> &'a T {
        self.backing_array
            .get(id.0.get())
            .expect("specified id should have existed")
    }
    pub fn get_mut(&'a mut self, id: &Id) -> &'a mut T {
        self.backing_array
            .get_mut(id.0.get())
            .expect("specified id should have existed")
    }
}
