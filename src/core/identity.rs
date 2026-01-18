#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Position-based identity (hierarchical UUID v5)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct NodeId(pub Uuid);

impl NodeId {
    /// Create root node ID
    pub fn root(namespace: &str) -> Self {
        Self(Uuid::new_v5(
            &Uuid::NAMESPACE_OID,
            namespace.as_bytes(),
        ))
    }
    
    /// Create child node ID
    pub fn child(parent: Self, index: usize) -> Self {
        Self(Uuid::new_v5(
            &parent.0,
            &index.to_le_bytes(),
        ))
    }
    
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

/// Content-based identity (Blake3 hash)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ContentHash(pub u128);

impl ContentHash {
    pub fn from_content(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        let bytes: [u8; 16] = hash.as_bytes()[..16].try_into().unwrap();
        Self(u128::from_le_bytes(bytes))
    }
    
    pub fn to_hex(&self) -> String {
        format!("{:032x}", self.0)
    }
}

/// Interned string handle (Blake3 hash of text)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Spur(pub u128);

impl Spur {
    pub fn from_text(text: &str) -> Self {
        let hash = blake3::hash(text.as_bytes());
        let bytes: [u8; 16] = hash.as_bytes()[..16].try_into().unwrap();
        Self(u128::from_le_bytes(bytes))
    }
    
    pub fn to_hex(&self) -> String {
        format!("{:032x}", self.0)
    }
}
