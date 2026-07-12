# Examples

Each `.lino` file is a transformer program written in Links Notation. Run one
with the CLI, feeding input on standard input:

```sh
echo "(a before b)" | cargo run -- run --program examples/swap.lino
```

> **Note.** Links Notation v0.13 has no comment syntax and treats a blank line
> as a document separator, so the programs below are kept compact and
> comment-free. The explanations live here instead.

## `swap.lino`

Reorders a binary `before` relation. Because a symmetric rewrite ping-pongs
forever under the ordered-first restart strategy, the rule is capped with
`(maxApplications: 1)` so the run reaches a normal form.

```sh
echo "(a before b)" | cargo run -- run --program examples/swap.lino
# => (b before a)
```

## `peano-add.lino`

Unary (Peano) addition. Numbers are `z` (zero) and `(s N)` (successor), so `2`
is `(s (s z))`. Two rules define `add`:

- `(add z $b)` → `$b`
- `(add (s $a) $b)` → `(s (add $a $b))`

```sh
echo "(add (s (s z)) (s z))" | cargo run -- run --program examples/peano-add.lino
# => (s (s (s z)))      i.e. 2 + 1 = 3
```

## `peano-mul.lino`

Multiplication built on top of the addition rules, showing rule composition:

- `(mul z $b)` → `z`
- `(mul (s $a) $b)` → `(add $b (mul $a $b))`

```sh
echo "(mul (s (s z)) (s (s (s z))))" | cargo run -- run --program examples/peano-mul.lino
# => (s (s (s (s (s (s z))))))     i.e. 2 * 3 = 6
```

## `list-reverse.lino`

Reverses a cons list `(c head tail)` with `e` as the empty list, using an
accumulator:

```sh
echo "(reverse (c a (c b (c c e))))" | cargo run -- run --program examples/list-reverse.lino
# => (c c (c b (c a e)))
```

## `turing-bb2.lino` — the universality demonstration

A full Turing machine simulated purely by structural link rewriting. The
configuration is `(tm left state head right)` where `left` and `right` are cons
lists (nearest cell first) and `head` is the current symbol. This program
encodes the **2-state busy beaver**, which halts after exactly six steps
leaving four `1`s on the tape.

```sh
echo "(tm e A 0 e)" | cargo run -- run --program examples/turing-bb2.lino --trace
# => (tm (c 1 (c 1 e)) H 1 (c 1 e))    (halts in 6 steps)
```

Each transition needs two rules — one for when the adjacent tape cell is the
empty list `e` (a fresh blank) and one for a non-empty cons cell — because
structural matching does not branch on "empty or non-empty" within a single
pattern. The machine halts by entering state `H`, for which no rule applies, so
the run ends in a normal form.

## `turing-runaway.lino` — bounded non-termination

A single-state machine that moves right forever, with no halting state. It
demonstrates that bounded mode stops a non-terminating program with a typed
reason and a non-zero exit code instead of looping indefinitely:

```sh
echo "(tm e R 0 e)" | cargo run -- run --program examples/turing-runaway.lino --max-steps 5
# => stopped: step limit reached after 5 steps   (exit code 3)
```
