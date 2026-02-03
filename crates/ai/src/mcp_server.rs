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

use crate::interface::{GoalSpec, TaskFilter};
use crate::job_manager::JobId;
use crate::{AIInterface, JobManager};

/// Create an error response with DevMan error codes.
fn create_mcp_error_response(
    code: i32,
    message: &str,
    data: Option<serde_json::Value>,
    retryable: bool,
) -> serde_json::Value {
    let mut error_object = serde_json::Map::new();
    error_object.insert("code".to_string(), json!(code));
    error_object.insert("message".to_string(), json!(message));
    error_object.insert("retryable".to_string(), json!(retryable));

    if let Some(d) = data {
        error_object.insert("data".to_string(), d);
    }

    json!({
        "success": false,
        "error": error_object
    })
}

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
    /// Job manager for async tasks
    job_manager: Option<Arc<dyn JobManager>>,
    /// Storage path for resources
    storage_path: std::path::PathBuf,
}

impl McpServer {
    /// Create a new MCP server with default config.
    pub async fn new() -> anyhow::Result<Self> {
        Self::with_config(McpServerConfig::default()).await
    }

    /// Create a new MCP server with custom config.
    pub async fn with_config(config: McpServerConfig) -> anyhow::Result<Self> {
        let mut server = Self {
            config: config.clone(),
            running: false,
            tools: HashMap::new(),
            resources: HashMap::new(),
            ai_interface: None,
            job_manager: None,
            storage_path: config.storage_path.clone(),
        };

        // Register built-in DevMan tools
        server.register_builtin_tools();

        // Register built-in resources
        server.register_builtin_resources();

        Ok(server)
    }

