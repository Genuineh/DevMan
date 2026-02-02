//! Interactive AI interface with strict task state control.
//!
//! This module provides the core trait and types for AI assistants to interact
//! with DevMan in a guided, state-managed workflow.

use async_trait::async_trait;
use devman_core::{
    GoalId, KnowledgeId, PhaseId, QualityCheckId, QualityCheckType,
    TaskId, WorkRecordId,
};
use devman_knowledge::KnowledgeService;
use devman_quality::QualityEngine;
use devman_tools::ToolExecutor;
use std::sync::Arc;

// ==================== Re-exports ====================

pub use devman_core::{
    AbandonReason, ChangeImpact, QualityCheckResult, TaskState,
    WorkRecord, Task, TaskProgress,
};

// ==================== Core Trait ====================

/// Interactive AI interface - strict state control and guided workflow.
#[async_trait]
pub trait InteractiveAI: Send + Sync {
    // ==================== Task Lifecycle ====================

    /// Create a new task
    async fn create_task(&self, request: CreateTaskRequest) -> Result<TaskId, anyhow::Error>;

    /// Abandon task (unified entry point for all termination scenarios)
    async fn abandon_task(
        &self,
        task_id: TaskId,
        reason: AbandonReason,
    ) -> Result<AbandonResult, anyhow::Error>;

    /// Complete task
    async fn complete_task(
        &self,
        task_id: TaskId,
        summary: TaskCompletionSummary,
    ) -> Result<(), anyhow::Error>;

    // ==================== Task Guidance ====================

    /// Get task guidance - AI should call this before any operation
    async fn get_task_guidance(&self, task_id: TaskId) -> Result<TaskGuidance, anyhow::Error>;

    /// List tasks
    async fn list_tasks(&self, filter: TaskFilter) -> Result<Vec<TaskSummary>, anyhow::Error>;

    // ==================== Normal Workflow ====================

    /// Stage 1: Read task context (Created -> ContextRead)
    async fn read_task_context(&self, task_id: TaskId) -> Result<TaskContext, anyhow::Error>;

    /// Stage 2: Review knowledge (ContextRead -> KnowledgeReviewed)
    async fn review_knowledge(
        &self,
        task_id: TaskId,
        query: &str,
    ) -> Result<KnowledgeReviewResult, anyhow::Error>;

    /// Confirm knowledge review completed
    async fn confirm_knowledge_reviewed(
        &self,
        task_id: TaskId,
        knowledge_ids: Vec<KnowledgeId>,
    ) -> Result<(), anyhow::Error>;

    /// Stage 3: Start execution (KnowledgeReviewed -> InProgress)
    async fn start_execution(&self, task_id: TaskId) -> Result<ExecutionSession, anyhow::Error>;

    /// Log work progress
    async fn log_work(&self, task_id: TaskId, log: WorkLogEntry) -> Result<(), anyhow::Error>;

    /// Submit work (InProgress -> WorkRecorded)
    async fn finish_work(
        &self,
        task_id: TaskId,
        result: WorkSubmission,
    ) -> Result<WorkRecordId, anyhow::Error>;

    /// Stage 4: Run quality check (WorkRecorded -> QualityChecking)
    async fn run_quality_check(
        &self,
        task_id: TaskId,
        checks: Vec<QualityCheckType>,
    ) -> Result<QualityCheckId, anyhow::Error>;

    /// Get quality check result
    async fn get_quality_result(
        &self,
        check_id: QualityCheckId,
    ) -> Result<QualityCheckResult, anyhow::Error>;

    /// Confirm quality result and decide next action
    async fn confirm_quality_result(
        &self,
        task_id: TaskId,
        check_id: QualityCheckId,
        decision: QualityDecision,
    ) -> Result<(), anyhow::Error>;

    // ==================== Task Control ====================

    /// Pause task
    async fn pause_task(&self, task_id: TaskId, reason: String) -> Result<(), anyhow::Error>;

    /// Resume task
    async fn resume_task(&self, task_id: TaskId) -> Result<(), anyhow::Error>;

    // ==================== Requirement Changes ====================

    /// Handle requirement change
    async fn handle_requirement_change(
        &self,
        task_id: TaskId,
        change: RequirementChange,
    ) -> Result<ChangeHandlingResult, anyhow::Error>;

