//! Task model - the core unit of work in DevMan.

use crate::{TaskId, Time};
use serde::{Deserialize, Serialize};

/// A task represents a unit of work with intent and hypothesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier
    pub id: TaskId,

    /// Why this task exists
    pub intent: String,

    /// What change is expected
    pub hypothesis: String,

    /// Current status
    pub status: TaskStatus,

    /// Confidence in success (0-1)
    pub confidence: f32,

    /// Priority (0-255, higher = more important)
    pub priority: u8,

    /// Relationships to other tasks
    pub links: Vec<TaskLink>,

    /// Associated event log
    pub logs: Vec<crate::EventId>,

    /// Creation timestamp
    pub created_at: Time,

    /// Last update timestamp
    pub updated_at: Time,
}

impl Task {
    /// Create a new task with the given intent and hypothesis.
    pub fn new(intent: impl Into<String>, hypothesis: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: TaskId::new(),
            intent: intent.into(),
            hypothesis: hypothesis.into(),
            status: TaskStatus::Idea,
            confidence: 0.5,
            priority: 128,
            links: Vec::new(),
            logs: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// The state machine for task lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Initial idea, not yet queued
    Idea,
    /// Queued for execution
    Queued,
    /// Currently being executed
    Active,
    /// Blocked by dependencies
    Blocked,
    /// Awaiting review/reflection
    Review,
    /// Completed successfully
    Done,
    /// Abandoned (won't be pursued)
    Abandoned,
}

/// A relationship between two tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLink {
    /// The target task
    pub to: TaskId,

    /// Type of relationship
    pub kind: LinkKind,
}

/// Types of relationships between tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkKind {
    /// This task depends on `to` completing first
    DependsOn,
    /// This task blocks `to` from starting
    Blocks,
    /// Loosely related tasks
    RelatedTo,
    /// This task was derived from `to`
    DerivesFrom,
}

/// Filter for querying tasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskFilter {
    /// Filter by status
    pub status: Option<Vec<TaskStatus>>,
    /// Filter by minimum priority
    pub min_priority: Option<u8>,
    /// Filter by minimum confidence
    pub min_confidence: Option<f32>,
}
