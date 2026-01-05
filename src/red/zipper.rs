use crate::core::NodeId;
use serde::{Deserialize, Serialize};

/// Navigation state (ephemeral)
#[derive(Clone, Debug)]
pub struct Zipper {
    pub focus: NodeId,
    pub path: Vec<Crumb>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Crumb {
    pub node_id: NodeId,
    pub abs_offset: u32,
    pub child_index: usize,
}

impl Zipper {
    pub fn new(root: NodeId) -> Self {
        Self {
            focus: root,
            path: Vec::new(),
        }
    }
    
    pub fn up(&mut self) -> bool {
        if let Some(crumb) = self.path.pop() {
            self.focus = crumb.node_id;
            true
        } else {
            false
        }
    }
    
    pub fn down(&mut self, child_id: NodeId, child_index: usize, abs_offset: u32) {
        self.path.push(Crumb {
            node_id: self.focus,
            abs_offset,
            child_index,
        });
        self.focus = child_id;
    }
}
