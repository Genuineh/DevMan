//! Tool abstraction.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A tool that can be executed.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get tool name.
    fn name(&self) -> &str;

    /// Get tool description.
    fn description(&self) -> &str;

    /// Execute the tool.
    async fn execute(&self, input: &ToolInput) -> Result<ToolOutput, anyhow::Error>;

    /// Get tool schema (for AI discovery).
    fn schema(&self) -> ToolSchema;
}

/// Tool executor - runs tools by name.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool by name.
    async fn execute_tool(&self, tool: &str, input: ToolInput) -> Result<ToolOutput, anyhow::Error>;
}

/// Input to a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInput {
    /// Command arguments
    pub args: Vec<String>,

    /// Environment variables
    pub env: HashMap<String, String>,

    /// Standard input
    pub stdin: Option<String>,

    /// Timeout
    pub timeout: Option<std::time::Duration>,
}

/// Output from a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Exit code
    pub exit_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Execution duration
    pub duration: std::time::Duration,
}

/// Tool schema for AI discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Tool name
    pub name: String,

    /// Description
    pub description: String,

    /// Parameters
    pub parameters: Vec<Parameter>,

    /// Usage examples
    pub examples: Vec<Example>,
}

/// A tool parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,

    /// Description
    pub description: String,

    /// Type
    pub param_type: String,

    /// Required
    pub required: bool,

    /// Default value
    pub default: Option<serde_json::Value>,
}

/// A usage example.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// Example description
    pub description: String,

    /// Example input
    pub input: ToolInput,
}
