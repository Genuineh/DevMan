//! MCP Server for AI integration.
//!
//! This module implements the Model Context Protocol (MCP) server
//! for DevMan, enabling AI assistants to interact with the system.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::UnixStream;
use tracing::{debug, error, info};

use crate::AIInterface;

/// MCP Protocol version
pub const MCP_VERSION: &str = "2024-11-05";

/// DevMan MCP server configuration.
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Storage path for DevMan data
    pub storage_path: std::path::PathBuf,
    /// Server name for MCP identification
    pub server_name: String,
    /// Server version
    pub version: String,
    /// Unix socket path for stdio transport
    pub socket_path: Option<std::path::PathBuf>,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            storage_path: ".devman".into(),
            server_name: "devman".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            socket_path: None,
        }
    }
}

/// Tool definition for MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema for the tool
    pub input_schema: serde_json::Value,
}

/// Resource definition for MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    pub description: String,
    /// MIME type
    pub mime_type: Option<String>,
}

/// MCP Request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum McpRequest {
    /// Initialize request
    #[serde(rename = "initialize")]
    Initialize {
        protocol_version: String,
        capabilities: serde_json::Value,
    },

    /// List tools request
    #[serde(rename = "tools/list")]
    ToolsList,

    /// Call tool request
    #[serde(rename = "tools/call")]
    ToolsCall {
        name: String,
        arguments: serde_json::Value,
    },

    /// List resources request
    #[serde(rename = "resources/list")]
    ResourcesList,

    /// Read resource request
    #[serde(rename = "resources/read")]
    ResourcesRead { uri: String },

    /// Ping request
    #[serde(rename = "ping")]
    Ping,
}

/// MCP Response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// Request id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Result data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

/// MCP Error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// DevMan MCP server.
pub struct McpServer {
    /// Configuration
    config: McpServerConfig,
    /// Whether server is running
    running: bool,
    /// Registered tools
    tools: HashMap<String, McpTool>,
    /// Registered resources
    resources: HashMap<String, McpResource>,
    /// AI interface reference
    ai_interface: Option<Arc<dyn AIInterface>>,
}

impl McpServer {
    /// Create a new MCP server with default config.
    pub async fn new() -> anyhow::Result<Self> {
        Self::with_config(McpServerConfig::default()).await
    }

    /// Create a new MCP server with custom config.
    pub async fn with_config(config: McpServerConfig) -> anyhow::Result<Self> {
        let mut server = Self {
            config,
            running: false,
            tools: HashMap::new(),
            resources: HashMap::new(),
            ai_interface: None,
        };

        // Register built-in DevMan tools
        server.register_builtin_tools();

        // Register built-in resources
        server.register_builtin_resources();

        Ok(server)
    }

    /// Get the server configuration.
    pub fn config(&self) -> &McpServerConfig {
        &self.config
    }

    /// Check if server is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Set AI interface for tool execution.
    pub fn set_ai_interface(&mut self, ai: Arc<dyn AIInterface>) {
        self.ai_interface = Some(ai);
    }

    /// Register a tool with the MCP server.
    pub fn register_tool(&mut self, tool: McpTool) {
        let name = tool.name.clone();
        self.tools.insert(name.clone(), tool);
        debug!("Registered tool: {}", name);
    }

    /// Register a resource with the MCP server.
    pub fn register_resource(&mut self, resource: McpResource) {
        let uri = resource.uri.clone();
        self.resources.insert(uri.clone(), resource);
        debug!("Registered resource: {}", uri);
    }

