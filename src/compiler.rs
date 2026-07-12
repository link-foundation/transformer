//! Compiles a Links Notation program into the immutable [`Program`] model.
//!
//! The program dialect is itself Links Notation (requirement R2). A program is
//! a document whose top-level elements are identifier links:
//!
//! ```lino
//! (programVersion: 1)
//! (rule:
//!   (name: swap)
//!   (match: ($x before $y))
//!   (replace: ($y before $x))
//!   (maxApplications: 1)
//!   (terminal: false)
//!   (priority: 0)
//!   (guard: (notEqual $x $y)))
//! ```
//!
//! Variables are references beginning with `$`; `_` is the anonymous wildcard.
//! The compiler validates variable usage and metadata, rejecting unknown
//! attributes and templates that reference unbound variables (requirement R25).

use crate::diagnostics::{Diagnostic, Diagnostics, Location};
use crate::model::{is_wildcard, variable_name, Guard, Node, Program, Rule};
use links_notation::{parse_lino, LiNo};
use std::collections::BTreeSet;

/// Compiles `source` into a [`Program`], or returns every diagnostic found.
pub fn compile_program(source: &str) -> Result<Program, Diagnostics> {
    let document = match parse_lino(source) {
        Ok(document) => document,
        Err(error) => {
            let mut diagnostics = Diagnostics::new();
            diagnostics.push(Diagnostic::new(
                Location::Program,
                format!("failed to parse program: {}", error),
            ));
            return Err(diagnostics);
        }
    };

    let top_level = match document {
        LiNo::Link { values, .. } => values,
        LiNo::Ref(_) => {
            let mut diagnostics = Diagnostics::new();
            diagnostics.push(Diagnostic::new(
                Location::Program,
                "a program must be a sequence of links, not a bare reference",
            ));
            return Err(diagnostics);
        }
    };

    let mut diagnostics = Diagnostics::new();
    let mut version: Option<u64> = None;
    let mut rules: Vec<Rule> = Vec::new();

    for (index, element) in top_level.iter().enumerate() {
        match element {
            LiNo::Link {
                id: Some(id),
                values,
            } if id == "programVersion" => {
                version = compile_version(values, index, &mut diagnostics);
            }
            LiNo::Link {
                id: Some(id),
                values,
            } if id == "rule" => {
                let order = rules.len();
                if let Some(rule) = compile_rule(values, order, &mut diagnostics) {
                    rules.push(rule);
                }
            }
            LiNo::Link { id: Some(id), .. } => {
                diagnostics.push(Diagnostic::new(
                    Location::TopLevel { index },
                    format!(
                        "unknown top-level element '{}'; expected 'programVersion' or 'rule'",
                        id
                    ),
                ));
            }
            _ => {
                diagnostics.push(Diagnostic::new(
                    Location::TopLevel { index },
                    "top-level elements must be identifier links such as (rule: ...)",
                ));
            }
        }
    }

    if rules.is_empty() && diagnostics.is_empty() {
        diagnostics.push(Diagnostic::new(
            Location::Program,
            "a program must declare at least one rule",
        ));
    }

    if let Some(v) = version {
        if v > Program::SUPPORTED_VERSION {
            diagnostics.push(Diagnostic::new(
                Location::Program,
                format!(
                    "program version {} is newer than the supported version {}",
                    v,
                    Program::SUPPORTED_VERSION
                ),
            ));
        }
    }

    if diagnostics.is_empty() {
        Ok(Program { version, rules })
    } else {
        Err(diagnostics)
    }
}

fn compile_version(values: &[Node], index: usize, diagnostics: &mut Diagnostics) -> Option<u64> {
    match single_reference(values) {
        Some(text) => match text.parse::<u64>() {
            Ok(v) => Some(v),
            Err(_) => {
                diagnostics.push(Diagnostic::new(
                    Location::TopLevel { index },
                    format!(
                        "programVersion must be a non-negative integer, found '{}'",
                        text
                    ),
                ));
                None
            }
        },
        None => {
            diagnostics.push(Diagnostic::new(
                Location::TopLevel { index },
                "programVersion must have exactly one integer value",
            ));
            None
        }
    }
}