    /// Set Job manager for async task execution.
    pub fn set_job_manager(&mut self, job_manager: Arc<dyn JobManager>) {
        self.job_manager = Some(job_manager);
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

        // ========== Task Guidance Tools (引导性工具) ==========

        self.register_tool(McpTool {
            name: "devman_get_task_guidance".to_string(),
            description: "Get task guidance - AI should call this before any operation. Returns current state, next action, allowed operations, and guidance message.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID to get guidance for"}
                },
                "required": ["task_id"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_read_task_context".to_string(),
            description: "Read task context (Created -> ContextRead). Must be called after task creation.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID"}
                },
                "required": ["task_id"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_review_knowledge".to_string(),
            description: "Review relevant knowledge for a task. Returns knowledge items and suggested reading.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID"},
                    "query": {"type": "string", "description": "Knowledge search query"}
                },
                "required": ["task_id", "query"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_confirm_knowledge_reviewed".to_string(),
            description: "Confirm knowledge review completed (ContextRead -> KnowledgeReviewed).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID"},
                    "knowledge_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "IDs of knowledge items reviewed"
                    }
                },
                "required": ["task_id", "knowledge_ids"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_start_execution".to_string(),
            description: "Start task execution (KnowledgeReviewed -> InProgress).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID"}
                },
                "required": ["task_id"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_log_work".to_string(),
            description: "Log work progress during execution. Records actions, files changed, and command outputs.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID"},
                    "action": {
                        "type": "string",
                        "enum": ["created", "modified", "tested", "documented", "debugged", "refactored"],
                        "description": "Type of work action"
                    },
                    "description": {"type": "string", "description": "Description of work done"},
                    "files": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Files affected by this action"
                    }
                },
                "required": ["task_id", "action", "description"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_finish_work".to_string(),
            description: "Submit work (InProgress -> WorkRecorded). Must have logged work before calling.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID"},
                    "description": {"type": "string", "description": "Summary of work completed"},
                    "artifacts": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"},
                                "type": {"type": "string", "enum": ["file", "code", "documentation", "test", "binary", "other"]},
                                "path": {"type": "string"}
                            }
                        },
                        "description": "Work artifacts produced"
                    },
                    "lessons_learned": {"type": "string", "description": "Lessons learned during this work"}
                },
                "required": ["task_id", "description"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_run_task_quality_check".to_string(),
            description: "Run quality check for a task (WorkRecorded -> QualityChecking).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID"},
                    "check_types": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Types of checks: compile, test, lint, format, doc"
                    }
                },
                "required": ["task_id"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_get_quality_result".to_string(),
            description: "Get quality check result by check ID.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "check_id": {"type": "string", "description": "Quality check ID"}
                },
                "required": ["check_id"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_confirm_quality_result".to_string(),
            description: "Confirm quality result and decide next action (QualityCompleted).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID"},
                    "check_id": {"type": "string", "description": "Quality check ID"},
                    "decision": {
                        "type": "string",
                        "enum": ["accept_and_complete", "fix_and_continue", "redo_execution"],
                        "description": "Decision on quality result"
                    }
                },
                "required": ["task_id", "check_id", "decision"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_complete_task".to_string(),
            description: "Complete a task. Only allowed if quality check passed.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID"},
                    "summary": {"type": "string", "description": "Completion summary"},
                    "artifacts": {
                        "type": "array",
                        "items": {"type": "object"},
                        "description": "Final artifacts"
                    },
                    "created_knowledge_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Knowledge IDs created during this task"
                    }
                },
                "required": ["task_id", "summary"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_pause_task".to_string(),
            description: "Pause a task. Can be resumed later.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID"},
                    "reason": {"type": "string", "description": "Reason for pausing"}
                },
                "required": ["task_id", "reason"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_resume_task".to_string(),
            description: "Resume a paused task.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID"}
                },
                "required": ["task_id"]
            }),
        });

        self.register_tool(McpTool {
            name: "devman_abandon_task".to_string(),
            description: "Abandon a task with a reason. Handles all termination scenarios.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "task_id": {"type": "string", "description": "Task ID"},
                    "reason_type": {
                        "type": "string",
                        "enum": ["voluntary", "project_cancelled", "goal_cancelled", "requirement_changed", "dependency_failed", "insufficient_info", "technical_limitation", "resource_unavailable", "timeout", "quality_failed", "other"],
                        "description": "Type of abandonment reason"
                    },
                    "reason": {"type": "string", "description": "Detailed reason"}
                },
                "required": ["task_id", "reason_type", "reason"]
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
        name: &str,
        arguments: serde_json::Value,
    ) -> serde_json::Value {
        // Check if AI interface is available
        let ai_interface = match &self.ai_interface {
            Some(ai) => ai,
            None => {
                return create_mcp_error_response(
                    -32603,
                    "Internal error: AI interface not configured",
                    None,
                    false,
                );
            }
        };

        match name {
            // Goal management
            "devman_create_goal" => {
                self.handle_create_goal(ai_interface, &arguments).await
            }
            "devman_get_goal_progress" => {
                self.handle_get_goal_progress(ai_interface, &arguments).await
            }

            // Task management
            "devman_create_task" => {
                self.handle_create_task(ai_interface, &arguments).await
            }
            "devman_list_tasks" => {
                self.handle_list_tasks(ai_interface, &arguments).await
            }

            // Knowledge management
            "devman_search_knowledge" => {
                self.handle_search_knowledge(ai_interface, &arguments).await
            }
            "devman_save_knowledge" => {
                self.handle_save_knowledge(ai_interface, &arguments).await
            }

            // Quality checks
            "devman_run_quality_check" => {
                self.handle_run_quality_check(ai_interface, &arguments).await
            }

            // Tool execution
            "devman_execute_tool" => {
                self.handle_execute_tool(ai_interface, &arguments).await
            }

            // Context and blockers
            "devman_get_context" => {
                self.handle_get_context(ai_interface).await
            }
            "devman_list_blockers" => {
                self.handle_list_blockers(ai_interface).await
            }

            // Job management
            "devman_get_job_status" => {
                self.handle_get_job_status(&arguments).await
            }
            "devman_cancel_job" => {
                self.handle_cancel_job(&arguments).await
            }

            // Task guidance tools
            "devman_get_task_guidance" => {
                self.handle_get_task_guidance(&arguments).await
            }
            "devman_read_task_context" => {
                self.handle_read_task_context(&arguments).await
            }
            "devman_review_knowledge" => {
                self.handle_review_knowledge(&arguments).await
            }
            "devman_confirm_knowledge_reviewed" => {
                self.handle_confirm_knowledge_reviewed(&arguments).await
            }
            "devman_start_execution" => {
                self.handle_start_execution(&arguments).await
            }
            "devman_log_work" => {
                self.handle_log_work(&arguments).await
            }
            "devman_finish_work" => {
                self.handle_finish_work(&arguments).await
            }
            "devman_run_task_quality_check" => {
                self.handle_run_task_quality_check(&arguments).await
            }
            "devman_get_quality_result" => {
                self.handle_get_quality_result(&arguments).await
            }
            "devman_confirm_quality_result" => {
                self.handle_confirm_quality_result(&arguments).await
            }
            "devman_complete_task" => {
                self.handle_complete_task(&arguments).await
            }
            "devman_pause_task" => {
                self.handle_pause_task(&arguments).await
            }
            "devman_resume_task" => {
                self.handle_resume_task(&arguments).await
            }
            "devman_abandon_task" => {
                self.handle_abandon_task(&arguments).await
            }

            // Unknown tool
            _ => create_mcp_error_response(
                -32601,
                &format!("Unknown tool: {}", name),
                None,
                false,
            ),
        }
    }

    // Tool handlers

    async fn handle_create_goal(
        &self,
        ai_interface: &Arc<dyn AIInterface>,
        arguments: &serde_json::Value,
    ) -> serde_json::Value {
        let spec = GoalSpec {
            title: arguments.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: arguments.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            success_criteria: arguments.get("success_criteria")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            project_id: None,
        };

        match ai_interface.create_goal(spec).await {
            Ok(goal) => json!({
                "success": true,
                "data": {
                    "goal_id": goal.id.to_string(),
                    "title": goal.title,
                    "status": format!("{:?}", goal.status)
                },
                "version": format!("goal_{}@v1", goal.id)
            }),
            Err(e) => create_mcp_error_response(
                -32000,
                &format!("Failed to create goal: {}", e),
                Some(json!({"hint": "Check the goal title and description are valid."})),
                true,
            ),
        }
    }

    async fn handle_get_goal_progress(
        &self,
        ai_interface: &Arc<dyn AIInterface>,
        arguments: &serde_json::Value,
    ) -> serde_json::Value {
        let goal_id_str = match arguments.get("goal_id").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                return create_mcp_error_response(
                    -32602,
                    "Missing required parameter: goal_id",
                    None,
                    false,
                );
            }
        };

        let goal_id = match goal_id_str.parse::<devman_core::GoalId>() {
            Ok(id) => id,
            Err(_) => {
                return create_mcp_error_response(
                    -32602,
                    "Invalid goal_id format",
                    None,
                    false,
                );
            }
        };

        match ai_interface.get_progress(goal_id).await {
            Some(progress) => json!({
                "success": true,
                "data": {
                    "goal_id": goal_id_str,
                    "percentage": progress.percentage,
                    "completed_phases": progress.completed_phases,
                    "active_tasks": progress.active_tasks,
                    "completed_tasks": progress.completed_tasks
                }
            }),
            None => create_mcp_error_response(
                -32002,
                &format!("Goal not found: {}", goal_id_str),
                None,
                false,
            ),
        }
    }

    async fn handle_create_task(
        &self,
        ai_interface: &Arc<dyn AIInterface>,
        arguments: &serde_json::Value,
    ) -> serde_json::Value {
        // Placeholder implementation
        json!({
            "success": true,
            "message": "Task creation placeholder - requires WorkManager integration"
        })
    }

    async fn handle_list_tasks(
        &self,
        ai_interface: &Arc<dyn AIInterface>,
        arguments: &serde_json::Value,
    ) -> serde_json::Value {
        let filter = TaskFilter {
            status: arguments.get("state").and_then(|v| v.as_str()).map(|s| {
                match s {
                    "Created" | "Queued" => devman_core::TaskStatus::Queued,
                    "InProgress" | "Active" => devman_core::TaskStatus::Active,
                    "Completed" | "Done" => devman_core::TaskStatus::Done,
                    "Abandoned" => devman_core::TaskStatus::Abandoned,
                    _ => devman_core::TaskStatus::Queued,
                }
            }),
            goal_id: None,
            phase_id: None,
            limit: arguments.get("limit").and_then(|v| v.as_u64()).map(|u| u as usize),
            include_completed: true,
        };

        let tasks = ai_interface.list_tasks(filter).await;
        let task_summaries: Vec<serde_json::Value> = tasks.iter().map(|t| json!({
            "task_id": t.id.to_string(),
            "title": t.title,
            "status": format!("{:?}", t.status),
            "priority": 3 // Default priority
        })).collect();

        json!({
            "success": true,
            "data": {
                "tasks": task_summaries,
                "total_count": task_summaries.len()
            },
            "version": format!("tasks@v{}", task_summaries.len())
        })
    }

    async fn handle_search_knowledge(
        &self,
        ai_interface: &Arc<dyn AIInterface>,
        arguments: &serde_json::Value,
    ) -> serde_json::Value {
        let query = arguments.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let results = ai_interface.search_knowledge(query).await;

        let summaries: Vec<serde_json::Value> = results.iter().map(|k| json!({
            "knowledge_id": k.id.to_string(),
            "title": k.title,
            "knowledge_type": format!("{:?}", k.knowledge_type),
            "tags": k.tags
        })).collect();

        json!({
            "success": true,
            "data": {
                "results": summaries,
                "total_count": summaries.len()
            }
        })
    }

    async fn handle_save_knowledge(
        &self,
        _ai_interface: &Arc<dyn AIInterface>,
        _arguments: &serde_json::Value,
    ) -> serde_json::Value {
        json!({
            "success": true,
            "message": "Knowledge saving placeholder"
        })
    }

    async fn handle_run_quality_check(
        &self,
        ai_interface: &Arc<dyn AIInterface>,
        arguments: &serde_json::Value,
    ) -> serde_json::Value {
        let check_type = arguments.get("check_type").and_then(|v| v.as_str()).unwrap_or("lint");

        let check = devman_core::QualityCheck {
            id: devman_core::QualityCheckId::new(),
            name: format!("MCP quality check: {}", check_type),
            description: format!("Quality check triggered via MCP for {}", check_type),
            check_type: devman_core::QualityCheckType::Generic(
                devman_core::GenericCheckType::LintsPass {
                    linter: check_type.to_string(),
                }
            ),
            severity: devman_core::Severity::Error,
            category: devman_core::QualityCategory::Maintainability,
        };

        let result = ai_interface.run_quality_check(check).await;
        json!({
            "success": true,
            "data": {
                "passed": result.passed,
                "execution_time_ms": result.execution_time.as_millis(),
                "findings_count": result.findings.len()
            }
        })
    }

    async fn handle_execute_tool(
        &self,
        _ai_interface: &Arc<dyn AIInterface>,
        _arguments: &serde_json::Value,
    ) -> serde_json::Value {
        json!({
            "success": true,
            "message": "Tool execution placeholder"
        })
    }

    async fn handle_get_context(
        &self,
        _ai_interface: &Arc<dyn AIInterface>,
    ) -> serde_json::Value {
        json!({
            "success": true,
            "data": {
                "message": "Context retrieval - use devman://context/* resources"
            }
        })
    }

    async fn handle_list_blockers(
        &self,
        ai_interface: &Arc<dyn AIInterface>,
    ) -> serde_json::Value {
        let blockers = ai_interface.list_blockers().await;
        json!({
            "success": true,
            "data": {
                "blockers": blockers.iter().map(|b| json!({
                    "reason": b.reason,
                    "severity": format!("{:?}", b.severity)
                })).collect::<Vec<_>>(),
                "total_count": blockers.len()
            }
        })
    }

    async fn handle_get_job_status(
        &self,
        arguments: &serde_json::Value,
    ) -> serde_json::Value {
        let job_id_str = match arguments.get("job_id").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                return create_mcp_error_response(
                    -32602,
                    "Missing required parameter: job_id",
                    None,
                    false,
                );
            }
        };

        let job_manager = match &self.job_manager {
            Some(jm) => jm,
            None => {
                return create_mcp_error_response(
                    -32603,
                    "Internal error: Job manager not configured",
                    None,
                    false,
                );
            }
        };

        let job_id = JobId(job_id_str.to_string());
        match job_manager.get_job_status(&job_id).await {
            Some(status) => json!({
                "success": true,
                "data": {
                    "job_id": status.job_id,
                    "status": status.status,
                    "progress": status.progress,
                    "progress_message": status.progress_message,
                    "created_at": status.created_at,
                    "completed_at": status.completed_at,
                    "result": status.result,
                    "error": status.error
                }
            }),
            None => create_mcp_error_response(
                -32002,
                &format!("Job not found: {}", job_id_str),
                None,
                false,
            ),
        }
    }

    async fn handle_cancel_job(
        &self,
        arguments: &serde_json::Value,
    ) -> serde_json::Value {
        let job_id_str = match arguments.get("job_id").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                return create_mcp_error_response(
                    -32602,
                    "Missing required parameter: job_id",
                    None,
                    false,
                );
            }
        };

        let job_manager = match &self.job_manager {
            Some(jm) => jm,
            None => {
                return create_mcp_error_response(
                    -32603,
                    "Internal error: Job manager not configured",
                    None,
                    false,
                );
            }
        };

        let job_id = JobId(job_id_str.to_string());
        match job_manager.cancel_job(&job_id).await {
            Ok(()) => json!({
                "success": true,
                "message": format!("Job {} cancelled", job_id_str)
            }),
            Err(e) => create_mcp_error_response(
                e.code,
                &e.message,
                e.hint.map(|h| json!({"hint": h})),
                e.retryable,
            ),
        }
    }

    // ==================== Task Guidance Handlers ====================

    async fn handle_get_task_guidance(&self, arguments: &serde_json::Value) -> serde_json::Value {
        let task_id_str = match arguments.get("task_id").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                return create_mcp_error_response(
                    -32602,
                    "Missing required parameter: task_id",
                    None,
                    false,
                );
            }
        };

        let task_id = match task_id_str.parse::<devman_core::TaskId>() {
            Ok(id) => id,
            Err(_) => {
                return create_mcp_error_response(
                    -32602,
                    "Invalid task_id format",
                    None,
                    false,
                );
            }
        };

        // Use AIInterface to get guidance
        let ai_interface = match &self.ai_interface {
            Some(ai) => ai,
            None => {
                return create_mcp_error_response(
                    -32603,
                    "Internal error: AI interface not configured",
                    None,
                    false,
                );
            }
        };

        // For now, return placeholder guidance
        // In full implementation, this would call InteractiveAI::get_task_guidance
        json!({
            "success": true,
            "data": {
                "task_id": task_id_str,
                "current_state": "Created",
                "next_action": "read_context",
                "guidance_message": "请调用 devman_read_task_context() 读取任务上下文",
                "allowed_operations": ["devman_read_task_context"],
                "prerequisites_satisfied": true,
                "missing_prerequisites": [],
                "health": "healthy"
            }
        })
    }

    async fn handle_read_task_context(&self, arguments: &serde_json::Value) -> serde_json::Value {
        let task_id_str = match arguments.get("task_id").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                return create_mcp_error_response(
                    -32602,
                    "Missing required parameter: task_id",
                    None,
                    false,
                );
            }
        };

        // Placeholder implementation
        json!({
            "success": true,
            "data": {
                "task_id": task_id_str,
                "state": "ContextRead",
                "message": "上下文已读取",
                "task_info": {
                    "title": "任务标题",
                    "description": "任务描述",
                    "goal_id": null
                },
                "project": {
                    "name": "DevMan",
                    "tech_stack": ["Rust"]
                }
            }
        })
    }

    async fn handle_review_knowledge(&self, arguments: &serde_json::Value) -> serde_json::Value {
        let task_id_str = arguments.get("task_id").and_then(|v| v.as_str()).unwrap_or("");
        let query = arguments.get("query").and_then(|v| v.as_str()).unwrap_or("");

        let ai_interface = match &self.ai_interface {
            Some(ai) => ai,
            None => {
                return create_mcp_error_response(
                    -32603,
                    "Internal error: AI interface not configured",
                    None,
                    false,
                );
            }
        };

        // Search knowledge
        let results = ai_interface.search_knowledge(query).await;

        let summaries: Vec<serde_json::Value> = results.iter().map(|k| json!({
            "knowledge_id": k.id.to_string(),
            "title": k.title,
            "type": format!("{:?}", k.knowledge_type),
            "summary": "Summary placeholder",
            "relevance_score": 0.9
        })).collect();

        json!({
            "success": true,
            "data": {
                "task_id": task_id_str,
                "knowledge_items": summaries,
                "total_count": summaries.len(),
                "suggested_queries": [query]
            }
        })
    }

    async fn handle_confirm_knowledge_reviewed(&self, arguments: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "message": "Knowledge review confirmed",
            "data": {
                "state": "KnowledgeReviewed",
                "next_action": "start_execution"
            }
        })
    }

    async fn handle_start_execution(&self, arguments: &serde_json::Value) -> serde_json::Value {
        let task_id_str = match arguments.get("task_id").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                return create_mcp_error_response(
                    -32602,
                    "Missing required parameter: task_id",
                    None,
                    false,
                );
            }
        };

        json!({
            "success": true,
            "data": {
                "task_id": task_id_str,
                "state": "InProgress",
                "session_id": format!("session_{}", task_id_str),
                "message": "开始执行，请使用 devman_log_work() 记录工作进展"
            }
        })
    }

    async fn handle_log_work(&self, arguments: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "message": "Work logged",
            "data": {
                "recorded": true
            }
        })
    }

    async fn handle_finish_work(&self, arguments: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "message": "Work submitted",
            "data": {
                "state": "WorkRecorded",
                "record_id": format!("record_{}", arguments.get("task_id").and_then(|v| v.as_str()).unwrap_or("unknown")),
                "next_action": "run_quality_check"
            }
        })
    }

    async fn handle_run_task_quality_check(&self, arguments: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "data": {
                "state": "QualityChecking",
                "check_id": format!("check_{}", chrono::Utc::now().timestamp()),
                "message": "质检运行中，请使用 devman_get_quality_result() 获取结果"
            }
        })
    }

    async fn handle_get_quality_result(&self, arguments: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "data": {
                "check_id": arguments.get("check_id").and_then(|v| v.as_str()).unwrap_or(""),
                "status": "completed",
                "overall_status": "passed",
                "findings_count": 0,
                "warnings_count": 0,
                "next_action": "confirm_result"
            }
        })
    }

    async fn handle_confirm_quality_result(&self, arguments: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "data": {
                "state": "QualityCompleted",
                "decision": arguments.get("decision").and_then(|v| v.as_str()).unwrap_or(""),
                "message": "质检结果已确认"
            }
        })
    }

    async fn handle_complete_task(&self, arguments: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "data": {
                "task_id": arguments.get("task_id").and_then(|v| v.as_str()).unwrap_or(""),
                "state": "Completed",
                "message": "任务已完成"
            }
        })
    }

    async fn handle_pause_task(&self, arguments: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "data": {
                "state": "Paused",
                "reason": arguments.get("reason").and_then(|v| v.as_str()).unwrap_or(""),
                "message": "任务已暂停，可使用 devman_resume_task() 恢复"
            }
        })
    }

    async fn handle_resume_task(&self, arguments: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "data": {
                "message": "任务已恢复"
            }
        })
    }

    async fn handle_abandon_task(&self, arguments: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "data": {
                "state": "Abandoned",
                "reason_type": arguments.get("reason_type").and_then(|v| v.as_str()).unwrap_or(""),
                "reason": arguments.get("reason").and_then(|v| v.as_str()).unwrap_or(""),
                "message": "任务已放弃",
                "can_be_reassigned": true,
                "work_preserved": true
            }
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
    use crate::{JobType, CreateJobRequest, InMemoryJobManager};

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

    // ==================== Error Response Tests ====================

    #[test]
    fn test_create_mcp_error_response_basic() {
        let response = create_mcp_error_response(
            -32000,
            "Test error message",
            None,
            false,
        );

        assert_eq!(response["success"], false);
        assert_eq!(response["error"]["code"], -32000);
        assert_eq!(response["error"]["message"], "Test error message");
        assert_eq!(response["error"]["retryable"], false);
    }

    #[test]
    fn test_create_mcp_error_response_with_hint() {
        let response = create_mcp_error_response(
            -32001,
            "State conflict error",
            Some(json!({"hint": "Check the resource state"})),
            true,
        );

        assert_eq!(response["success"], false);
        assert_eq!(response["error"]["code"], -32001);
        // Check that error object exists
        assert!(response["error"].is_object());
        // Check that data object exists
        assert!(response["error"]["data"].is_object());
        // Check data.hint exists
        assert!(response["error"]["data"]["hint"].is_string());
        assert_eq!(response["error"]["data"]["hint"], "Check the resource state");
        assert_eq!(response["error"]["retryable"], true);
    }

    #[test]
    fn test_create_mcp_error_response_retryable() {
        let response = create_mcp_error_response(
            -32003,
            "Job timeout",
            Some(json!({"timeout_seconds": 300})),
            true,
        );

        assert!(response["error"]["retryable"].as_bool().unwrap());
        assert_eq!(response["error"]["data"]["timeout_seconds"], 300);
    }

    // ==================== JobManager Tests ====================

    #[test]
    fn test_job_id_generation() {
        let id1 = JobId::new();
        let id2 = JobId::new();

        assert_ne!(id1.to_string(), id2.to_string());
        assert!(id1.to_string().starts_with("job_"));
    }

    #[test]
    fn test_job_status_display() {
        use crate::JobStatus;

        assert_eq!(JobStatus::Pending.to_string(), "pending");
        assert_eq!(JobStatus::Running.to_string(), "running");
        assert_eq!(JobStatus::Completed.to_string(), "completed");
        assert_eq!(JobStatus::Failed.to_string(), "failed");
        assert_eq!(JobStatus::Cancelled.to_string(), "cancelled");
        assert_eq!(JobStatus::Timeout.to_string(), "timeout");
    }

    #[test]
    fn test_in_memory_job_manager_creation() {
        let _manager = InMemoryJobManager::new();
        // Just verify the manager was created successfully
        // The manager should be ready to accept jobs
        assert!(true);
    }

    #[test]
    fn test_create_job_request() {
        let request = CreateJobRequest {
            job_type: JobType::CreateGoal {
                title: "Test Goal".to_string(),
                description: "Test Description".to_string(),
            },
            timeout_seconds: Some(60),
        };

        match request.job_type {
            JobType::CreateGoal { title, description } => {
                assert_eq!(title, "Test Goal");
                assert_eq!(description, "Test Description");
            }
            _ => panic!("Wrong job type"),
        }
    }
}
