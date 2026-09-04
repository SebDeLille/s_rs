# AGENTS.md

Minimal Scheme (R5RS subset) interpreter in Rust. Cargo workspace, no external
runtime dependencies.

## Workspace

| Path | Role |
| --- | --- |
| `libsrs/src/types/` | `SrsValue` (`core.rs`), errors (`error.rs`), scoped environment (`env.rs`) |
| `libsrs/src/interpretor/` | `lexical_analyzer.rs` (lexer), `translator.rs` (parser), `evaluator.rs` |
| `libsrs/tests/r5rs/` | Integration tests driving the full pipeline |
| `libsrs/doc/` | Design notes (`lexical_analysis.md`: lexer state machine) |
| `srs/` | CLI binary (`src/main.rs`) and CLI tests (`tests/cli.rs`) |

## Build and test

```bash
cargo build
cargo test                      # whole workspace
cargo test -p libsrs            # library unit + integration tests
cargo run -p srs -- "(+ 2 3)"   # one-shot evaluation
cargo run -p srs                # REPL
```

## Pipeline

`get_lexemes(&str) -> Vec<Lexeme>` → `translate_all(Vec<Lexeme>) -> Vec<SrsValue>`
→ `Evaluator::eval(&SrsValue) -> SrsResult<SrsValue>`.

Every stage returns `SrsResult<T>`; the CLI is a thin driver over these three calls.

## Data model

`SrsValue` (`libsrs/src/types/core.rs`) is the single value enum — no trait objects,
no `Box<dyn ...>`:

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

Errors are `SrsError::Error { kind: SrsErrorKind, message }` or `SrsError::Exit(i64)`;
`Exit` is a control-flow signal, not a failure — the CLI turns it into a process exit code.

`Env` is an `Rc`-linked chain (`Env::root()` / `Env::child(&parent)`) with interior
mutability; it currently holds only the primitive bindings.

## Conventions

- Short, explicit names; no abbreviations beyond the `Srs` prefix.
- Return `SrsResult<T>` for anything fallible; never `panic!`/`unwrap` in library code.
- Implement `Default` for public structs when a sensible default exists.
- Unit tests live in the same file under `#[cfg(test)] mod tests`; cross-stage tests go
  in `libsrs/tests/r5rs/`, CLI behaviour in `srs/tests/cli.rs`.
- Primitives are registered as `SrsValue::Id("__add")`-style markers (single source of
  truth: `Evaluator::register_primitives`) and dispatched by name in `Evaluator::apply`;
  add a new primitive in both places.
- Operator lexemes (`+`, `-`, ...) are translated to `SrsValue::Id("+")` by the translator.

## Current capabilities

- **Lexer**: integers, floats, identifiers, strings, chars, booleans (`#t`/`#f`),
  parentheses, quote, `#`, arithmetic and comparison tokens (`= < <= > >= not`).
- **Parser**: nested s-expressions into `SrsValue::List(...)`; multiple top-level forms.
- **Evaluator**: self-evaluating literals, arithmetic `+ - * /` with int/float coercion
  and a division-by-zero guard, `exit` / `(exit <integer>)`.
- **CLI**: REPL or one-shot argument; `exit`, `quit`, Ctrl-D and `(exit [code])` all quit.

Comparison operators, `define`, `lambda`, `if`, `let` and quoting are lexed but **not**
evaluated yet.

## Extending the evaluator

- Validate primitive argument counts and types explicitly (see `Evaluator::exit` for the
  slice-pattern style).
- Add a recursion/depth guard before introducing new recursive descent.
- The interpreter is **not hardened**; read `SECURITY.md` before touching numeric
  primitives, recursion or input handling.

## Planned work

See `ROADMAP.md`. Do not start a roadmap item without a matching GitHub issue.

## Contributing

- One branch per GitHub issue, changes land through a PR, never push to `main`
  (see `CONTRIBUTING.md`).
- Prefer `gh` over raw `git` for GitHub operations (PRs, issues, checks).
