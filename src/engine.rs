//! The ordered-first execution engine: matching, replacement, scheduling,
//! limits, and stop reasons.
//!
//! Execution follows the ordered-first strategy documented in the case study
//! (requirements R3 and R10): each step considers rules by priority then
//! declaration order, applies exactly one replacement at the first matching
//! node found in a stable pre-order traversal, and then restarts rule
//! selection. This reproduces the classic Markov-style ordered rewriting model
//! of the predecessor while operating on link structure instead of text.

use crate::matcher::{evaluate_guard, match_node, Bindings};
use crate::model::{Node, Program, Rule};
use crate::replacer::{instantiate, replace_at};
use crate::trace::TraceEvent;
use links_notation::LiNo;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Bounds and toggles that keep bounded-mode execution safe (requirements R12,
/// R23, R24). Defaults are conservative so an untrusted program cannot run away
/// with CPU, memory, or output growth.
#[derive(Clone, Default)]
pub struct Options {
    /// Maximum number of rewrite steps. `None` means unbounded (use only for
    /// trusted programs); the default is a finite bound.
    pub max_steps: Option<u64>,
    /// Maximum number of nodes permitted in the working graph. Exceeding it
    /// stops the run with [`StopReason::NodeLimit`].
    pub max_nodes: Option<usize>,
    /// Wall-clock deadline. Reaching it stops the run with
    /// [`StopReason::Deadline`].
    pub deadline: Option<Instant>,
    /// When true, a repeated graph state stops the run with
    /// [`StopReason::CycleDetected`] (requirement R24).
    pub detect_cycles: bool,
    /// When true, record a [`TraceEvent`] per applied rewrite (requirement
    /// R18).
    pub trace: bool,
    /// Optional cooperative cancellation flag (requirement R16).
    pub cancel: Option<Arc<AtomicBool>>,
}

impl Options {
    /// The default bounded-mode step limit.
    pub const DEFAULT_MAX_STEPS: u64 = 100_000;

    /// Bounded defaults suitable for untrusted programs.
    pub fn bounded() -> Self {
        Options {
            max_steps: Some(Self::DEFAULT_MAX_STEPS),
            ..Options::default()
        }
    }

    /// Unbounded execution. Only appropriate for trusted programs that are
    /// known to terminate; termination is undecidable in general (requirement
    /// R24).
    pub fn unbounded() -> Self {
        Options {
            max_steps: None,
            ..Options::default()
        }
    }
}

/// Why a run stopped. The variants distinguish the successful terminal states
/// from the bounded-mode safety stops (requirement R17).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopReason {
    /// No rule applied; the graph is in normal form.
    NormalForm,
    /// A terminal rule applied and halted the run (requirement R11).
    TerminalRule,
    /// The step limit was reached (requirement R12/R23).
    StepLimit,
    /// The node-count limit was reached (requirement R23).
    NodeLimit,
    /// The wall-clock deadline was reached (requirement R23).
    Deadline,
    /// A previously seen graph state recurred (requirement R24).
    CycleDetected,
    /// Execution was cancelled cooperatively (requirement R16).
    Cancelled,
}

impl StopReason {
    /// Whether this stop reason represents natural completion rather than a
    /// bound or cancellation.
    pub fn is_complete(&self) -> bool {
        matches!(self, StopReason::NormalForm | StopReason::TerminalRule)
    }
}

impl std::fmt::Display for StopReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            StopReason::NormalForm => "normal form (no rule applies)",
            StopReason::TerminalRule => "terminal rule applied",
            StopReason::StepLimit => "step limit reached",
            StopReason::NodeLimit => "node limit reached",
            StopReason::Deadline => "deadline reached",
            StopReason::CycleDetected => "cycle detected",
            StopReason::Cancelled => "cancelled",
        };
        f.write_str(text)
    }
}

/// The outcome of a single [`Execution::step`] call.
#[derive(Debug, Clone, PartialEq)]
pub enum StepOutcome {
    /// A rule applied. Carries the applied rule index and whether it was
    /// terminal.
    Applied {
        rule_index: usize,
        rule_label: String,
        path: Vec<usize>,
        terminal: bool,
    },
    /// No rule applied; the graph is in normal form.
    NormalForm,
}

