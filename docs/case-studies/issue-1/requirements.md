# Requirements catalogue

The identifiers below are stable so implementation issues and tests can cite them.
“Explicit” means stated by issue 1 or the repository description. “Derived” means
necessary to make an explicit requirement testable, safe, or compatible with the
named predecessor.

## Product and language requirements

| ID | Requirement | Origin | Acceptance evidence |
|---|---|---|---|
| R1 | Transform link structures through ordered pattern/replacement rules | Explicit | A program rewrites a parsed LiNo input and returns the expected LiNo graph |
| R2 | Use Links Notation for input data, patterns, replacements, and serializable programs | Explicit | Parser/formatter round trips fixtures and rule files contain no required JSON sidecar |
| R3 | Preserve the predecessor's essential ordered and stepped execution model | Explicit | Compatibility tests cover priority, restart/advance behavior, repeat limits, and terminal rules |
| R4 | Provide substitution/query syntax informed by `link-cli` | Explicit | Variables, nested structures and wildcard matching have documented LiNo examples |
| R5 | Be computationally universal under documented unbounded semantics | Explicit | A documented encoding simulates a known universal model; bounded mode reports limit exhaustion |
| R6 | Match structure and reference identity, not only serialized text | Derived | Equivalent formatting yields identical matches; shared and cyclic structures have tests |
| R7 | Bind variables consistently across repeated occurrences | Derived | A repeated variable rejects unequal references and is available to replacement construction |
| R8 | Support literal, wildcard and nested/deep structural patterns | Derived from `link-cli` | Positive and negative conformance fixtures cover each pattern kind |
| R9 | Build replacements from literals, matched subgraphs and bound references | Derived | Tests cover reuse, insertion, deletion and replacement of substructures |
| R10 | Define rule priority, match order and traversal strategy | Derived | The same program produces byte-equivalent normalized output across repeated runs and runtimes |
| R11 | Support terminal rules and explicit normal-form completion | Predecessor-derived | Result distinguishes terminal-rule completion from no-applicable-rule completion |
| R12 | Support per-rule and whole-run repetition/step limits | Predecessor-derived/safety | Infinite-cycle fixture stops with a typed limit result and partial graph |
| R13 | Support predicates/guards without making regex the graph matcher | Derived | A scalar predicate can accept/reject a structural match through an extension interface |

## API, operations, and observability

| ID | Requirement | Origin | Acceptance evidence |
|---|---|---|---|
| R14 | Expose a reusable library independent of CLI and storage | Derived from predecessor | Unit tests invoke parse/compile/step/run without process or filesystem access |
| R15 | Expose a CLI for applying a program to LiNo input | Predecessor/link-cli-derived | End-to-end tests cover stdin/files, stdout, diagnostics and exit codes |
| R16 | Provide `step`, `run`, reset/resume state and cancellation | Predecessor-derived | A multi-step fixture can pause, inspect, resume and reach the same final result as `run` |
| R17 | Return structured stop reasons, step count, applied rule and diagnostics | Derived | Public result type distinguishes normal form, terminal, cancelled, limit and error |
| R18 | Emit optional structured trace events with tracing disabled by default | Derived/safety | Enabling trace records match/rewrite events; default run allocates no retained trace |
| R19 | Keep parsing, matching, scheduling, storage and presentation separable | Derived | Architecture/API tests use in-memory graph and mock event sink |
| R20 | Support canonical output plus preservation-oriented formatting where feasible | Derived from LiNo | Canonical formatter is deterministic; limitations of trivia preservation are documented |
| R21 | Offer import/export interoperability with maintained LiNo packages | Ecosystem-derived | Shared fixture corpus parses and formats through the chosen upstream package |
| R22 | Make persistent link storage optional behind an adapter | `link-cli`-derived | Same conformance suite runs against memory; persistence contract tests run separately |

## Quality and operational requirements

| ID | Requirement | Origin | Acceptance evidence |
|---|---|---|---|
| R23 | Prevent accidental runaway CPU, memory and output growth in bounded mode | Derived from Turing completeness | Configurable steps, deadline, graph-size and trace-size limits each have tests |
| R24 | Detect or surface trivial cycles without promising general termination | Derived | Repeated-state option reports a cycle; docs state that termination is undecidable in general |
| R25 | Produce precise parse/compile/runtime diagnostics with source spans | Derived | Invalid-program golden tests assert rule name, location and actionable message |
| R26 | Be Unicode-safe | LiNo/grammar ecosystem-derived | Non-ASCII reference and scalar fixtures pass without byte-boundary corruption |
| R27 | Version the program grammar and public API | Derived | Program declares/infers a supported version; unsupported versions fail clearly |
| R28 | Maintain a language-neutral conformance corpus | Derived | Fixtures specify program, input, expected events/result/output and run in every implementation |
| R29 | Test determinism, property invariants, malformed input and adversarial limits | Derived | Unit, property/fuzz and integration suites run in CI |
| R30 | Benchmark matcher and rewrite workloads before optimization | Derived | Reproducible benchmarks cover flat, nested, shared, cyclic and no-match graphs |
| R31 | Publish security guidance for untrusted programs | Derived | Threat model documents denial-of-service, predicate/plugin trust and safe defaults |
| R32 | Use the repository's Unlicense-compatible conventions and document dependencies | Ecosystem-derived | License scan and dependency inventory are part of release checks |

## Documentation and research deliverables

| ID | Requirement | Origin | Acceptance evidence |
|---|---|---|---|
| R33 | Compile issue-related data under `docs/case-studies/issue-1` | Explicit | This directory contains the evidence log and analysis |
| R34 | Research online facts and existing components | Explicit | `sources.md` records upstream revisions and external references |
| R35 | Enumerate every issue requirement | Explicit | This catalogue maps explicit and necessary derived requirements |
| R36 | Propose solution options and a plan for every requirement | Explicit | `architecture.md` compares options; `roadmap.md` maps every ID to work packages |

## Non-goals for the first implementation milestone

- A new LiNo parser, database engine, or general regex engine.
- Implicit nondeterministic/parallel execution.
- A claim that all programs terminate.
- Network service, browser UI, transactions, and version control in the semantic
  kernel. They remain compatible future adapters.
- Exact source-whitespace preservation in the canonical graph formatter.
