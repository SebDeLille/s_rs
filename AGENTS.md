# AGENTS.md

Minimal Scheme (R5RS subset) interpreter in Rust. Cargo workspace, no external
runtime dependencies except for the GTK4 system libraries required by the
optional `srsgtk` crate.

## Crates

- `libsrs` — core interpreter library (lexer, reader, evaluator, types,
  startup-script loader).
- `srs` — command-line REPL built on `libsrs`.
- `srsgtk` — GTK4 graphical frontend with a drawing canvas and an integrated
  REPL.

## Contributing

- One branch per GitHub issue, changes land through a PR, never push to `main`
  (see `CONTRIBUTING.md`).
- Prefer `gh` over raw `git` for GitHub operations (PRs, issues, checks).