/// The result of running to completion or to a bound.
#[derive(Debug, Clone, PartialEq)]
pub struct RunResult {
    /// The final graph.
    pub graph: Node,
    /// Why the run stopped.
    pub stop_reason: StopReason,
    /// Total number of applied steps.
    pub steps: u64,
    /// The label of the last applied rule, if any.
    pub last_rule: Option<String>,
    /// Recorded trace events (empty unless tracing was enabled).
    pub trace: Vec<TraceEvent>,
}

/// A resettable, resumable execution over a program and a working graph
/// (requirement R16).
pub struct Execution {
    program: Program,
    graph: Node,
    options: Options,
    scheduling: Vec<usize>,
    applications: Vec<u64>,
    steps: u64,
    last_rule: Option<String>,
    trace: Vec<TraceEvent>,
    seen_states: HashSet<String>,
}

impl Execution {
    /// Creates a new execution over `graph` using `program` and `options`.
    pub fn new(program: Program, graph: Node, options: Options) -> Self {
        let scheduling = program.scheduling_order();
        let applications = vec![0; program.rules.len()];
        Execution {
            program,
            graph,
            options,
            scheduling,
            applications,
            steps: 0,
            last_rule: None,
            trace: Vec::new(),
            seen_states: HashSet::new(),
        }
    }

    /// The current working graph.
    pub fn graph(&self) -> &Node {
        &self.graph
    }

    /// The number of steps applied so far.
    pub fn steps(&self) -> u64 {
        self.steps
    }

    /// Resets the working graph and all run state, keeping the program and
    /// options (requirement R16).
    pub fn reset(&mut self, graph: Node) {
        self.graph = graph;
        self.applications.iter_mut().for_each(|c| *c = 0);
        self.steps = 0;
        self.last_rule = None;
        self.trace.clear();
        self.seen_states.clear();
    }

    /// Whether a rule is still eligible given its per-rule application cap.
    fn eligible(&self, rule_index: usize) -> bool {
        match self.program.rules[rule_index].max_applications {
            Some(limit) => self.applications[rule_index] < limit,
            None => true,
        }
    }

    /// Applies a single step under the ordered-first strategy.
    pub fn step(&mut self) -> StepOutcome {
        for &rule_index in &self.scheduling {
            if !self.eligible(rule_index) {
                continue;
            }
            let rule = &self.program.rules[rule_index];
            if let Some((path, bindings)) = find_match(&self.graph, rule) {
                let replacement =
                    instantiate(&rule.template, &bindings).expect("compiler validated template");
                let nodes_before = count_nodes(&self.graph);
                self.graph = replace_at(&self.graph, &path, replacement);
                self.steps += 1;
                self.applications[rule_index] += 1;
                let label = rule.label();
                self.last_rule = Some(label.clone());
                if self.options.trace {
                    self.trace.push(TraceEvent {
                        step: self.steps,
                        rule: label.clone(),
                        path: path.clone(),
                        nodes_before,
                        nodes_after: count_nodes(&self.graph),
                        terminal: rule.terminal,
                    });
                }
                return StepOutcome::Applied {
                    rule_index,
                    rule_label: label,
                    path,
                    terminal: rule.terminal,
                };
            }
        }
        StepOutcome::NormalForm
    }

    /// Runs until the graph reaches a terminal state or a bound is hit.
    pub fn run(&mut self) -> RunResult {
        let stop_reason = loop {
            // Cooperative cancellation is checked first so a caller can stop a
            // long run promptly.
            if let Some(flag) = &self.options.cancel {
                if flag.load(Ordering::Relaxed) {
                    break StopReason::Cancelled;
                }
            }
            if let Some(max) = self.options.max_steps {
                if self.steps >= max {
                    break StopReason::StepLimit;
                }
            }
            if let Some(max) = self.options.max_nodes {
                if count_nodes(&self.graph) > max {
                    break StopReason::NodeLimit;
                }
            }
            if let Some(deadline) = self.options.deadline {
                if Instant::now() >= deadline {
                    break StopReason::Deadline;
                }
            }
            if self.options.detect_cycles {
                let canonical = self.graph.to_string();
                if !self.seen_states.insert(canonical) {
                    break StopReason::CycleDetected;
                }
            }
            match self.step() {
                StepOutcome::Applied { terminal, .. } => {
                    if terminal {
                        break StopReason::TerminalRule;
                    }
                }
                StepOutcome::NormalForm => break StopReason::NormalForm,
            }
        };
        RunResult {
            graph: self.graph.clone(),
            stop_reason,
            steps: self.steps,
            last_rule: self.last_rule.clone(),
            trace: self.trace.clone(),
        }
    }
}