    /// Register built-in DevMan tools.
    fn register_builtin_tools(&mut self) {
        // Goal management tools
        self.register_tool(McpTool {
            name: "devman_create_goal".to_string(),
            description: "Create a new goal in DevMan".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": {"type": "string", "description": "Goal title"},
                    "description": {"type": "string", "description": "Goal description"},
                    "success_criteria": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Success criteria for the goal"
                    }
                },
                "required": ["title"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_get_goal_progress".to_string(),
            description: "Get progress of a goal".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "goal_id": {"type": "string", "description": "Goal ID"}
                },
                "required": ["goal_id"]
            }),
        });

        // Task management tools
        self.register_tool(McpTool {
            name: "devman_create_task".to_string(),
            description: "Create a new task".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": {"type": "string", "description": "Task title"},
                    "description": {"type": "string", "description": "Task description"},
                    "goal_id": {"type": "string", "description": "Associated goal ID"},
                    "phase_id": {"type": "string", "description": "Associated phase ID"}
                },
                "required": ["title"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_list_tasks".to_string(),
            description: "List tasks with optional filters".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "state": {
                        "type": "string",
                        "enum": ["Created", "InProgress", "Completed", "Abandoned"],
                        "description": "Filter by task state"
                    },
                    "limit": {"type": "integer", "description": "Maximum results"}
                }
            }),
        });

        // Knowledge tools
        self.register_tool(McpTool {
            name: "devman_search_knowledge".to_string(),
            description: "Search knowledge base".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"},
                    "limit": {"type": "integer", "description": "Maximum results"}
                },
                "required": ["query"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_save_knowledge".to_string(),
            description: "Save new knowledge to the knowledge base".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": {"type": "string", "description": "Knowledge title"},
                    "knowledge_type": {
                        "type": "string",
                        "enum": ["LessonLearned", "BestPractice", "CodePattern", "Solution", "Template", "Decision"]
                    },
                    "content": {"type": "string", "description": "Knowledge content"},
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Tags for categorization"
                    }
                },
                "required": ["title", "knowledge_type", "content"]
            }),
        });

        // Quality tools
        self.register_tool(McpTool {
            name: "devman_run_quality_check".to_string(),
            description: "Run quality checks on the project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "check_type": {
                        "type": "string",
                        "enum": ["compile", "test", "lint", "format", "doc"],
                        "description": "Type of quality check"
                    },
                    "target": {"type": "string", "description": "Optional target"}
                }
            }),
        });

        // Tool execution
        self.register_tool(McpTool {
            name: "devman_execute_tool".to_string(),
            description: "Execute a tool (cargo, git, etc.)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "tool": {
                        "type": "string",
                        "enum": ["cargo", "git", "npm", "fs"],
                        "description": "Tool to execute"
                    },
                    "command": {"type": "string", "description": "Command to run"},
                    "args": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Command arguments"
                    },
                    "timeout": {"type": "integer", "description": "Timeout in seconds"}
                },
                "required": ["tool", "command"]
            }),
        });

        // Context and progress
        self.register_tool(McpTool {
            name: "devman_get_context".to_string(),
            description: "Get current work context".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        });

        self.register_tool(McpTool {
            name: "devman_list_blockers".to_string(),
            description: "List current blockers".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        });
    }

    /// Register built-in DevMan resources.
    fn register_builtin_resources(&mut self) {
        // Project context resource
        self.register_resource(McpResource {
            uri: "devman://context/project".to_string(),
            name: "Current Project Context".to_string(),
            description: "Current project configuration and state".to_string(),
            mime_type: Some("application/json".to_string()),
        });

        // Active goal resource
        self.register_resource(McpResource {
            uri: "devman://context/goal".to_string(),
            name: "Active Goal".to_string(),
            description: "Currently active goal and its progress".to_string(),
            mime_type: Some("application/json".to_string()),
        });

        // Task queue resource
        self.register_resource(McpResource {
            uri: "devman://tasks/queue".to_string(),
            name: "Task Queue".to_string(),
            description: "All pending and in-progress tasks".to_string(),
            mime_type: Some("application/json".to_string()),
        });

        // Recent knowledge resource
        self.register_resource(McpResource {
            uri: "devman://knowledge/recent".to_string(),
            name: "Recent Knowledge".to_string(),
            description: "Recently added or updated knowledge".to_string(),
            mime_type: Some("application/json".to_string()),
        });
    }

    /// Handle an MCP request.
    async fn handle_request(&self, request: McpRequest, id: Option<String>) -> McpResponse {
        match request {
            McpRequest::Initialize { protocol_version, capabilities: _ } => {
                debug!("MCP Initialize request - version: {}", protocol_version);
                McpResponse {
                    id,
                    result: Some(json!({
                        "protocolVersion": protocol_version,
                        "capabilities": {
                            "tools": json!({}),
                            "resources": json!({})
                        },
                        "serverInfo": {
                            "name": self.config.server_name,
                            "version": self.config.version
                        }
                    })),
                    error: None,
                }
            }

            McpRequest::ToolsList => {
                let tools: Vec<_> = self.tools.values().cloned().collect();
                McpResponse {
                    id,
                    result: Some(json!({ "tools": tools })),
                    error: None,
                }
            }

            McpRequest::ToolsCall { name, arguments } => {
                debug!("Tool call: {} with args: {:?}", name, arguments);
                let result = self.execute_tool(&name, arguments).await;
                McpResponse {
                    id,
                    result: Some(result),
                    error: None,
                }
            }

            McpRequest::ResourcesList => {
                let resources: Vec<_> = self.resources.values().cloned().collect();
                McpResponse {
                    id,
                    result: Some(json!({ "resources": resources })),
                    error: None,
                }
            }

            McpRequest::ResourcesRead { uri } => {
                debug!("Resource read: {}", uri);
                let result = self.read_resource(&uri).await;
                McpResponse {
                    id,
                    result: Some(result),
                    error: None,
                }
            }

            McpRequest::Ping => {
                McpResponse {
                    id,
                    result: Some(json!({ "status": "pong" })),
                    error: None,
                }
            }
        }
    }

    /// Execute a tool.
    async fn execute_tool(
        &self,
        _name: &str,
        _arguments: serde_json::Value,
    ) -> serde_json::Value {
        // Default response - tools would be executed via AI interface in full implementation
        json!({
            "success": true,
            "message": "Tool executed successfully (placeholder)"
        })
    }

    /// Read a resource.
    async fn read_resource(&self, _uri: &str) -> serde_json::Value {
        // Default response - resources would be loaded from storage in full implementation
        json!({
            "contents": [{
                "uri": _uri,
                "mimeType": "application/json",
                "text": "{}"
            }]
        })
    }

    /// Start the MCP server with stdio transport.
    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.start_with_stdio().await
    }

    /// Start with stdio transport.
    async fn start_with_stdio(&mut self) -> anyhow::Result<()> {
        info!("Starting DevMan MCP Server v{} (stdio transport)", self.config.version);

        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();
        let mut stdout = BufWriter::new(tokio::io::stdout());

        self.running = true;

        while let Some(line_result) = lines.next_line().await? {
            // Parse the message
            if line_result.trim().is_empty() {
                continue;
            }

            let request: McpRequest = match serde_json::from_str(&line_result) {
                Ok(req) => req,
                Err(e) => {
                    error!("Failed to parse request: {}", e);
                    let error_response = McpResponse {
                        id: None,
                        result: None,
                        error: Some(McpError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                            data: None,
                        }),
                    };
                    let error_json = serde_json::to_string(&error_response)
                        .unwrap_or_else(|_| "{}".to_string());
                    if let Err(e) = stdout.write_all(error_json.as_bytes()).await {
                        error!("Failed to write error response: {}", e);
                        break;
                    }
                    if let Err(e) = stdout.write_all(b"\n").await {
                        error!("Failed to write newline: {}", e);
                        break;
                    }
                    if let Err(e) = stdout.flush().await {
                        error!("Failed to flush: {}", e);
                    }
                    continue;
                }
            };

            // Handle the request
            let response = self.handle_request(request, None).await;
            let response_json = serde_json::to_string(&response)
                .unwrap_or_else(|_| "{}".to_string());

            if let Err(e) = stdout.write_all(response_json.as_bytes()).await {
                error!("Failed to write response: {}", e);
                break;
            }
            if let Err(e) = stdout.write_all(b"\n").await {
                error!("Failed to write newline: {}", e);
                break;
            }
            if let Err(e) = stdout.flush().await {
                error!("Failed to flush: {}", e);
            }
        }

        self.running = false;
        info!("MCP Server stopped");
        Ok(())
    }

    /// Start with Unix socket transport.
    pub async fn start_with_socket(&mut self, socket_path: &std::path::Path) -> anyhow::Result<()> {
        info!("Starting DevMan MCP Server v{} (socket transport)", self.config.version);

        // Remove existing socket file
        if socket_path.exists() {
            std::fs::remove_file(socket_path)?;
        }

        let listener = tokio::net::UnixListener::bind(socket_path)?;
        self.running = true;

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            self.handle_connection(stream).await?;
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        }

        self.running = false;
        info!("MCP Server stopped");
        Ok(())
    }

    /// Handle a client connection.
    async fn handle_connection(&self, stream: UnixStream) -> anyhow::Result<()> {
        let (reader, mut writer) = stream.into_split();
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();

        while let Some(line_result) = lines.next_line().await? {
            if line_result.trim().is_empty() {
                continue;
            }

            let request: McpRequest = match serde_json::from_str(&line_result) {
                Ok(req) => req,
                Err(e) => {
                    error!("Failed to parse request: {}", e);
                    continue;
                }
            };

            let response = self.handle_request(request, None).await;
            let response_json = serde_json::to_string(&response)
                .unwrap_or_else(|_| "{}".to_string());

            if let Err(e) = writer.write_all(response_json.as_bytes()).await {
                error!("Failed to write response: {}", e);
                break;
            }
            if let Err(e) = writer.write_all(b"\n").await {
                error!("Failed to write newline: {}", e);
                break;
            }
        }

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
    fn test_mcp_tool_definition() {
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "param": {"type": "string"}
                }
            }),
        };

        assert_eq!(tool.name, "test_tool");
        assert!(tool.input_schema.is_object());
    }

    #[test]
    fn test_mcp_resource_definition() {
        let resource = McpResource {
            uri: "devman://test/resource".to_string(),
            name: "Test Resource".to_string(),
            description: "A test resource".to_string(),
            mime_type: Some("application/json".to_string()),
        };

        assert_eq!(resource.uri, "devman://test/resource");
        assert_eq!(resource.mime_type, Some("application/json".to_string()));
    }

    #[test]
    fn test_mcp_error_definition() {
        let error = McpError {
            code: -32600,
            message: "Invalid request".to_string(),
            data: None,
        };

        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid request");
    }

    #[test]
    fn test_mcp_request_initialize() {
        let request = McpRequest::Initialize {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: json!({}),
        };

        match request {
            McpRequest::Initialize { protocol_version, .. } => {
                assert_eq!(protocol_version, MCP_VERSION);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_mcp_request_tools_call() {
        let request = McpRequest::ToolsCall {
            name: "test_tool".to_string(),
            arguments: json!({ "param": "value" }),
        };

        match request {
            McpRequest::ToolsCall { name, arguments } => {
                assert_eq!(name, "test_tool");
                assert_eq!(arguments["param"], "value");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_mcp_response_success() {
        let response = McpResponse {
            id: Some("1".to_string()),
            result: Some(json!({ "status": "ok" })),
            error: None,
        };

        assert_eq!(response.id, Some("1".to_string()));
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_mcp_response_error() {
        let response = McpResponse {
            id: Some("2".to_string()),
            result: None,
            error: Some(McpError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        };

        assert_eq!(response.id, Some("2".to_string()));
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

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
            socket_path: Some("/tmp/custom.sock".into()),
        };
        assert_eq!(config.server_name, "custom_devman");
        assert_eq!(config.socket_path, Some(std::path::PathBuf::from("/tmp/custom.sock")));
    }

    #[test]
    fn test_mcp_server_builtin_tools_registration() {
        // Create server synchronously using Default
        let config = McpServerConfig::default();
        let server = futures::executor::block_on(McpServer::with_config(config)).unwrap();
        // Check that some built-in tools are registered
        assert!(server.tools.contains_key("devman_create_goal"));
        assert!(server.tools.contains_key("devman_create_task"));
        assert!(server.tools.contains_key("devman_search_knowledge"));
        assert!(server.tools.contains_key("devman_run_quality_check"));
    }

    #[test]
    fn test_mcp_server_builtin_resources_registration() {
        let config = McpServerConfig::default();
        let server = futures::executor::block_on(McpServer::with_config(config)).unwrap();
        // Check that some built-in resources are registered
        assert!(server.resources.contains_key("devman://context/project"));
        assert!(server.resources.contains_key("devman://tasks/queue"));
        assert!(server.resources.contains_key("devman://knowledge/recent"));
    }
}
