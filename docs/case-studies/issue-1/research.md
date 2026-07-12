# Research synthesis

## What the issue asks for

Issue 1 combines a product direction with a research assignment. The product is a
rule-based, Turing-complete transformer based on Links Notation, analogous to a
previous regular-expression transformer and potentially borrowing the query-language
style of `link-cli`. The required output at this stage is collected evidence, deep
analysis, an exhaustive requirement list, a survey of reusable components, solution
options, and plans under this case-study directory.

The request does not define a programming language, a final rule grammar, persistence,
or a UI. Those are design choices, not implied requirements. Treating them as explicit
would prematurely constrain the product.

## Predecessor feature map

| Predecessor behavior | Keep | Adapt for Links Notation | Defer/drop |
|---|---|---|---|
| Ordered substitution rules | Yes | Match link structures rather than regex text | — |
| One-step execution object | Yes | Return graph, bindings, rule and structured event | — |
| Repeated run to completion | Yes | Add stop reasons and whole-run safety limits | — |
| Per-rule maximum repeats | Yes | Count structural applications | — |
| Regex captures/replacements | Concept | Replace with variable unification/templates | Raw regex only as optional scalar guard |
| File path filtering | Host feature | CLI/storage selection, outside rule semantics | Defer from kernel |
| File transformation/logging | Host feature | CLI adapter and event sink | Defer until CLI milestone |
| C#, Python and Nim ports | Portability goal | Share a language-neutral corpus | Defer ports until semantics stabilize |
| Markov algorithm tests | Yes | Translate symbols/tape into LiNo sequences | — |

The predecessor's rule loop is simple but its exact advance/restart behavior must be
captured in fixtures before claiming compatibility. “Similar” should mean compatible
capabilities and mental model, not identical classes or textual regex semantics.

## `link-cli` lessons

`link-cli` provides stronger evidence for authoring and storage than for scheduling.
Its valuable concepts are:

- one substitution abstraction spanning create/read/update/delete;
- variables in link positions, wildcards, and nested patterns;
- names layered over numeric link identity;
- LiNo import/export and copyable change output;
- optional storage decorators for transactions and version control; and
- separate C# and Rust implementations checked for behavior parity.

The proposed transformer can compile a friendly `link-cli`-style rule dialect into a
smaller immutable program model. It should not make a CLI query processor the semantic
core or require persistent storage for pure graph transformations.

## Structural rewriting versus text rewriting

Two LiNo strings can differ in indentation, labels, or reference spelling while
representing equivalent structure. Conversely, identical-looking nested text does not
by itself expose whether nodes are shared or cyclic. Matching serialized text would
therefore make formatting semantically observable and lose graph identity.

Structural matching solves this by operating on parsed links and references. A binding
maps a pattern variable to an identity or subgraph, repeated variables impose equality,
and the replacement template explicitly reuses or constructs nodes. Canonical text is
an output view, not the execution state.

## Rewrite-strategy research

Term-rewriting systems commonly separate declarative rules from a strategy that
controls where and when they apply. The Stratego documentation, for example,
distinguishes rules from traversal strategies such as innermost application. That
separation is directly useful here: the matcher answers whether a rule applies, while
the scheduler owns ordered-first, traversal, repetition, and stopping.

Graph rewriting introduces sharing and cycles that tree rewriting can ignore. This
makes candidate ordering, identity reuse, overlapping matches, and reachability part of
the specification. The recommended first strategy applies only one match per step;
parallel/non-overlapping application can be added later without ambiguity.

## Computability and termination

Term/string rewriting can express universal computation, but universality does not
follow merely from calling an engine “rule based.” The chosen rule system must retain
unbounded state and repetition and must demonstrate a simulation of a known universal
model. At the same time, no general termination detector can make arbitrary universal
programs safe. Practical execution needs explicit resource bounds and cancellation;
simple repeated-state detection is only a useful diagnostic.

## Component conclusions

1. Reuse `links-notation` for syntax and serialization.
2. Reuse predecessor behavior as compatibility fixtures, not code architecture.
3. Reuse `link-cli` concepts and investigate its lower-level storage/restriction APIs
   only after the pure interfaces exist.
4. Use `grammar-expressions` only for optional scalar/text predicates or migration.
5. Implement the minimal identity-aware matcher/replacer because none of the surveyed
   components supplies that exact semantic kernel.
6. Keep storage, CLI, tracing, WASM, service, and UI as adapters around the kernel.

## Open questions and proposed defaults

| Question | Proposed default | Why it is reversible |
|---|---|---|
| Reference runtime | Rust | Semantics live in fixtures and interfaces; another runtime can follow |
| Match strategy | Ordered-first, stable traversal | Scheduler interface can add named strategies |
| Storage | In-memory graph | Persistent adapter can implement the same graph/edit contracts |
| Variable spelling | `$name`, `_` wildcard | Compiler can version concrete syntax |
| Safe execution | Bounded by default | Explicit unbounded option preserves theoretical expressiveness |
| Text matching | Optional guard | Does not contaminate structural equality or identity |

The requirement catalogue converts these findings into acceptance evidence, and the
roadmap schedules decisions before implementation so prototypes can invalidate syntax
assumptions cheaply.
