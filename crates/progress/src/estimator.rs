//! Completion time estimation for AI workflows.
//!
//! Provides AI-friendly time estimation with minute-level precision:
//! - Based on task complexity and execution steps
//! - Progress-based refinement
//! - Phase and goal aggregation

use chrono::{DateTime, Utc, Duration};
use devman_core::{Goal, Phase, Task, TaskStatus};

/// AI-friendly completion estimation result.
#[derive(Debug, Clone)]
pub struct TimeEstimation {
    /// Estimated completion time
    pub estimated_completion: DateTime<Utc>,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Estimated duration in minutes
    pub duration_minutes: i64,
    /// Factors that influenced the estimation
    pub factors: Vec<String>,
}

/// Task complexity scoring for AI tasks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskComplexity {
    /// Trivial task, few tokens
    Trivial,
    /// Simple task, straightforward
    Simple,
    /// Moderate complexity
    Moderate,
    /// Complex task with multiple steps
    Complex,
    /// Very complex, may take significant time
    VeryComplex,
}

impl TaskComplexity {
    /// Get base duration in minutes for this complexity level.
    pub fn base_minutes(&self) -> i64 {
        match self {
            TaskComplexity::Trivial => 5,
            TaskComplexity::Simple => 15,
            TaskComplexity::Moderate => 30,
            TaskComplexity::Complex => 60,
            TaskComplexity::VeryComplex => 120,
        }
    }

    /// Get confidence modifier for this complexity level.
    pub fn confidence_modifier(&self) -> f32 {
        match self {
            TaskComplexity::Trivial => 0.95,
            TaskComplexity::Simple => 0.85,
            TaskComplexity::Moderate => 0.75,
            TaskComplexity::Complex => 0.60,
            TaskComplexity::VeryComplex => 0.45,
        }
    }
}

/// Completion time estimator for AI workflows.
#[derive(Clone, Default)]
pub struct CompletionEstimator;

impl CompletionEstimator {
    /// Base duration per step in minutes (AI is fast at execution).
    const MINUTES_PER_STEP: i64 = 2;

    /// Estimate goal completion time with minute precision.
    pub fn estimate_goal(&self, goal: &Goal) -> TimeEstimation {
        let active_tasks = goal.progress.active_tasks;

        if active_tasks == 0 {
            return TimeEstimation {
                estimated_completion: Utc::now(),
                confidence: 1.0,
                duration_minutes: 0,
                factors: vec!["Goal completed".to_string()],
            };
        }

        // Estimate based on task count and complexity
        let avg_complexity = TaskComplexity::Moderate;
        let base_minutes = avg_complexity.base_minutes();

        let total_minutes = base_minutes * active_tasks as i64;
        let confidence = avg_complexity.confidence_modifier();

        let factors = vec![
            format!("Active tasks: {}", active_tasks),
            format!("Average complexity: {:?}", avg_complexity),
        ];

        TimeEstimation {
            estimated_completion: Utc::now() + Duration::minutes(total_minutes),
            confidence,
            duration_minutes: total_minutes,
            factors,
        }
    }

    /// Estimate phase completion time with minute precision.
    pub fn estimate_phase(&self, phase: &Phase) -> TimeEstimation {
        let remaining = phase.progress.total_tasks - phase.progress.completed_tasks;

        if remaining == 0 {
            return TimeEstimation {
                estimated_completion: Utc::now(),
                confidence: 1.0,
                duration_minutes: 0,
                factors: vec!["Phase completed".to_string()],
            };
        }

        // Estimate based on task count
        let avg_minutes = 30; // 30 minutes per task on average for AI
        let total_minutes = avg_minutes * remaining as i64;

        let factors = vec![
            format!("Remaining tasks: {}", remaining),
            format!("Est. {} min/task", avg_minutes),
        ];

        TimeEstimation {
            estimated_completion: Utc::now() + Duration::minutes(total_minutes),
            confidence: 0.75,
            duration_minutes: total_minutes,
            factors,
        }
    }

