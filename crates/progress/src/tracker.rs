//! Progress tracking service.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use devman_core::{
    Goal, GoalId, GoalProgress, Phase, PhaseId, PhaseProgress, Task, TaskId, TaskProgress,
};
use devman_storage::Storage;

/// Progress tracking service.
#[async_trait]
pub trait ProgressTracker: Send + Sync {
    /// Get goal progress.
    async fn get_goal_progress(&self, goal_id: GoalId) -> Option<GoalProgress>;

    /// Get phase progress.
    async fn get_phase_progress(&self, phase_id: PhaseId) -> Option<PhaseProgress>;

    /// Get task progress.
    async fn get_task_progress(&self, task_id: TaskId) -> Option<TaskProgress>;

    /// Take a progress snapshot.
    async fn snapshot(&self) -> ProgressSnapshot;
}

/// A snapshot of progress at a point in time.
#[derive(Debug, Clone)]
pub struct ProgressSnapshot {
    /// When snapshot was taken
    pub timestamp: DateTime<Utc>,

    /// Goal progress by goal ID
    pub goal_progress: Vec<(GoalId, GoalProgress)>,

    /// Phase progress by phase ID
    pub phase_progress: Vec<(PhaseId, PhaseProgress)>,

    /// Task progress by task ID
    pub task_progress: Vec<(TaskId, TaskProgress)>,
}

/// Basic progress tracker implementation.
pub struct BasicProgressTracker<S: Storage> {
    storage: std::sync::Arc<S>,
}

impl<S: Storage> BasicProgressTracker<S> {
    /// Create a new progress tracker.
    pub fn new(storage: S) -> Self {
        Self {
            storage: std::sync::Arc::new(storage),
        }
    }

    /// Calculate goal progress from its phases.
    async fn calculate_goal_progress(&self, goal: &Goal) -> GoalProgress {
        let mut total_phases = 0;
        let mut total_tasks = 0;
        let mut completed_tasks = 0;
        let mut completed_phase_ids = Vec::new();

        // Load project to get phases
        if let Ok(Some(project)) = self.storage.load_project(goal.project_id).await {
            for phase_id in &project.phases {
                if let Ok(Some(phase)) = self.storage.load_phase(*phase_id).await {
                    total_phases += 1;
                    if matches!(phase.status, devman_core::PhaseStatus::Completed) {
                        completed_phase_ids.push(*phase_id);
                    }

                    for task_id in &phase.tasks {
                        total_tasks += 1;
                        if let Ok(Some(task)) = self.storage.load_task(*task_id).await {
                            if matches!(
                                task.status,
                                devman_core::TaskStatus::Done | devman_core::TaskStatus::Abandoned
                            ) {
                                completed_tasks += 1;
                            }
                        }
                    }
                }
            }
        }

        let percentage = if total_tasks > 0 {
            (completed_tasks as f32 / total_tasks as f32) * 100.0
        } else {
            0.0
        };

        GoalProgress {
            percentage,
            completed_phases: completed_phase_ids,
            active_tasks: total_tasks - completed_tasks,
            completed_tasks,
            estimated_completion: None,
            blockers: Vec::new(),
        }
    }

    /// Calculate phase progress from its tasks.
    async fn calculate_phase_progress(&self, phase: &Phase) -> PhaseProgress {
        let mut completed = 0;
        let total = phase.tasks.len();

        for task_id in &phase.tasks {
            if let Ok(Some(task)) = self.storage.load_task(*task_id).await {
                if matches!(
                    task.status,
                    devman_core::TaskStatus::Done | devman_core::TaskStatus::Abandoned
                ) {
                    completed += 1;
                }
            }
        }

        let percentage = if total > 0 {
            (completed as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        PhaseProgress {
            completed_tasks: completed,
            total_tasks: total,
            percentage,
        }
    }
}

#[async_trait]
impl<S: Storage + 'static> ProgressTracker for BasicProgressTracker<S> {
    async fn get_goal_progress(&self, goal_id: GoalId) -> Option<GoalProgress> {
        let goal = self.storage.load_goal(goal_id).await.ok().flatten()?;
        Some(self.calculate_goal_progress(&goal).await)
    }

    async fn get_phase_progress(&self, phase_id: PhaseId) -> Option<PhaseProgress> {
        let phase = self.storage.load_phase(phase_id).await.ok().flatten()?;
        Some(self.calculate_phase_progress(&phase).await)
    }

    async fn get_task_progress(&self, task_id: TaskId) -> Option<TaskProgress> {
        let task = self.storage.load_task(task_id).await.ok().flatten()?;
        Some(task.progress)
    }

    async fn snapshot(&self) -> ProgressSnapshot {
        // Collect all progress
        let goals = self.storage.list_goals().await.unwrap_or_default();
        let mut goal_progress = Vec::new();
        let mut phase_progress = Vec::new();
        let mut task_progress = Vec::new();

        for goal in goals {
            let progress = self.calculate_goal_progress(&goal).await;
            goal_progress.push((goal.id, progress));
        }

        // TODO: Collect phases and tasks too

        ProgressSnapshot {
            timestamp: Utc::now(),
            goal_progress,
            phase_progress,
            task_progress,
        }
    }
}
