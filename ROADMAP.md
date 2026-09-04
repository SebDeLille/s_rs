# Roadmap

Planned work, roughly in order. Items are promoted to GitHub issues when started;
see `CONTRIBUTING.md` for the branch and PR workflow.

## Special forms

- `if` conditional.
- `define` for variables, backed by the existing `Env`.
- `lambda` and procedure application, with `Env::child` for the call frame.
- `let` / `let*` bindings.

## Primitives

- Comparison operators `= < <= > >=` and `not` — currently lexed but not evaluated.
- Boolean logic `and` / `or`.
- List primitives `car`, `cdr`, `cons`, `list`, `null?`.

## Reader

- `quote` and the `'` shorthand — currently lexed but not translated.
- Quasiquote / unquote.
- Vector literals `#(...)` — `SrsValue::Vector` exists but is rejected by the evaluator.

## Diagnostics and robustness

- Line/column locations in `SrsError`.
- Recursion depth guard in the translator and the evaluator.
- Input size limit and an instruction budget.
- Checked integer arithmetic instead of raw `i64` operators.

See `SECURITY.md` for the current limitations these items address.
