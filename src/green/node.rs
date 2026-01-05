use crate::core::{ContentHash, Span, SyntaxId, Spur};
use serde::{Deserialize, Serialize};

/// Content-addressed node (stored in SurrealDB)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GreenNode {
    pub content_hash: ContentHash,
    pub syntax_id: SyntaxId,
    pub span: Span,
    pub data: GreenData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GreenData {
    Branch {
        children: Vec<ContentHash>,
    },
    Token {
        text: Spur,
    },
}

impl GreenNode {
    pub fn new_branch(
        syntax_id: SyntaxId,
        span: Span,
        children: Vec<ContentHash>,
    ) -> Self {
        let data = GreenData::Branch { children };
        let content_hash = Self::compute_hash(syntax_id, span, &data);
        
        Self {
            content_hash,
            syntax_id,
            span,
            data,
        }
    }
    
    pub fn new_token(
        syntax_id: SyntaxId,
        span: Span,
        text: Spur,
    ) -> Self {
        let data = GreenData::Token { text };
        let content_hash = Self::compute_hash(syntax_id, span, &data);
        
        Self {
            content_hash,
            syntax_id,
            span,
            data,
        }
    }
    
    fn compute_hash(syntax_id: SyntaxId, span: Span, data: &GreenData) -> ContentHash {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&syntax_id.0.to_le_bytes());
        hasher.update(&span.start.to_le_bytes());
        hasher.update(&span.len().to_le_bytes());
        
        match data {
            GreenData::Branch { children } => {
                hasher.update(b"branch");
                for child in children {
                    hasher.update(&child.0.to_le_bytes());
                }
            }
            GreenData::Token { text } => {
                hasher.update(b"token");
                hasher.update(&text.0.to_le_bytes());
            }
        }
        
        let hash = hasher.finalize();
        let bytes: [u8; 16] = hash.as_bytes()[..16].try_into().unwrap();
        ContentHash(u128::from_le_bytes(bytes))
    }
}
