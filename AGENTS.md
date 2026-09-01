# AGENTS.md

## Project

Minimal Scheme interpreter in Rust.
Workspace layout:

- `libsrs/` — core library
  - `src/types/` — `SrsValue` enum, errors, memory environment
  - `src/interpretor/` — lexer, parser (translator), evaluator
- `srs/` — CLI binary

## Build & Test

```bash
cargo build
cargo test
cargo run -- "(+ 2 3)"
```

## Data Model

Single enum `SrsValue` (see `libsrs/src/types/core.rs`). No trait objects.

```rust
pub enum SrsValue {
    Nil,
    Integer(i64),
    Float(f64),
    String(String),
    Id(String),
    Bool(bool),
    List(Vec<SrsValue>),
    Vector(Vec<SrsValue>),
}
```

Removed legacy files: `types/{id,integer,list,string}.rs`, `interpretor/error.rs`.

## Conventions

- Short, explicit names.
- `SrsResult<T>` for fallible operations.
- `Default` implemented for public structs when sensible.
- Tests live in the same file under `#[cfg(test)] mod tests`.
- Operator lexemes (e.g. `+`, `-`) are translated to `SrsValue::Id("+")` etc.

## Contributing

See `CONTRIBUTING.md` for branch and pull-request workflow.
Prefer `gh` over raw `git` for GitHub operations (PRs, issues, checks).

## Current Capabilities

- Lexer: integers, floats, identifiers, strings, booleans (`#t`/`#f`), parentheses, operators, comparison tokens.
- Parser: nested s-expressions into `SrsValue::List(...)`.
- Evaluator: literals, primitive arithmetic `+ - * /` with int/float coercion.

## Security Notes

Known issues that must be addressed before untrusted input:

- **Division by zero** can panic (integer `i64` division / `i64::MIN / -1`).
- **No recursion depth limit** in parser/evaluator → stack overflow on deeply nested input.
- **No input size limits** → OOM on huge expressions or huge token counts.
- **No timeout / instruction budget** → trivial denial of service.

When extending evaluator:

- Validate primitive argument counts and types.
- Add recursion/depth guards before calling recursively.
- Never expose host filesystem, network, or env vars to interpreted code without explicit sandbox.

## TODO

- Define/lambda/if/let bindings
- Comparison operators beyond lexing
- Quote / quasiquote
- Proper error locations (line/column)
- Resource limits and division-by-zero guards
