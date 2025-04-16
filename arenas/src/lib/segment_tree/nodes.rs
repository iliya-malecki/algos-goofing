pub use crate::arena::Id;

#[derive(Debug)]
pub struct SparseBinaryTreeNode<Data> {
    pub data: Data,
    pub left: Option<Id>,
    pub right: Option<Id>,
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
