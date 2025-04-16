// i was thinking about reafctoring this with
// trait BinaryTreeNodeReference
// {
//     type Data;
//     fn get_left(&self) -> Self;
//     fn get_right(&self) -> Self;
//     fn get_data(&self) -> Self::Data;
//     fn is_leaf(&self) -> bool;
// }
// but then it hit me that left() and right() returning Option<Self> vs Self is a
// fundamental difference so trying to marry them will be a Java moment.
// So we writin 2 different traits boys
use crate::arena::{Arena, Id};
use std::{cell::RefCell, collections::VecDeque, ops::Add, rc::Rc};

trait BinaryTreeNodeReference
where
    Self: Sized,
{
    type Data;
    fn get_left(&self) -> Option<Self>;
    fn get_right(&self) -> Option<Self>;
    fn get_data(&self) -> Self::Data;
    fn set_data(&self, value: Self::Data);
    fn is_leaf(&self) -> bool;
}

#[derive(Clone)]
struct PhysicalIdBinaryTreeNodeReference<Data> {
    arena: Rc<RefCell<Arena<PhysicalIdBinaryTreeNode<Data>>>>,
    id: Id,
}
impl<Data> PartialEq for PhysicalIdBinaryTreeNodeReference<Data> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<Data> BinaryTreeNodeReference for PhysicalIdBinaryTreeNodeReference<Data>
where
    Data: Clone,
{
    type Data = Data;

    fn get_left(&self) -> Option<Self> {
        let left = self.arena.borrow().get(self.id).left?;
        Some(Self {
            arena: self.arena.clone(),
            id: left,
        })
    }

    fn get_right(&self) -> Option<Self> {
        let right = self.arena.borrow().get(self.id).right?;
        Some(Self {
            arena: self.arena.clone(),
            id: right,
        })
    }

    fn get_data(&self) -> Self::Data {
        self.arena.borrow().get(self.id).data.clone()
    }

    fn is_leaf(&self) -> bool {
        let arena = self.arena.borrow();
        let node = arena.get(self.id);
        node.is_leaf()
    }
    fn set_data(&self, value: Data) {
        self.arena.borrow_mut().get_mut(self.id).data = value;
    }
}

#[derive(Debug)]
struct PhysicalIdBinaryTreeNode<Data> {
    data: Data,
    left: Option<Id>,
    right: Option<Id>,
}