    // ==================== Task Reassignment ====================

    /// Request task reassignment
    async fn request_reassignment(
        &self,
        task_id: TaskId,
        reason: String,
    ) -> Result<ReassignmentRequest, anyhow::Error>;

    /// Accept reassigned task
    async fn accept_reassigned_task(
        &self,
        task_id: TaskId,
        request_id: ReassignmentRequestId,
    ) -> Result<TaskHandover, anyhow::Error>;
}

// ==================== Request/Response Types ====================

/// Create task request
#[derive(Debug, Clone)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub goal_id: Option<GoalId>,
    pub phase_id: Option<PhaseId>,
    pub estimated_duration: Option<String>,
    pub dependencies: Vec<TaskId>,
}

/// Task guidance information
#[derive(Debug, Clone)]
pub struct TaskGuidance {
    pub current_state: TaskState,
    pub next_action: NextAction,
    pub prerequisites_satisfied: bool,
    pub missing_prerequisites: Vec<String>,
    pub allowed_operations: Vec<String>,
    pub guidance_message: String,
    pub task_health: TaskHealth,
}

/// Next action for AI
#[derive(Debug, Clone)]
pub enum NextAction {
    ReadContext,
    ReviewKnowledge { suggested_queries: Vec<String> },
    StartExecution { suggested_workflow: Option<String> },
    ContinueExecution { required_logs: Vec<String> },
    SubmitWork,
    RunQualityCheck { required_checks: Vec<QualityCheckType> },
    FixQualityIssues { issues: Vec<String> },
    CompleteTask,
    TaskFinished,
}

/// Task health status
#[derive(Debug, Clone)]
pub enum TaskHealth {
    Healthy,
    Warning { warnings: Vec<String> },
    Attention { issues: Vec<TaskIssue> },
    Critical { blockers: Vec<String> },
}

/// Task issue
#[derive(Debug, Clone)]
pub struct TaskIssue {
    pub severity: IssueSeverity,
    pub description: String,
    pub suggested_action: String,
}

#[derive(Debug, Clone)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Task summary
#[derive(Debug, Clone)]
pub struct TaskSummary {
    pub id: TaskId,
    pub title: String,
    pub state: TaskState,
    pub progress: TaskProgress,
    pub created_at: devman_core::Time,
}

/// Task filter
#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    pub states: Option<Vec<TaskState>>,
    pub limit: Option<usize>,
}

/// Task context (detailed information for AI)
#[derive(Debug, Clone)]
pub struct TaskContext {
    pub task: Task,
    pub project: ProjectContext,
    pub dependencies: Vec<TaskDependency>,
    pub quality_requirements: Vec<QualityRequirement>,
}

/// Project context
#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub name: String,
    pub description: String,
    pub tech_stack: Vec<String>,
    pub current_phase: PhaseInfo,
}

/// Phase info
#[derive(Debug, Clone)]
pub struct PhaseInfo {
    pub id: PhaseId,
    pub name: String,
    pub status: String,
}

/// Task dependency
#[derive(Debug, Clone)]
pub struct TaskDependency {
    pub task_id: TaskId,
    pub title: String,
    pub status: TaskState,
    pub is_blocking: bool,
}

/// Quality requirement
#[derive(Debug, Clone)]
pub struct QualityRequirement {
    pub check_type: QualityCheckType,
    pub description: String,
    pub required: bool,
}

/// Knowledge review result
#[derive(Debug, Clone)]
pub struct KnowledgeReviewResult {
    pub knowledge_items: Vec<KnowledgeItem>,
    pub required_reading: Vec<KnowledgeId>,
    pub reviewed_knowledge_ids: Vec<KnowledgeId>,
}

/// Knowledge item (simplified)
#[derive(Debug, Clone)]
pub struct KnowledgeItem {
    pub id: KnowledgeId,
    pub title: String,
    pub knowledge_type: String,
    pub summary: String,
    pub detail: String,
    pub relevance_score: f64,
}

/// Execution session
#[derive(Debug, Clone)]
pub struct ExecutionSession {
    pub session_id: String,
    pub started_at: devman_core::Time,
    pub timeout: Option<std::time::Duration>,
}

