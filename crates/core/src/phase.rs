//! Phase model - goal stages with acceptance criteria.

use serde::{Deserialize, Serialize};
use crate::id::{PhaseId, TaskId, QualityCheckId};
use crate::Time;

/// A phase is a stage of a project with specific objectives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    /// Unique identifier
    pub id: PhaseId,

    /// Phase name
    pub name: String,

    /// Description
    pub description: String,

    /// Phase objectives
    pub objectives: Vec<String>,

    /// Acceptance criteria
    pub acceptance_criteria: Vec<AcceptanceCriterion>,

    /// Tasks in this phase
    pub tasks: Vec<TaskId>,

    /// Phase dependencies
    pub depends_on: Vec<PhaseId>,

    /// Phase status
    pub status: PhaseStatus,

    /// Progress
    pub progress: PhaseProgress,

    /// Estimated duration
    pub estimated_duration: Option<std::time::Duration>,

    /// Actual duration
    pub actual_duration: Option<std::time::Duration>,

    /// Created at
    pub created_at: Time,
}

/// Phase status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
    Cancelled,
}

/// Acceptance criterion for a phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    /// Description
    pub description: String,

    /// Required quality checks
    pub quality_checks: Vec<QualityCheckId>,
}

/// Progress tracking for a phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseProgress {
    /// Completed tasks
    pub completed_tasks: usize,

    /// Total tasks
    pub total_tasks: usize,

    /// Percentage complete
    pub percentage: f32,
}

impl Default for PhaseProgress {
    fn default() -> Self {
        Self {
            completed_tasks: 0,
            total_tasks: 0,
            percentage: 0.0,
        }
    }
}
