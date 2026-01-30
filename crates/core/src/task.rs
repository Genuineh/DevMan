//! Task model - the core unit of work in DevMan.

use serde::{Deserialize, Serialize};
use crate::id::{TaskId, PhaseId, WorkRecordId, GoalId};
use crate::Time;

/// A task represents a unit of work that can be executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier
    pub id: TaskId,

    /// Task title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// AI's understanding of the task intent
    pub intent: TaskIntent,

    /// Execution steps (tool calls to reduce token usage)
    pub steps: Vec<ExecutionStep>,

    /// Inputs required for this task
    pub inputs: Vec<Input>,

    /// Expected outputs
    pub expected_outputs: Vec<ExpectedOutput>,

    /// Quality gates (checkpoints)
    pub quality_gates: Vec<QualityGate>,

    /// Current status
    pub status: TaskStatus,

    /// Progress tracking
    pub progress: TaskProgress,

    /// Associated phase
    pub phase_id: PhaseId,

    /// Dependencies
    pub depends_on: Vec<TaskId>,

    /// Tasks this blocks
    pub blocks: Vec<TaskId>,

    /// Work records from executions
    pub work_records: Vec<WorkRecordId>,

    /// Creation timestamp
    pub created_at: Time,

    /// Last update timestamp
    pub updated_at: Time,
}

/// AI's understanding of task intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskIntent {
    /// Natural language description
    pub natural_language: String,

    /// Context information
    pub context: TaskContext,

    /// Success criteria
    pub success_criteria: Vec<String>,
}

/// Context information for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    /// Related knowledge
    pub relevant_knowledge: Vec<KnowledgeId>,

    /// Similar tasks
    pub similar_tasks: Vec<TaskId>,

    /// Affected files/modules
    pub affected_files: Vec<String>,
}

/// An execution step (tool call).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step order
    pub order: usize,

    /// Description
    pub description: String,

    /// Tool invocation
    pub tool: ToolInvocation,

    /// Verification (optional)
    pub verify: Option<Verification>,
}

/// Tool invocation specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    /// Tool name
    pub tool: String,

    /// Arguments
    pub args: Vec<String>,

    /// Environment variables
    pub env: Vec<(String, String)>,

    /// Timeout
    pub timeout: Option<std::time::Duration>,
}

/// Verification for a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    /// What to check
    pub check: String,

    /// Expected value
    pub expected: String,
}

/// A quality gate (checkpoint during task execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    /// Gate name
    pub name: String,

    /// Description
    pub description: String,

    /// Quality checks to run
    pub checks: Vec<QualityCheckId>,

    /// Pass condition
    pub pass_condition: PassCondition,

    /// Action on failure
    pub on_failure: FailureAction,
}

/// Pass condition for a quality gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PassCondition {
    /// All checks must pass
    AllPassed,

    /// At least N checks must pass
    AtLeast { count: usize },

    /// Custom expression
    Custom { expression: String },
}

/// Action to take on quality gate failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureAction {
    /// Block further progress
    Block,

    /// Warn but continue
    Warn,

    /// Escalate to human
    Escalate,
}

/// Input specification for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    /// Input name
    pub name: String,

    /// Input type
    pub input_type: String,

    /// Description
    pub description: String,

    /// Required
    pub required: bool,
}

/// Expected output from a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOutput {
    /// Output name
    pub name: String,

    /// Output type
    pub output_type: String,

    /// Description
    pub description: String,
}

/// Task state machine - strict state control for interactive AI workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskState {
    /// Task created, waiting to read context
    Created {
        created_at: Time,
        created_by: String,
    },

    /// Context read, waiting to review knowledge
    ContextRead {
        read_at: Time,
    },

    /// Knowledge reviewed, ready to start execution
    KnowledgeReviewed {
        knowledge_ids: Vec<KnowledgeId>,
        reviewed_at: Time,
    },

    /// Task in progress
    InProgress {
        started_at: Time,
        checkpoint: Option<String>,
    },

    /// Work recorded, waiting for quality check
    WorkRecorded {
        record_id: WorkRecordId,
        recorded_at: Time,
    },

    /// Quality check in progress
    QualityChecking {
        check_id: QualityCheckId,
        started_at: Time,
    },

    /// Quality check completed
    QualityCompleted {
        result: QualityCheckResult,
        completed_at: Time,
    },

    /// Task paused (can be resumed)
    Paused {
        paused_at: Time,
        reason: String,
        previous_state: Box<TaskState>,
    },

    /// Task abandoned (unified handling for all termination scenarios)
    Abandoned {
        abandoned_at: Time,
        reason: AbandonReason,
    },

    /// Task completed
    Completed {
        completed_at: Time,
        completed_by: String,
    },
}

/// Reason for task abandonment (covers all scenarios where task cannot continue).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbandonReason {
    /// Voluntary abandonment by AI/developer
    Voluntary {
        reason: String,
        can_be_reassigned: bool,
    },

    /// Project cancelled
    ProjectCancelled {
        reason: String,
        cancelled_by: String,
    },

    /// Goal cancelled
    GoalCancelled {
        goal_id: GoalId,
        reason: String,
    },

    /// Requirement changed (cannot adapt)
    RequirementChanged {
        old_requirement: String,
        new_requirement: String,
        impact: ChangeImpact,
    },

    /// Dependency task failed
    DependencyFailed {
        dependency_task_id: TaskId,
        failure_reason: String,
    },

    /// Insufficient information
    InsufficientInformation {
        missing_info: Vec<String>,
    },

    /// Technical limitation
    TechnicalLimitation {
        limitation: String,
        suggested_alternative: Option<String>,
    },

    /// Resource unavailable
    ResourceUnavailable {
        resource: String,
        reason: String,
    },

    /// Timeout
    Timeout {
        deadline: Time,
        actual_completion: Option<Time>,
    },

    /// Quality check failed repeatedly
    QualityCheckFailed {
        attempts: usize,
        remaining_issues: Vec<String>,
    },

    /// Other reason
    Other {
        reason: String,
        details: Option<String>,
    },
}