/// Work log entry
#[derive(Debug, Clone)]
pub struct WorkLogEntry {
    pub timestamp: devman_core::Time,
    pub action: WorkAction,
    pub description: String,
    pub files: Vec<String>,
    pub command_output: Option<CommandExecution>,
}

/// Work action type
#[derive(Debug, Clone)]
pub enum WorkAction {
    Created,
    Modified,
    Tested,
    Documented,
    Debugged,
    Refactored,
}

/// Command execution record
#[derive(Debug, Clone)]
pub struct CommandExecution {
    pub command: String,
    pub args: Vec<String>,
    pub exit_code: i32,
    pub output: String,
    pub timestamp: devman_core::Time,
}

/// Work submission
#[derive(Debug, Clone)]
pub struct WorkSubmission {
    pub description: String,
    pub artifacts: Vec<Artifact>,
    pub commands_executed: Vec<CommandExecution>,
    pub lessons_learned: Option<String>,
}

/// Artifact
#[derive(Debug, Clone)]
pub struct Artifact {
    pub name: String,
    pub artifact_type: ArtifactType,
    pub path: Option<String>,
    pub content: Option<String>,
}

/// Artifact type
#[derive(Debug, Clone)]
pub enum ArtifactType {
    File,
    Code,
    Documentation,
    Test,
    Binary,
    Other,
}

/// Quality decision
#[derive(Debug, Clone)]
pub enum QualityDecision {
    AcceptAndComplete,
    FixIssuesAndContinue,
    RedoExecution,
}

/// Abandon result
#[derive(Debug, Clone)]
pub struct AbandonResult {
    pub success: bool,
    pub can_be_reassigned: bool,
    pub work_reusable: bool,
    pub suggestions_for_next: Vec<String>,
    pub new_state: TaskState,
}

/// Requirement change
#[derive(Debug, Clone)]
pub struct RequirementChange {
    pub description: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub change_type: RequirementChangeType,
    pub impact: ChangeImpact,
}

/// Requirement change type
#[derive(Debug, Clone)]
pub enum RequirementChangeType {
    FeatureChange,
    PriorityChange,
    DeadlineChange,
    DependencyChange,
    QualityRequirementChange,
}

/// Change handling result
#[derive(Debug, Clone)]
pub enum ChangeHandlingResult {
    CanContinue,
    NeedsReview { suggested_knowledge: Vec<String> },
    NeedsReexecution { affected_work: Vec<String> },
    RecommendNewTask { reason: String, reusable_content: Vec<String> },
}

/// Reassignment request
#[derive(Debug, Clone)]
pub struct ReassignmentRequest {
    pub id: ReassignmentRequestId,
    pub task_id: TaskId,
    pub requested_by: String,
    pub reason: String,
    pub created_at: devman_core::Time,
    pub status: ReassignmentStatus,
}

/// Reassignment request ID
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReassignmentRequestId(pub String);

/// Reassignment status
#[derive(Debug, Clone)]
pub enum ReassignmentStatus {
    PendingApproval,
    AwaitingAcceptance,
    Accepted { accepted_by: String, accepted_at: devman_core::Time },
    Rejected { reason: String },
}

/// Task handover (for reassignment)
#[derive(Debug, Clone)]
pub struct TaskHandover {
    pub task: Task,
    pub current_state: TaskState,
    pub completed_work: Vec<WorkLogEntry>,
    pub reviewed_knowledge: Vec<KnowledgeId>,
    pub abandonment_reason: Option<String>,
    pub suggestions: Vec<String>,
    pub warnings: Vec<String>,
    pub reusable_artifacts: Vec<Artifact>,
}

/// Task completion summary
#[derive(Debug, Clone)]
pub struct TaskCompletionSummary {
    pub summary: String,
    pub artifacts: Vec<Artifact>,
    pub lessons_learned: Option<String>,
    pub created_knowledge: Option<Vec<KnowledgeId>>,
}

// ==================== Basic Implementation ====================

/// Basic implementation of InteractiveAI
pub struct BasicInteractiveAI {
    storage: Arc<dyn devman_storage::Storage>,
    knowledge_service: Arc<dyn KnowledgeService>,
    quality_engine: Arc<dyn QualityEngine>,
    tool_executor: Arc<dyn ToolExecutor>,
}