fn compile_rule(values: &[Node], order: usize, diagnostics: &mut Diagnostics) -> Option<Rule> {
    let mut name: Option<String> = None;
    let mut pattern: Option<Node> = None;
    let mut template: Option<Node> = None;
    let mut guard: Option<Guard> = None;
    let mut priority: i64 = 0;
    let mut max_applications: Option<u64> = None;
    let mut terminal = false;
    let location = |name: &Option<String>| Location::Rule {
        index: order,
        name: name.clone(),
    };

    for attribute in values {
        let (id, attr_values) = match attribute {
            LiNo::Link {
                id: Some(id),
                values,
            } => (id.as_str(), values.as_slice()),
            _ => {
                diagnostics.push(Diagnostic::new(
                    location(&name),
                    "each rule attribute must be an identifier link such as (match: ...)",
                ));
                continue;
            }
        };

        match id {
            "name" => match single_reference(attr_values) {
                Some(text) => name = Some(text.to_string()),
                None => diagnostics.push(Diagnostic::new(
                    location(&name),
                    "name must be a single reference",
                )),
            },
            "match" => match single_value(attr_values) {
                Some(node) => pattern = Some(node.clone()),
                None => diagnostics.push(Diagnostic::new(
                    location(&name),
                    "match must contain exactly one pattern",
                )),
            },
            "replace" => match single_value(attr_values) {
                Some(node) => template = Some(node.clone()),
                None => diagnostics.push(Diagnostic::new(
                    location(&name),
                    "replace must contain exactly one template",
                )),
            },
            "guard" => match single_value(attr_values) {
                Some(node) => match compile_guard(node) {
                    Ok(g) => guard = Some(g),
                    Err(message) => diagnostics.push(Diagnostic::new(location(&name), message)),
                },
                None => diagnostics.push(Diagnostic::new(
                    location(&name),
                    "guard must contain exactly one expression",
                )),
            },
            "priority" => match single_reference(attr_values).and_then(|t| t.parse::<i64>().ok()) {
                Some(value) => priority = value,
                None => diagnostics.push(Diagnostic::new(
                    location(&name),
                    "priority must be a single integer",
                )),
            },
            "maxApplications" => {
                match single_reference(attr_values).and_then(|t| t.parse::<u64>().ok()) {
                    Some(value) => max_applications = Some(value),
                    None => diagnostics.push(Diagnostic::new(
                        location(&name),
                        "maxApplications must be a single non-negative integer",
                    )),
                }
            }
            "terminal" => match single_reference(attr_values).and_then(parse_bool) {
                Some(value) => terminal = value,
                None => diagnostics.push(Diagnostic::new(
                    location(&name),
                    "terminal must be 'true' or 'false'",
                )),
            },
            other => diagnostics.push(Diagnostic::new(
                location(&name),
                format!("unknown rule attribute '{}'", other),
            )),
        }
    }

    let pattern = match pattern {
        Some(pattern) => pattern,
        None => {
            diagnostics.push(Diagnostic::new(
                location(&name),
                "rule is missing a match pattern",
            ));
            return None;
        }
    };
    let template = match template {
        Some(template) => template,
        None => {
            diagnostics.push(Diagnostic::new(
                location(&name),
                "rule is missing a replace template",
            ));
            return None;
        }
    };

    // Validate that the template and guard only reference variables the pattern
    // binds, and that the template contains no wildcard (requirement R25).
    let pattern_vars = collect_variables(&pattern);
    let mut valid = true;
    for var in collect_variables(&template) {
        if !pattern_vars.contains(&var) {
            diagnostics.push(Diagnostic::new(
                location(&name),
                format!("replace references unbound variable '${}'", var),
            ));
            valid = false;
        }
    }
    if contains_wildcard(&template) {
        diagnostics.push(Diagnostic::new(
            location(&name),
            "replace must not contain the anonymous wildcard '_'",
        ));
        valid = false;
    }
    if let Some(guard) = &guard {
        for var in collect_guard_variables(guard) {
            if !pattern_vars.contains(&var) {
                diagnostics.push(Diagnostic::new(
                    location(&name),
                    format!("guard references unbound variable '${}'", var),
                ));
                valid = false;
            }
        }
    }

    if !valid {
        return None;
    }

    Some(Rule {
        name,
        pattern,
        template,
        guard,
        priority,
        order,
        max_applications,
        terminal,
    })
}

