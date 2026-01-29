//! Task model - the core unit of work in DevMan.

use serde::{Deserialize, Serialize};
use crate::id::{TaskId, PhaseId, WorkRecordId};
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

use crate::{KnowledgeId, QualityCheckId};
