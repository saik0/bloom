use ra_ap_syntax::{SourceFile, SyntaxNode};
use ra_ap_syntax::Edition::Edition2021;
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

/// Tree node stored in SurrealDB - has both parent and children for bidirectional nav
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TreeNode {
    pub kind: String,
    pub text: String,
    pub parent: Option<String>,   // Parent record ID (None for root)
    pub children: Vec<String>,    // Child record IDs (actual IDs, not random!)
    pub child_index: usize,       // This node's index within parent's children
}

/// Intermediate structure for building the tree synchronously
struct PendingNode {
    id: String,
    node: TreeNode,
}

/// Parse Rust source and store tree in DB. Returns root node ID.
pub async fn parse_rust(db: &Surreal<Db>, source: &str, _file_name: &str) -> Result<String, String> {
    // SYNCHRONOUS PHASE: Walk the tree and build all nodes in memory
    let (pending_nodes, root_id) = {
        let parsed = SourceFile::parse(source, Edition2021);
        let root = parsed.syntax_node();

        let mut pending = Vec::new();
        let root_id = build_tree_sync(&root, None, 0, &mut pending);

        (pending, root_id)
    };
    // SyntaxNode is now dropped, safe to await

    // ASYNC PHASE: Insert all nodes into DB
    for pending in pending_nodes {
        let _: Option<TreeNode> = db
            .create(("nodes", &pending.id))
            .content(pending.node)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(root_id)
}

/// Synchronously walk the tree, building nodes bottom-up so children IDs are known
fn build_tree_sync(
    node: &SyntaxNode,
    parent_id: Option<String>,
    child_index: usize,
    pending: &mut Vec<PendingNode>,
) -> String {
    // Generate ID for this node
    let node_id = uuid::Uuid::new_v4().to_string();

    // Recursively process children first to get their IDs
    let child_ids: Vec<String> = node
        .children()
        .enumerate()
        .map(|(idx, child)| build_tree_sync(&child, Some(node_id.clone()), idx, pending))
        .collect();

    // Now create this node with actual child IDs
    let tree_node = TreeNode {
        kind: format!("{:?}", node.kind()),
        text: node.text().to_string(),
        parent: parent_id,
        children: child_ids,
        child_index,
    };

    pending.push(PendingNode {
        id: node_id.clone(),
        node: tree_node,
    });

    node_id
}

/// Get a single node by ID - returns (id, node) tuple
pub async fn get_node(db: &Surreal<Db>, id: &str) -> Result<(String, TreeNode), String> {
    let mut result = db
        .query("SELECT * FROM type::thing('nodes', $id)")
        .bind(("id", id.to_string()))
        .await
        .map_err(|e| e.to_string())?;

    let node: Option<TreeNode> = result.take(0).map_err(|e| e.to_string())?;
    let node = node.ok_or_else(|| format!("Node {} not found", id))?;

    Ok((id.to_string(), node))
}

/// Get children of a node by querying their IDs directly
pub async fn get_children(db: &Surreal<Db>, parent_id: &str) -> Result<Vec<(String, TreeNode)>, String> {
    // First get the parent to find child IDs
    let (_, parent) = get_node(db, parent_id).await?;

    // Fetch each child in order
    let mut children = Vec::new();
    for child_id in &parent.children {
        match get_node(db, child_id).await {
            Ok(child) => children.push(child),
            Err(e) => eprintln!("Warning: couldn't fetch child {}: {}", child_id, e),
        }
    }

    Ok(children)
}

/// Update node text
pub async fn update_node_text(db: &Surreal<Db>, id: &str, new_text: String) -> Result<(), String> {
    let _: Option<TreeNode> = db
        .update(("nodes", id))
        .merge(serde_json::json!({ "text": new_text }))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}