use crate::core::{ContentHash, NodeId, Span, SyntaxId, Spur};
use crate::green::GreenNode;
use ra_ap_syntax::{SourceFile, SyntaxNode};
use ra_ap_syntax::Edition::Edition2021;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::local::Mem, Surreal};
use surrealdb::engine::local::Db;
use surrealdb::sql::Thing;

pub async fn parse_rust(db: &Surreal<Db>, source: &str, file_name: &str) -> Result<NodeId, String> {
    // --- SYNCHRONOUS PHASE ---
    // The !Send 'root' is confined to this block
    let (flat_nodes, root_id) = {
        let parsed = SourceFile::parse(source, Edition2021);
        let root = parsed.syntax_node();

        let nodes = flatten_tree(&root);
        let root_id = NodeId::root(file_name);

        (nodes, root_id)
    }; // <--- 'root' is dropped here!

    // --- ASYNCHRONOUS PHASE ---
    // Now it is safe to use .await because the !Send data is gone
    for node in flat_nodes {
        let id = node.node_id.clone();

        // Don't try to deserialize the response - just create and move on
        let _: Option<FlatNode> = db
            .create(("nodes", &id))
            .content(node)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(root_id)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FlatNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,  // SurrealDB's record ID
    pub node_id: String,    // Our application ID
    pub kind: String,
    pub text: String,
    pub children: Vec<String>,
}

fn flatten_tree(node: &SyntaxNode) -> Vec<FlatNode> {
    let mut flat_nodes = Vec::new();

    // Simple recursive or iterative traversal
    for child in node.children() {
        flat_nodes.push(FlatNode {
            id: None,  // SurrealDB will set this
            node_id: uuid::Uuid::new_v4().to_string(),
            kind: format!("{:?}", child.kind()),
            text: child.text().to_string(),
            children: child.children().map(|c| uuid::Uuid::new_v4().to_string()).collect(),
        });

        // Recurse
        flat_nodes.extend(flatten_tree(&child));
    }

    flat_nodes
}

// Optional helper for later querying
pub async fn get_node(db: &Surreal<Db>, id: &str) -> Result<FlatNode, String> {
    let node: Option<FlatNode> = db
        .select(("nodes", id))
        .await
        .map_err(|e| e.to_string())?;

    node.ok_or_else(|| format!("Node {} not found", id))
}

// Optional helper for querying all nodes
pub async fn get_all_nodes(db: &Surreal<Db>) -> Result<Vec<FlatNode>, String> {
    let nodes: Vec<FlatNode> = db
        .select("nodes")
        .await
        .map_err(|e| e.to_string())?;

    Ok(nodes)
}