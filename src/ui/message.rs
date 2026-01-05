use crate::app::InitResult;
use crate::parse::rust::TreeNode;

#[derive(Debug, Clone)]
pub enum Message {
    // Initialization
    Initialized(Result<InitResult, String>),

    // Focus loaded: (focused_node, children)
    FocusLoaded(Result<(String, TreeNode, Vec<(String, TreeNode)>), String>),

    // Navigation
    NavigateUp,
    NavigateDown(String, usize),  // (child_id, child_index)

    // Editing
    StartEdit(String),
    UpdateEditText(String),
    CommitEdit,
    CancelEdit,

    // No-op
    None,
}