    /// Estimate task completion time with minute precision.
    pub fn estimate_task(&self, task: &Task) -> TimeEstimation {
        // Already done or abandoned
        if matches!(
            task.status,
            TaskStatus::Done | TaskStatus::Abandoned
        ) {
            return TimeEstimation {
                estimated_completion: task.updated_at,
                confidence: 1.0,
                duration_minutes: 0,
                factors: vec!["Task completed".to_string()],
            };
        }

        // Calculate complexity based on multiple factors
        let complexity = self.calculate_task_complexity(task);
        let mut minutes = complexity.base_minutes();

        // Adjust for number of execution steps
        let step_count = task.steps.len();
        if step_count > 0 {
            let step_minutes = step_count as i64 * Self::MINUTES_PER_STEP;
            minutes = minutes.max(step_minutes);
        }

        // Adjust for dependencies (blocking adds uncertainty)
        let dep_factor = task.depends_on.len() as f32 * 0.1;
        minutes = (minutes as f32 * (1.0 + dep_factor)) as i64;

        // Adjust based on current progress
        let progress_factor = match task.progress.percentage {
            p if p > 75.0 => 0.3, // Near completion, quick finish
            p if p > 50.0 => 0.5,
            p if p > 25.0 => 0.7,
            _ => 1.0,
        };
        minutes = (minutes as f32 * progress_factor) as i64;

        let confidence = complexity.confidence_modifier()
            * (1.0 - (task.depends_on.len() as f32 * 0.05))
            .clamp(0.3, 1.0);

        let factors = vec![
            format!("Complexity: {:?}", complexity),
            format!("Steps: {}", step_count),
            format!("Dependencies: {}", task.depends_on.len()),
            format!("Progress: {:.0}%", task.progress.percentage),
        ];

        TimeEstimation {
            estimated_completion: Utc::now() + Duration::minutes(minutes),
            confidence,
            duration_minutes: minutes,
            factors,
        }
    }

    /// Estimate task complexity based on task characteristics.
    fn calculate_task_complexity(&self, task: &Task) -> TaskComplexity {
        // Base complexity on step count
        let step_count = task.steps.len();

        if step_count <= 2 && task.depends_on.is_empty() {
            TaskComplexity::Trivial
        } else if step_count <= 5 && task.depends_on.len() <= 1 {
            TaskComplexity::Simple
        } else if step_count <= 10 && task.depends_on.len() <= 2 {
            TaskComplexity::Moderate
        } else if step_count <= 20 && task.depends_on.len() <= 3 {
            TaskComplexity::Complex
        } else {
            TaskComplexity::VeryComplex
        }
    }

    /// Format duration in human-readable format.
    pub fn format_duration(&self, minutes: i64) -> String {
        if minutes < 60 {
            format!("{}m", minutes)
        } else if minutes < 1440 {
            let hours = minutes / 60;
            let mins = minutes % 60;
            if mins > 0 {
                format!("{}h {}m", hours, mins)
            } else {
                format!("{}h", hours)
            }
        } else {
            let days = minutes / 1440;
            let hours = (minutes % 1440) / 60;
            if hours > 0 {
                format!("{}d {}h", days, hours)
            } else {
                format!("{}d", days)
            }
        }
    }

