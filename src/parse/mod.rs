//! Parsing module
//! 
//! Thin wrapper around ra_ap_syntax. The parsed tree IS the editor state.

use ra_ap_syntax::{SourceFile, SyntaxNode, Edition};

/// Parse Rust source code into a syntax tree.
/// 
/// Returns the root SyntaxNode which owns the entire tree.
/// This is the source of truth for the editor.
pub fn parse_rust(source: &str) -> SyntaxNode {
    let parse = SourceFile::parse(source, Edition::Edition2021);
    parse.syntax_node()
}

/// Get parse errors if any
pub fn parse_errors(source: &str) -> Vec<String> {
    let parse = SourceFile::parse(source, Edition::Edition2021);
    parse.errors().iter().map(|e| e.to_string()).collect()
}
