# Delivery roadmap

This plan maps all requirements to concrete work. Each milestone is independently
reviewable and leaves the main branch usable.

## M0 — Specification and fixtures

Requirements: R2–R13, R17, R20, R23–R28, R33–R36.

1. Review and accept the rule information model and reserved variable syntax.
2. Define graph identity, match root, replacement, reachability and canonical output.
3. Define ordered-first execution and every stop reason/limit.
4. Create language-neutral YAML or LiNo conformance manifests whose payloads and
   programs remain LiNo; include source/expected graph and event sequences.
5. Translate the predecessor's ordering, repetition and Markov examples.
6. Add universal-machine design fixtures and proof outline.

Exit: grammar/semantics ADR accepted; invalid and valid fixture corpus reviewed; no
implementation question is hidden in an example.

## M1 — Pure semantic kernel

Requirements: R1–R14, R16–R21, R23–R30.

1. Scaffold Rust library and CI with formatter, linter, tests, dependency audit and
   license checks.
2. Integrate the upstream Rust LiNo parser behind an adapter.
3. Implement program compiler and source-span diagnostics.
4. Write matcher tests first, then implement literals, wildcard, nested/deep patterns,
   identity and repeated-variable unification.
5. Write replacement tests first, then implement bound-reference reuse and graph edits.
6. Implement ordered-first scheduler, `step`/`run`, terminal rules, limits,
   cancellation, cycle observation and structured outcomes.
7. Add optional event sink and deterministic canonical formatter.
8. Run fixture corpus, property tests, fuzz smoke jobs and baseline benchmarks.

Exit: all semantic conformance tests pass; library performs no filesystem/network I/O;
bounded infinite fixture stops correctly; API docs define invariants.

## M2 — CLI and compatibility

Requirements: R3–R5, R15, R17–R21, R23, R25–R28, R31–R32.

1. Add CLI integration tests before commands.
2. Implement `check`, `step`, and `run` commands with stdin/file inputs, canonical
   output, trace selection, limits and stable exit codes.
3. Add a `link-cli`-inspired compact query/rule authoring mode only if it compiles to
   the same immutable program model.
4. Add an optional `grammar-expressions` scalar predicate feature, disabled for
   untrusted safe mode.
5. Publish migration examples from predecessor regex rules and clarify cases requiring
   structural patterns.
6. Complete security, dependency, license and versioning documentation.

Exit: end-to-end tests cover success, malformed input, cancellation and every limit;
translated predecessor cases pass; packages are release-ready.

## M3 — Persistence adapter

Requirements: R19, R21–R23, R29–R31.

1. Specify storage transaction and node-identity contracts.
2. Prototype against the same doublets ecosystem used by `link-cli`; measure rather
   than assume reuse feasibility.
3. Run the semantic conformance suite against in-memory and persistent graphs.
4. Guarantee atomic application/cancellation and document garbage collection.
5. Benchmark load, match, edit and commit independently.

Exit: persistence changes no semantic result; crash/cancellation tests cannot expose a
partially applied step; performance data supports the dependency choice.

## M4 — Universality and additional hosts

Requirements: R5, R14, R18–R21, R23, R26–R32.

1. Complete and independently review the universal-model encoding and proof sketch.
2. Add WebAssembly bindings using the same conformance corpus.
3. Consider service/UI hosts only from measured user need; keep them adapters.
4. Evaluate additional execution strategies as separately versioned features.

Exit: universality claim is linked to executable fixtures; new hosts match native
results; alternative strategies cannot change ordered-first behavior.

## Requirement-to-work-package matrix

| Requirements | Primary work package |
|---|---|
| R1–R4 | M0 semantics, M1 compiler/matcher/replacer, M2 compatibility |
| R5 | M0 encoding design, M4 executable proof |
| R6–R13 | M0 fixtures, M1 semantic kernel |
| R14 | M1 library boundary |
| R15 | M2 CLI |
| R16–R18 | M1 execution and event model |
| R19–R22 | M1 adapters, M2 CLI, M3 storage |
| R23–R25 | M0 contracts, M1 limits/diagnostics, M2 hostile end-to-end tests |
| R26–R30 | M0 corpus, M1 quality suite, M3 storage benchmarks |
| R31–R32 | M1 CI foundations, M2 release/security docs |
| R33–R36 | This case study and its review |

## Suggested issue decomposition

Create one tracking issue per milestone and small implementation issues for compiler,
matcher, replacer, scheduler, formatter, trace sink, CLI and storage adapter. Every
implementation issue must cite requirement IDs, begin with a failing conformance test,
and state which stop/error behavior it introduces. Avoid parallel runtime ports until
M2 semantics are stable; the conformance corpus is the portability contract.

## Definition of done for the product

- Every R1–R32 acceptance item has automated or reviewable evidence.
- The universality claim has an executable encoding and bounded non-halting test.
- Local and CI formatting, lint, unit, property, fuzz-smoke, integration and license
  checks pass.
- Public docs match CLI help and library types.
- The complete diff removes no previously supported behavior without an explicit,
  documented migration decision.
