# Issue 1 case study: a Links Notation transformer

Issue: [link-foundation/transformer#1](https://github.com/link-foundation/transformer/issues/1)

## Executive summary

The requested product is an ordered, rule-based graph/term rewriting engine whose
programs and data are expressed in Links Notation (LiNo). It should preserve the
useful execution model of
[`RegularExpressions.Transformer`](https://github.com/linksplatform/RegularExpressions.Transformer)
without treating serialized LiNo as an opaque string. Rules should match link
structures, bind references, construct replacement structures, and run under an
explicit strategy until a normal form, a terminal rule, or a resource limit is
reached.

The recommended first implementation is a Rust library and CLI. Rust already has
maintained LiNo and link-storage components in the ecosystem and can later target
WebAssembly. The public core should remain language-neutral: immutable program
model, matcher, substitution builder, deterministic scheduler, execution limits,
and structured trace events. A textual grammar matcher can be an optional predicate,
but structural link matching is the semantic foundation.

This pull request intentionally establishes the specification and delivery contract,
not a speculative implementation. The issue explicitly asks to collect evidence,
enumerate all requirements, analyze solutions and existing components, and propose
plans. The documents here make those outputs reviewable before an API or syntax is
made expensive to change.

## Case-study contents

- [research.md](research.md) records source evidence and lessons from predecessor
  systems and adjacent components.
- [requirements.md](requirements.md) is the complete, traceable requirement
  catalogue with acceptance criteria.
- [architecture.md](architecture.md) proposes semantics, data flow, APIs, safety,
  testing, and alternatives.
- [roadmap.md](roadmap.md) turns every requirement into staged work packages and
  exit criteria.
- [sources.md](sources.md) records source URLs, revisions, access dates, and the
  distinction between observed facts and design inferences.

## Recommended semantic kernel

```text
LiNo program ──parse──> rules + strategy
                              │
LiNo input ────parse──> link graph ──match──> bindings
                              ▲                  │
                              └──── replace ─────┘
                                      │
                         trace + result + stop reason
```

One step selects the first applicable rule under the configured traversal and
priority policy, applies one replacement, and restarts selection. This matches the
predecessor's observable ordered-step behavior while replacing regex capture groups
with bindings over references and links. Determinism is the default; alternative
strategies must be selected explicitly.

## Decisions required before milestone 1

1. Confirm Rust as the reference implementation language.
2. Approve the minimal rule dialect proposed in `architecture.md`, especially how
   variables and rule metadata are represented in LiNo.
3. Decide whether rewriting is purely in-memory in version 1 or also targets the
   persistent doublets store used by `link-cli`.
4. Define the exact claim meant by “Turing complete.” The roadmap requires a small
   machine encoding and conformance suite rather than relying on the label alone.

These are bounded product choices, not missing research. The proposed defaults allow
implementation to start immediately after review.