impl<Data> PhysicalIdBinaryTreeNode<Data>
where
    Data: Clone,
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
    arena: Rc<RefCell<Arena<PhysicalIdBinaryTreeNode<Data>>>>,
    len: usize,
    root: Id,
    degree: usize,
}
impl<Data> SegmentTreeWithRealId<Data>
where
    Data: Default + Clone + Add<Output = Data>,
{
    pub fn new(items: Vec<Data>) -> Self {
        let mut arena = Arena::<PhysicalIdBinaryTreeNode<Data>>::with_capacity(items.len() * 4);
        let mut nodes = items
            .iter()
            .map(PhysicalIdBinaryTreeNode::<Data>::leaf)
            .map(|el| arena.alloc(el))
            .collect::<Vec<_>>();
        for degree in 1usize.. {
            nodes = Self::build_level(&nodes, &mut arena);
            if nodes.len() == 1 {
                return Self {
                    arena: Rc::new(RefCell::new(arena)),
                    len: items.len(),
                    root: nodes[0],
                    degree: degree,
                };
            }
        }
        unreachable!();
    }

    fn build_level(nodes: &Vec<Id>, arena: &mut Arena<PhysicalIdBinaryTreeNode<Data>>) -> Vec<Id> {
        (0..nodes.len())
            .step_by(2)
            .map(|i| {
                let left_agg = arena.get(nodes[i]).data.clone();
                let (right_agg, right_id) = if i + 1 >= nodes.len() {
                    (Data::default(), None)
                } else {
                    (arena.get(nodes[i + 1]).data.clone(), Some(nodes[i + 1]))
                };
                arena.alloc(PhysicalIdBinaryTreeNode {
                    data: left_agg + right_agg,
                    left: Some(nodes[i]),
                    right: right_id,
                })
            })
            .collect()
    }

    pub fn setitem(&mut self, at: usize, item: &Data) {
        assert!(at <= self.len);
        let start = PhysicalIdBinaryTreeNodeReference {
            arena: self.arena.clone(),
            id: self.root,
        };
        let mut visited: VecDeque<_> = std::iter::once(start.clone())
            .chain(NodePath::from_index(at, start, self.degree))
            .collect();
        let leaf = visited.pop_back().expect("why would it be empty");
        assert!(leaf.is_leaf());
        leaf.set_data(item.clone());

        while !visited.is_empty() {
            let node = visited
                .pop_back()
                .expect("cant have empty nodes in a chain of nodes");
            let left_data = node.get_left().map_or(Data::default(), |v| v.get_data());
            let right_data = node.get_right().map_or(Data::default(), |v| v.get_data());
            node.set_data(left_data + right_data);
        }
    }

    pub fn range_sum(&self, lower: usize, upper: usize) -> Data {
        let (lower_hops, upper_hops) = NodePath::without_common_hops(
            lower,
            upper,
            PhysicalIdBinaryTreeNodeReference::<Data> {
                arena: self.arena.clone(),
                id: self.root,
            },
            self.degree,
        );
        self.sum_half(lower_hops, WalkDirection::Up)
            + self.sum_half(upper_hops, WalkDirection::Down)
    }

    fn sum_half(&self, hops: NodePath<Data>, direction: WalkDirection) -> Data {
        let mut visited: VecDeque<_> = hops.collect();
        let mut last = visited.pop_back().unwrap();
        assert!(last.is_leaf());
        let mut total = last.get_data();
        while !visited.is_empty() {
            let parent = visited.pop_back().unwrap();
            let child = match direction {
                WalkDirection::Up => parent.get_right(),
                WalkDirection::Down => parent.get_left(),
            };
            if child.is_some() && child != Some(last) {
                total = total + child.unwrap().get_data();
            }
            last = parent;
        }
        total
    }
}

enum WalkDirection {
    Up,
    Down,
}
struct NodePath<Data> {
    node_ref: PhysicalIdBinaryTreeNodeReference<Data>,
    index: usize,
    offset: usize,
    stop: usize,
}
impl<Data> NodePath<Data>
where
    Data: Clone,
{
    pub fn from_index(
        index: usize,
        start: PhysicalIdBinaryTreeNodeReference<Data>,
        tree_degree: usize,
    ) -> Self {
        Self {
            node_ref: start,
            index,
            offset: tree_degree,
            stop: 0,
        }
    }

    pub fn without_common_hops(
        left: usize,
        right: usize,
        start: PhysicalIdBinaryTreeNodeReference<Data>,
        tree_degree: usize,
    ) -> (Self, Self) {
        let equal_bits = !(left ^ right);
        let leading_ones = equal_bits.leading_ones();
        let stop_index = size_of::<usize>() * 8 - leading_ones as usize;

        let new_start = Self {
            node_ref: start.clone(),
            index: left,
            offset: tree_degree,
            stop: stop_index,
        }
        .last()
        .unwrap_or(start);

        (
            Self {
                node_ref: new_start.clone(),
                index: left,
                offset: stop_index,
                stop: 0,
            },
            Self {
                node_ref: new_start,
                index: right,
                offset: stop_index,
                stop: 0,
            },
        )
    }
}

impl<Data> Iterator for NodePath<Data>
where
    Data: Clone,
{
    type Item = PhysicalIdBinaryTreeNodeReference<Data>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.stop {
            return None;
        }
        match self.index >> (self.offset - 1) & 1 {
            0 => {
                self.node_ref = self
                    .node_ref
                    .get_left()
                    .expect("iterating not done so children cant be None");
            }
            1 => {
                self.node_ref = self
                    .node_ref
                    .get_right()
                    .expect("iterating not done so children cant be None");
            }
            _ => todo!(),
        }
        self.offset -= 1;
        Some(self.node_ref.clone())
    }
}
