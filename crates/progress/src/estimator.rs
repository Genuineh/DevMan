//! Completion time estimation.

use chrono::{DateTime, Utc, Duration};
use devman_core::{Goal, GoalId, Phase, PhaseId, Task};

/// Completion time estimator.
pub struct CompletionEstimator;

impl CompletionEstimator {
    /// Estimate goal completion time.
    pub fn estimate_goal(&self, goal: &Goal) -> Option<DateTime<Utc>> {
        // Simple estimation: current time + average remaining duration
        let remaining = goal.progress.active_tasks as i64;
        if remaining == 0 {
            return Some(Utc::now());
        }

        // Assume 1 day per remaining task (very rough)
        let days = remaining * 24;
        Some(Utc::now() + Duration::hours(days))
    }

    /// Estimate phase completion time.
    pub fn estimate_phase(&self, phase: &Phase) -> Option<DateTime<Utc>> {
        let remaining = phase.progress.total_tasks - phase.progress.completed_tasks;
        if remaining == 0 {
            return Some(Utc::now());
        }

        let hours = remaining as i64 * 8; // 8 hours per task
        Some(Utc::now() + Duration::hours(hours))
    }

    /// Estimate task completion time.
    pub fn estimate_task(&self, task: &Task) -> Option<DateTime<Utc>> {
        if matches!(
            task.status,
            devman_core::TaskStatus::Done | devman_core::TaskStatus::Abandoned
        ) {
            return Some(task.updated_at);
        }

        // Assume 4 hours for incomplete tasks
        Some(Utc::now() + Duration::hours(4))
    }
}

impl Default for CompletionEstimator {
    fn default() -> Self {
        Self
    }
}
