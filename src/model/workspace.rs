use uuid::Uuid;
use crate::model::split_tree::SplitNode;

pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub split_tree: SplitNode,
}

impl Workspace {
    pub fn new(name: String, initial_surface_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            split_tree: SplitNode::Leaf { surface_id: initial_surface_id },
        }
    }
}
