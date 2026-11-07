# arxiv-mcp-rs

Stateless Rust MCP server for arXiv literature work: upstream-aligned **tools and prompts** (per [SPEC.md](./SPEC.md)), fresh on-disk store, **stdio + Streamable HTTP** in v1, built-in PDF path (`pdf-extract` + `lopdf`).

**Upstream inspiration:** [blazickjp/arxiv-mcp-server](https://github.com/blazickjp/arxiv-mcp-server) (behavioral reference; not a cache drop-in).

## Status

Milestone **M1** scaffold: workspace, ID validation, store layout shell, MCP `ping` tool, stdio and stateless HTTP (`legacy_session_mode = false`).

## Requirements

- Rust **1.89+** ([rust-toolchain.toml](./rust-toolchain.toml))
- Linux / macOS / Windows for builds (CI: Linux x86_64)

## Build and test

```bash
cargo build --release
cargo test --workspace
cargo run -- serve --transport stdio
```

### Streamable HTTP (v1)

```bash
TRANSPORT=http PORT=8000 cargo run -- serve --transport http --host 127.0.0.1 --port 8000
```

- MCP: `http://127.0.0.1:8000/mcp`
- Health: `GET http://127.0.0.1:8000/healthz` → `ok`

### MCP client (stdio)

```json
{
  "mcpServers": {
    "arxiv": {
      "type": "stdio",
      "command": "/path/to/target/release/arxiv-mcp-rs",
      "args": ["serve", "--storage-path", "/path/to/.arxiv-mcp-rs/papers"]
    }
  }
}
```

Default store: `~/.arxiv-mcp-rs/papers` (fresh layout; does not read Python `~/.arxiv-mcp-server/papers`).

## Crate layout

| Crate | Role |
|-------|------|
| `arxiv-mcp-rs` | CLI binary |
| `arxiv-mcp` | MCP handlers and transports |
| `arxiv-core` | Config and arXiv ID parsing |
| `arxiv-store` | `StoreBackend` + local FS v1 |
| `arxiv-fetch` | Allowlisted HTTP (WIP) |
| `arxiv-convert` | HTML/PDF → Markdown (WIP) |

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md): work on `main` or normal feature branches—no `cursor/` prefixes; no agent co-authors on commits.

## License

Apache-2.0. See [LICENSE](./LICENSE) and [NOTICE](./NOTICE).
