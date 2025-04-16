use super::nodes::SparseBinaryTreeNode;
use crate::{Arena, Id};
use std::iter::Rev;

pub enum WalkDirection {
    Up,
    Down,
}
pub struct NodePath<'a, Data> {
    id: Id,
    arena: &'a Arena<SparseBinaryTreeNode<Data>>,
    index: usize,
    offset: usize,
    stop: usize,
}
impl<'a, Data> NodePath<'a, Data> {
    pub fn from_index(
        index: usize,
        start: Id,
        arena: &'a Arena<SparseBinaryTreeNode<Data>>,
        tree_degree: usize,
    ) -> Self {
        Self {
            id: start,
            arena,
            index,
            offset: tree_degree,
            stop: 0,
        }
    }

    pub fn without_common_hops(
        left: usize,
        right: usize,
        start: Id,
        arena: &'a Arena<SparseBinaryTreeNode<Data>>,
        tree_degree: usize,
    ) -> (Self, Self) {
        let equal_bits = !(left ^ right);
        let leading_ones = equal_bits.leading_ones();
        let stop_index = size_of::<usize>() * 8 - leading_ones as usize;

        let new_start = Self {
            id: start,
            arena,
            index: left,
            offset: tree_degree,
            stop: stop_index,
        }
        .last()
        .unwrap_or(start);

        (
            Self {
                id: new_start,
                arena,
                index: left,
                offset: stop_index,
                stop: 0,
            },
            Self {
                id: new_start,
                arena,
                index: right,
                offset: stop_index,
                stop: 0,
            },
        )
    }
}

impl<'a, Data> Iterator for NodePath<'a, Data> {
    type Item = Id;
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.stop {
            return None;
        }
        let node = self.arena.get(self.id);
        match self.index >> (self.offset - 1) & 1 {
            0 => {
                self.id = node
                    .left
                    .expect("iterating not done so children cant be None");
            }
            1 => {
                self.id = node
                    .right
                    .expect("iterating not done so children cant be None");
            }
            _ => todo!(),
        }
        self.offset -= 1;
        Some(self.id)
    }
}

pub trait ReverseStacking: Iterator + Sized {
    fn reverse_stacking(self) -> Rev<std::vec::IntoIter<Self::Item>> {
        let items: Vec<_> = self.collect();
        items.into_iter().rev()
    }
}
impl<I: Iterator> ReverseStacking for I {}
