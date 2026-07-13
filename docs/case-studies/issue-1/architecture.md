# Architecture and solution analysis

## Proposed rule model

A rule is a named, immutable value with:

- structural `match` pattern;
- structural `replace` template;
- optional guard/predicate;
- priority and declaration order;
- optional per-rule application limit;
- terminal flag; and
- metadata/source span for diagnostics.

The concrete LiNo spelling should be finalized with parser prototypes. This strawman
shows the information model, not frozen syntax:

```lino
(programVersion: 1)
(rule swap
  (match: ($x before $y))
  (replace: ($y before $x))
  (maximumApplications: 1))
```

Variables use a reserved namespace such as `$x`; `_` is an anonymous wildcard. A
compiler resolves variable declarations, validates replacement references, rejects
unknown metadata, and produces an efficient internal program. Reserved syntax must be
tested against ordinary LiNo references before adoption.

## Execution semantics

1. Parse LiNo program and input into identity-aware graphs.
2. Compile patterns and validate every rule.
3. Traverse candidate roots in a documented stable order.
4. Examine rules by priority then declaration order.
5. Match structurally and unify repeated variables.
6. Evaluate the optional guard.
7. Construct the replacement while preserving explicitly reused identities.
8. Emit a step event and restart rule selection according to strategy.
9. Stop at normal form, terminal rule, cancellation, error, repeated state, or limit.

Default strategy is `ordered-first`: one match per step, first eligible rule, stable
candidate order, then restart at the first rule. Later strategies (`innermost`,
`outermost`, `all-non-overlapping`) belong behind the scheduler interface and must not
silently change default results.

Deletion and sharing require precise graph semantics. Version 1 should define a
replacement as changing the selected root edge/subgraph while retaining reachable
bound nodes. Garbage collection is a storage concern: the in-memory result may omit
unreachable nodes during canonical serialization, while a persistent adapter must use
its explicit deletion policy.

## Component boundaries

| Component | Responsibility | Must not own |
|---|---|---|
| LiNo adapter | Parse/format and source mapping | Rule scheduling |
| Program compiler | Validate syntax, variables and metadata; compile patterns | Graph mutation |
| Matcher | Candidate search, structural unification, bindings | Global rule priority |
| Replacer | Instantiate template and produce graph edit | I/O or run loop |
| Scheduler | Strategy, limits, cancellation and stop reasons | Parsing |
| Event sink | Optional trace/log/metrics consumption | Semantic decisions |
| Storage adapter | Load, commit and transaction boundary | Pattern semantics |
| CLI | Files/stdin, flags, exit codes and rendering | Business logic |

Suggested library surface (names illustrative):

```text
compile_program(source) -> Program | Diagnostics
parse_graph(source) -> Graph | Diagnostics
Execution::new(program, graph, Options)
Execution::step() -> StepOutcome
Execution::run() -> RunResult
```

`RunResult` carries graph, stop reason, step count, last rule, diagnostics and optional
trace summary. Limits include maximum steps, applications per rule, deadline, maximum
nodes/links, maximum serialized output, and maximum retained trace bytes.

## Existing component evaluation

| Option | Strengths | Gaps | Recommendation |
|---|---|---|---|
| `links-notation` Rust crate | Maintained parser/formatter, ecosystem format, Unicode, Rust/WASM path | No rewriting semantics | Adopt as the default syntax adapter |
| `link-cli` query processor | Proven structural variables, nested/wildcard queries, stores and decorators | Coupled to CRUD/database query purpose; no general rule scheduler | Reuse concepts and, after API review, low-level restrictions/storage components |
| `grammar-expressions` | Named text captures, ordered bounded rules, Rust/JS parity | Operates on text, not shared/cyclic graphs | Optional scalar predicate and compatibility adapter |
| Port predecessor directly | Known API and behavior; Markov tests | Regex/text identity, old multi-language duplication | Port behavior via conformance tests, not implementation |
| Build matcher from scratch over LiNo graph | Correct semantic fit and controllable API | Highest initial engineering cost | Necessary for the small semantic kernel; keep it focused |
| General graph rewriting framework | Rich theory/tooling | Foreign data model, runtime weight, impedance mismatch | Use as design literature, not an initial dependency |

## Alternative product shapes

### A. Textual regex over serialized LiNo

Fastest demo, but formatting changes alter behavior, nested structures are fragile,
and shared/cyclic identity is unavailable. Reject as the core. It may be a documented
legacy adapter.

### B. Embed transformations in `link-cli`

Reuses mature storage and query machinery. It risks coupling a reusable transformer
to CLI/database concerns and makes stepwise pure evaluation harder. Keep optional
integration downstream after the library contract stabilizes.

### C. Structural rewrite library plus adapters (recommended)

Provides the smallest correct abstraction, deterministic tests, and later CLI,
storage, WASM, or service hosts. It costs a dedicated matcher but avoids semantic debt.

### D. Nondeterministic graph-rewrite exploration

Useful for search and theorem-oriented applications, but result explosion and unclear
compatibility make it unsuitable as the default. A future strategy can return a stream
of successor states under strict bounds.

## Turing-completeness claim

The unbounded ordered rewrite semantics should be capable of encoding a known
universal rewriting model (for example, a tag system or a small Turing machine). The
project should ship:

1. a written encoding from machine state/tape to LiNo links;
2. rules for transition and halt;
3. fixtures for several machines, including a non-halting case; and
4. a proof sketch that each machine transition corresponds to transformer steps.

Production defaults remain bounded. “Turing complete” describes expressive power,
not a promise to decide termination or a reason to omit safeguards.

## Security and operational model

Untrusted rule programs can cause catastrophic backtracking in optional textual
predicates, state explosion, infinite rewriting, oversized output, or expensive trace
retention. Safe mode therefore disables native/plugin predicates, uses bounded
matchers, enforces all resource limits, supports cancellation/deadlines, and returns a
partial result with a typed stop reason. Persistent adapters must stage a step and
commit atomically so cancellation cannot leave a half-applied graph edit.

Tracing is opt-in and event-driven. Events include execution start/stop, candidate
match, guard decision, applied rule, graph-size delta, and limit/cycle detection.
Sensitive scalar values should be redactable by the sink.

## Verification strategy

- Golden conformance fixtures for parse, compile, match, replace, step, run and errors.
- Compatibility fixtures derived from predecessor Markov tests, translated from text
  symbols to LiNo link sequences.
- Unit tests for identity sharing, cycles, repeated variables, wildcard depth,
  priority, terminal rules and each stop reason.
- Property tests: parse/format/parse equivalence, deterministic replay, unchanged
  graph when no rule applies, and references in results resolve.
- Fuzz parsers, compiler and matcher with strict budgets.
- Integration tests for CLI streams/files and optional storage atomicity.
- Benchmarks for flat/nested/shared/cyclic graphs, many rules, failed matches and
  output growth. Optimize only after profiles identify bottlenecks.
