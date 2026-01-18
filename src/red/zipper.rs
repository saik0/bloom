use serde::{Deserialize, Serialize};

/// Navigation cursor into the tree - can have multiple independent Zippers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Zipper {
    pub focus: String,        // Current node ID
    pub path: Vec<Crumb>,     // Breadcrumb trail for going back up
}

/// Breadcrumb: remembers where we came from
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Crumb {
    pub node_id: String,      // The parent node ID we descended from
    pub child_index: usize,   // Which child slot we went into
}

impl Zipper {
    pub fn new(root_id: String) -> Self {
        Self {
            focus: root_id,
            path: Vec::new(),
        }
    }

    /// Move up to parent. Returns true if successful, false if already at root.
    pub fn up(&mut self) -> bool {
        if let Some(crumb) = self.path.pop() {
            self.focus = crumb.node_id;
            true
        } else {
            false
        }
    }

    /// Move down into a child node.
    pub fn down(&mut self, child_id: String, child_index: usize) {
        let parent_id = std::mem::replace(&mut self.focus, child_id);
        self.path.push(Crumb {
            node_id: parent_id,
            child_index,
        });
    }

    /// Current depth in the tree (0 = root)
    pub fn depth(&self) -> usize {
        self.path.len()
    }

    /// Get the path as a list of node IDs (for breadcrumb display)
    #[allow(dead_code)]
    pub fn path_ids(&self) -> Vec<&str> {
        self.path.iter().map(|c| c.node_id.as_str()).collect()
    }
}