/// Impact of a change on task progress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeImpact {
    /// Can continue with current work
    CanContinue,

    /// Needs to review knowledge again
    NeedsReview,

    /// Needs to re-execute
    NeedsReexecution,

    /// Needs to completely restart
    NeedsRestart,
}

/// Quality check result summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheckResult {
    pub overall_status: QualityOverallStatus,
    pub findings_count: usize,
    pub warnings_count: usize,
}

/// Overall quality status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityOverallStatus {
    NotChecked,
    Passed,
    PassedWithWarnings,
    Failed,
    PendingReview,
}

/// State transition result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateTransition {
    /// Transition allowed
    Allowed,

    /// Transition rejected - missing precondition
    RejectedMissingPrecondition {
        required: String,
        hint: String,
    },

    /// Transition rejected - needs specific action
    RejectedRequiredAction {
        action: String,
        guidance: String,
    },
}

impl TaskState {
    /// Check if this state can be paused.
    pub fn can_be_paused(&self) -> bool {
        !matches!(self, Self::Completed { .. } | Self::Abandoned { .. })
    }

    /// Get guidance message for current state.
    pub fn get_guidance(&self) -> &'static str {
        match self {
            Self::Created { .. } => {
                "请先调用 read_task_context() 读取任务上下文，了解项目信息、依赖关系和质检要求。"
            }
            Self::ContextRead { .. } => {
                "请调用 review_knowledge() 查询相关知识，学习最佳实践和类似实现。"
            }
            Self::KnowledgeReviewed { .. } => {
                "现在可以开始执行任务了。调用 start_execution() 开始，并使用 log_work() 记录工作进展。"
            }
            Self::InProgress { .. } => {
                "继续执行任务，使用 log_work() 记录工作。完成后调用 finish_work() 提交工作记录。"
            }
            Self::WorkRecorded { .. } => {
                "工作已记录，请调用 run_quality_check() 运行质检。"
            }
            Self::QualityChecking { .. } => {
                "质检正在运行，请等待结果..."
            }
            Self::QualityCompleted { result, .. } => {
                match result.overall_status {
                    QualityOverallStatus::Passed => "质检通过！调用 complete_task() 完成任务。",
                    _ => "质检未通过，请修复问题后调用 start_execution() 重新开始执行。",
                }
            }
            Self::Paused { .. } => {
                "任务已暂停。调用 resume_task() 恢复执行。"
            }
            Self::Abandoned { .. } => {
                "任务已放弃。"
            }
            Self::Completed { .. } => {
                "任务已完成。"
            }
        }
    }

    /// Get allowed operations for current state.
    pub fn allowed_operations(&self) -> Vec<&'static str> {
        match self {
            Self::Created { .. } => vec!["read_task_context", "abandon_task"],
            Self::ContextRead { .. } => vec!["review_knowledge", "abandon_task"],
            Self::KnowledgeReviewed { .. } => vec!["start_execution", "abandon_task"],
            Self::InProgress { .. } => vec!["log_work", "finish_work", "pause_task", "abandon_task"],
            Self::WorkRecorded { .. } => vec!["run_quality_check", "abandon_task"],
            Self::QualityChecking { .. } => vec!["get_quality_result"],
            Self::QualityCompleted { .. } => vec!["complete_task", "start_execution", "abandon_task"],
            Self::Paused { .. } => vec!["resume_task", "abandon_task"],
            Self::Abandoned { .. } => vec![],
            Self::Completed { .. } => vec![],
        }
    }
}

/// Legacy TaskStatus for backward compatibility (use TaskState for new code).
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

impl From<TaskState> for TaskStatus {
    fn from(state: TaskState) -> Self {
        match state {
            TaskState::Created { .. } => TaskStatus::Idea,
            TaskState::ContextRead { .. } | TaskState::KnowledgeReviewed { .. } => TaskStatus::Queued,
            TaskState::InProgress { .. } => TaskStatus::Active,
            TaskState::WorkRecorded { .. } | TaskState::QualityChecking { .. } => TaskStatus::Review,
            TaskState::QualityCompleted { .. } => TaskStatus::Review,
            TaskState::Paused { .. } => TaskStatus::Blocked,
            TaskState::Abandoned { .. } => TaskStatus::Abandoned,
            TaskState::Completed { .. } => TaskStatus::Done,
        }
    }
}

/// Progress tracking for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    /// Percentage complete (0-100)
    pub percentage: f32,

    /// Current step index
    pub current_step: Option<usize>,

    /// Total steps
    pub total_steps: usize,

    /// Status message
    pub message: String,
}

impl Default for TaskProgress {
    fn default() -> Self {
        Self {
            percentage: 0.0,
            current_step: None,
            total_steps: 0,
            message: String::new(),
        }
    }
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

// Re-exports for compatibility
pub use crate::{KnowledgeId, QualityCheckId};
