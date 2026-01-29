//! MCP Server for AI integration.

use tracing::info;

/// DevMan MCP server.
pub struct McpServer {
    /// Storage path
    storage_path: std::path::PathBuf,
}

impl McpServer {
    /// Create a new MCP server.
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            storage_path: ".devman".into(),
        })
    }

    /// Start the MCP server.
    pub async fn start(&mut self) -> anyhow::Result<()> {
        info!("Starting DevMan MCP Server");

        // TODO: Implement actual MCP stdio transport
        info!("MCP Server ready (placeholder)");

        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        info!("Shutting down");

        Ok(())
    }
}
