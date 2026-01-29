//! Work management service.

use async_trait::async_trait;
use devman_core::{
    PhaseId, QualityGate, Task, TaskId, TaskProgress, WorkEvent, WorkRecord, WorkRecordId,
    WorkResult,
};
use devman_storage::Storage;

/// Work management service.
#[async_trait]
pub trait WorkManager: Send + Sync {
    /// Create a new task.
    async fn create_task(&mut self, spec: TaskSpec) -> Result<Task, anyhow::Error>;

    /// Execute a task.
    async fn execute_task(
        &mut self,
        task_id: TaskId,
        executor: Executor,
    ) -> Result<WorkRecord, anyhow::Error>;

    /// Record a work event.
    async fn record_event(
        &mut self,
        task_id: TaskId,
        event: WorkEvent,
    ) -> Result<(), anyhow::Error>;

    /// Update task progress.
    async fn update_progress(
        &mut self,
        task_id: TaskId,
        progress: TaskProgress,
    ) -> Result<(), anyhow::Error>;

    /// Complete a task.
    async fn complete_task(
        &mut self,
        task_id: TaskId,
        result: WorkResult,
    ) -> Result<(), anyhow::Error>;
}

/// Specification for creating a task.
#[derive(Debug, Clone)]
pub struct TaskSpec {
    pub title: String,
    pub description: String,
    pub intent: devman_core::TaskIntent,
    pub phase_id: PhaseId,
    pub quality_gates: Vec<QualityGate>,
}

/// Who/what is executing work.
#[derive(Debug, Clone)]
pub enum Executor {
    AI { model: String },
    Human { name: String },
    Hybrid { ai: String, human: String },
}

/// Basic work manager implementation.
pub struct BasicWorkManager<S: Storage> {
    storage: std::sync::Arc<tokio::sync::Mutex<S>>,
    quality_engine: Option<std::sync::Arc<dyn devman_quality::QualityEngine>>,
}

impl<S: Storage> BasicWorkManager<S> {
    /// Create a new work manager.
    pub fn new(storage: S) -> Self {
        Self {
            storage: std::sync::Arc::new(tokio::sync::Mutex::new(storage)),
            quality_engine: None,
        }
    }

    /// Set quality engine.
    pub fn with_quality_engine(
        mut self,
        engine: std::sync::Arc<dyn devman_quality::QualityEngine>,
    ) -> Self {
        self.quality_engine = Some(engine);
        self
    }
}

#[async_trait]
impl<S: Storage + 'static> WorkManager for BasicWorkManager<S> {
    async fn create_task(&mut self, spec: TaskSpec) -> Result<Task, anyhow::Error> {
        let task = Task {
            id: devman_core::TaskId::new(),
            title: spec.title,
            description: spec.description,
            intent: spec.intent,
            steps: Vec::new(),
            inputs: Vec::new(),
            expected_outputs: Vec::new(),
            quality_gates: spec.quality_gates,
            status: devman_core::TaskStatus::Queued,
            progress: devman_core::TaskProgress::default(),
            phase_id: spec.phase_id,
            depends_on: Vec::new(),
            blocks: Vec::new(),
            work_records: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.storage.lock().await.save_task(&task).await?;
        Ok(task)
    }

    async fn execute_task(
        &mut self,
        task_id: TaskId,
        executor: Executor,
    ) -> Result<WorkRecord, anyhow::Error> {
        let mut task = self
            .storage
            .lock()
            .await
            .load_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.status = devman_core::TaskStatus::Active;
        self.storage.lock().await.save_task(&task).await?;

        let started_at = chrono::Utc::now();
        let mut work_record = WorkRecord {
            id: devman_core::WorkRecordId::new(),
            task_id,
            executor: match executor {
                Executor::AI { model } => devman_core::Executor::AI { model },
                Executor::Human { name } => devman_core::Executor::Human { name },
                Executor::Hybrid { ai, human } => devman_core::Executor::Hybrid { ai, human },
            },
            started_at,
            completed_at: None,
            duration: None,
            events: Vec::new(),
            result: devman_core::WorkResult {
                status: devman_core::CompletionStatus::Running,
                outputs: Vec::new(),
                metrics: devman_core::WorkMetrics {
                    token_used: None,
                    time_spent: std::time::Duration::ZERO,
                    tools_invoked: 0,
                    quality_checks_run: 0,
                    quality_checks_passed: 0,
                },
            },
            artifacts: Vec::new(),
            issues: Vec::new(),
            resolutions: Vec::new(),
        };

        self.storage.lock().await.save_work_record(&work_record).await?;

        Ok(work_record)
    }

    async fn record_event(
        &mut self,
        task_id: TaskId,
        event: WorkEvent,
    ) -> Result<(), anyhow::Error> {
        let mut task = self
            .storage
            .lock()
            .await
            .load_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // Get active work record
        let record_id = *task.work_records.last().ok_or_else(|| {
            anyhow::anyhow!("No active work record for task")
        })?;

        let mut record = self
            .storage
            .lock()
            .await
            .load_work_record(record_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Work record not found"))?;

        record.events.push(event);
        self.storage.lock().await.save_work_record(&record).await?;

        Ok(())
    }

    async fn update_progress(
        &mut self,
        task_id: TaskId,
        progress: TaskProgress,
    ) -> Result<(), anyhow::Error> {
        let mut task = self
            .storage
            .lock()
            .await
            .load_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.progress = progress;
        task.updated_at = chrono::Utc::now();
        self.storage.lock().await.save_task(&task).await?;

        Ok(())
    }

    async fn complete_task(
        &mut self,
        task_id: TaskId,
        result: WorkResult,
    ) -> Result<(), anyhow::Error> {
        let mut task = self
            .storage
            .lock()
            .await
            .load_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.status = match result.status {
            devman_core::CompletionStatus::Success => devman_core::TaskStatus::Done,
            devman_core::CompletionStatus::Failed => devman_core::TaskStatus::Review,
            _ => devman_core::TaskStatus::Review,
        };
        task.updated_at = chrono::Utc::now();
        self.storage.lock().await.save_task(&task).await?;

        // Update work record
        let record_id = *task.work_records.last().ok_or_else(|| {
            anyhow::anyhow!("No active work record for task")
        })?;

        let mut record = self
            .storage
            .lock()
            .await
            .load_work_record(record_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Work record not found"))?;

        record.completed_at = Some(chrono::Utc::now());
        record.duration = Some(record.completed_at.unwrap() - record.started_at);
        record.result = result;
        self.storage.lock().await.save_work_record(&record).await?;

        Ok(())
    }
}
