//! Tool Integration
//!
//! Execute external tools (cargo, npm, git, etc.) to reduce token usage.

#![warn(missing_docs)]

pub mod r#trait;
pub mod builtin;

pub use r#trait::{Tool, ToolExecutor, ToolInput, ToolOutput, ToolSchema};
pub use builtin::{CargoTool, NpmTool, GitTool, FsTool};
