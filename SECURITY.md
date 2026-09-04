# Security

## Status

`s_rs` is an experimental Scheme interpreter. It is **not hardened** and must not be
used to evaluate untrusted input.

## Known limitations

- **Integer overflow** — `+`, `-` and `*` use plain `i64` operations, as does
  `i64::MIN / -1`. These panic in debug builds and wrap silently in release builds.
- **No recursion depth limit** — the translator and the evaluator recurse on nested
  s-expressions, so deeply nested input overflows the stack.
- **No input size limit** — arbitrarily large expressions or token counts can exhaust
  memory.
- **No timeout or instruction budget** — a long-running evaluation cannot be interrupted.
- **No error locations** — `SrsError` carries a kind and a message, but no line/column,
  which makes malformed input hard to diagnose.

Division by zero *is* guarded: `Evaluator::div` rejects a zero divisor before folding.

## Rules when extending the interpreter

- Validate primitive argument counts and types explicitly; never `unwrap`/`panic!` in
  library code, return `SrsResult<T>`.
- Add a recursion or depth guard before introducing new recursive descent.
- Prefer checked arithmetic (`checked_add`, `checked_div`, ...) over raw operators when
  touching numeric primitives.
- Never expose the host filesystem, network or environment variables to interpreted code
  without an explicit sandbox.

## Reporting a vulnerability

Open a GitHub issue on this repository. Since the project is experimental and has no
release channel, there is no private disclosure process.
