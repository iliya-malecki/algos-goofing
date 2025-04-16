mod node_path;
mod nodes;
use crate::arena::{Arena, Id};
use node_path::{NodePath, ReverseStacking, WalkDirection};
use nodes::SparseBinaryTreeNode;
use std::ops::Add;

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
        let mut visited = std::iter::once(self.root)
            .chain(NodePath::from_index(
                at,
                self.root,
                &self.arena,
                self.degree,
            ))
            .reverse_stacking();
        let leaf_id = visited.next().expect("why would it be empty");
        let leaf = self.arena.get_mut(leaf_id);
        assert!(leaf.is_leaf());
        leaf.data = item.clone();
        for id in visited {
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
        let mut visited = hops.reverse_stacking();
        let mut last_id = visited.next().unwrap();
        let last = self.arena.get(last_id);
        assert!(last.is_leaf());
        let mut total = last.data.clone();
        for parent in visited {
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
