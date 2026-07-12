//! # lino-transformer
//!
//! A rule-based, Turing-complete structural transformer for
//! [Links Notation](https://github.com/link-foundation/links-notation) (LiNo).
//!
//! The transformer rewrites link structures through ordered pattern/replacement
//! rules. Both the data and the rule programs are written in LiNo. Unlike its
//! text-and-regex predecessor
//! [`RegularExpressions.Transformer`](https://github.com/linksplatform/RegularExpressions.Transformer),
//! matching is structural and identity-aware rather than textual, so equivalent
//! formatting yields identical matches.
//!
//! ## Layers
//!
//! The crate keeps parsing, matching, scheduling, and presentation separable
//! (requirement R19):
//!
//! - [`compiler`] turns LiNo source into an immutable [`model::Program`].
//! - [`matcher`] performs structural matching with variable binding.
//! - [`replacer`] instantiates replacement templates and edits the graph.
//! - [`engine`] schedules rules under the ordered-first strategy with bounds and
//!   typed stop reasons.
//! - [`trace`] carries optional structured trace events.
//!
//! ## Quick start
//!
//! ```
//! use lino_transformer::{canonical, transform, Options};
//!
//! // A symmetric swap loops forever under ordered-first restart, so cap it to a
//! // single application to reach a normal form.
//! let program = "(rule: (maxApplications: 1) \
//!                (match: ($x before $y)) (replace: ($y before $x)))";
//! let result = transform(program, "(a before b)", Options::bounded()).unwrap();
//! assert_eq!(canonical(&result.graph), "(b before a)");
//! ```

pub mod compiler;
pub mod diagnostics;
pub mod engine;
pub mod format;
pub mod matcher;
pub mod model;
pub mod replacer;
pub mod trace;

pub use compiler::compile_program;
pub use diagnostics::{Diagnostic, Diagnostics, Location};
pub use engine::{Execution, Options, RunResult, StepOutcome, StopReason};
pub use format::canonical;
pub use model::{Guard, Node, Program, Rule};

// Re-export the upstream LiNo essentials so embedders do not need a direct
// dependency for common tasks (requirement R21).
pub use links_notation::{parse_lino, LiNo, ParseError};

/// Errors that can arise from the [`transform`] convenience entry point.
#[derive(Debug)]
pub enum TransformError {
    /// The program failed to compile.
    Compile(Diagnostics),
    /// The input data failed to parse as LiNo.
    Parse(ParseError),
}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformError::Compile(diagnostics) => {
                write!(f, "program did not compile:\n{}", diagnostics)
            }
            TransformError::Parse(error) => write!(f, "input did not parse: {}", error),
        }
    }
}

impl std::error::Error for TransformError {}

/// Compiles `program`, parses `input`, and runs to completion or to a bound.
///
/// This is the one-call convenience over [`compile_program`], [`parse_lino`],
/// and [`Execution`]. Library embedders that need stepping, cancellation, or
/// custom graphs should use those pieces directly.
pub fn transform(
    program: &str,
    input: &str,
    options: Options,
) -> Result<RunResult, TransformError> {
    let compiled = compile_program(program).map_err(TransformError::Compile)?;
    let graph = parse_lino(input).map_err(TransformError::Parse)?;
    Ok(Execution::new(compiled, graph, options).run())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_runs_end_to_end() {
        // A symmetric swap loops forever, so cap it to a single application to
        // reach a normal form (mirrors the case-study strawman).
        let program = "(rule: (name: swap) (maxApplications: 1) \
                       (match: ($x before $y)) (replace: ($y before $x)))";
        let result = transform(program, "(a before b)", Options::bounded()).unwrap();
        assert_eq!(canonical(&result.graph), "(b before a)");
        assert!(result.stop_reason.is_complete());
    }

    #[test]
    fn transform_surfaces_compile_errors() {
        let error = transform(
            "(rule: (match: ($x)) (replace: ($y)))",
            "(a)",
            Options::bounded(),
        )
        .unwrap_err();
        assert!(matches!(error, TransformError::Compile(_)));
    }
}
