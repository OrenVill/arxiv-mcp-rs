# arxiv-mcp-rs

Stateless Rust MCP server for arXiv literature work: upstream-aligned **tools and prompts** (per [SPEC.md](./SPEC.md)), fresh on-disk store, **stdio + Streamable HTTP** in v1, built-in PDF path (`pdf-extract` + `lopdf`).

**Origin:** https://cursor.com/codebase/oren/arxiv-mcp-rs (**Private** by default; change visibility in repo settings)

**Upstream inspiration:** [blazickjp/arxiv-mcp-server](https://pypi.org/project/arxiv-mcp-server/) on PyPI (behavioral reference; not a cache drop-in).

## Status

| Milestone | Deliverable | State |
|-----------|-------------|-------|
| **M1** | Workspace scaffold, ID validation, stdio + stateless HTTP MCP shell, `ping` | **Done** |
| **M2** | `StoreBackend` v1 — manifest layout, atomic writes, optional `--quota-mb` | **Done** |
| **M3** | `search_papers`, `get_abstract`, Atom fixtures, in-memory cache, tool `inputSchema` contracts | **Done** |
| **M4** | `download_paper` (HTML), `list_papers`, `read_paper`, in-flight dedup | **Done** |
| **M5** | PDF fallback + golden Markdown vs upstream | **Done** — HTML-first `download_paper`; PDF fallback; golden fixture under `tests/golden/` |
| **M6** | Seven upstream prompts + SHA-256 snapshot tests | **Done** |
| **M7** | Security scenario suite + untrusted content policy | **Done** |
| **M8** | Benchmarks, README, v0.1.0 release | **This branch** |

v0.1.0 tag is next after release checklist (CI workflow on GitHub when `workflow` scope is available).

## Security scenario matrix (summary)

Automated coverage for [SPEC.md §10.6](./SPEC.md) release gate. **arxiv-mcp-rs** enforces these before network or store I/O where noted; Python upstream behavior is the reference for parity, not a guarantee of identical hardening.

| Scenario | arxiv-mcp-rs | Test / location |
|----------|--------------|-----------------|
| `paper_id` = `../../../etc/passwd` | Rejected (ID parse) | `crates/arxiv-core/tests/security_id_matrix.rs` |
| `paper_id` = `https://evil.com/x` | Rejected (ID parse) | same |
| Redirect to non-allowlisted host | Blocked | `crates/arxiv-fetch/tests/redirect_allowlist.rs` |
| Symlink paper directory | Blocked | `crates/arxiv-store/src/local.rs` (`rejects_symlink_paper_dir`) |
| Store quota exceeded | Bounded error (`quota exceeded`) | `crates/arxiv-store` unit tests |
| Untrusted banner + sanitize defaults | `CONTENT_WARNING` on first chunk; NFC + invisible-char strip | `crates/arxiv-core/tests/security_untrusted_policy.rs` |
| Oversized PDF download | Capped via fetch/converter limits (see env table in SPEC §10.3) | Policy in code; live size gates expanded with M5 |
| stdout contains non-JSON-RPC during stdio | Logs on stderr only (`tracing`); MCP on stdout | `crates/arxiv-mcp-rs/src/main.rs` |

## Clone locally (Windows → WSL)

See [docs/ORIGIN.md](./docs/ORIGIN.md) for Origin CLI install, auth, and `origin repo clone oren/arxiv-mcp-rs`.

## Requirements

- Rust **1.89+** ([rust-toolchain.toml](./rust-toolchain.toml))
- Linux / macOS / Windows for builds (CI: Linux x86_64)

## Build and test

```bash
cargo build --release
cargo test --workspace
cargo run -- serve --transport stdio
```

### Benchmarks (M8 stub)

Micro-benchmark for hot-path ID parsing (full cold-start / RSS / binary-size table ships after M5 golden job):

```bash
cargo bench -p arxiv-core --bench id_parse
```

See [benches/README.md](./benches/README.md) for manual measurement recipes vs `uvx arxiv-mcp-server`.

### Streamable HTTP (v1)

```bash
TRANSPORT=http PORT=8000 cargo run -- serve --transport http --host 127.0.0.1 --port 8000
```

- MCP: `http://127.0.0.1:8000/mcp`
- Health: `GET http://127.0.0.1:8000/healthz` → `ok`

Bind defaults to loopback; put TLS and auth in front of HTTP in production ([SPEC.md §12.3](./SPEC.md)).

## Configuration

| Setting | CLI | Environment | Default |
|---------|-----|-------------|---------|
| Storage root | `--storage-path` | `ARXIV_STORAGE_PATH` | `~/.arxiv-mcp-rs/papers` |
| Transport | `--transport` | `TRANSPORT` | `stdio` |
| HTTP bind | `--host`, `--port` | `HOST`, `PORT` | `127.0.0.1:8000` |
| Contact (User-Agent) | `--contact-email` | `ARXIV_CONTACT_EMAIL` | unset |
| Store quota | — (CLI extension planned) | `STORE_QUOTA_MB` | none |
| Strict sanitize | — | `STRICT_SANITIZE` | `0` |
| Default page size | — | `DEFAULT_MAX_CHARS` | `12000` |

Default store: `~/.arxiv-mcp-rs/papers` (**D2:** does not read or migrate Python `~/.arxiv-mcp-server/papers`; `check` warns if legacy flat cache exists).

### MCP client — Cursor / VS Code (`mcp.json`)

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

Edit `claude_desktop_config.json` (macOS: `~/Library/Application Support/Claude/`, Windows: `%APPDATA%\Claude\`):

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

Project or user MCP config (stdio):

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

### HTTP client (Streamable MCP)

Point the client at `http://127.0.0.1:8000/mcp` when the server runs with `TRANSPORT=http`. No auth in v1—terminate TLS and access control at your proxy.

## Tools and prompts (v1 surface)

**Tools:** `search_papers`, `get_abstract`, `download_paper`, `list_papers`, `read_paper` (plus scaffold `ping`).

**Prompts:** `literature_review`, `research_discovery`, `research_question`, `literature_synthesis`, `summarize_paper`, `deep_paper_analysis`, `compare_papers` (snapshot-tested).

## Crate layout

| Crate | Role |
|-------|------|
| `arxiv-mcp-rs` | CLI binary (`serve`, `check`, `fetch`) |
| `arxiv-mcp` | MCP handlers, tools, prompts, transports |
| `arxiv-core` | Config, Atom parsing, ID validation, untrusted policy |
| `arxiv-store` | `StoreBackend` + local FS v1 |
| `arxiv-fetch` | Allowlisted HTTP client + redirect policy |
| `arxiv-convert` | HTML → Markdown; PDF extraction stub |

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md): work on `main` or normal feature branches—no `cursor/` prefixes; no agent co-authors on commits.

## License

Apache-2.0. See [LICENSE](./LICENSE) and [NOTICE](./NOTICE).
