#![allow(dead_code)]
use serde::{Deserialize, Serialize};

/// Syntax kind identifier
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct SyntaxId(pub u16);

impl SyntaxId {
    pub fn from_ra(kind: ra_ap_syntax::SyntaxKind) -> Self {
        Self(kind.into())
    }
}

impl std::fmt::Display for SyntaxId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
