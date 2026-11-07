use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Context;
use arxiv_core::{legacy_flat_store_detected, legacy_python_storage_path, Config};
use arxiv_mcp::{run_http, run_stdio, AppState};
use arxiv_store::StoreBackend;
use clap::{Parser, Subcommand, ValueEnum};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, ValueEnum)]
enum Transport {
    Stdio,
    Http,
}

#[derive(Parser)]
#[command(name = "arxiv-mcp-rs", about = "Stateless arXiv MCP server")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the MCP server (default subcommand).
    Serve {
        #[arg(long, default_value = "stdio", env = "TRANSPORT")]
        transport: Transport,
        #[arg(long, env = "ARXIV_STORAGE_PATH")]
        storage_path: Option<PathBuf>,
        #[arg(long, default_value = "127.0.0.1", env = "HOST")]
        host: String,
        #[arg(long, default_value_t = 8000, env = "PORT")]
        port: u16,
        #[arg(long, env = "ARXIV_CONTACT_EMAIL")]
        contact_email: Option<String>,
    },
    /// Local smoke checks (expanded in SPEC milestone M1).
    Check {
        #[arg(long, env = "ARXIV_STORAGE_PATH")]
        storage_path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Serve {
            transport,
            storage_path,
            host,
            port,
            contact_email,
        } => {
            let mut config = Config::default();
            if let Some(path) = storage_path {
                config.storage_path = path;
            }
            config.contact_email = contact_email;
            let state = AppState::new(config)?;
            state.store.ensure_root().await?;

            match transport {
                Transport::Stdio => run_stdio(state).await,
                Transport::Http => {
                    let addr: SocketAddr = format!("{host}:{port}")
                        .parse()
                        .context("invalid HOST/PORT")?;
                    run_http(state, addr).await
                }
            }
        }
        Commands::Check { storage_path } => {
            let mut config = Config::default();
            if let Some(path) = storage_path {
                config.storage_path = path;
            }
            let state = AppState::new(config)?;
            state.store.ensure_root().await?;
            tracing::info!(root = %state.store.root().display(), "store ok");
            tracing::info!("check: allowlist hosts export.arxiv.org, arxiv.org, www.arxiv.org");
            warn_if_legacy_python_store();
            Ok(())
        }
    }
}

/// D2: legacy Python cache is never mounted or migrated automatically.
fn warn_if_legacy_python_store() {
    let legacy = legacy_python_storage_path();
    if !legacy_flat_store_detected(&legacy) {
        return;
    }
    tracing::warn!(
        path = %legacy.display(),
        "detected blazickjp/arxiv-mcp-server flat cache (D2: not read or migrated); \
         use store v1 at ~/.arxiv-mcp-rs/papers and download_paper to repopulate"
    );
}
