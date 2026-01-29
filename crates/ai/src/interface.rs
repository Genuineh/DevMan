//! High-level AI interface.

use async_trait::async_trait;
use devman_core::{
    Goal, GoalId, GoalProgress, Knowledge, Phase, PhaseId, QualityCheck, QualityCheckId,
    QualityStatus, Task, TaskId, WorkRecord, WorkResult,
};
use devman_knowledge::KnowledgeService;
use devman_progress::ProgressTracker;
use devman_quality::{QualityEngine, engine::WorkContext as QualityWorkContext};
use devman_tools::ToolInput;
use devman_work::{WorkManager, TaskSpec, WorkManagementContext};
use std::sync::Arc;

/// High-level interface for AI assistants.
#[async_trait]
pub trait AIInterface: Send + Sync {
    // === Context Query ===

    /// Get current work context.
    async fn get_current_context(&self) -> WorkManagementContext;

    // === Knowledge Retrieval ===

    /// Search knowledge by semantic query.
    async fn search_knowledge(&self, query: &str) -> Vec<Knowledge>;

    /// Get best practices for a domain.
    async fn get_best_practices(&self, domain: &str) -> Vec<Knowledge>;

    // === Progress Query ===

    /// Get goal progress.
    async fn get_progress(&self, goal_id: GoalId) -> Option<GoalProgress>;

    /// List current blockers.
    async fn list_blockers(&self) -> Vec<devman_core::Blocker>;

    // === Task Operations ===

    /// Create a new task.
    async fn create_task(&self, spec: TaskSpec) -> Result<Task, anyhow::Error>;

    /// Start executing a task.
    async fn start_task(&self, task_id: TaskId) -> Result<WorkRecord, anyhow::Error>;

    /// Complete a task with result.
    async fn complete_task(&self, task_id: TaskId, result: WorkResult) -> Result<(), anyhow::Error>;

    // === Quality Operations ===

    /// Run a quality check.
    async fn run_quality_check(
        &self,
        check: QualityCheck,
    ) -> devman_core::QualityCheckResult;

    /// Get quality status for a task.
    async fn get_quality_status(&self, task_id: TaskId) -> QualityStatus;

    // === Tool Execution ===

    /// Execute a tool (reduces token usage).
    async fn execute_tool(&self, tool: String, input: ToolInput) -> devman_tools::ToolOutput;

    // === Knowledge Saving ===

    /// Save new knowledge.
    async fn save_knowledge(&self, knowledge: Knowledge) -> Result<(), anyhow::Error>;
}

/// Basic AI interface implementation.
pub struct BasicAIInterface {
    work_manager: Arc<tokio::sync::Mutex<dyn WorkManager>>,
    progress_tracker: Arc<dyn ProgressTracker>,
    knowledge_service: Arc<dyn KnowledgeService>,
    quality_engine: Arc<dyn QualityEngine>,
    tool_executor: Arc<dyn devman_tools::ToolExecutor>,
}

impl BasicAIInterface {
    /// Create a new AI interface.
    pub fn new(
        work_manager: Arc<tokio::sync::Mutex<dyn WorkManager>>,
        progress_tracker: Arc<dyn ProgressTracker>,
        knowledge_service: Arc<dyn KnowledgeService>,
        quality_engine: Arc<dyn QualityEngine>,
        tool_executor: Arc<dyn devman_tools::ToolExecutor>,
    ) -> Self {
        Self {
            work_manager,
            progress_tracker,
            knowledge_service,
            quality_engine,
            tool_executor,
        }
    }
}

#[async_trait]
impl AIInterface for BasicAIInterface {
    async fn get_current_context(&self) -> WorkManagementContext {
        // Return empty context for now
        WorkManagementContext::new()
    }

    async fn search_knowledge(&self, query: &str) -> Vec<Knowledge> {
        self.knowledge_service.search_semantic(query, 10).await
    }

    async fn get_best_practices(&self, domain: &str) -> Vec<Knowledge> {
        self.knowledge_service.get_best_practices(domain).await
    }

    async fn get_progress(&self, goal_id: GoalId) -> Option<GoalProgress> {
        self.progress_tracker.get_goal_progress(goal_id).await
    }

    async fn list_blockers(&self) -> Vec<devman_core::Blocker> {
        // TODO: Implement blocker detection
        Vec::new()
    }

    async fn create_task(&self, spec: TaskSpec) -> Result<Task, anyhow::Error> {
        self.work_manager
            .lock()
            .await
            .create_task(spec)
            .await
    }

    async fn start_task(&self, task_id: TaskId) -> Result<WorkRecord, anyhow::Error> {
        self.work_manager
            .lock()
            .await
            .execute_task(task_id, devman_work::Executor::AI {
                model: "default".to_string(),
            })
            .await
    }

    async fn complete_task(&self, task_id: TaskId, result: WorkResult) -> Result<(), anyhow::Error> {
        self.work_manager
            .lock()
            .await
            .complete_task(task_id, result)
            .await
    }

    async fn run_quality_check(
        &self,
        check: QualityCheck,
    ) -> devman_core::QualityCheckResult {
        // Use a default task_id for quality checks without context
        let task_id = devman_core::TaskId::new();
        self.quality_engine
            .run_check(&check, &QualityWorkContext::new(task_id))
            .await
    }

    async fn get_quality_status(&self, task_id: TaskId) -> QualityStatus {
        // TODO: Implement quality status
        QualityStatus {
            task_id,
            total_checks: 0,
            passed_checks: 0,
            failed_checks: 0,
            warnings: 0,
            overall_status: devman_core::QualityOverallStatus::NotChecked,
            pending_human_review: false,
        }
    }

    async fn execute_tool(&self, tool: String, input: ToolInput) -> devman_tools::ToolOutput {
        self.tool_executor.execute_tool(&tool, input).await.unwrap_or_else(
            |e| devman_tools::ToolOutput {
                exit_code: -1,
                stdout: String::new(),
                stderr: e.to_string(),
                duration: std::time::Duration::ZERO,
            },
        )
    }

    async fn save_knowledge(&self, knowledge: Knowledge) -> Result<(), anyhow::Error> {
        // TODO: Implement knowledge saving
        Ok(())
    }
}
