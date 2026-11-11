# arxiv-mcp-rs

Rust MCP server for arXiv literature work. It exposes the same **tool and prompt names** as the popular Python [arxiv-mcp-server](https://pypi.org/project/arxiv-mcp-server/) reference, with a **stateless** process model, a **fresh on-disk store** under your home directory, and **stdio or Streamable HTTP** transports.

Search papers, fetch abstracts, download HTML (with PDF fallback) into Markdown, list what you have locally, and read cached content—plus seven workflow **prompts** for agents.

**Origin:** https://cursor.com/codebase/oren/arxiv-mcp-rs  
**GitHub mirror:** https://github.com/OrenVill/arxiv-mcp-rs

## Quick start

**Requirements:** Rust **1.89+** (see [rust-toolchain.toml](./rust-toolchain.toml)).

```bash
# Build
cargo build --release

# Smoke test (store path, legacy-cache warning, allowlist)
cargo run -- check

# Run MCP on stdio (what Cursor / Claude attach to)
cargo run --release -- serve

# Optional: download one paper from the shell (no MCP client needed)
cargo run --release -- fetch 1706.03762
```

Install the binary on your `PATH` if you prefer:

```bash
cargo install --path crates/arxiv-mcp-rs
arxiv-mcp-rs serve
```

**HTTP mode** (Streamable MCP, stateless sessions):

```bash
cargo run --release -- serve --transport http --host 127.0.0.1 --port 8000
```

| Endpoint | URL |
|----------|-----|
| MCP | `http://127.0.0.1:8000/mcp` |
| Health | `GET http://127.0.0.1:8000/healthz` → `ok` |

Bind stays on loopback by default; put TLS and auth in front for anything beyond local use ([SPEC.md §12.3](./SPEC.md)).

## What you get

### Tools

| Tool | Purpose |
|------|---------|
| `search_papers` | Query arXiv (categories, dates, pagination) |
| `get_abstract` | Metadata + abstract for one ID |
| `download_paper` | HTML-first download → Markdown on disk; PDF fallback |
| `list_papers` | Index of cached papers |
| `read_paper` | Read cached Markdown (bounded pagination) |

Paper text is **untrusted external content**; responses may include a content warning on the first chunk.

### Prompts

`research-discovery`, `deep-paper-analysis`, `summarize_paper`, `compare_papers`, `literature_review`, `literature-synthesis`, `research-question` — instruction templates for agents (no model runs inside the server).

## Storage

- Default store: `~/.arxiv-mcp-rs/papers` (per-paper `content.md` + `manifest.json`).
- Does **not** import the Python server’s flat cache; `check` warns if `~/.arxiv-mcp-server/papers` exists.
- Override with `--storage-path` or `ARXIV_STORAGE_PATH`.

## Configuration

| Setting | CLI | Environment | Default |
|---------|-----|-------------|---------|
| Storage root | `--storage-path` | `ARXIV_STORAGE_PATH` | `~/.arxiv-mcp-rs/papers` |
| Transport | `--transport` | `TRANSPORT` | `stdio` |
| HTTP bind | `--host`, `--port` | `HOST`, `PORT` | `127.0.0.1:8000` |
| User-Agent contact | `--contact-email` | `ARXIV_CONTACT_EMAIL` | unset |
| Store quota | — | `STORE_QUOTA_MB` | no cap |
| Strict sanitize | — | `STRICT_SANITIZE` | off |
| Read page size | — | `DEFAULT_MAX_CHARS` | `12000` |

More env vars and limits: [SPEC.md](./SPEC.md).

## Connect an MCP client

### Cursor / VS Code (`mcp.json`)

```json
{
  "mcpServers": {
    "arxiv": {
      "type": "stdio",
      "command": "/path/to/target/release/arxiv-mcp-rs",
      "args": ["serve", "--storage-path", "/path/to/.arxiv-mcp-rs/papers"],
      "env": {
        "ARXIV_CONTACT_EMAIL": "you@example.com",
        "RUST_LOG": "arxiv_mcp_rs=info"
      }
    }
  }
}
```

### Claude Desktop

`claude_desktop_config.json` (macOS: `~/Library/Application Support/Claude/`, Windows: `%APPDATA%\Claude\`):

```json
{
  "mcpServers": {
    "arxiv": {
      "command": "/path/to/target/release/arxiv-mcp-rs",
      "args": ["serve", "--storage-path", "/path/to/.arxiv-mcp-rs/papers"]
    }
  }
}
```

### Claude Code

```json
{
  "mcpServers": {
    "arxiv": {
      "type": "stdio",
      "command": "arxiv-mcp-rs",
      "args": ["serve", "--storage-path", "/absolute/path/to/store"]
    }
  }
}
```

### HTTP client

Point the client at `http://127.0.0.1:8000/mcp` when the server runs with `--transport http`.

## Develop

```bash
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

Clone from Origin: [docs/ORIGIN.md](./docs/ORIGIN.md). Architecture and contracts: [SPEC.md](./SPEC.md). Contributing: [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

Apache-2.0. See [LICENSE](./LICENSE) and [NOTICE](./NOTICE).
