use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Db;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityRecord {
    pub salsa_id: String,
    pub content_hash: String,
    pub path: Vec<u32>,
    pub parent: Option<String>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRootRecord {
    pub path: String,
    pub root_hash: String,
}

pub async fn upsert_identity(
    db: &Surreal<Db>,
    record: &IdentityRecord,
) -> Result<(), String> {
    let _: Option<IdentityRecord> = db
        .update(("identity", record.salsa_id.as_str()))
        .content(record.clone())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn get_identity_by_salsa_id(
    db: &Surreal<Db>,
    salsa_id: &str,
) -> Result<Option<IdentityRecord>, String> {
    db.select(("identity", salsa_id))
        .await
        .map_err(|e| e.to_string())
}

pub async fn upsert_source_root(
    db: &Surreal<Db>,
    record: &SourceRootRecord,
) -> Result<(), String> {
    let _: Option<SourceRootRecord> = db
        .update(("source_root", record.path.as_str()))
        .content(record.clone())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn relate_identity_to_node(
    db: &Surreal<Db>,
    salsa_id: &str,
    node_id: &Thing,
) -> Result<(), String> {
    db.query("RELATE type::thing('identity', $id) -> identity_of -> $node")
        .bind(("id", salsa_id.to_string()))
        .bind(("node", node_id.clone()))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn get_node_record_id_by_salsa_id(
    db: &Surreal<Db>,
    salsa_id: &str,
) -> Result<Option<Thing>, String> {
    #[derive(Deserialize)]
    struct IdentityLink {
        out: Thing,
    }

    let mut result = db
        .query("SELECT out FROM identity_of WHERE in = type::thing('identity', $id)")
        .bind(("id", salsa_id.to_string()))
        .await
        .map_err(|e| e.to_string())?;

    let links: Vec<IdentityLink> = result.take(0).map_err(|e| e.to_string())?;
    Ok(links.into_iter().next().map(|link| link.out))
}

pub async fn get_node_by_salsa_id(
    db: &Surreal<Db>,
    salsa_id: &str,
) -> Result<Option<serde_json::Value>, String> {
    let Some(node_id) = get_node_record_id_by_salsa_id(db, salsa_id).await? else {
        return Ok(None);
    };

    let mut result = db
        .query("SELECT * FROM $node")
        .bind(("node", node_id))
        .await
        .map_err(|e| e.to_string())?;

    let mut rows: Vec<serde_json::Value> = result.take(0).map_err(|e| e.to_string())?;
    Ok(rows.pop())
}

pub async fn get_children_by_salsa_id(
    db: &Surreal<Db>,
    salsa_id: &str,
) -> Result<Vec<serde_json::Value>, String> {
    let Some(node_id) = get_node_record_id_by_salsa_id(db, salsa_id).await? else {
        return Ok(Vec::new());
    };

    let mut result = db
        .query("SELECT out.* FROM child_of WHERE in = $node ORDER BY index")
        .bind(("node", node_id))
        .await
        .map_err(|e| e.to_string())?;

    result.take(0).map_err(|e| e.to_string())
}

pub fn identity_watch_query(salsa_id: &str) -> String {
    format!("LIVE SELECT * FROM identity WHERE salsa_id = '{}'", salsa_id)
}
