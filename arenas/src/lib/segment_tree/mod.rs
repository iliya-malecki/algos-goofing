mod nodes;
use crate::arena::{Arena, Id};
use nodes::CompleteBinaryTreeNode;
use std::ops::Add;

pub struct SegmentTreeWithEphemeralId<Data>
where
    Data: Default,
{
    arena: Arena<CompleteBinaryTreeNode<Data>>,
    len: usize,
    root: Id,
}
impl<Data> SegmentTreeWithEphemeralId<Data>
where
    Data: Default + Clone + Add<Output = Data>,
{
    // these look backwards because the root is at the self.array.len() and not at 0

    fn reverse_of(&self, id: Id) -> Id {
        self.arena.len() + 1 - id
    }
    fn left_of(&self, id: Id) -> Id {
        self.reverse_of((self.reverse_of(id)) * 2 + 1)
    }
    fn right_of(&self, id: Id) -> Id {
        self.reverse_of(self.reverse_of(id) * 2)
    }
    fn parent_of(&self, id: Id) -> Id {
        self.reverse_of(self.reverse_of(id) / 2)
    }
    fn id_of_leaf(&self, array_index: usize) -> Option<Id> {
        if array_index >= self.len {
            None
        } else {
            Some(Id::from_array_index(array_index))
        }
    }
    fn common_ancestor_of(&self, mut this: Id, mut that: Id) -> Id {
        while this != that {
            this = self.parent_of(this);
            that = self.parent_of(that);
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
        for _ in 0..distance_to_nearest_power_of_2(nodes.len()) {
            nodes.push(arena.alloc(CompleteBinaryTreeNode::default()));
        }
        loop {
            nodes = Self::build_level(&nodes, &mut arena);
            if nodes.len() == 1 {
                return Self {
                    arena: arena,
                    len: items.len(),
                    root: nodes[0],
                };
            }
        }
    }

    fn build_level(nodes: &Vec<Id>, arena: &mut Arena<CompleteBinaryTreeNode<Data>>) -> Vec<Id> {
        let mut level: Vec<_> = (0..nodes.len())
            .step_by(2)
            .map(|i| {
                let left_agg = arena.get(nodes[i]).data.clone();
                let right_agg = if i + 1 >= nodes.len() {
                    Data::default()
                } else {
                    arena.get(nodes[i + 1]).data.clone()
                };
                arena.alloc(CompleteBinaryTreeNode {
                    data: left_agg + right_agg,
                })
            })
            .collect();
        for _ in 0..distance_to_nearest_power_of_2(nodes.len()) {
            level.push(arena.alloc(CompleteBinaryTreeNode::default()));
        }
        level
    }

    pub fn setitem(&mut self, at: usize, item: &Data) {
        let mut id = self.id_of_leaf(at).expect("`at` index makes no sense");
        self.arena.get_mut(id).data = item.clone();

        while id != self.root {
            id = self.parent_of(id);
            let left_id = self.left_of(id);
            let right_id = self.right_of(id);
            let left_data = self.arena.get(left_id).data.clone();
            let right_data = self.arena.get(right_id).data.clone();
            let node = self.arena.get_mut(id);
            node.data = left_data + right_data;
        }
    }

    pub fn range_sum(&self, lower: usize, upper: usize) -> Data {
        let lower_id = self
            .id_of_leaf(lower)
            .expect("`lower` index makes no sense");
        let upper_id = self
            .id_of_leaf(upper)
            .expect("`upper` index makes no sense");
        let common_ancestor = self.common_ancestor_of(lower_id, upper_id);
        self.sum_half(lower_id, self.left_of(common_ancestor), WalkDirection::Up)
            + self.sum_half(
                upper_id,
                self.right_of(common_ancestor),
                WalkDirection::Down,
            )
    }

    fn sum_half(&self, start: Id, stop: Id, direction: WalkDirection) -> Data {
        let mut current = start;
        let mut total = self.arena.get(start).data.clone();
        while current != stop {
            let parent = self.parent_of(current);
            let child = match direction {
                WalkDirection::Up => self.right_of(parent),
                WalkDirection::Down => self.left_of(parent),
            };
            if child != current {
                total = total + self.arena.get(child).data.clone();
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

fn distance_to_nearest_power_of_2(n: usize) -> usize {
    if n.count_ones() <= 1 {
        return 0;
    }
    let upper = 1 << (size_of::<usize>() * 8 - n.leading_zeros() as usize);
    upper - n
}
