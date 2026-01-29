//! DevMan MCP Server - Model Context Protocol integration.
//!
//! This allows AI assistants (like Claude) to directly interact with DevMan.

use tracing::{info, Level};

/// DevMan MCP server.
struct McpServer {
    _storage_path: std::path::PathBuf,
}

impl McpServer {
    /// Create a new MCP server.
    async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            _storage_path: ".devman".into(),
        })
    }

    /// Handle an MCP request (placeholder).
    async fn handle_request(&mut self, _request: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "content": [{
                    "type": "text",
                    "text": "DevMan MCP Server - placeholder implementation"
                }]
            }
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting DevMan MCP server");

    let mut _server = McpServer::new().await?;

    // TODO: Implement actual MCP transport (stdio)
    // For now, this is a placeholder that demonstrates the API surface
    info!("MCP server ready (placeholder implementation)");

    // Placeholder: wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    info!("Shutting down");

    Ok(())
}