fn compile_guard(node: &Node) -> Result<Guard, String> {
    match node {
        LiNo::Link { id: None, values } => match values.split_first() {
            Some((LiNo::Ref(op), rest)) => match op.as_str() {
                "equal" if rest.len() == 2 => Ok(Guard::Equal(rest[0].clone(), rest[1].clone())),
                "notEqual" if rest.len() == 2 => {
                    Ok(Guard::NotEqual(rest[0].clone(), rest[1].clone()))
                }
                "all" => {
                    let mut guards = Vec::new();
                    for operand in rest {
                        guards.push(compile_guard(operand)?);
                    }
                    Ok(Guard::All(guards))
                }
                "equal" | "notEqual" => Err(format!("guard '{}' expects exactly two operands", op)),
                other => Err(format!("unknown guard operator '{}'", other)),
            },
            _ => Err("guard must start with an operator (equal, notEqual, all)".to_string()),
        },
        _ => Err("guard must be a link such as (notEqual $x $y)".to_string()),
    }
}

/// Returns the single value if `values` has exactly one element.
fn single_value(values: &[Node]) -> Option<&Node> {
    match values {
        [value] => Some(value),
        _ => None,
    }
}

/// Returns the single reference string if `values` is exactly one reference.
fn single_reference(values: &[Node]) -> Option<&str> {
    match values {
        [LiNo::Ref(text)] => Some(text.as_str()),
        _ => None,
    }
}

fn parse_bool(text: &str) -> Option<bool> {
    match text {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Collects every variable name that appears in a node.
fn collect_variables(node: &Node) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    collect_variables_into(node, &mut set);
    set
}

fn collect_variables_into(node: &Node, set: &mut BTreeSet<String>) {
    match node {
        LiNo::Ref(reference) => {
            if let Some(name) = variable_name(reference) {
                set.insert(name.to_string());
            }
        }
        LiNo::Link { values, .. } => {
            for value in values {
                collect_variables_into(value, set);
            }
        }
    }
}

fn collect_guard_variables(guard: &Guard) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    match guard {
        Guard::Equal(a, b) | Guard::NotEqual(a, b) => {
            collect_variables_into(a, &mut set);
            collect_variables_into(b, &mut set);
        }
        Guard::All(guards) => {
            for guard in guards {
                set.extend(collect_guard_variables(guard));
            }
        }
    }
    set
}

fn contains_wildcard(node: &Node) -> bool {
    match node {
        LiNo::Ref(reference) => is_wildcard(reference),
        LiNo::Link { values, .. } => values.iter().any(contains_wildcard),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_a_complete_rule() {
        let program = compile_program(
            "(programVersion: 1)\n\
             (rule: (name: swap) (priority: 2) (maxApplications: 3) (terminal: false) \
             (match: ($x before $y)) (replace: ($y before $x)) (guard: (notEqual $x $y)))",
        )
        .expect("compiles");
        assert_eq!(program.version, Some(1));
        assert_eq!(program.rules.len(), 1);
        let rule = &program.rules[0];
        assert_eq!(rule.name.as_deref(), Some("swap"));
        assert_eq!(rule.priority, 2);
        assert_eq!(rule.max_applications, Some(3));
        assert!(!rule.terminal);
        assert!(rule.guard.is_some());
    }

    #[test]
    fn rejects_unbound_template_variable() {
        let error = compile_program("(rule: (match: ($x)) (replace: ($y)))").unwrap_err();
        assert!(error.to_string().contains("unbound variable '$y'"));
    }

    #[test]
    fn rejects_wildcard_in_template() {
        let error = compile_program("(rule: (match: ($x)) (replace: (_)))").unwrap_err();
        assert!(error.to_string().contains("anonymous wildcard"));
    }

    #[test]
    fn rejects_unknown_attribute() {
        let error =
            compile_program("(rule: (match: (a)) (replace: (b)) (color: red))").unwrap_err();
        assert!(error.to_string().contains("unknown rule attribute 'color'"));
    }

    #[test]
    fn rejects_unknown_top_level_element() {
        let error = compile_program("(mystery: 1)").unwrap_err();
        assert!(error
            .to_string()
            .contains("unknown top-level element 'mystery'"));
    }

    #[test]
    fn rejects_missing_pattern() {
        let error = compile_program("(rule: (replace: (b)))").unwrap_err();
        assert!(error.to_string().contains("missing a match pattern"));
    }

    #[test]
    fn rejects_empty_program() {
        let error = compile_program("(programVersion: 1)").unwrap_err();
        assert!(error.to_string().contains("at least one rule"));
    }

    #[test]
    fn rejects_future_version() {
        let error = compile_program("(programVersion: 999)\n(rule: (match: (a)) (replace: (b)))")
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("newer than the supported version"));
    }

    #[test]
    fn reports_multiple_diagnostics_at_once() {
        let error =
            compile_program("(rule: (match: ($x)) (replace: ($y)) (bad: attr))").unwrap_err();
        assert!(error.len() >= 2);
    }
}
