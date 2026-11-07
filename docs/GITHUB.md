# GitHub (public) — option 3

You create the repo; the agent (or you) push `main` once `gh` can authenticate.

## 1. Create the repo (you, in the browser)

On [github.com/new](https://github.com/new):

| Field | Value |
|-------|--------|
| Owner | `orenvill` |
| Name | `arxiv-mcp-rs` |
| Visibility | **Public** |
| README / .gitignore / license | **None** (empty repo) |

Note the clone URL, e.g. `https://github.com/orenvill/arxiv-mcp-rs.git`.

## 2. Give the Cloud Agent `gh` access

Add **`GH_TOKEN`** (or `GITHUB_TOKEN`) in [Cursor Secrets](https://cursor.com/settings) with `repo` scope, bind it to your Cloud Agent **environment**, then **start a new agent run** (secrets are picked up on boot).

Or complete device login in the agent terminal: `gh auth login -h github.com -p https -w`.

## 3. Push from the agent

Replace the URL if your repo name or owner differs:

```bash
cd /workspace
git remote remove github 2>/dev/null || true
git remote add github https://github.com/orenvill/arxiv-mcp-rs.git
git push -u github main
```

With a token in the environment, `gh` uses it automatically:

```bash
gh auth status
git push -u github main
```

## 4. Verify

- Repo shows `SPEC.md`, `crates/`, and CI workflow.
- **Actions** tab runs on `main` (fmt, clippy, test).

## Remotes

| Remote | Purpose |
|--------|---------|
| `origin` | Cursor Origin (agent workspace) |
| `github` | Public GitHub (after step 3) |
