//! DevMan core data models.
//!
//! This crate defines the fundamental data structures that power the
//! AI cognitive work management system.

#![warn(missing_docs)]

// Core identities
mod id;

// Goal and project management
mod goal;
mod project;
mod phase;

// Task execution
mod task;
mod work_record;
mod event;

// Knowledge and quality
mod knowledge;
mod quality;

// Re-exports
pub use id::*;

// Goal & Project
pub use goal::{Goal, GoalStatus, GoalProgress, SuccessCriterion, CriterionStatus};
pub use project::{Project, ProjectConfig, DirStructure, ToolConfig, BuildTool, TestFramework};
pub use phase::{Phase, PhaseStatus, PhaseProgress, AcceptanceCriterion};

// Task & Work
pub use task::{
    Task, TaskStatus, TaskState, AbandonReason, ChangeImpact, TaskProgress, TaskLink, LinkKind, TaskFilter,
    TaskIntent, TaskContext, ExecutionStep, QualityGate, PassCondition, FailureAction,
    Input, ExpectedOutput, StateTransition,
    // Task module's simplified quality types
    QualityCheckResult as TaskQualityCheckResult,
    QualityOverallStatus as TaskQualityOverallStatus,
};
pub use work_record::{
    WorkRecord, WorkEvent, WorkEventType, Executor, WorkResult,
    CompletionStatus, Output, Artifact, Issue, Resolution, WorkMetrics,
    Severity,
};
pub use event::Event;

// Knowledge & Quality
pub use knowledge::{
    Knowledge, KnowledgeType, KnowledgeContent, KnowledgeMetadata,
    UsageStats, Feedback, CodeSnippet, TemplateContent, TemplateParameter,
};
pub use quality::{
    QualityCheck, QualityCheckType, GenericCheckType, CustomCheckSpec,
    CommandSpec, ValidationSpec, OutputParser, MetricExtractor, QualityCategory,
    HumanReviewSpec, ReviewQuestion, AnswerType, AnswerValue,
    HumanReviewResult, ReviewAnswer, NotificationChannel,
    QualityCheckResult, CheckDetails, Finding, FileLocation, Metric,
    QualityProfile, GateStrategy, PhaseGate,
    QualityStatus, QualityOverallStatus,
};

// Progress tracking
pub use progress::{Blocker, BlockedItem};

/// Timestamp type
pub type Time = chrono::DateTime<chrono::Utc>;

/// Progress module exports
mod progress {
    pub use super::work_record::Blocker;
    pub use super::work_record::BlockedItem;
}
