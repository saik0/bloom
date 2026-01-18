#![allow(dead_code)]
use serde::{Deserialize, Serialize};

/// Byte range in source text
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
    
    pub fn len(&self) -> u32 {
        self.end - self.start
    }
}
