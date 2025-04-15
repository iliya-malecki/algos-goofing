use crate::arena::{Arena, Id};
use std::{collections::VecDeque, ops::Add};

#[derive(Debug)]
struct SparseBinaryTreeNode<Data> {
    data: Data,
    left: Option<Id>,
    right: Option<Id>,
}

impl<Data> SparseBinaryTreeNode<Data>
where
    Data: Default + Clone,
{
    pub fn leaf(value: &Data) -> Self {
        Self {
            data: value.clone(),
            left: None,
            right: None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

pub struct SegmentTreeWithRealId<Data>
where
    Data: Default,
{
    arena: Arena<SparseBinaryTreeNode<Data>>,
    len: usize,
    root: Id,
    degree: usize,
}
impl<Data> SegmentTreeWithRealId<Data>
where
    Data: Default + Clone + Add<Output = Data>,
{
    pub fn new(items: Vec<Data>) -> Self {
        let mut arena = Arena::<SparseBinaryTreeNode<Data>>::with_capacity(items.len() * 4);
        let mut nodes = items
            .iter()
            .map(SparseBinaryTreeNode::<Data>::leaf)
            .map(|el| arena.alloc(el))
            .collect::<Vec<_>>();
        for degree in 1usize.. {
            nodes = Self::build_level(&nodes, &mut arena);
            if nodes.len() == 1 {
                return Self {
                    arena: arena,
                    len: items.len(),
                    root: nodes[0],
                    degree: degree,
                };
            }
        }
        unreachable!();
    }

    fn build_level(nodes: &Vec<Id>, arena: &mut Arena<SparseBinaryTreeNode<Data>>) -> Vec<Id> {
        (0..nodes.len())
            .step_by(2)
            .map(|i| {
                let left_agg = arena.get(nodes[i]).data.clone();
                let (right_agg, right_id) = if i + 1 >= nodes.len() {
                    (Data::default(), None)
                } else {
                    (arena.get(nodes[i + 1]).data.clone(), Some(nodes[i + 1]))
                };
                arena.alloc(SparseBinaryTreeNode {
                    data: left_agg + right_agg,
                    left: Some(nodes[i]),
                    right: right_id,
                })
            })
            .collect()
    }

    pub fn setitem(&mut self, at: usize, item: &Data) {
        assert!(at <= self.len);
        let mut visited: VecDeque<_> = std::iter::once(self.root)
            .chain(NodePath::from_index(
                at,
                self.root,
                &self.arena,
                self.degree,
            ))
            .collect();
        let leaf_id = visited.pop_back().expect("why would it be empty");
        let leaf = self.arena.get_mut(leaf_id);
        assert!(leaf.is_leaf());
        leaf.data = item.clone();
        while !visited.is_empty() {
            let id = visited
                .pop_back()
                .expect("cant have empty nodes in a chain of nodes");
            let (left, right) = {
                let node = self.arena.get(id);
                (node.left, node.right)
            };
            let left_data = left.map_or(Data::default(), |v| self.arena.get(v).data.clone());
            let right_data = right.map_or(Data::default(), |v| self.arena.get(v).data.clone());
            let node = self.arena.get_mut(id);
            node.data = left_data + right_data;
        }
    }

    pub fn range_sum(&self, lower: usize, upper: usize) -> Data {
        let (lower_hops, upper_hops) =
            NodePath::without_common_hops(lower, upper, self.root, &self.arena, self.degree);
        self.sum_half(lower_hops, WalkDirection::Up)
            + self.sum_half(upper_hops, WalkDirection::Down)
    }

    fn sum_half(&self, hops: NodePath<Data>, direction: WalkDirection) -> Data {
        let mut visited: VecDeque<_> = hops.collect();
        let mut last_id = visited.pop_back().unwrap();
        let last = self.arena.get(last_id);
        assert!(last.is_leaf());
        let mut total = last.data.clone();
        while !visited.is_empty() {
            let parent = visited.pop_back().unwrap();
            let child = match direction {
                WalkDirection::Up => self.arena.get(parent).right,
                WalkDirection::Down => self.arena.get(parent).left,
            };
            if child.is_some() && child != Some(last_id) {
                total = total + self.arena.get(child.unwrap()).data.clone();
            }
            last_id = parent;
        }
        total
    }
}

enum WalkDirection {
    Up,
    Down,
}
struct NodePath<'a, Data> {
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