/// Counts the total number of nodes (references and links) in a graph.
pub fn count_nodes(node: &Node) -> usize {
    match node {
        LiNo::Ref(_) => 1,
        LiNo::Link { values, .. } => 1 + values.iter().map(count_nodes).sum::<usize>(),
    }
}

/// Finds the first node (in pre-order) where `rule` matches and its guard, if
/// any, passes. Returns the path to that node and the resulting bindings.
fn find_match(root: &Node, rule: &Rule) -> Option<(Vec<usize>, Bindings)> {
    let mut path = Vec::new();
    find_match_inner(root, rule, &mut path)
}

fn find_match_inner(
    node: &Node,
    rule: &Rule,
    path: &mut Vec<usize>,
) -> Option<(Vec<usize>, Bindings)> {
    let mut bindings = Bindings::new();
    if match_node(&rule.pattern, node, &mut bindings) {
        let guard_ok = rule
            .guard
            .as_ref()
            .map(|g| evaluate_guard(g, &bindings))
            .unwrap_or(true);
        if guard_ok {
            return Some((path.clone(), bindings));
        }
    }
    if let LiNo::Link { values, .. } = node {
        for (index, child) in values.iter().enumerate() {
            path.push(index);
            if let Some(found) = find_match_inner(child, rule, path) {
                return Some(found);
            }
            path.pop();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compile_program;
    use crate::format::canonical;

    fn run_program(source: &str, input: &str, options: Options) -> RunResult {
        let program = compile_program(source).expect("program compiles");
        let graph = links_notation::parse_lino(input).expect("valid input");
        Execution::new(program, graph, options).run()
    }

    #[test]
    fn applies_a_single_swap_rule() {
        // A symmetric swap loops forever, so cap it at one application to reach
        // a normal form (requirement R12).
        let program = "(rule: (name: swap) (maxApplications: 1) \
                       (match: ($x before $y)) (replace: ($y before $x)))";
        let result = run_program(program, "(a before b)", Options::bounded());
        assert_eq!(canonical(&result.graph), "(b before a)");
        assert_eq!(result.stop_reason, StopReason::NormalForm);
        assert_eq!(result.steps, 1);
        assert_eq!(result.last_rule.as_deref(), Some("swap"));
    }

    #[test]
    fn symmetric_rule_without_cap_loops_until_step_limit() {
        // Documents that ordered-first restart makes a symmetric rule
        // non-terminating; bounded mode stops it (requirements R10, R23).
        let program = "(rule: (match: ($x before $y)) (replace: ($y before $x)))";
        let options = Options {
            max_steps: Some(10),
            ..Options::default()
        };
        let result = run_program(program, "(a before b)", options);
        assert_eq!(result.stop_reason, StopReason::StepLimit);
        assert_eq!(result.steps, 10);
    }

    #[test]
    fn terminal_rule_reports_terminal_completion() {
        // Two rules: a non-terminal that rewrites a->b, and a terminal that
        // matches b. Requirement R11: distinguish terminal completion.
        let program = "\
            (rule: (name: grow) (match: (seed)) (replace: (sprout)))\n\
            (rule: (name: done) (terminal: true) (match: (sprout)) (replace: (flower)))";
        let result = run_program(program, "(seed)", Options::bounded());
        // In LiNo, a lone parenthesized atom `(flower)` is the reference
        // `flower`; the canonical form drops the collapsing parentheses.
        assert_eq!(canonical(&result.graph), "flower");
        assert_eq!(result.stop_reason, StopReason::TerminalRule);
        assert_eq!(result.last_rule.as_deref(), Some("done"));
    }

    #[test]
    fn step_limit_stops_infinite_program() {
        // Requirement R12/R23: an infinite rewrite stops with a typed limit and
        // returns the partial graph.
        let program = "(rule: (name: grow) (match: (n $x)) (replace: (n (n $x))))";
        let options = Options {
            max_steps: Some(5),
            ..Options::default()
        };
        let result = run_program(program, "(n z)", options);
        assert_eq!(result.stop_reason, StopReason::StepLimit);
        assert_eq!(result.steps, 5);
    }

    #[test]
    fn per_rule_application_cap_limits_a_rule() {
        // maxApplications caps a single rule (requirement R12). After the cap
        // the rule is skipped and the run reaches normal form.
        let program = "(rule: (name: once) (maxApplications: 1) (match: (a $x)) (replace: (b $x)))";
        let result = run_program(program, "(a (a z))", Options::bounded());
        // Only the outermost application fires; the inner (a z) survives.
        assert_eq!(result.steps, 1);
        assert_eq!(result.stop_reason, StopReason::NormalForm);
        assert_eq!(canonical(&result.graph), "(b (a z))");
    }

    #[test]
    fn cycle_detection_reports_repeated_state() {
        // A ping-pong program never terminates but repeats a state; requirement
        // R24 surfaces the cycle instead of promising termination.
        let program = "\
            (rule: (name: flip) (match: (ping)) (replace: (pong)))\n\
            (rule: (name: flop) (match: (pong)) (replace: (ping)))";
        let options = Options {
            max_steps: Some(1000),
            detect_cycles: true,
            ..Options::default()
        };
        let result = run_program(program, "(ping)", options);
        assert_eq!(result.stop_reason, StopReason::CycleDetected);
    }

    #[test]
    fn no_applicable_rule_leaves_graph_unchanged() {
        let program = "(rule: (match: (never)) (replace: (matches)))";
        let result = run_program(program, "(something else)", Options::bounded());
        assert_eq!(canonical(&result.graph), "(something else)");
        assert_eq!(result.stop_reason, StopReason::NormalForm);
        assert_eq!(result.steps, 0);
    }

    #[test]
    fn priority_overrides_declaration_order() {
        // The later-declared rule has higher priority and must win.
        let program = "\
            (rule: (name: low) (match: (x)) (replace: (low_ran)))\n\
            (rule: (name: high) (priority: 10) (match: (x)) (replace: (high_ran)))";
        let result = run_program(program, "(x)", Options::bounded());
        assert_eq!(canonical(&result.graph), "high_ran");
        assert_eq!(result.last_rule.as_deref(), Some("high"));
    }

    #[test]
    fn guard_can_reject_a_structural_match() {
        // Requirement R13: a guard rejects a match that structurally fits.
        let program = "\
            (rule: (name: distinct) (match: (pair $x $y)) (guard: (notEqual $x $y)) (replace: (ok)))";
        // Equal operands: guard fails, no rule applies.
        let equal = run_program(program, "(pair a a)", Options::bounded());
        assert_eq!(canonical(&equal.graph), "(pair a a)");
        assert_eq!(equal.steps, 0);
        // Distinct operands: guard passes.
        let distinct = run_program(program, "(pair a b)", Options::bounded());
        assert_eq!(canonical(&distinct.graph), "ok");
    }

    #[test]
    fn step_and_run_reach_the_same_result() {
        // Requirement R16: stepping manually reaches the same final graph as a
        // single run().
        let program_source =
            "(rule: (name: grow) (maxApplications: 3) (match: (n $x)) (replace: (n (n $x))))";
        let program = compile_program(program_source).expect("compiles");
        let input = links_notation::parse_lino("(n z)").unwrap();

        let mut stepper = Execution::new(program.clone(), input.clone(), Options::bounded());
        loop {
            match stepper.step() {
                StepOutcome::Applied { terminal, .. } if terminal => break,
                StepOutcome::Applied { .. } => continue,
                StepOutcome::NormalForm => break,
            }
        }

        let mut runner = Execution::new(program, input, Options::bounded());
        let run_result = runner.run();
        assert_eq!(canonical(stepper.graph()), canonical(&run_result.graph));
    }

    #[test]
    fn cancellation_stops_a_run() {
        // Requirement R16: a cancelled run returns promptly with Cancelled.
        let program = compile_program("(rule: (match: (n $x)) (replace: (n (n $x))))").unwrap();
        let graph = links_notation::parse_lino("(n z)").unwrap();
        let flag = Arc::new(AtomicBool::new(true));
        let options = Options {
            cancel: Some(flag),
            ..Options::bounded()
        };
        let result = Execution::new(program, graph, options).run();
        assert_eq!(result.stop_reason, StopReason::Cancelled);
        assert_eq!(result.steps, 0);
    }
}
