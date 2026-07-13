# transformer

A rule-based, Turing-complete structural transformer built on
[Links Notation](https://github.com/link-foundation/links-notation) (LiNo).

Both the data and the rewrite rules are written in LiNo. The transformer applies
ordered pattern/replacement rules to link structures. Unlike its text-and-regex
predecessor
[`RegularExpressions.Transformer`](https://github.com/linksplatform/RegularExpressions.Transformer),
matching is **structural and identity-aware** rather than textual, so equivalent
formatting yields identical matches.

## Quick start

Build the crate and run one of the shipped example programs, feeding input on
standard input:

```sh
echo "(a before b)" | cargo run -- run --program examples/swap.lino
# => (b before a)
```

Compute `2 + 1` with unary (Peano) arithmetic:

```sh
echo "(add (s (s z)) (s z))" | cargo run -- run --program examples/peano-add.lino
# => (s (s (s z)))
```

Simulate a 2-state busy-beaver Turing machine — the universality demonstration —
which halts after exactly six steps leaving four `1`s on the tape:

```sh
echo "(tm e A 0 e)" | cargo run -- run --program examples/turing-bb2.lino --trace
# => (tm (c 1 (c 1 e)) H 1 (c 1 e))
```

See [`examples/README.md`](examples/README.md) for the full walkthrough.

## Writing a program

A program is a LiNo document of ordered rules. Each rule has a `match` pattern
and a `replace` template; variables are references beginning with `$`, and `_`
is an anonymous wildcard.

```lino
(programVersion: 1)
(rule:
  (name: swap)
  (maxApplications: 1)
  (match: ($x before $y))
  (replace: ($y before $x)))
```

Supported rule attributes: `name`, `match`, `replace`, `guard`
(`equal` / `notEqual` / `all`), `priority`, `maxApplications`, and `terminal`.

Execution follows the **ordered-first** strategy: each step considers rules by
priority then declaration order, applies exactly one replacement at the first
matching node in a stable pre-order traversal, then restarts rule selection.
This reproduces the classic Markov-style ordered rewriting model of the
predecessor while operating on link structure instead of text. Because term
rewriting with structural matching, variables, and nesting is Turing complete,
so is this transformer — the busy-beaver example is a concrete witness.

Since termination is undecidable, bounded mode (the default) caps steps, node
count, and wall-clock time, and reports a typed [`StopReason`]. Use
`--unbounded` only for trusted programs known to terminate.

## Command-line interface

```
lino-transformer <COMMAND> --program <FILE> [OPTIONS]

COMMANDS:
    check    Compile a program and report diagnostics
    run      Apply a program to input and print the resulting graph
    step     Apply a single rewrite step and print the resulting graph

OPTIONS:
    -p, --program <FILE>   Program file written in Links Notation (required)
    -i, --input <FILE>     Input data file (default: standard input)
        --max-steps <N>    Step limit for run (default: 100000)
        --max-nodes <N>    Node-count limit for run
        --unbounded        Remove the step limit (trusted programs only)
        --detect-cycles    Stop run when a graph state repeats
        --trace            Print structured trace events to stderr

EXIT CODES:
    0  success            2  usage/compile/parse error            3  stopped at a bound
```

## Library

The crate also exposes a library (`lino_transformer`) with a layered API —
`compiler`, `matcher`, `replacer`, `engine`, `trace`, and `format` — plus a
one-call [`transform`] convenience:

```rust
use lino_transformer::{canonical, transform, Options};

let program = "(rule: (maxApplications: 1) \
               (match: ($x before $y)) (replace: ($y before $x)))";
let result = transform(program, "(a before b)", Options::bounded()).unwrap();
assert_eq!(canonical(&result.graph), "(b before a)");
```

## Design

The evidence, complete requirement catalogue, architecture, alternatives, and
delivery plan are captured in the
[issue 1 case study](docs/case-studies/issue-1/README.md).

## License

Released into the public domain under the [Unlicense](LICENSE).
