# Contributing

## Branches

- Use `main` as the integration branch.
- Feature branches use short descriptive names (for example `fetch-allowlist`, `tool-search`).
- Do **not** use `cursor/` prefixed branch names.

## Commits

- Write clear commit messages in complete sentences.
- Do not add `Co-authored-by` trailers for bots or agents.

## Development

```bash
rustup toolchain install 1.89.0
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

See [SPEC.md](./SPEC.md) for scope, architecture, and milestones.
