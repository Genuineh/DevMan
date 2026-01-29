//! Work context management.

use devman_core::{Goal, GoalId, Phase, PhaseId, Project, ProjectId, Task, TaskId};

/// Current work context (for work management).
#[derive(Debug, Clone)]
pub struct WorkManagementContext {
    /// Current goal
    pub goal: Option<Goal>,

    /// Current project
    pub project: Option<Project>,

    /// Current phase
    pub phase: Option<Phase>,

    /// Active task
    pub active_task: Option<Task>,

    /// Recent changes
    pub recent_changes: Vec<Change>,

    /// Working directory
    pub work_dir: Option<String>,

    /// Environment variables
    pub env: std::collections::HashMap<String, String>,
}

impl Default for WorkManagementContext {
    fn default() -> Self {
        Self {
            goal: None,
            project: None,
            phase: None,
            active_task: None,
            recent_changes: Vec::new(),
            work_dir: None,
            env: std::collections::HashMap::new(),
        }
    }
}

impl WorkManagementContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set goal.
    pub fn with_goal(mut self, goal: Goal) -> Self {
        self.goal = Some(goal);
        self
    }

    /// Set project.
    pub fn with_project(mut self, project: Project) -> Self {
        self.project = Some(project);
        self
    }

    /// Set phase.
    pub fn with_phase(mut self, phase: Phase) -> Self {
        self.phase = Some(phase);
        self
    }

    /// Set active task.
    pub fn with_active_task(mut self, task: Task) -> Self {
        self.active_task = Some(task);
        self
    }

    /// Add a change.
    pub fn add_change(mut self, change: Change) -> Self {
        self.recent_changes.push(change);
        self
    }
}

/// A change in the work context.
#[derive(Debug, Clone)]
pub struct Change {
    /// What changed
    pub what: String,

    /// Type of change
    pub change_type: ChangeType,

    /// When it happened
    pub when: chrono::DateTime<chrono::Utc>,

    /// Who made the change
    pub who: String,
}

/// Type of change.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChangeType {
    Create,
    Update,
    Delete,
    Execute,
}