    /// Format confidence as percentage string.
    pub fn format_confidence(&self, confidence: f32) -> String {
        format!("{:.0}%", confidence * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devman_core::TaskContext;
    use chrono::Utc;

    fn create_test_task_with_steps(id: devman_core::TaskId, title: &str, step_count: usize, deps: usize) -> Task {
        let steps_vec: Vec<devman_core::ExecutionStep> = (0..step_count)
            .map(|i| devman_core::ExecutionStep {
                order: i,
                description: format!("Step {}", i),
                tool: devman_core::ToolInvocation {
                    tool: format!("tool_{}", i),
                    args: vec![],
                    env: vec![],
                    timeout: None,
                },
                verify: None,
            })
            .collect();

        let dep_vec: Vec<devman_core::TaskId> = (0..deps)
            .map(|_| devman_core::TaskId::new())
            .collect();

        Task {
            id,
            title: title.to_string(),
            description: "Test task".to_string(),
            intent: devman_core::TaskIntent {
                natural_language: format!("Test task: {}", title),
                context: TaskContext {
                    relevant_knowledge: vec![],
                    similar_tasks: vec![],
                    affected_files: vec![],
                },
                success_criteria: vec![],
            },
            status: TaskStatus::Active,
            steps: steps_vec,
            inputs: vec![],
            expected_outputs: vec![],
            quality_gates: vec![],
            progress: devman_core::TaskProgress {
                percentage: 0.0,
                current_step: Some(0),
                total_steps: step_count,
                message: "In progress".to_string(),
            },
            phase_id: devman_core::PhaseId::new(),
            depends_on: dep_vec,
            blocks: vec![],
            work_records: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_task_complexity_trivial() {
        let task = create_test_task_with_steps(devman_core::TaskId::new(), "trivial", 1, 0);
        let estimator = CompletionEstimator::default();
        let complexity = estimator.calculate_task_complexity(&task);
        assert_eq!(complexity, TaskComplexity::Trivial);
    }

    #[test]
    fn test_task_complexity_simple() {
        let task = create_test_task_with_steps(devman_core::TaskId::new(), "simple", 3, 1);
        let estimator = CompletionEstimator::default();
        let complexity = estimator.calculate_task_complexity(&task);
        assert_eq!(complexity, TaskComplexity::Simple);
    }

    #[test]
    fn test_task_complexity_moderate() {
        let task = create_test_task_with_steps(devman_core::TaskId::new(), "moderate", 7, 2);
        let estimator = CompletionEstimator::default();
        let complexity = estimator.calculate_task_complexity(&task);
        assert_eq!(complexity, TaskComplexity::Moderate);
    }

    #[test]
    fn test_task_complexity_complex() {
        let task = create_test_task_with_steps(devman_core::TaskId::new(), "complex", 15, 3);
        let estimator = CompletionEstimator::default();
        let complexity = estimator.calculate_task_complexity(&task);
        assert_eq!(complexity, TaskComplexity::Complex);
    }

    #[test]
    fn test_task_complexity_very_complex() {
        let task = create_test_task_with_steps(devman_core::TaskId::new(), "very_complex", 25, 5);
        let estimator = CompletionEstimator::default();
        let complexity = estimator.calculate_task_complexity(&task);
        assert_eq!(complexity, TaskComplexity::VeryComplex);
    }

    #[test]
    fn test_estimate_task_completed() {
        let task = create_test_task_with_steps(devman_core::TaskId::new(), "done", 5, 0);
        let estimator = CompletionEstimator::default();
        let result = estimator.estimate_task(&task);
        assert!(result.duration_minutes > 0);

        // Now test completed task
        let mut completed_task = create_test_task_with_steps(devman_core::TaskId::new(), "done", 5, 0);
        completed_task.status = TaskStatus::Done;
        let result = estimator.estimate_task(&completed_task);
        assert_eq!(result.duration_minutes, 0);
        assert_eq!(result.confidence, 1.0);
        assert!(result.factors.contains(&"Task completed".to_string()));
    }

    #[test]
    fn test_format_duration() {
        let estimator = CompletionEstimator::default();

        assert_eq!(estimator.format_duration(30), "30m");
        assert_eq!(estimator.format_duration(90), "1h 30m");
        assert_eq!(estimator.format_duration(120), "2h");
        assert_eq!(estimator.format_duration(150), "2h 30m");
        assert_eq!(estimator.format_duration(1440), "1d");
        assert_eq!(estimator.format_duration(2880), "2d");
    }

    #[test]
    fn test_format_confidence() {
        let estimator = CompletionEstimator::default();

        assert_eq!(estimator.format_confidence(0.75), "75%");
        assert_eq!(estimator.format_confidence(0.95), "95%");
        assert_eq!(estimator.format_confidence(0.5), "50%");
    }

    #[test]
    fn test_estimate_task_with_steps() {
        let task = create_test_task_with_steps(devman_core::TaskId::new(), "multi_step", 10, 2);
        let estimator = CompletionEstimator::default();
        let result = estimator.estimate_task(&task);

        // Should have step-related factors
        assert!(result.factors.iter().any(|f| f.contains("Steps")));
        assert!(result.factors.iter().any(|f| f.contains("Dependencies")));
        assert!(result.duration_minutes > 0);
    }

    #[test]
    fn test_completion_estimator_default() {
        let estimator = CompletionEstimator::default();
        // Create a goal with active tasks for testing
        let goal = devman_core::Goal {
            id: devman_core::GoalId::new(),
            title: "Test Goal".to_string(),
            description: "Test".to_string(),
            project_id: devman_core::ProjectId::new(),
            success_criteria: vec![],
            progress: devman_core::GoalProgress {
                percentage: 25.0,
                completed_phases: vec![],
                active_tasks: 3,
                completed_tasks: 1,
                estimated_completion: None,
                blockers: vec![],
            },
            current_phase: devman_core::PhaseId::new(),
            status: devman_core::GoalStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let result = estimator.estimate_goal(&goal);
        assert!(result.estimated_completion > Utc::now());
        assert!(result.duration_minutes > 0);
    }

    #[test]
    fn test_time_estimation_fields() {
        let estimator = CompletionEstimator::default();
        let task = create_test_task_with_steps(devman_core::TaskId::new(), "test", 5, 0);
        let result = estimator.estimate_task(&task);

        assert!(result.estimated_completion > Utc::now());
        assert!(result.confidence > 0.0 && result.confidence <= 1.0);
        assert!(result.duration_minutes > 0);
        assert!(!result.factors.is_empty());
    }

    #[test]
    fn test_estimate_phase_completed() {
        let estimator = CompletionEstimator::default();
        let phase = devman_core::Phase {
            id: devman_core::PhaseId::new(),
            name: "Test Phase".to_string(),
            description: "Test".to_string(),
            objectives: vec![],
            acceptance_criteria: vec![],
            tasks: vec![],
            depends_on: vec![],
            status: devman_core::PhaseStatus::Completed,
            progress: devman_core::PhaseProgress {
                completed_tasks: 5,
                total_tasks: 5,
                percentage: 100.0,
            },
            estimated_duration: None,
            actual_duration: None,
            created_at: Utc::now(),
        };
        let result = estimator.estimate_phase(&phase);

        assert_eq!(result.duration_minutes, 0);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_task_complexity_base_minutes() {
        assert_eq!(TaskComplexity::Trivial.base_minutes(), 5);
        assert_eq!(TaskComplexity::Simple.base_minutes(), 15);
        assert_eq!(TaskComplexity::Moderate.base_minutes(), 30);
        assert_eq!(TaskComplexity::Complex.base_minutes(), 60);
        assert_eq!(TaskComplexity::VeryComplex.base_minutes(), 120);
    }

    #[test]
    fn test_task_complexity_confidence_modifier() {
        assert_eq!(TaskComplexity::Trivial.confidence_modifier(), 0.95);
        assert_eq!(TaskComplexity::Simple.confidence_modifier(), 0.85);
        assert_eq!(TaskComplexity::Moderate.confidence_modifier(), 0.75);
        assert_eq!(TaskComplexity::Complex.confidence_modifier(), 0.60);
        assert_eq!(TaskComplexity::VeryComplex.confidence_modifier(), 0.45);
    }

    #[test]
    fn test_estimate_phase_in_progress() {
        let estimator = CompletionEstimator::default();
        let phase = devman_core::Phase {
            id: devman_core::PhaseId::new(),
            name: "Test Phase".to_string(),
            description: "Test".to_string(),
            objectives: vec![],
            acceptance_criteria: vec![],
            tasks: vec![devman_core::TaskId::new(), devman_core::TaskId::new()],
            depends_on: vec![],
            status: devman_core::PhaseStatus::InProgress,
            progress: devman_core::PhaseProgress {
                completed_tasks: 1,
                total_tasks: 2,
                percentage: 50.0,
            },
            estimated_duration: None,
            actual_duration: None,
            created_at: Utc::now(),
        };
        let result = estimator.estimate_phase(&phase);

        assert!(result.duration_minutes > 0);
        assert!(result.estimated_completion > Utc::now());
    }
}
