# Contributing

## Git workflow

- Each GitHub issue is handled in a dedicated branch.
- Changes are sent through a Pull Request.
- A Pull Request must reference the issue it addresses (e.g. "Closes #42").
- Do not push directly to `main`.

## Checks

- `cargo fmt` must pass with no diff before opening a PR.
- `cargo clippy` must pass with no warnings before opening a PR.
- `cargo test` must pass before opening a PR.
