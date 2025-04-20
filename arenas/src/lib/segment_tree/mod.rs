mod nodes;
use crate::arena::{Arena, Id};
use nodes::CompleteBinaryTreeNode;
use std::ops::{Add, Deref};

pub struct SegmentTreeWithEphemeralId<Data>
where
    Data: Default,
{
    arena: Arena<CompleteBinaryTreeNode<Data>>,
    len: usize,
    root: Id,
    level_offsets: Vec<usize>,
}
impl<'a, Data> SegmentTreeWithEphemeralId<Data>
where
    Data: Default + Clone + Add<Output = Data>,
{
    fn leaf_to_noderef(
        array_index: usize,
        level_offsets: &'a Vec<usize>,
        leaf_count: usize,
        node_count: usize,
    ) -> Option<NodeReference<'a>> {
        if array_index >= leaf_count {
            None
        } else {
            Some(NodeReference::new(
                Id::from_array_index(array_index),
                0,
                node_count,
                level_offsets,
            ))
        }
    }
    fn common_ancestor_of(
        mut this: NodeReference<'a>,
        mut that: NodeReference<'a>,
    ) -> NodeReference<'a> {
        while this.id != that.id {
            this = this.parent();
            that = that.parent();
        }
        this
    }

    fn get_total_node_count(len: usize) -> usize {
        let mut current = len;
        let mut total = len;
        while current > 0 {
            if current % 2 == 0 {
                current += 1;
            }
            total += current;
            current /= 2;
        }
        total
    }

    pub fn new(items: Vec<Data>) -> Self {
        let capacity = Self::get_total_node_count(items.len());
        let mut arena = Arena::<CompleteBinaryTreeNode<Data>>::with_capacity(capacity);
        let mut nodes = items
            .iter()
            .map(CompleteBinaryTreeNode::<Data>::new)
            .map(|el| arena.alloc(el))
            .collect::<Vec<_>>();
        if nodes.len() % 2 != 0 {
            nodes.push(arena.alloc(CompleteBinaryTreeNode::default()));
        }

        let starting_degree = log2usize_roundup(nodes.len());
        let mut level_offsets = vec![2usize.pow(starting_degree) - nodes.len()];
        for level in 1usize.. {
            nodes = Self::build_level(&nodes, &mut arena);
            level_offsets.push(2usize.pow(starting_degree - level as u32) - nodes.len());
            if nodes.len() == 1 {
                return Self {
                    arena: arena,
                    len: items.len(),
                    root: nodes[0],
                    level_offsets: precalculate_level_offsets(level_offsets),
                };
            }
        }
        unreachable!();
    }

    fn build_level(nodes: &Vec<Id>, arena: &mut Arena<CompleteBinaryTreeNode<Data>>) -> Vec<Id> {
        let mut level: Vec<_> = (0..nodes.len())
            .step_by(2)
            .map(|i| {
                let left_agg = arena.get(&nodes[i]).data.clone();
                let right_agg = if i + 1 >= nodes.len() {
                    Data::default()
                } else {
                    arena.get(&nodes[i + 1]).data.clone()
                };
                arena.alloc(CompleteBinaryTreeNode {
                    data: left_agg + right_agg,
                })
            })
            .collect();
        if level.len() > 1 && level.len() % 2 != 0 {
            level.push(arena.alloc(CompleteBinaryTreeNode::default()));
        }
        level
    }

    pub fn setitem(&mut self, at: usize, item: &Data) {
        let mut node_ref =
            Self::leaf_to_noderef(at, &self.level_offsets, self.len, self.arena.len())
                .expect("`at` index makes no sense");
        self.arena.get_mut(&(node_ref.id)).data = item.clone();

        while node_ref.id != self.root {
            node_ref = node_ref.parent();
            let left_id = node_ref.left();
            let right_id = node_ref.right();
            let left_data = self.arena.get(&left_id).data.clone();
            let right_data = self.arena.get(&right_id).data.clone();
            let node = self.arena.get_mut(&node_ref);
            node.data = left_data + right_data;
        }
    }

    pub fn range_sum(&self, lower: usize, upper: usize) -> Data {
        let lower_id =
            Self::leaf_to_noderef(lower, &self.level_offsets, self.len, self.arena.len())
                .expect("`lower` index makes no sense");
        let upper_id =
            Self::leaf_to_noderef(upper, &self.level_offsets, self.len, self.arena.len())
                .expect("`upper` index makes no sense");
        let common_ancestor = Self::common_ancestor_of(lower_id.clone(), upper_id.clone());
        self.sum_half(lower_id, common_ancestor.left(), WalkDirection::Up)
            + self.sum_half(upper_id, common_ancestor.right(), WalkDirection::Down)
    }

    fn sum_half(
        &self,
        start: NodeReference,
        stop: NodeReference,
        direction: WalkDirection,
    ) -> Data {
        let mut current = start.clone();
        let mut total = self.arena.get(&start).data.clone();
        while current.id != stop.id {
            let parent = current.parent();
            let child = match direction {
                WalkDirection::Up => parent.right(),
                WalkDirection::Down => parent.left(),
            };
            if child.id != current.id {
                total = total + self.arena.get(&child).data.clone();
            };
            current = parent;
        }
        total
    }
}

enum WalkDirection {
    Up,
    Down,
}

/// Helper Id wrapper for this specific tree. Keeps track of tree levels,
/// allows easy traversal of the tree with skipped nodes
#[derive(Clone)]
struct NodeReference<'a> {
    id: Id,
    level: usize,
    total_len: usize,
    offsets: &'a [usize],
}
impl<'a> NodeReference<'a> {
    fn new(id: Id, level: usize, total_len: usize, offsets: &'a [usize]) -> Self {
        Self {
            id,
            level,
            total_len,
            offsets,
        }
    }
    /// the root is at the self.array.len() and not at 1
    /// so to keep the tree indexing logic we flip it before and after each lookup
    fn backwards(&self) -> Self {
        Self::new(
            self.total_len + 1 - self.id,
            self.level,
            self.total_len,
            self.offsets,
        )
    }
    fn left(&self) -> Self {
        Self::new(
            (self.backwards().id) * 2 + 1 - self.offsets[self.level - 1],
            self.level - 1,
            self.total_len,
            self.offsets,
        )
        .backwards()
    }
    fn right(&self) -> Self {
        Self::new(
            (self.backwards().id) * 2 - self.offsets[self.level - 1],
            self.level - 1,
            self.total_len,
            self.offsets,
        )
        .backwards()
    }
    fn parent(&self) -> Self {
        Self::new(
            (self.backwards().id + self.offsets[self.level]) / 2,
            self.level + 1,
            self.total_len,
            self.offsets,
        )
        .backwards()
    }
}
impl<'a> Deref for NodeReference<'a> {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

fn log2usize_roundup(n: usize) -> u32 {
    if n.is_power_of_two() {
        n as u32
    } else {
        size_of::<usize>() as u32 * 8 - n.leading_zeros()
    }
}

/// the reason for why offsets[position] - offsets[..position].sum() is the law -
/// no idea, it just seems to be the case from my napkin doodling
fn precalculate_level_offsets(level_offsets: Vec<usize>) -> Vec<usize> {
    let backwards_cum_sum = level_offsets
        .iter()
        .rev()
        .scan(0, |acc, el| Some(*acc + *el))
        .collect::<Vec<_>>()
        .into_iter()
        .rev();
    let lag = backwards_cum_sum.skip(1).chain(std::iter::once(0));

    level_offsets
        .iter()
        .zip(lag)
        .map(|(original, transformed)| original - transformed)
        .collect()
}
