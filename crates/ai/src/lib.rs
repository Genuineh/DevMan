//! AI Interface (MCP Server)
//!
//! High-level API for AI assistants to interact with DevMan.

#![warn(missing_docs)]

pub mod r#interface;
pub mod interactive;
pub mod mcp_server;

pub use r#interface::AIInterface;
pub use interactive::{InteractiveAI, BasicInteractiveAI};
