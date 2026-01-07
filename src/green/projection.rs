//! GREEN Tree Projection to SurrealDB
//!
//! This is a SIDE-CHANNEL for analytics and queries that rowan can't do.
//! The rowan SyntaxNode is the source of truth; this is a derived projection.
//!
//! Use cases:
//! - Cross-file symbol graphs
//! - Temporal history ("when did this function change?")
//! - RAG embeddings for AI features
//! - Complex graph queries (call graphs, dependency analysis)

use ra_ap_syntax::SyntaxNode;
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

/// A projected node in SurrealDB - derived from rowan, not source of truth
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectedNode {
    pub content_hash: String,
    pub kind: String,
    pub text: String,
    pub span_start: u32,
    pub span_len: u32,
    pub children: Vec<String>,  // Content hashes of children
}

/// Compute content hash for a syntax node
fn content_hash(node: &SyntaxNode) -> String {
    let mut hasher = blake3::Hasher::new();
    
    // Hash the kind
    let kind: u16 = node.kind().into();
    hasher.update(&kind.to_le_bytes());
    
    // Hash the span
    let range = node.text_range();
    hasher.update(&u32::from(range.start()).to_le_bytes());
    hasher.update(&u32::from(range.len()).to_le_bytes());
    
    // Hash children or text
    let children: Vec<_> = node.children().collect();
    if children.is_empty() {
        hasher.update(b"token:");
        hasher.update(node.text().to_string().as_bytes());
    } else {
        hasher.update(b"branch:");
        for child in &children {
            hasher.update(content_hash(child).as_bytes());
        }
    }
    
    let hash = hasher.finalize();
    hex::encode(&hash.as_bytes()[..16])
}

/// Extract projection data from a rowan tree (synchronous, happens on main thread)
/// 
/// Returns Vec of nodes ready to be inserted into DB
pub fn extract_projection(root: &SyntaxNode) -> (Vec<ProjectedNode>, String) {
    let mut nodes = Vec::new();
    collect_nodes(root, &mut nodes);
    let root_hash = nodes.last().map(|n| n.content_hash.clone()).unwrap_or_default();
    (nodes, root_hash)
}

/// Insert pre-extracted projection into DB (async, can happen on background thread)
pub async fn insert_projection(
    db: &Surreal<Db>, 
    nodes: Vec<ProjectedNode>, 
    file_path: String,
    root_hash: String,
) -> Result<(), String> {
    // Batch upsert - same content = no-op (deduplication)
    for node in nodes {
        let _: Option<ProjectedNode> = db
            .upsert(("nodes", &node.content_hash))
            .content(node)
            .await
            .map_err(|e| e.to_string())?;
    }
    
    // Store file -> root mapping
    db.query("UPSERT INTO file_roots { file: $file, root_hash: $hash, updated_at: time::now() }")
        .bind(("file", file_path))
        .bind(("hash", root_hash))
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

fn collect_nodes(node: &SyntaxNode, out: &mut Vec<ProjectedNode>) {
    let children: Vec<_> = node.children().collect();
    
    // Process children first (bottom-up for hash computation)
    for child in &children {
        collect_nodes(child, out);
    }
    
    let range = node.text_range();
    let child_hashes: Vec<String> = children.iter().map(content_hash).collect();
    
    out.push(ProjectedNode {
        content_hash: content_hash(node),
        kind: format!("{:?}", node.kind()),
        text: node.text().to_string(),
        span_start: range.start().into(),
        span_len: range.len().into(),
        children: child_hashes,
    });
}

/// Schema for the projection database
pub const SCHEMA: &str = r#"
-- Projected GREEN nodes (derived from rowan, not source of truth)
DEFINE TABLE nodes SCHEMAFULL;
DEFINE FIELD content_hash ON nodes TYPE string;
DEFINE FIELD kind ON nodes TYPE string;
DEFINE FIELD text ON nodes TYPE string;
DEFINE FIELD span_start ON nodes TYPE int;
DEFINE FIELD span_len ON nodes TYPE int;
DEFINE FIELD children ON nodes TYPE array<string>;

-- File to root mapping
DEFINE TABLE file_roots SCHEMAFULL;
DEFINE FIELD file ON file_roots TYPE string;
DEFINE FIELD root_hash ON file_roots TYPE string;
DEFINE FIELD updated_at ON file_roots TYPE datetime;
DEFINE INDEX file_idx ON file_roots FIELDS file UNIQUE;
"#;
