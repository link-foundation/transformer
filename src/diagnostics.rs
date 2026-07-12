//! Structured diagnostics produced while compiling a transformer program.
//!
//! Requirement R25 asks for precise compile/runtime diagnostics. The upstream
//! `links-notation` parser does not currently expose byte-level source spans, so
//! a diagnostic locates a problem by the offending rule (name and declaration
//! index) and, where useful, the serialized form of the sub-link that failed to
//! validate. This is documented as a known limitation in the case study.

use std::fmt;

/// Where a diagnostic originates within a program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    /// The problem concerns the program as a whole (for example, no rules).
    Program,
    /// The problem concerns a specific top-level element by index.
    TopLevel { index: usize },
    /// The problem concerns a rule identified by declaration order and name.
    Rule { index: usize, name: Option<String> },
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Program => write!(f, "program"),
            Location::TopLevel { index } => write!(f, "top-level element #{}", index + 1),
            Location::Rule { index, name } => match name {
                Some(name) => write!(f, "rule #{} ('{}')", index + 1, name),
                None => write!(f, "rule #{}", index + 1),
            },
        }
    }
}

/// A single compile-time diagnostic with an actionable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub location: Location,
    pub message: String,
}

impl Diagnostic {
    pub fn new(location: Location, message: impl Into<String>) -> Self {
        Diagnostic {
            location,
            message: message.into(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.location, self.message)
    }
}

/// A collection of diagnostics returned when a program fails to compile.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Diagnostics {
    pub items: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics { items: Vec::new() }
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.items.push(diagnostic);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", item)?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}
