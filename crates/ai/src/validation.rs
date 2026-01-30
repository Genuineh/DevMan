//! Task state validation and transition logic.

use devman_core::{TaskState, TaskId, StateTransition};

/// Context for state transitions.
pub struct TransitionContext {
    pub caller: String,
    pub permissions: Vec<String>,
    pub reason: Option<String>,
}

impl TransitionContext {
    pub fn new(caller: impl Into<String>) -> Self {
        Self {
            caller: caller.into(),
            permissions: vec![],
            reason: None,
        }
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn can_abandon(&self) -> bool {
        self.permissions.contains(&"abandon".to_string()) || self.permissions.contains(&"*".to_string())
    }

    pub fn has_cancel_permission(&self) -> bool {
        self.permissions.contains(&"cancel".to_string()) || self.permissions.contains(&"*".to_string())
    }

    pub fn has_goal_change_permission(&self) -> bool {
        self.permissions.contains(&"change_goal".to_string()) || self.permissions.contains(&"*".to_string())
    }
}

/// Task state validator.
pub struct TaskStateValidator;

impl TaskStateValidator {
    /// Validate a state transition.
    pub fn validate_transition(
        current: &TaskState,
        new_state: &TaskState,
        context: &TransitionContext,
    ) -> StateTransition {
        match (current, new_state) {
            // Normal flow: Created → ContextRead
            (TaskState::Created { .. }, TaskState::ContextRead { .. }) => {
                StateTransition::Allowed
            }

            // Normal flow: ContextRead → KnowledgeReviewed
            (TaskState::ContextRead { .. }, TaskState::KnowledgeReviewed { .. }) => {
                StateTransition::Allowed
            }

            // Normal flow: KnowledgeReviewed → InProgress
            (TaskState::KnowledgeReviewed { .. }, TaskState::InProgress { .. }) => {
                StateTransition::Allowed
            }

            // Normal flow: InProgress → WorkRecorded
            (TaskState::InProgress { .. }, TaskState::WorkRecorded { .. }) => {
                StateTransition::Allowed
            }

            // Normal flow: WorkRecorded → QualityChecking
            (TaskState::WorkRecorded { .. }, TaskState::QualityChecking { .. }) => {
                StateTransition::Allowed
            }

            // Normal flow: QualityChecking → QualityCompleted
            (TaskState::QualityChecking { .. }, TaskState::QualityCompleted { .. }) => {
                StateTransition::Allowed
            }

            // Normal flow: QualityCompleted (passed) → Completed
            (TaskState::QualityCompleted { result, .. }, TaskState::Completed { .. }) => {
                if matches!(result.overall_status, devman_core::TaskQualityOverallStatus::Passed) {
                    StateTransition::Allowed
                } else {
                    StateTransition::RejectedRequiredAction {
                        action: "修复质检问题".to_string(),
                        guidance: "质检未通过，请修复问题后重新质检或放弃任务".to_string(),
                    }
                }
            }

            // Fix cycle: QualityCompleted → InProgress
            (TaskState::QualityCompleted { .. }, TaskState::InProgress { .. }) => {
                StateTransition::Allowed
            }

            // Pause: Any pausable state → Paused
            (state, TaskState::Paused { .. }) if state.can_be_paused() => {
                StateTransition::Allowed
            }

            // Resume: Paused → back to previous state
            (TaskState::Paused { previous_state, .. }, new_state) => {
                // Check if new_state matches the previous state by comparing discriminants
                let is_same_state = match (&**previous_state, new_state) {
                    (TaskState::Created { .. }, TaskState::Created { .. }) => true,
                    (TaskState::ContextRead { .. }, TaskState::ContextRead { .. }) => true,
                    (TaskState::KnowledgeReviewed { .. }, TaskState::KnowledgeReviewed { .. }) => true,
                    (TaskState::InProgress { .. }, TaskState::InProgress { .. }) => true,
                    (TaskState::WorkRecorded { .. }, TaskState::WorkRecorded { .. }) => true,
                    (TaskState::QualityChecking { .. }, TaskState::QualityChecking { .. }) => true,
                    (TaskState::QualityCompleted { .. }, TaskState::QualityCompleted { .. }) => true,
                    _ => false,
                };

                if is_same_state {
                    StateTransition::Allowed
                } else {
                    StateTransition::RejectedRequiredAction {
                        action: format!("恢复到 {:?}", new_state),
                        guidance: format!("只能恢复到暂停前的状态: {:?}", previous_state),
                    }
                }
            }

            // Abandon: Any state → Abandoned
            (_, TaskState::Abandoned { .. }) => {
                if context.can_abandon() {
                    StateTransition::Allowed
                } else {
                    StateTransition::RejectedRequiredAction {
                        action: "放弃任务".to_string(),
                        guidance: "放弃任务需要提供详细原因".to_string(),
                    }
                }
            }

            // Invalid transitions
            (current, new) => StateTransition::RejectedRequiredAction {
                action: format!("{:?} → {:?}", current, new),
                guidance: Self::get_guidance_for_state(current),
            },
        }
    }

