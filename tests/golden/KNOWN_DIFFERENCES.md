# Golden Markdown differences (v1)

This repository uses `pdf-extract` with `lopdf` for PDF fallback instead of pymupdf4llm.
Expected quality deltas versus the Python upstream:

- Two-column layout may interleave columns in text order.
- Mathematical notation is often lossy (no LaTeX reconstruction).
- Figures, tables, and headers are not preserved as structured Markdown.
- Scanned PDFs without a text layer yield empty or error responses (no OCR in v1).

Refresh golden files with `UPDATE_GOLDEN=1 cargo test golden_markdown`.
