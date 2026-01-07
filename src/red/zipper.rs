//! RED Tree Navigation via Zipper
//!
//! The Zipper navigates rowan's SyntaxNode directly. Rowan IS the source of truth.
//! No copying, no separate "TreeNode" struct - just a cursor into the live syntax tree.

use ra_ap_syntax::SyntaxNode;

/// Navigation cursor into rowan's syntax tree.
/// 
/// This is the RED tree - ephemeral, position-aware, with parent pointers.
/// Rowan provides all of this; we just wrap it for ergonomic navigation.
#[derive(Clone)]
pub struct Zipper {
    /// Current focused node in the rowan tree
    focus: SyntaxNode,
    /// Breadcrumb trail: (parent_node, child_index we descended into)
    path: Vec<(SyntaxNode, usize)>,
}

impl Zipper {
    /// Create a zipper focused on the root of a syntax tree
    pub fn new(root: SyntaxNode) -> Self {
        Self {
            focus: root,
            path: Vec::new(),
        }
    }

    /// Get the currently focused node
    pub fn focus(&self) -> &SyntaxNode {
        &self.focus
    }

    /// Move up to parent. Returns true if successful, false if at root.
    pub fn up(&mut self) -> bool {
        if let Some((parent, _idx)) = self.path.pop() {
            self.focus = parent;
            true
        } else {
            false
        }
    }

    /// Move down into the nth child. Returns true if successful.
    pub fn down(&mut self, child_index: usize) -> bool {
        if let Some(child) = self.focus.children().nth(child_index) {
            let parent = self.focus.clone();
            self.path.push((parent, child_index));
            self.focus = child;
            true
        } else {
            false
        }
    }

    /// Move to next sibling. Returns true if successful.
    pub fn next_sibling(&mut self) -> bool {
        if let Some(sibling) = self.focus.next_sibling() {
            // Update the child index in the path
            if let Some((_, idx)) = self.path.last_mut() {
                *idx += 1;
            }
            self.focus = sibling;
            true
        } else {
            false
        }
    }

    /// Move to previous sibling. Returns true if successful.
    pub fn prev_sibling(&mut self) -> bool {
        if let Some(sibling) = self.focus.prev_sibling() {
            if let Some((_, idx)) = self.path.last_mut() {
                *idx = idx.saturating_sub(1);
            }
            self.focus = sibling;
            true
        } else {
            false
        }
    }

    /// Current depth in the tree (0 = root)
    pub fn depth(&self) -> usize {
        self.path.len()
    }

    /// Get the path as (kind, child_index) pairs for breadcrumb display
    pub fn breadcrumbs(&self) -> Vec<(String, usize)> {
        self.path
            .iter()
            .map(|(node, idx)| (format!("{:?}", node.kind()), *idx))
            .collect()
    }

    /// Get children of the focused node
    pub fn children(&self) -> impl Iterator<Item = SyntaxNode> + '_ {
        self.focus.children()
    }

    /// Get child count
    pub fn child_count(&self) -> usize {
        self.focus.children().count()
    }
}

impl std::fmt::Debug for Zipper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Zipper")
            .field("focus", &format!("{:?}", self.focus.kind()))
            .field("depth", &self.depth())
            .finish()
    }
}
