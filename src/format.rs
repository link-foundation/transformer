//! Canonical, deterministic serialization of a working graph (requirement R20).
//!
//! A transformer graph is rooted at an implicit document container — the
//! anonymous top-level link produced by the LiNo parser. Formatting that
//! container with the upstream `Display` implementation wraps multi-value
//! top-level links in an extra pair of parentheses, which is not what a user
//! who wrote `(a before b)` expects back. The canonical formatter therefore
//! treats a top-level anonymous link as a document and prints each top-level
//! element on its own line.
//!
//! Trivia such as source whitespace and comments is intentionally not preserved
//! (documented non-goal); the formatter is deterministic so identical graphs
//! always render identically.

use crate::model::Node;
use links_notation::LiNo;

/// Formats `node` canonically. When `node` is the anonymous document container,
/// its top-level elements are joined by newlines; otherwise the node is
/// formatted directly.
pub fn canonical(node: &Node) -> String {
    match node {
        LiNo::Link { id: None, values } => values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Node {
        links_notation::parse_lino(source).expect("valid lino")
    }

    #[test]
    fn document_is_not_double_wrapped() {
        assert_eq!(canonical(&parse("(a before b)")), "(a before b)");
        assert_eq!(canonical(&parse("(pair a a)")), "(pair a a)");
    }

    #[test]
    fn multiple_top_level_links_join_by_newline() {
        assert_eq!(canonical(&parse("(a b)\n(c d)")), "(a b)\n(c d)");
    }

    #[test]
    fn is_deterministic() {
        let node = parse("(outer (inner a b) c)");
        assert_eq!(canonical(&node), canonical(&node));
    }
}
