//! DevMan MCP Server - AI Interface for DevMan
//!
//! This binary provides an MCP (Model Context Protocol) server
//! for AI assistants to interact with DevMan.

use anyhow::Result;
use clap::{Parser, Subcommand};
use devman_ai::mcp_server::McpServer;
use devman_ai::mcp_server::McpServerConfig;
use tracing::{info, Level};
use tracing_subscriber::fmt;

#[derive(Parser)]
#[command(name = "devman-ai")]
#[command(about = "DevMan MCP Server - AI interface for work management", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Storage path for DevMan data
    #[arg(short, long, default_value = ".devman")]
    storage: std::path::PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Start MCP Server in stdio mode
    Stdio,

    /// Start MCP Server with Unix socket
    Socket {
        /// Socket path
        path: std::path::PathBuf,
    },

    /// List available tools
    ListTools,

    /// Print server info
    Info,
}

fn init_logging(to_stderr: bool) {
    if to_stderr {
        // Socket mode: disable logging entirely to avoid interfering with MCP protocol
        // The VSCode MCP extension captures both stdout and stderr
        tracing_subscriber::fmt()
            .with_max_level(Level::ERROR)
            .with_writer(std::io::sink)
            .init();
    } else {
        // Stdio/info mode: log to stdout
        fmt::Subscriber::builder()
            .with_max_level(Level::INFO)
            .init();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Create storage path
    std::fs::create_dir_all(&cli.storage).ok();

    let mut server = McpServer::with_config(
        McpServerConfig {
            storage_path: cli.storage,
            server_name: "devman".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            socket_path: None,
        }
    ).await?;

    match cli.command {
        Commands::Stdio => {
            init_logging(false);
            info!("Starting DevMan MCP Server (stdio mode)");
            server.start().await?;
        }

        Commands::Socket { path } => {
            init_logging(true);
            info!("Starting DevMan MCP Server (socket mode)");
            server.start_with_socket(&path).await?;
        }

        Commands::ListTools => {
            init_logging(false);
            let tools: Vec<_> = server.tools.values().collect();
            println!("Available tools ({}):", tools.len());
            for tool in tools {
                println!("  - {}", tool.name);
            }
        }

        Commands::Info => {
            init_logging(false);
            println!("DevMan MCP Server v{}", env!("CARGO_PKG_VERSION"));
            println!("Protocol: MCP 2024-11-05");
            println!("Transport: stdio / Unix socket");
            println!("Tools: {}", server.tools.len());
            println!("Resources: {}", server.resources.len());
        }
    }

    Ok(())
}