impl BasicInteractiveAI {
    pub fn new(
        storage: Arc<dyn devman_storage::Storage>,
        knowledge_service: Arc<dyn KnowledgeService>,
        quality_engine: Arc<dyn QualityEngine>,
        tool_executor: Arc<dyn ToolExecutor>,
    ) -> Self {
        Self {
            storage,
            knowledge_service,
            quality_engine,
            tool_executor,
        }
    }
}

#[async_trait]
impl InteractiveAI for BasicInteractiveAI {
    async fn create_task(&self, _request: CreateTaskRequest) -> Result<TaskId, anyhow::Error> {
        // TODO: Implement task creation
        Ok(TaskId::new())
    }

    async fn abandon_task(&self, _task_id: TaskId, _reason: AbandonReason) -> Result<AbandonResult, anyhow::Error> {
        // TODO: Implement task abandonment
        Ok(AbandonResult {
            success: true,
            can_be_reassigned: false,
            work_reusable: true,
            suggestions_for_next: vec![],
            new_state: TaskState::Abandoned {
                abandoned_at: chrono::Utc::now(),
                reason: AbandonReason::Other {
                    reason: "placeholder".to_string(),
                    details: None,
                },
            },
        })
    }

    async fn complete_task(&self, _task_id: TaskId, _summary: TaskCompletionSummary) -> Result<(), anyhow::Error> {
        // TODO: Implement task completion
        Ok(())
    }

