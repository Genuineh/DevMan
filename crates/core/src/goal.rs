//! Goal model - top-level objective with progress tracking.

use serde::{Deserialize, Serialize};
use crate::id::{GoalId, ProjectId, PhaseId, CriterionId, QualityCheckId};
use crate::Time;
use crate::work_record::Blocker;

/// A goal represents the top-level objective AI wants to achieve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    /// Unique identifier
    pub id: GoalId,

    /// Goal title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Success criteria for this goal
    pub success_criteria: Vec<SuccessCriterion>,

    /// Progress tracking
    pub progress: GoalProgress,

    /// Associated project
    pub project_id: ProjectId,

    /// Current phase
    pub current_phase: PhaseId,

    /// Goal status
    pub status: GoalStatus,

    /// When created
    pub created_at: Time,

    /// Last updated
    pub updated_at: Time,
}

/// Goal status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalStatus {
    /// Goal is active
    Active,
    /// Goal completed
    Completed,
    /// Goal paused
    Paused,
    /// Goal cancelled
    Cancelled,
}

/// A success criterion for a goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    /// Unique identifier
    pub id: CriterionId,

    /// Description
    pub description: String,

    /// How to verify this criterion
    pub verification: VerificationMethod,

    /// Current status
    pub status: CriterionStatus,
}

/// How to verify a success criterion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    /// Automated verification via quality check
    Automated(QualityCheckId),

    /// Manual verification by human
    Manual { reviewer: String },

    /// Hybrid: automated + human review
    Hybrid {
        automated: QualityCheckId,
        reviewer: String,
    },
}

/// Criterion status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CriterionStatus {
    NotStarted,
    InProgress,
    Met,
    NotMet,
}

/// Progress tracking for a goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProgress {
    /// Percentage complete (0-100)
    pub percentage: f32,

    /// Completed phases
    pub completed_phases: Vec<PhaseId>,

    /// Active (non-completed) tasks
    pub active_tasks: usize,

    /// Completed tasks
    pub completed_tasks: usize,

    /// Estimated completion time
    pub estimated_completion: Option<Time>,

    /// Current blockers
    pub blockers: Vec<Blocker>,
}


impl Default for GoalProgress {
    fn default() -> Self {
        Self {
            percentage: 0.0,
            completed_phases: Vec::new(),
            active_tasks: 0,
            completed_tasks: 0,
            estimated_completion: None,
            blockers: Vec::new(),
        }
    }
}
