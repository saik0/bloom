use ra_ap_syntax::{SourceFile, SyntaxNode};
use ra_ap_syntax::Edition::Edition2021;
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

/// Tree node stored in SurrealDB - content-addressed by hash
///
/// The node ID is derived from a blake3 hash of:
/// - syntax kind
/// - text content (for tokens) or children hashes (for branches)
/// - span (start, len)
///
/// This means identical subtrees get the same ID = automatic deduplication.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TreeNode {
    pub kind: String,
    pub text: String,
    pub span_start: u32,
    pub span_len: u32,
    pub parent: Option<String>,   // Parent content hash (None for root)
    pub children: Vec<String>,    // Child content hashes
    pub child_index: usize,       // This node's index within parent's children
}

/// Intermediate structure for building the tree synchronously
struct PendingNode {
    id: String,  // Content hash
    node: TreeNode,
}

/// Compute content-addressed ID for a syntax node
///
/// Hash includes: kind + span + (children hashes OR text content)
/// This gives us stable identity: same content = same hash
fn compute_content_hash(
    kind: u16,
    span_start: u32,
    span_len: u32,
    text: &str,
    children: &[String],
) -> String {
    let mut hasher = blake3::Hasher::new();

    // Include syntax kind
    hasher.update(&kind.to_le_bytes());

    // Include span
    hasher.update(&span_start.to_le_bytes());
    hasher.update(&span_len.to_le_bytes());

    if children.is_empty() {
        // Token: hash the text content
        hasher.update(b"token:");
        hasher.update(text.as_bytes());
    } else {
        // Branch: hash children's content hashes
        hasher.update(b"branch:");
        for child_hash in children {
            hasher.update(child_hash.as_bytes());
        }
    }

    let hash = hasher.finalize();
    // Use first 16 bytes as hex string for readable IDs
    hex::encode(&hash.as_bytes()[..16])
}

/// Parse Rust source and store tree in DB. Returns root node ID (content hash).
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

    // ASYNC PHASE: Insert all nodes into DB (upsert - same content = no-op)
    for pending in pending_nodes {
        // Use INSERT ... ON DUPLICATE KEY UPDATE for idempotent writes
        // If node with this content hash exists, it's the same content, so skip
        let _: Option<TreeNode> = db
            .upsert(("nodes", &pending.id))
            .content(pending.node)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(root_id)
}

/// Synchronously walk the tree, building nodes bottom-up so children hashes are known
fn build_tree_sync(
    node: &SyntaxNode,
    parent_hash: Option<String>,
    child_index: usize,
    pending: &mut Vec<PendingNode>,
) -> String {
    let range = node.text_range();
    let span_start = u32::from(range.start());
    let span_len = u32::from(range.len());
    let kind: u16 = node.kind().into();
    let text = node.text().to_string();

    // Recursively process children first to get their content hashes
    let child_hashes: Vec<String> = node
        .children()
        .enumerate()
        .map(|(idx, child)| {
            // Parent hash will be set after we compute our own hash
            build_tree_sync(&child, None, idx, pending)
        })
        .collect();

    // Compute content-addressed ID
    let content_hash = compute_content_hash(kind, span_start, span_len, &text, &child_hashes);

    // Now create the node
    let tree_node = TreeNode {
        kind: format!("{:?}", node.kind()),
        text,
        span_start,
        span_len,
        parent: parent_hash,
        children: child_hashes,
        child_index,
    };

    pending.push(PendingNode {
        id: content_hash.clone(),
        node: tree_node,
    });

    content_hash
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

/// Update node text (MUTATES IN PLACE)
///
/// # Content-Addressing Trade-off
///
/// In a pure content-addressed system, changing text would create a NEW node
/// with a new hash, and require updating all ancestors (path copying).
///
/// For Phase 2B simplicity, we mutate in place. This means:
/// - The node ID no longer matches its content hash after edits
/// - Deduplication won't work for edited nodes
/// - Re-parsing the same source would create a different tree
///
/// **Phase 3 TODO**: Implement proper structural sharing with path copying.
/// Edit operations should return the new root hash, and the tree should be
/// rebuilt bottom-up with new hashes for the edited path.
pub async fn update_node_text(db: &Surreal<Db>, id: &str, new_text: String) -> Result<(), String> {
    let _: Option<TreeNode> = db
        .update(("nodes", id))
        .merge(serde_json::json!({ "text": new_text }))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Recompute content hash for a node (for validation/debugging)
pub fn recompute_hash(node: &TreeNode) -> String {
    // We need the syntax kind as u16, but we only store the debug string
    // This is a limitation - for proper content addressing we'd need to store the raw kind
    // For now, hash the kind string instead
    let mut hasher = blake3::Hasher::new();
    hasher.update(node.kind.as_bytes());
    hasher.update(&node.span_start.to_le_bytes());
    hasher.update(&node.span_len.to_le_bytes());

    if node.children.is_empty() {
        hasher.update(b"token:");
        hasher.update(node.text.as_bytes());
    } else {
        hasher.update(b"branch:");
        for child_hash in &node.children {
            hasher.update(child_hash.as_bytes());
        }
    }

    let hash = hasher.finalize();
    hex::encode(&hash.as_bytes()[..16])
}