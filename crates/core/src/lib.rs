//! DevMan core data models.
//!
//! This crate defines the fundamental data structures that power the
//! AI execution cognitive framework.

#![warn(missing_docs, unused_crate_dependencies)]

mod task;
mod event;
mod knowledge;
mod id;

pub use task::{Task, TaskStatus, TaskLink, LinkKind, TaskFilter};
pub use event::{Event, AgentId};
pub use knowledge::{KnowledgeNode, KnowledgeUpdate};
pub use id::{TaskId, EventId, NodeId};

/// Timestamp type
pub type Time = chrono::DateTime<chrono::Utc>;
