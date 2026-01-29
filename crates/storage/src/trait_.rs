//! Storage trait abstraction.

use async_trait::async_trait;
use devman_core::{Event, KnowledgeNode, Task, TaskFilter, TaskId, EventId, NodeId};

/// Error type for storage operations.
pub type Result<T> = std::result::Result<T, StorageError>;

/// Errors that can occur during storage operations.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Git operation error
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    /// Item not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Storage abstraction for DevMan data.
///
/// This trait allows different storage backends to be plugged in.
#[async_trait]
pub trait Storage: Send + Sync {
    // === Task operations ===

    /// Save a task (create or update).
    async fn save_task(&mut self, task: &Task) -> Result<()>;

    /// Load a task by ID.
    async fn load_task(&self, id: TaskId) -> Result<Option<Task>>;

    /// List tasks matching the filter.
    async fn list_tasks(&self, filter: &TaskFilter) -> Result<Vec<Task>>;

    /// Delete a task.
    async fn delete_task(&mut self, id: TaskId) -> Result<()>;

    // === Event operations ===

    /// Save an event.
    async fn save_event(&mut self, event: &Event) -> Result<()>;

    /// Load an event by ID.
    async fn load_event(&self, id: EventId) -> Result<Option<Event>>;

    /// List all events.
    async fn list_events(&self) -> Result<Vec<Event>>;

    // === Knowledge operations ===

    /// Save a knowledge node.
    async fn save_knowledge(&mut self, node: &KnowledgeNode) -> Result<()>;

    /// Load a knowledge node by ID.
    async fn load_knowledge(&self, id: NodeId) -> Result<Option<KnowledgeNode>>;

    /// List all knowledge nodes.
    async fn list_knowledge(&self) -> Result<Vec<KnowledgeNode>>;

    // === Transaction support ===

    /// Commit pending changes with a message.
    async fn commit(&mut self, message: &str) -> Result<()>;

    /// Rollback pending changes.
    async fn rollback(&mut self) -> Result<()>;
}
