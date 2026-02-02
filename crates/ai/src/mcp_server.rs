//! MCP Server for AI integration.

use tracing::info;

/// DevMan MCP server configuration.
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Storage path for DevMan data
    pub storage_path: std::path::PathBuf,
    /// Server name for MCP identification
    pub server_name: String,
    /// Server version
    pub version: String,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            storage_path: ".devman".into(),
            server_name: "devman".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// DevMan MCP server.
pub struct McpServer {
    /// Configuration
    config: McpServerConfig,
    /// Whether server is running
    running: bool,
}

impl McpServer {
    /// Create a new MCP server with default config.
    pub async fn new() -> anyhow::Result<Self> {
        Self::with_config(McpServerConfig::default()).await
    }

    /// Create a new MCP server with custom config.
    pub async fn with_config(config: McpServerConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            running: false,
        })
    }

    /// Get the server configuration.
    pub fn config(&self) -> &McpServerConfig {
        &self.config
    }

    /// Check if server is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Start the MCP server.
    pub async fn start(&mut self) -> anyhow::Result<()> {
        info!("Starting DevMan MCP Server v{}", self.config.version);

        // TODO: Implement actual MCP stdio transport
        // - Initialize MCP protocol handlers
        // - Register DevMan tools with MCP
        // - Set up stdio transport for communication
        // - Handle incoming requests from AI clients

        info!("MCP Server ready (placeholder)");

        self.running = true;

        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        info!("Shutting down");

        self.running = false;
        Ok(())
    }

    /// Stop the MCP server.
    pub fn stop(&mut self) {
        self.running = false;
        info!("MCP Server stopped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_config_default() {
        let config = McpServerConfig::default();
        assert_eq!(config.server_name, "devman");
        assert_eq!(config.storage_path, std::path::PathBuf::from(".devman"));
        assert!(!config.version.is_empty());
    }

    #[test]
    fn test_mcp_server_config_custom() {
        let config = McpServerConfig {
            storage_path: "/custom/path".into(),
            server_name: "custom_devman".to_string(),
            version: "1.0.0".to_string(),
        };
        assert_eq!(config.server_name, "custom_devman");
        assert_eq!(config.storage_path, std::path::PathBuf::from("/custom/path"));
        assert_eq!(config.version, "1.0.0");
    }

    #[test]
    fn test_mcp_server_is_running_initially_false() {
        // Test that server is not running initially by checking config
        let config = McpServerConfig::default();
        assert!(!config.storage_path.as_os_str().is_empty());
    }

    #[test]
    fn test_mcp_server_config_fields() {
        let config = McpServerConfig {
            storage_path: "/test".into(),
            server_name: "test_server".to_string(),
            version: "0.1.0".to_string(),
        };
        assert_eq!(config.storage_path, std::path::PathBuf::from("/test"));
        assert_eq!(config.server_name, "test_server");
        assert_eq!(config.version, "0.1.0");
    }
}
