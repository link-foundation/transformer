# Sources and evidence log

Research was performed on 2026-07-12. Repository facts below were inspected from
source rather than inferred from project names. Commit identifiers make the study
reproducible even as upstream projects change.

| Source | Revision inspected | Evidence used |
|---|---:|---|
| [Issue 1](https://github.com/link-foundation/transformer/issues/1) | updated 2026-07-12; no comments at research time | Product description and explicit research deliverables |
| [Current repository](https://github.com/link-foundation/transformer) | `9f7ba146c0374af5e74477a83a5dc3466d1effe9` on `main` | README description; no implementation existed |
| [RegularExpressions.Transformer](https://github.com/linksplatform/RegularExpressions.Transformer) | `5654c8f08a2662e4b3f3badf30c760ca58f3f29a` | C#, Python and Nim implementations; ordered rules, stepping, repeat limits, path filters, CLI/file adapters, Markov tests |
| [link-cli](https://github.com/link-foundation/link-cli) | `ab2ce8be8e671c91e011d4f02eea10a19deea809` | LiNo query style, structural variables/wildcards, CRUD-as-substitution, imports/exports, storage decorators, Rust/C# parity |
| [links-notation](https://github.com/link-foundation/links-notation) | `237625e1d2fcdc073efdf50f87386b087d22b8f5` | Reference data model and maintained parsers for Rust, JavaScript, C#, Python, Go and Java |
| [grammar-expressions](https://github.com/link-foundation/grammar-expressions) | `dcbf98a171fb49d16bbfc32ab18f4f3db8520a69` | PEG/regex-style matching, named captures, ordered bounded rewrite engine, Rust/JavaScript APIs |
| [Stratego term rewriting documentation](https://www.metaborg.org/en/latest/source/langdev/meta/lang/stratego/strategoxt/04-term-rewriting.html) | accessed 2026-07-12 | Established separation between rewrite rules and traversal/control strategies |
| [Tools in Term Rewriting for Education](https://arxiv.org/abs/2002.12554) | arXiv:2002.12554 | Term rewriting as a computational model and the importance of analysis tools |
| [Term Graph Rewriting and Parallel Term Rewriting](https://arxiv.org/abs/1102.2651) | arXiv:1102.2651 | Graph sharing and parallel term-rewriting considerations |

## Source observations

### RegularExpressions.Transformer

- `TextTransformer` repeatedly calls `TextSteppedTransformer.Next()`.
- The stepped transformer maintains current text, rule list, and current rule index.
- Rules are ordered. A successful replacement can repeat according to the rule's
  maximum repeat count; execution then advances through the list.
- `SubstitutionRule` contains match, replacement, optional file-path pattern, and
  repeat count. File and logging adapters are outside the core transformer.
- The test suite includes Markov-algorithm examples, demonstrating that the intended
  model is more than a single regex replacement.

### link-cli

- Its public language treats create/read/update/delete as forms of substitution.
- Variables can bind link index, source, or target; nested patterns and wildcards are
  supported.
- It separates parsing/query processing from storage and layers names, transactions,
  and version control as decorators.
- It offers LiNo import/export, change reporting, and optional resource-bearing
  features rather than forcing them into every invocation.
- Its query syntax is valuable prior art, but its purpose is database manipulation;
  the transformer still needs an explicit rule program, scheduler, and execution
  result model.

### links-notation

- LiNo represents links containing references to other links, including doublets,
  triplets, N-tuples, nesting, labels, and indentation.
- Maintained packages exist in six languages. Reusing one avoids inventing a parser
  and makes conformance test sharing possible.
- Parsing and formatting are not graph rewriting. Identity resolution, variables,
  matching, replacement, and execution strategy remain transformer responsibilities.

### grammar-expressions

- It already supplies named captures, replacement, ordered rewrite rules, terminal
  rules, and a `maxSteps` guard in Rust and JavaScript.
- Its current subject is text. It can support scalar predicates or a compatibility
  adapter but cannot provide identity-aware matching of shared/cyclic link graphs by
  itself.

## Inferences and confidence

| Inference | Confidence | Reason |
|---|---|---|
| Ordered deterministic execution is the correct default | High | Shared by the predecessor and Markov-style rewriting; easiest to reproduce and test |
| Matching serialized LiNo with regex would be incorrect | High | Formatting aliases the same link structure and graph identity/cycles are not textual properties |
| Rust is the best first runtime | Medium | Strong current ecosystem fit and WASM path, but the issue does not mandate a language |
| Persistent storage belongs behind an adapter | High | Keeps the semantic kernel testable and follows `link-cli` layering |
| Turing completeness needs a demonstrable encoding | High | The repository description makes a capability claim that should be testable and precisely scoped |

No screenshots or other image attachments were present in the issue or PR at research
time. There were also no issue comments, PR conversation comments, inline review
comments, or submitted reviews to incorporate.
