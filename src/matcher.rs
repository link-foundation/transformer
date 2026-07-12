//! Structural pattern matching with variable binding.
//!
//! The matcher compares a pattern node against a data node structurally rather
//! than by serialized text (requirement R6): two links match when their ids and
//! their ordered children match. Variables bind sub-nodes and unify across
//! repeated occurrences (requirement R7); the anonymous wildcard matches any
//! node without binding (requirement R8).

use crate::model::{is_wildcard, variable_name, Guard, Node};
use links_notation::LiNo;
use std::collections::BTreeMap;

/// A set of variable bindings produced by a successful match. A `BTreeMap`
/// keeps the ordering deterministic for tracing and tests.
pub type Bindings = BTreeMap<String, Node>;

/// Attempts to match `pattern` against `data`, extending `bindings`.
///
/// On success the bindings map contains every variable captured by the pattern.
/// On failure the function returns `false`; callers should discard any partial
/// bindings by cloning before the call when they need to retry.
pub fn match_node(pattern: &Node, data: &Node, bindings: &mut Bindings) -> bool {
    match pattern {
        LiNo::Ref(reference) => {
            if is_wildcard(reference) {
                return true;
            }
            if let Some(name) = variable_name(reference) {
                return bind_variable(name, data, bindings);
            }
            // A literal reference matches only an equal literal reference.
            matches!(data, LiNo::Ref(value) if value == reference)
        }
        LiNo::Link {
            id: pattern_id,
            values: pattern_values,
        } => match data {
            LiNo::Link {
                id: data_id,
                values: data_values,
            } => {
                if !ids_match(pattern_id.as_deref(), data_id.as_deref()) {
                    return false;
                }
                if pattern_values.len() != data_values.len() {
                    return false;
                }
                pattern_values
                    .iter()
                    .zip(data_values.iter())
                    .all(|(p, d)| match_node(p, d, bindings))
            }
            LiNo::Ref(_) => false,
        },
    }
}

/// Ids match when both are absent, both are equal, or the pattern id is the
/// anonymous wildcard.
fn ids_match(pattern_id: Option<&str>, data_id: Option<&str>) -> bool {
    match (pattern_id, data_id) {
        (None, None) => true,
        (Some(p), _) if is_wildcard(p) => true,
        (Some(p), Some(d)) => p == d,
        _ => false,
    }
}

/// Binds `name` to `data`, unifying with any existing binding (requirement R7).
fn bind_variable(name: &str, data: &Node, bindings: &mut Bindings) -> bool {
    match bindings.get(name) {
        Some(existing) => existing == data,
        None => {
            bindings.insert(name.to_string(), data.clone());
            true
        }
    }
}

/// Resolves an operand node against the current bindings: variables become
/// their bound value, wildcards are rejected (they cannot be compared), and
/// every other node is returned as-is.
fn resolve_operand(node: &Node, bindings: &Bindings) -> Option<Node> {
    match node {
        LiNo::Ref(reference) => {
            if is_wildcard(reference) {
                None
            } else if let Some(name) = variable_name(reference) {
                bindings.get(name).cloned()
            } else {
                Some(node.clone())
            }
        }
        LiNo::Link { id, values } => {
            let mut resolved = Vec::with_capacity(values.len());
            for value in values {
                resolved.push(resolve_operand(value, bindings)?);
            }
            Some(LiNo::Link {
                id: id.clone(),
                values: resolved,
            })
        }
    }
}

/// Evaluates a guard against the given bindings (requirement R13). An operand
/// that cannot be resolved (an unbound variable or a wildcard) fails the guard
/// rather than panicking.
pub fn evaluate_guard(guard: &Guard, bindings: &Bindings) -> bool {
    match guard {
        Guard::Equal(a, b) => match (resolve_operand(a, bindings), resolve_operand(b, bindings)) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        },
        Guard::NotEqual(a, b) => {
            match (resolve_operand(a, bindings), resolve_operand(b, bindings)) {
                (Some(a), Some(b)) => a != b,
                _ => false,
            }
        }
        Guard::All(guards) => guards.iter().all(|g| evaluate_guard(g, bindings)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Node {
        // Every fixture here is a single top-level link, so unwrap the document
        // wrapper to get at the link the test cares about.
        let document = links_notation::parse_lino(source).expect("valid lino");
        match document {
            LiNo::Link { values, .. } => values.into_iter().next().expect("one link"),
            other => other,
        }
    }

    fn matches(pattern: &str, data: &str) -> Option<Bindings> {
        let mut bindings = Bindings::new();
        if match_node(&parse(pattern), &parse(data), &mut bindings) {
            Some(bindings)
        } else {
            None
        }
    }

    #[test]
    fn literal_reference_matches_equal_reference() {
        assert!(matches("(a b)", "(a b)").is_some());
        assert!(matches("(a b)", "(a c)").is_none());
    }

    #[test]
    fn wildcard_matches_anything_without_binding() {
        let bindings = matches("(a _)", "(a (deep structure))").expect("match");
        assert!(bindings.is_empty());
    }

    #[test]
    fn variable_binds_matched_subtree() {
        let bindings = matches("(a $x)", "(a (b c))").expect("match");
        assert_eq!(bindings.get("x"), Some(&parse("(b c)")));
    }

    #[test]
    fn repeated_variable_must_unify() {
        assert!(matches("($x before $x)", "(p before p)").is_some());
        assert!(matches("($x before $x)", "(p before q)").is_none());
    }

    #[test]
    fn different_arity_does_not_match() {
        assert!(matches("(a b)", "(a b c)").is_none());
    }

    #[test]
    fn structural_match_ignores_formatting() {
        // Requirement R6: equivalent structures match regardless of spelling of
        // the serialized text (extra whitespace here).
        assert!(matches("($x before $y)", "(  a   before   b )").is_some());
    }

    #[test]
    fn identifier_links_match_on_id() {
        assert!(matches("(loves: $x $y)", "(loves: a b)").is_some());
        assert!(matches("(loves: $x $y)", "(hates: a b)").is_none());
        // Wildcard id matches any id.
        assert!(matches("(_: $x $y)", "(hates: a b)").is_some());
    }

    #[test]
    fn guard_equality_uses_bindings() {
        let mut bindings = Bindings::new();
        assert!(match_node(
            &parse("($x $y)"),
            &parse("(a a)"),
            &mut bindings
        ));
        let eq = Guard::Equal(LiNo::Ref("$x".into()), LiNo::Ref("$y".into()));
        assert!(evaluate_guard(&eq, &bindings));
        let ne = Guard::NotEqual(LiNo::Ref("$x".into()), LiNo::Ref("$y".into()));
        assert!(!evaluate_guard(&ne, &bindings));
    }
}
