//! Work record model - detailed execution log.

use serde::{Deserialize, Serialize};
use crate::id::{WorkRecordId, TaskId, IssueId, BlockerId, GoalId, PhaseId};
use crate::Time;

/// A work record is a detailed log of task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkRecord {
    /// Unique identifier
    pub id: WorkRecordId,

    /// Associated task
    pub task_id: TaskId,

    /// Who executed this work
    pub executor: Executor,

    /// When started
    pub started_at: Time,

    /// When completed
    pub completed_at: Option<Time>,

    /// Duration
    pub duration: Option<chrono::Duration>,

    /// Timeline of events
    pub events: Vec<WorkEvent>,

    /// Final result
    pub result: WorkResult,

    /// Generated artifacts
    pub artifacts: Vec<Artifact>,

    /// Issues encountered
    pub issues: Vec<Issue>,

    /// Resolutions applied
    pub resolutions: Vec<Resolution>,
}

/// Who/what executed the work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Executor {
    AI { model: String },
    Human { name: String },
    Hybrid { ai: String, human: String },
}

/// An event in the work timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkEvent {
    /// When it happened
    pub timestamp: Time,

    /// Event type
    pub event_type: WorkEventType,

    /// Description
    pub description: String,

    /// Associated data
    pub data: serde_json::Value,
}

/// Types of work events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkEventType {
    StepStarted,
    StepCompleted,
    StepFailed,
    QualityCheckStarted,
    QualityCheckPassed,
    QualityCheckFailed,
    IssueDiscovered,
    IssueResolved,
    KnowledgeCreated,
}

/// The result of work execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkResult {
    /// Completion status
    pub status: CompletionStatus,

    /// Outputs produced
    pub outputs: Vec<Output>,

    /// Execution metrics
    pub metrics: WorkMetrics,
}

/// Overall completion status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompletionStatus {
    Running,
    Success,
    Failed,
    Cancelled,
}

/// An output artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    /// Output name
    pub name: String,

    /// Output value
    pub value: String,
}

/// Execution metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkMetrics {
    /// Tokens used (for AI execution)
    pub token_used: Option<usize>,

    /// Time spent
    pub time_spent: std::time::Duration,

    /// Tools invoked
    pub tools_invoked: usize,

    /// Quality checks run
    pub quality_checks_run: usize,

    /// Quality checks passed
    pub quality_checks_passed: usize,
}

/// An artifact generated during work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact name
    pub name: String,

    /// Artifact type
    pub artifact_type: String,

    /// Path/URL
    pub location: String,
}

/// An issue encountered during work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue ID
    pub id: IssueId,

    /// Description
    pub description: String,

    /// Severity
    pub severity: Severity,

    /// When discovered
    pub discovered_at: Time,

    /// Resolved
    pub resolved: bool,
}

/// A resolution applied to an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    /// Issue ID
    pub issue_id: IssueId,

    /// Description
    pub description: String,

    /// How it was resolved
    pub resolution_type: ResolutionType,

    /// When applied
    pub applied_at: Time,
}

/// How an issue was resolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionType {
    Fixed,
    Workaround,
    Deferred,
    Ignored,
}

/// Severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Blocker for progress tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blocker {
    /// Unique identifier
    pub id: BlockerId,

    /// What's blocked
    pub blocked_item: BlockedItem,

    /// Why it's blocked
    pub reason: String,

    /// Severity
    pub severity: Severity,

    /// When created
    pub created_at: Time,

    /// When resolved
    pub resolved_at: Option<Time>,
}

/// What is being blocked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockedItem {
    Task(TaskId),
    Phase(PhaseId),
    Goal(GoalId),
}
