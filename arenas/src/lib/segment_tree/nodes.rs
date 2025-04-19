#[derive(Debug, Default)]
pub struct CompleteBinaryTreeNode<Data> {
    pub data: Data,
}

impl<Data> CompleteBinaryTreeNode<Data>
where
    Data: Default + Clone,
{
    pub fn new(value: &Data) -> Self {
        Self {
            data: value.clone(),
        }
    }
}
