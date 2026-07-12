//! Replacement instantiation and in-place graph edits.
//!
//! Given a validated template and the bindings from a successful match, the
//! replacer builds a concrete node by substituting variables (requirement R9).
//! A separate helper replaces the matched node inside the working graph at the
//! path returned by the matcher.

use crate::matcher::Bindings;
use crate::model::{is_wildcard, variable_name, Node};
use links_notation::LiNo;

/// Errors that can occur while instantiating a template. The compiler validates
/// templates ahead of time, so these represent internal invariants rather than
/// user-facing conditions during a normal run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplaceError {
    /// A template referenced a variable that the pattern never bound.
    UnboundVariable(String),
    /// A template contained the anonymous wildcard, which cannot be produced.
    WildcardInTemplate,
}

impl std::fmt::Display for ReplaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplaceError::UnboundVariable(name) => {
                write!(f, "template references unbound variable '${}'", name)
            }
            ReplaceError::WildcardInTemplate => {
                write!(f, "template contains the anonymous wildcard '_'")
            }
        }
    }
}

impl std::error::Error for ReplaceError {}

/// Builds a concrete node from `template`, substituting variables from
/// `bindings`.
pub fn instantiate(template: &Node, bindings: &Bindings) -> Result<Node, ReplaceError> {
    match template {
        LiNo::Ref(reference) => {
            if is_wildcard(reference) {
                return Err(ReplaceError::WildcardInTemplate);
            }
            if let Some(name) = variable_name(reference) {
                return bindings
                    .get(name)
                    .cloned()
                    .ok_or_else(|| ReplaceError::UnboundVariable(name.to_string()));
            }
            Ok(LiNo::Ref(reference.clone()))
        }
        LiNo::Link { id, values } => {
            let mut new_values = Vec::with_capacity(values.len());
            for value in values {
                new_values.push(instantiate(value, bindings)?);
            }
            Ok(LiNo::Link {
                id: id.clone(),
                values: new_values,
            })
        }
    }
}

/// Returns a copy of `root` with the node located at `path` replaced by
/// `replacement`. An empty path replaces the root itself. Each path element is
/// an index into a link's `values`.
pub fn replace_at(root: &Node, path: &[usize], replacement: Node) -> Node {
    match path.split_first() {
        None => replacement,
        Some((&index, rest)) => match root {
            LiNo::Link { id, values } => {
                let mut new_values = values.clone();
                if let Some(child) = new_values.get(index) {
                    let updated = replace_at(child, rest, replacement);
                    new_values[index] = updated;
                }
                LiNo::Link {
                    id: id.clone(),
                    values: new_values,
                }
            }
            // A non-empty path cannot descend into a reference; return the root
            // unchanged. The matcher never produces such a path.
            LiNo::Ref(_) => root.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Node {
        let document = links_notation::parse_lino(source).expect("valid lino");
        match document {
            LiNo::Link { values, .. } => values.into_iter().next().expect("one link"),
            other => other,
        }
    }

    fn bindings(pairs: &[(&str, &str)]) -> Bindings {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), parse(v)))
            .collect()
    }

    #[test]
    fn substitutes_bound_variables() {
        let template = parse("($y before $x)");
        let bound = bindings(&[("x", "a"), ("y", "b")]);
        assert_eq!(
            instantiate(&template, &bound).unwrap(),
            parse("(b before a)")
        );
    }

    #[test]
    fn reuses_matched_subgraphs() {
        // Requirement R9: replacements can reuse whole matched substructures.
        let template = parse("(wrapped $x)");
        let bound = bindings(&[("x", "(deep (nested structure))")]);
        assert_eq!(
            instantiate(&template, &bound).unwrap(),
            parse("(wrapped (deep (nested structure)))")
        );
    }

    #[test]
    fn unbound_variable_is_an_error() {
        let template = parse("($missing)");
        let err = instantiate(&template, &Bindings::new()).unwrap_err();
        assert_eq!(err, ReplaceError::UnboundVariable("missing".into()));
    }

    #[test]
    fn replace_at_root_replaces_everything() {
        let root = parse("(a b)");
        assert_eq!(replace_at(&root, &[], parse("(c d)")), parse("(c d)"));
    }

    #[test]
    fn replace_at_descends_into_path() {
        let root = parse("(outer (inner target) other)");
        let updated = replace_at(&root, &[1, 1], parse("replaced"));
        assert_eq!(updated, parse("(outer (inner replaced) other)"));
    }
}
