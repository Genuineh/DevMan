//! Quality model - checks, gates, and human collaboration.

use serde::{Deserialize, Serialize};
use crate::id::{QualityCheckId, TaskId};
use crate::Time;
use crate::work_record::Severity;

/// A quality check that can be run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheck {
    /// Unique identifier
    pub id: QualityCheckId,

    /// Check name
    pub name: String,

    /// Description
    pub description: String,

    /// Check type
    pub check_type: QualityCheckType,

    /// Severity
    pub severity: Severity,

    /// Category
    pub category: QualityCategory,
}

/// Quality check types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum QualityCheckType {
    Generic(GenericCheckType),
    Custom(CustomCheckSpec),
}

/// Generic (built-in) quality checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenericCheckType {
    Compiles { target: String },
    TestsPass { test_suite: String, min_coverage: Option<f32> },
    Formatted { formatter: String },
    LintsPass { linter: String },
    DocumentationExists { paths: Vec<String> },
    TypeCheck {},
    DependenciesValid {},
    SecurityScan { scanner: String },
}

/// Custom check specification (user-extensible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCheckSpec {
    pub name: String,
    pub check_command: CommandSpec,
    pub validation: ValidationSpec,
    pub human_review: Option<HumanReviewSpec>,
}

/// Command specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpec {
    pub command: String,
    pub args: Vec<String>,
    pub timeout: std::time::Duration,
    pub expected_exit_code: Option<i32>,
}

/// Validation specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSpec {
    pub output_parser: OutputParser,
    pub pass_condition: String,
    pub extract_metrics: Vec<MetricExtractor>,
}

/// Output parser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputParser {
    JsonPath { path: String },
    Regex { pattern: String },
    LineContains { text: String },
    Custom { script: String },
}

/// Metric extractor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricExtractor {
    pub name: String,
    pub extractor: OutputParser,
    pub unit: Option<String>,
}

/// Human review specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanReviewSpec {
    pub reviewers: Vec<String>,
    pub review_guide: String,
    pub review_form: Vec<ReviewQuestion>,
    pub timeout: std::time::Duration,
    pub auto_pass_threshold: Option<f32>,
}

/// Review question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewQuestion {
    pub question: String,
    pub answer_type: AnswerType,
    pub required: bool,
}

/// Answer type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnswerType {
    YesNo,
    Rating { min: i32, max: i32 },
    Text,
    Choice { options: Vec<String> },
}

/// Answer value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnswerValue {
    YesNo(bool),
    Rating(i32),
    Text(String),
    Choice(String),
}

/// Quality category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QualityCategory {
    Correctness,
    Performance,
    Security,
    Maintainability,
    Documentation,
    Testing,
    Business,
    Compliance,
}

/// Quality check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheckResult {
    pub check_id: QualityCheckId,
    pub passed: bool,
    pub execution_time: std::time::Duration,
    pub details: CheckDetails,
    pub findings: Vec<Finding>,
    pub metrics: Vec<Metric>,
    pub human_review: Option<HumanReviewResult>,
}

/// Check details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckDetails {
    pub output: String,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}

/// A finding from a quality check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub category: QualityCategory,
    pub message: String,
    pub location: Option<FileLocation>,
    pub suggestion: Option<String>,
}

/// File location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLocation {
    pub file: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

/// A metric from a check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: Option<String>,
}

/// Human review result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanReviewResult {
    pub reviewer: String,
    pub reviewed_at: Time,
    pub answers: Vec<ReviewAnswer>,
    pub comments: String,
    pub approved: bool,
}

/// Review answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAnswer {
    pub question: String,
    pub answer: AnswerValue,
}

/// Notification channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email { recipients: Vec<String> },
    Slack { webhook: String },
    Webhook { url: String },
}

/// Quality profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityProfile {
    pub name: String,
    pub description: String,
    pub checks: Vec<QualityCheckId>,
    pub phase_gates: Vec<PhaseGate>,
    pub default_strategy: GateStrategy,
}

/// Phase gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseGate {
    pub phase: crate::PhaseId,
    pub checks: Vec<QualityCheckId>,
    pub strategy: GateStrategy,
}

/// Gate strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateStrategy {
    AllMustPass,
    WarningsAllowed { max_warnings: usize },
    ManualDecision,
    Custom { rule: String },
}

/// Quality status for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStatus {
    pub task_id: TaskId,
    pub total_checks: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
    pub warnings: usize,
    pub overall_status: QualityOverallStatus,
    pub pending_human_review: bool,
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