    fn get_guidance_for_state(state: &TaskState) -> String {
        state.get_guidance().to_string()
    }
}

/// Work log storage for tracking task progress.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkLogStorage {
    pub task_id: TaskId,
    pub logs: Vec<WorkLogEntry>,
    pub created_at: devman_core::Time,
    pub updated_at: devman_core::Time,
}

/// Work log entry stored in persistence layer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkLogEntry {
    pub timestamp: devman_core::Time,
    pub action: String,
    pub description: String,
    pub files: Vec<String>,
    pub command_output: Option<CommandExecutionRecord>,
}

/// Command execution record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandExecutionRecord {
    pub command: String,
    pub args: Vec<String>,
    pub exit_code: i32,
    pub output: String,
    pub timestamp: devman_core::Time,
}

#[cfg(test)]
mod tests {
    use super::*;
    use devman_core::{KnowledgeId, AbandonReason, TaskQualityCheckResult, TaskQualityOverallStatus};
    use chrono::Utc;

    fn make_context(caller: &str) -> TransitionContext {
        TransitionContext::new(caller).with_permissions(vec!["*".to_string()])
    }

    #[test]
    fn test_normal_flow_transitions() {
        let context = make_context("test_ai");

        // Created → ContextRead
        let created = TaskState::Created {
            created_at: Utc::now(),
            created_by: "test".to_string(),
        };
        let context_read = TaskState::ContextRead {
            read_at: Utc::now(),
        };
        assert!(matches!(
            TaskStateValidator::validate_transition(&created, &context_read, &context),
            StateTransition::Allowed
        ));

        // ContextRead → KnowledgeReviewed
        let knowledge_reviewed = TaskState::KnowledgeReviewed {
            knowledge_ids: vec![KnowledgeId::new()],
            reviewed_at: Utc::now(),
        };
        assert!(matches!(
            TaskStateValidator::validate_transition(&context_read, &knowledge_reviewed, &context),
            StateTransition::Allowed
        ));

        // KnowledgeReviewed → InProgress
        let in_progress = TaskState::InProgress {
            started_at: Utc::now(),
            checkpoint: None,
        };
        assert!(matches!(
            TaskStateValidator::validate_transition(&knowledge_reviewed, &in_progress, &context),
            StateTransition::Allowed
        ));
    }

    #[test]
    fn test_skip_context_read_rejected() {
        let context = make_context("test_ai");

        let created = TaskState::Created {
            created_at: Utc::now(),
            created_by: "test".to_string(),
        };
        let in_progress = TaskState::InProgress {
            started_at: Utc::now(),
            checkpoint: None,
        };

        let result = TaskStateValidator::validate_transition(&created, &in_progress, &context);
        assert!(matches!(result, StateTransition::RejectedRequiredAction { .. }));
    }

    #[test]
    fn test_pause_and_resume() {
        let context = make_context("test_ai");

        let in_progress = TaskState::InProgress {
            started_at: Utc::now(),
            checkpoint: None,
        };

        // Pause
        let paused = TaskState::Paused {
            paused_at: Utc::now(),
            reason: "testing".to_string(),
            previous_state: Box::new(in_progress.clone()),
        };
        assert!(matches!(
            TaskStateValidator::validate_transition(&in_progress, &paused, &context),
            StateTransition::Allowed
        ));

        // Resume
        assert!(matches!(
            TaskStateValidator::validate_transition(&paused, &in_progress, &context),
            StateTransition::Allowed
        ));
    }

    #[test]
    fn test_quality_passed_to_completed() {
        let context = make_context("test_ai");

        let quality_completed = TaskState::QualityCompleted {
            result: TaskQualityCheckResult {
                overall_status: TaskQualityOverallStatus::Passed,
                findings_count: 0,
                warnings_count: 0,
            },
            completed_at: Utc::now(),
        };

        let completed = TaskState::Completed {
            completed_at: Utc::now(),
            completed_by: "test".to_string(),
        };

        assert!(matches!(
            TaskStateValidator::validate_transition(&quality_completed, &completed, &context),
            StateTransition::Allowed
        ));
    }

    #[test]
    fn test_quality_failed_cannot_complete() {
        let context = make_context("test_ai");

        let quality_completed = TaskState::QualityCompleted {
            result: TaskQualityCheckResult {
                overall_status: TaskQualityOverallStatus::Failed,
                findings_count: 1,
                warnings_count: 0,
            },
            completed_at: Utc::now(),
        };

        let completed = TaskState::Completed {
            completed_at: Utc::now(),
            completed_by: "test".to_string(),
        };

        let result = TaskStateValidator::validate_transition(&quality_completed, &completed, &context);
        assert!(matches!(result, StateTransition::RejectedRequiredAction { .. }));
    }

    #[test]
    fn test_abandon_without_permission_rejected() {
        let context = TransitionContext::new("test_ai"); // No permissions

        let in_progress = TaskState::InProgress {
            started_at: Utc::now(),
            checkpoint: None,
        };

        let abandoned = TaskState::Abandoned {
            abandoned_at: Utc::now(),
            reason: AbandonReason::Other {
                reason: "test".to_string(),
                details: None,
            },
        };

        let result = TaskStateValidator::validate_transition(&in_progress, &abandoned, &context);
        assert!(matches!(result, StateTransition::RejectedRequiredAction { .. }));
    }
}
