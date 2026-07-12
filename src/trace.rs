//! Optional structured trace events (requirement R18).
//!
//! Tracing is disabled by default; when enabled the engine records one event
//! per applied rewrite. Events are plain data so any sink (logging, metrics,
//! tests) can consume them without the engine making presentation decisions.

/// A single structured trace event describing one applied rewrite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvent {
    /// 1-based step number.
    pub step: u64,
    /// The label of the rule that was applied.
    pub rule: String,
    /// The path (value indices) to the rewritten node in the pre-rewrite graph.
    pub path: Vec<usize>,
    /// Number of nodes in the graph before this rewrite.
    pub nodes_before: usize,
    /// Number of nodes in the graph after this rewrite.
    pub nodes_after: usize,
    /// Whether the applied rule was terminal.
    pub terminal: bool,
}

impl std::fmt::Display for TraceEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = if self.path.is_empty() {
            "root".to_string()
        } else {
            self.path
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(".")
        };
        write!(
            f,
            "step {}: applied {} at {} ({} -> {} nodes){}",
            self.step,
            self.rule,
            path,
            self.nodes_before,
            self.nodes_after,
            if self.terminal { " [terminal]" } else { "" }
        )
    }
}
