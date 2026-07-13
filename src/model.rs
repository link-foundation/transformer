//! The immutable program model produced by the compiler and consumed by the
//! execution engine.
//!
//! The model is intentionally independent of parsing, scheduling, I/O, and
//! storage (requirements R14 and R19). A [`Program`] is a plain value that can
//! be constructed by the compiler or, in tests, by hand.

use links_notation::LiNo;

/// The concrete node type used throughout the transformer.
///
/// Links Notation values are generic over the reference payload; the
/// transformer works with owned `String` references produced by the upstream
/// parser.
pub type Node = LiNo<String>;

/// The reserved prefix that marks a pattern reference as a binding variable.
///
/// For example `$x` in a match pattern binds the matched node to the name `x`
/// and unifies with any later `$x` in the same pattern (requirement R7).
pub const VARIABLE_PREFIX: char = '$';

/// The reserved anonymous wildcard reference. It matches any node without
/// binding a name (requirement R8).
pub const WILDCARD: &str = "_";

/// Returns the variable name if the reference string denotes a variable.
pub fn variable_name(reference: &str) -> Option<&str> {
    let mut chars = reference.chars();
    match chars.next() {
        Some(VARIABLE_PREFIX) if reference.len() > 1 => {
            Some(&reference[VARIABLE_PREFIX.len_utf8()..])
        }
        _ => None,
    }
}

/// Returns true when the reference string is the anonymous wildcard.
pub fn is_wildcard(reference: &str) -> bool {
    reference == WILDCARD
}

/// An optional structural guard evaluated after a pattern matches but before a
/// replacement is applied (requirement R13).
///
/// Guards operate purely on already-bound values and never execute native or
/// plugin code, keeping untrusted programs safe by construction.
#[derive(Debug, Clone, PartialEq)]
pub enum Guard {
    /// Passes when the two operands resolve to structurally equal nodes.
    Equal(Node, Node),
    /// Passes when the two operands resolve to structurally different nodes.
    NotEqual(Node, Node),
    /// Passes when every nested guard passes.
    All(Vec<Guard>),
}

/// A single ordered rewrite rule.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    /// Optional human-readable name used in diagnostics and traces.
    pub name: Option<String>,
    /// Structural match pattern.
    pub pattern: Node,
    /// Structural replacement template.
    pub template: Node,
    /// Optional guard evaluated against the bindings.
    pub guard: Option<Guard>,
    /// Higher priority rules are considered first; ties break on declaration
    /// order (requirement R10).
    pub priority: i64,
    /// Declaration order, assigned by the compiler. Lower runs first.
    pub order: usize,
    /// Optional cap on how many times this rule may apply in a run
    /// (requirement R12).
    pub max_applications: Option<u64>,
    /// When true, applying this rule stops the run in a terminal-completion
    /// state (requirement R11).
    pub terminal: bool,
}

impl Rule {
    /// A convenience constructor for tests and embedders.
    pub fn new(pattern: Node, template: Node) -> Self {
        Rule {
            name: None,
            pattern,
            template,
            guard: None,
            priority: 0,
            order: 0,
            max_applications: None,
            terminal: false,
        }
    }

    /// A stable label for diagnostics and traces.
    pub fn label(&self) -> String {
        match &self.name {
            Some(name) => name.clone(),
            None => format!("rule#{}", self.order + 1),
        }
    }
}

/// A compiled, immutable transformer program.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Declared program grammar version, if any (requirement R27).
    pub version: Option<u64>,
    /// Rules in declaration order.
    pub rules: Vec<Rule>,
}

impl Program {
    /// The program grammar version this build understands.
    pub const SUPPORTED_VERSION: u64 = 1;

    /// Returns rule indices ordered by scheduling priority: higher `priority`
    /// first, then declaration order. The returned indices point into
    /// [`Program::rules`].
    pub fn scheduling_order(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.rules.len()).collect();
        indices.sort_by(|&a, &b| {
            let ra = &self.rules[a];
            let rb = &self.rules[b];
            rb.priority
                .cmp(&ra.priority)
                .then_with(|| ra.order.cmp(&rb.order))
        });
        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_variables_and_wildcards() {
        assert_eq!(variable_name("$x"), Some("x"));
        assert_eq!(variable_name("$abc"), Some("abc"));
        assert_eq!(variable_name("$"), None);
        assert_eq!(variable_name("x"), None);
        assert!(is_wildcard("_"));
        assert!(!is_wildcard("_x"));
    }

    #[test]
    fn variable_name_is_unicode_safe() {
        // Requirement R26: non-ASCII references must not corrupt on byte
        // boundaries.
        assert_eq!(variable_name("$переменная"), Some("переменная"));
        assert_eq!(variable_name("$café"), Some("café"));
    }

    #[test]
    fn scheduling_orders_by_priority_then_declaration() {
        let mut low = Rule::new(LiNo::Ref("a".into()), LiNo::Ref("b".into()));
        low.order = 0;
        low.priority = 0;
        let mut high = Rule::new(LiNo::Ref("c".into()), LiNo::Ref("d".into()));
        high.order = 1;
        high.priority = 5;
        let mut mid = Rule::new(LiNo::Ref("e".into()), LiNo::Ref("f".into()));
        mid.order = 2;
        mid.priority = 5;
        let program = Program {
            version: Some(1),
            rules: vec![low, high, mid],
        };
        // high and mid share priority 5, so declaration order (1 then 2) breaks
        // the tie ahead of low (priority 0).
        assert_eq!(program.scheduling_order(), vec![1, 2, 0]);
    }
}
