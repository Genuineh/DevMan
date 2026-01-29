//! Storage trait abstraction.

use async_trait::async_trait;
use devman_core::{
    Goal, GoalId, Project, ProjectId, Phase, PhaseId, Task, TaskId, TaskFilter,
    Event, EventId, Knowledge, KnowledgeId, QualityCheck, QualityCheckId,
    WorkRecord, WorkRecordId, Blocker, BlockerId,
};

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
    // === Goal operations ===

    /// Save a goal.
    async fn save_goal(&mut self, goal: &Goal) -> Result<()>;

    /// Load a goal by ID.
    async fn load_goal(&self, id: GoalId) -> Result<Option<Goal>>;

    /// List all goals.
    async fn list_goals(&self) -> Result<Vec<Goal>>;

    // === Project operations ===

    /// Save a project.
    async fn save_project(&mut self, project: &Project) -> Result<()>;

    /// Load a project by ID.
    async fn load_project(&self, id: ProjectId) -> Result<Option<Project>>;

    // === Phase operations ===

    /// Save a phase.
    async fn save_phase(&mut self, phase: &Phase) -> Result<()>;

    /// Load a phase by ID.
    async fn load_phase(&self, id: PhaseId) -> Result<Option<Phase>>;

    // === Task operations ===

    /// Save a task.
    async fn save_task(&mut self, task: &Task) -> Result<()>;

    /// Load a task by ID.
    async fn load_task(&self, id: TaskId) -> Result<Option<Task>>;

    /// List tasks with optional filter.
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

    /// Save knowledge.
    async fn save_knowledge(&mut self, knowledge: &Knowledge) -> Result<()>;

    /// Load knowledge by ID.
    async fn load_knowledge(&self, id: KnowledgeId) -> Result<Option<Knowledge>>;

    /// List all knowledge.
    async fn list_knowledge(&self) -> Result<Vec<Knowledge>>;

    // === Quality Check operations ===

    /// Save a quality check.
    async fn save_quality_check(&mut self, check: &QualityCheck) -> Result<()>;

    /// Load a quality check by ID.
    async fn load_quality_check(&self, id: QualityCheckId) -> Result<Option<QualityCheck>>;

    /// List all quality checks.
    async fn list_quality_checks(&self) -> Result<Vec<QualityCheck>>;

    // === Work Record operations ===

    /// Save a work record.
    async fn save_work_record(&mut self, record: &WorkRecord) -> Result<()>;

    /// Load a work record by ID.
    async fn load_work_record(&self, id: WorkRecordId) -> Result<Option<WorkRecord>>;

    /// List work records for a task.
    async fn list_work_records(&self, task_id: TaskId) -> Result<Vec<WorkRecord>>;

    // === Transaction support ===

    /// Commit pending changes with a message.
    async fn commit(&mut self, message: &str) -> Result<()>;

    /// Rollback pending changes.
    async fn rollback(&mut self) -> Result<()>;
}

/// A transaction for atomic operations.
pub struct Transaction {
    // Placeholder for transaction support
    _private: (),
}
