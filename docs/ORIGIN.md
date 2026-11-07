# Origin repository

- **Browse:** https://cursor.com/codebase/oren/arxiv-mcp-rs  
- **Visibility:** Private (change under **Settings** on that page)  
- **Clone URL:** `https://origin.cursor.com/git/oren/arxiv-mcp-rs.git`

## Get the repository on your machine (Windows)

Origin CLI does **not** run in PowerShell. Use **WSL** and run:

```bash
# Run in WSL (Origin CLI is not available in PowerShell)
# Install the Origin CLI
curl -fsSL https://downloads.cursor.com/origin/install.sh | sh

# Sign in (also sets up git credentials)
origin auth login

# Clone the repository
origin repo clone oren/arxiv-mcp-rs
```

If `origin` is not found after install, persist `~/.local/bin` on `PATH` in WSL (bash):

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Origin CLI docs:** https://cursor.com/docs/origin/cli
