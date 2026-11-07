# Benchmark harness (M8)

Published numbers for v0.1.0 will live in the root [README](../README.md) after M5 golden Markdown work lands. Until then, use these manual recipes and the Criterion stub.

## Criterion (checked in)

```bash
cargo bench -p arxiv-core --bench id_parse
```

## Planned release metrics (SPEC §13.5)

| Metric | How to measure (draft) |
|--------|-------------------------|
| Cold start | Spawn `arxiv-mcp-rs serve`, send MCP `tools/list`, time to first response |
| RSS peak | `/usr/bin/time -v` or `ps` while running `download_paper` on a fixture PDF |
| Binary size | `ls -lh target/release/arxiv-mcp-rs` |
| Download + convert | Compare wall time vs `uvx arxiv-mcp-server` on the same `paper_id` (cold vs warm store) |

Set `ARXIV_LIVE_TESTS=1` only for controlled live comparisons; respect arXiv rate limits.
