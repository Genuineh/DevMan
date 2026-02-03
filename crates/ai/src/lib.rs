//! AI Interface (MCP Server)
//!
//! High-level API for AI assistants to interact with DevMan.

#![warn(missing_docs)]

pub mod r#interface;
pub mod interactive;
pub mod validation;
pub mod guidance;
pub mod mcp_server;
pub mod job_manager;

pub use r#interface::{AIInterface, GoalSpec, GoalFilter, TaskFilter};
pub use interactive::{InteractiveAI, BasicInteractiveAI};
pub use validation::{TaskStateValidator, TransitionContext, WorkLogStorage, WorkLogEntry, CommandExecutionRecord};
pub use guidance::{TaskGuidanceGenerator, TaskGuidanceInfo, GuidanceContext};
pub use job_manager::{JobManager, InMemoryJobManager, JobId, Job, JobStatus, JobType, JobError, JobStatusResponse, CreateJobRequest, JobFilter, error_codes};
pub use mcp_server::{McpServer, McpServerConfig, McpTool, McpResource, MCP_VERSION};