    async fn get_task_guidance(&self, task_id: TaskId) -> Result<TaskGuidance, anyhow::Error> {
        let task = self.storage.load_task(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // Convert TaskStatus to TaskState for guidance
        // For now, use a default state
        let state = TaskState::Created {
            created_at: task.created_at,
            created_by: "system".to_string(),
        };

        let guidance_message = state.get_guidance().to_string();

        Ok(TaskGuidance {
            current_state: state,
            next_action: NextAction::TaskFinished,
            prerequisites_satisfied: true,
            missing_prerequisites: vec![],
            allowed_operations: vec![],
            guidance_message,
            task_health: TaskHealth::Healthy,
        })
    }

    async fn list_tasks(&self, _filter: TaskFilter) -> Result<Vec<TaskSummary>, anyhow::Error> {
        // TODO: Implement task listing
        Ok(vec![])
    }

    async fn read_task_context(&self, _task_id: TaskId) -> Result<TaskContext, anyhow::Error> {
        // TODO: Implement context reading
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn review_knowledge(&self, _task_id: TaskId, _query: &str) -> Result<KnowledgeReviewResult, anyhow::Error> {
        // TODO: Implement knowledge review
        Ok(KnowledgeReviewResult {
            knowledge_items: vec![],
            required_reading: vec![],
            reviewed_knowledge_ids: vec![],
        })
    }

    async fn confirm_knowledge_reviewed(&self, _task_id: TaskId, _knowledge_ids: Vec<KnowledgeId>) -> Result<(), anyhow::Error> {
        // TODO: Implement knowledge review confirmation
        Ok(())
    }

    async fn start_execution(&self, _task_id: TaskId) -> Result<ExecutionSession, anyhow::Error> {
        // TODO: Implement execution start
        Ok(ExecutionSession {
            session_id: "session_001".to_string(),
            started_at: chrono::Utc::now(),
            timeout: None,
        })
    }

    async fn log_work(&self, _task_id: TaskId, _log: WorkLogEntry) -> Result<(), anyhow::Error> {
        // TODO: Implement work logging
        Ok(())
    }

    async fn finish_work(&self, _task_id: TaskId, _result: WorkSubmission) -> Result<WorkRecordId, anyhow::Error> {
        // TODO: Implement work submission
        Ok(WorkRecordId::new())
    }

    async fn run_quality_check(&self, _task_id: TaskId, _checks: Vec<QualityCheckType>) -> Result<QualityCheckId, anyhow::Error> {
        // TODO: Implement quality check
        Ok(QualityCheckId::new())
    }

    async fn get_quality_result(&self, _check_id: QualityCheckId) -> Result<QualityCheckResult, anyhow::Error> {
        // TODO: Implement quality result retrieval
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn confirm_quality_result(&self, _task_id: TaskId, _check_id: QualityCheckId, _decision: QualityDecision) -> Result<(), anyhow::Error> {
        // TODO: Implement quality result confirmation
        Ok(())
    }

    async fn pause_task(&self, _task_id: TaskId, _reason: String) -> Result<(), anyhow::Error> {
        // TODO: Implement task pause
        Ok(())
    }

    async fn resume_task(&self, _task_id: TaskId) -> Result<(), anyhow::Error> {
        // TODO: Implement task resume
        Ok(())
    }

    async fn handle_requirement_change(&self, _task_id: TaskId, _change: RequirementChange) -> Result<ChangeHandlingResult, anyhow::Error> {
        // TODO: Implement requirement change handling
        Ok(ChangeHandlingResult::CanContinue)
    }

    async fn request_reassignment(&self, task_id: TaskId, reason: String) -> Result<ReassignmentRequest, anyhow::Error> {
        // TODO: Implement reassignment request
        Ok(ReassignmentRequest {
            id: ReassignmentRequestId("req_001".to_string()),
            task_id,
            requested_by: "ai".to_string(),
            reason,
            created_at: chrono::Utc::now(),
            status: ReassignmentStatus::PendingApproval,
        })
    }

    async fn accept_reassigned_task(&self, _task_id: TaskId, _request_id: ReassignmentRequestId) -> Result<TaskHandover, anyhow::Error> {
        // TODO: Implement reassignment acceptance
        Err(anyhow::anyhow!("Not implemented"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // ==================== Type Tests ====================

    #[test]
    fn test_create_task_request_default() {
        let request = CreateTaskRequest {
            title: "Test Task".to_string(),
            description: "Test Description".to_string(),
            goal_id: None,
            phase_id: None,
            estimated_duration: None,
            dependencies: vec![],
        };
        assert_eq!(request.title, "Test Task");
        assert!(request.dependencies.is_empty());
    }

    #[test]
    fn test_task_guidance_structure() {
        let guidance = TaskGuidance {
            current_state: TaskState::Created {
                created_at: Utc::now(),
                created_by: "test".to_string(),
            },
            next_action: NextAction::ReadContext,
            prerequisites_satisfied: true,
            missing_prerequisites: vec![],
            allowed_operations: vec!["read_task_context".to_string()],
            guidance_message: "Test guidance".to_string(),
            task_health: TaskHealth::Healthy,
        };

        assert!(guidance.prerequisites_satisfied);
        assert!(guidance.allowed_operations.contains(&"read_task_context".to_string()));
    }

    #[test]
    fn test_task_health_variants() {
        let healthy = TaskHealth::Healthy;
        let warning = TaskHealth::Warning {
            warnings: vec!["Warning 1".to_string()],
        };
        let attention = TaskHealth::Attention {
            issues: vec![TaskIssue {
                severity: IssueSeverity::Medium,
                description: "Test issue".to_string(),
                suggested_action: "Fix it".to_string(),
            }],
        };
        let critical = TaskHealth::Critical {
            blockers: vec!["Blocker 1".to_string()],
        };

        assert!(matches!(healthy, TaskHealth::Healthy));
        assert!(matches!(warning, TaskHealth::Warning { .. }));
        assert!(matches!(attention, TaskHealth::Attention { .. }));
        assert!(matches!(critical, TaskHealth::Critical { .. }));
    }

    #[test]
    fn test_issue_severity_ordering() {
        assert_eq!(IssueSeverity::Low as u8, 0);
        assert_eq!(IssueSeverity::Medium as u8, 1);
        assert_eq!(IssueSeverity::High as u8, 2);
        assert_eq!(IssueSeverity::Critical as u8, 3);
    }

    #[test]
    fn test_next_action_variants() {
        let actions = vec![
            NextAction::ReadContext,
            NextAction::ReviewKnowledge {
                suggested_queries: vec!["query1".to_string()],
            },
            NextAction::StartExecution {
                suggested_workflow: Some("tdd".to_string()),
            },
            NextAction::ContinueExecution {
                required_logs: vec!["log1".to_string()],
            },
            NextAction::SubmitWork,
            NextAction::RunQualityCheck {
                required_checks: vec![],
            },
            NextAction::FixQualityIssues {
                issues: vec!["issue1".to_string()],
            },
            NextAction::CompleteTask,
            NextAction::TaskFinished,
        ];

        assert_eq!(actions.len(), 9);
    }

    #[test]
    fn test_task_filter_default() {
        let filter = TaskFilter::default();
        assert!(filter.states.is_none());
        assert!(filter.limit.is_none());
    }

    // ==================== Knowledge Review Tests ====================

    #[test]
    fn test_knowledge_review_result() {
        let result = KnowledgeReviewResult {
            knowledge_items: vec![],
            required_reading: vec![],
            reviewed_knowledge_ids: vec![],
        };

        assert!(result.knowledge_items.is_empty());
    }

    #[test]
    fn test_knowledge_item() {
        let item = KnowledgeItem {
            id: KnowledgeId::new(),
            title: "Test Knowledge".to_string(),
            knowledge_type: "best_practice".to_string(),
            summary: "Summary".to_string(),
            detail: "Detail".to_string(),
            relevance_score: 0.95,
        };

        assert!(item.relevance_score > 0.0 && item.relevance_score <= 1.0);
    }

    // ==================== Execution Session Tests ====================

    #[test]
    fn test_execution_session() {
        let session = ExecutionSession {
            session_id: "session_123".to_string(),
            started_at: Utc::now(),
            timeout: Some(std::time::Duration::from_secs(3600)),
        };

        assert_eq!(session.session_id, "session_123");
        assert!(session.timeout.is_some());
    }

    // ==================== Work Log Tests ====================

    #[test]
    fn test_work_log_entry() {
        let entry = WorkLogEntry {
            timestamp: Utc::now(),
            action: WorkAction::Created,
            description: "Created new file".to_string(),
            files: vec!["src/main.rs".to_string()],
            command_output: None,
        };

        assert!(matches!(entry.action, WorkAction::Created));
        assert!(entry.files.contains(&"src/main.rs".to_string()));
    }

    #[test]
    fn test_work_action_variants() {
        let actions = vec![
            WorkAction::Created,
            WorkAction::Modified,
            WorkAction::Tested,
            WorkAction::Documented,
            WorkAction::Debugged,
            WorkAction::Refactored,
        ];

        assert_eq!(actions.len(), 6);
    }

    #[test]
    fn test_command_execution() {
        let exec = CommandExecution {
            command: "cargo".to_string(),
            args: vec!["test".to_string(), "--".to_string(), "--nocapture".to_string()],
            exit_code: 0,
            output: "All tests passed".to_string(),
            timestamp: Utc::now(),
        };

        assert_eq!(exec.exit_code, 0);
        assert!(exec.output.contains("passed"));
    }

    #[test]
    fn test_work_submission() {
        let submission = WorkSubmission {
            description: "Implemented feature X".to_string(),
            artifacts: vec![Artifact {
                name: "feature_x.rs".to_string(),
                artifact_type: ArtifactType::Code,
                path: Some("src/feature_x.rs".to_string()),
                content: None,
            }],
            commands_executed: vec![],
            lessons_learned: Some("Learned about Y".to_string()),
        };

        assert!(submission.lessons_learned.is_some());
        assert!(matches!(submission.artifacts[0].artifact_type, ArtifactType::Code));
    }

    #[test]
    fn test_artifact_type_variants() {
        let types = vec![
            ArtifactType::File,
            ArtifactType::Code,
            ArtifactType::Documentation,
            ArtifactType::Test,
            ArtifactType::Binary,
            ArtifactType::Other,
        ];

        assert_eq!(types.len(), 6);
    }

    // ==================== Quality Decision Tests ====================

    #[test]
    fn test_quality_decision_variants() {
        let decisions = vec![
            QualityDecision::AcceptAndComplete,
            QualityDecision::FixIssuesAndContinue,
            QualityDecision::RedoExecution,
        ];

        assert_eq!(decisions.len(), 3);
    }

    // ==================== Abandon Result Tests ====================

    #[test]
    fn test_abandon_result() {
        let result = AbandonResult {
            success: true,
            can_be_reassigned: true,
            work_reusable: true,
            suggestions_for_next: vec!["Try a different approach".to_string()],
            new_state: TaskState::Abandoned {
                abandoned_at: Utc::now(),
                reason: AbandonReason::Other {
                    reason: "Out of scope".to_string(),
                    details: None,
                },
            },
        };

        assert!(result.success);
        assert!(result.can_be_reassigned);
        assert!(result.work_reusable);
    }

    // ==================== Requirement Change Tests ====================

    #[test]
    fn test_requirement_change_type_variants() {
        let types = vec![
            RequirementChangeType::FeatureChange,
            RequirementChangeType::PriorityChange,
            RequirementChangeType::DeadlineChange,
            RequirementChangeType::DependencyChange,
            RequirementChangeType::QualityRequirementChange,
        ];

        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_change_handling_result_variants() {
        let results = vec![
            ChangeHandlingResult::CanContinue,
            ChangeHandlingResult::NeedsReview {
                suggested_knowledge: vec!["doc1".to_string()],
            },
            ChangeHandlingResult::NeedsReexecution {
                affected_work: vec!["work1".to_string()],
            },
            ChangeHandlingResult::RecommendNewTask {
                reason: "Major refactor needed".to_string(),
                reusable_content: vec!["content1".to_string()],
            },
        ];

        assert_eq!(results.len(), 4);
    }

    // ==================== Reassignment Tests ====================

    #[test]
    fn test_reassignment_request() {
        let request = ReassignmentRequest {
            id: ReassignmentRequestId("req_001".to_string()),
            task_id: TaskId::new(),
            requested_by: "ai_assistant".to_string(),
            reason: "Specialized expertise needed".to_string(),
            created_at: Utc::now(),
            status: ReassignmentStatus::PendingApproval,
        };

        assert!(matches!(request.status, ReassignmentStatus::PendingApproval));
    }

    #[test]
    fn test_reassignment_request_id() {
        let id1 = ReassignmentRequestId("req_001".to_string());
        let id2 = ReassignmentRequestId("req_001".to_string());
        let id3 = ReassignmentRequestId("req_002".to_string());

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_reassignment_status_variants() {
        let statuses = vec![
            ReassignmentStatus::PendingApproval,
            ReassignmentStatus::AwaitingAcceptance,
            ReassignmentStatus::Accepted {
                accepted_by: "human".to_string(),
                accepted_at: Utc::now(),
            },
            ReassignmentStatus::Rejected {
                reason: "Not approved".to_string(),
            },
        ];

        assert_eq!(statuses.len(), 4);
    }

    // ==================== Task Completion Summary Tests ====================

    #[test]
    fn test_task_completion_summary() {
        let summary = TaskCompletionSummary {
            summary: "Completed feature X".to_string(),
            artifacts: vec![Artifact {
                name: "feature_x.rs".to_string(),
                artifact_type: ArtifactType::Code,
                path: None,
                content: Some("fn main() {}".to_string()),
            }],
            lessons_learned: Some("Use trait objects for polymorphism".to_string()),
            created_knowledge: Some(vec![KnowledgeId::new()]),
        };

        assert!(summary.lessons_learned.is_some());
        assert!(summary.created_knowledge.is_some());
    }

    // ==================== ID Generation Tests ====================

    #[test]
    fn test_task_id_generation() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        // Each call should generate a unique ID
        assert_ne!(id1.to_string(), id2.to_string());
        // ID should not be empty
        assert!(!id1.to_string().is_empty());
    }

    #[test]
    fn test_quality_check_id_generation() {
        let id1 = QualityCheckId::new();
        let id2 = QualityCheckId::new();
        assert_ne!(id1.to_string(), id2.to_string());
    }

    #[test]
    fn test_knowledge_id_generation() {
        let id1 = KnowledgeId::new();
        let id2 = KnowledgeId::new();
        assert_ne!(id1.to_string(), id2.to_string());
    }

    #[test]
    fn test_goal_id_generation() {
        let id1 = GoalId::new();
        let id2 = GoalId::new();
        assert_ne!(id1.to_string(), id2.to_string());
    }

    #[test]
    fn test_phase_id_generation() {
        let id1 = PhaseId::new();
        let id2 = PhaseId::new();
        assert_ne!(id1.to_string(), id2.to_string());
    }
}
