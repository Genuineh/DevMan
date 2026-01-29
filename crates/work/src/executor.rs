//! Task execution.

use async_trait::async_trait;
use devman_core::{PhaseId, Task, TaskId, WorkRecord, WorkRecordId};
use devman_storage::Storage;

/// Result of executing a single step.
#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_order: usize,
    pub success: bool,
    pub output: String,
    pub error: String,
    pub duration: std::time::Duration,
}

/// Task executor.
#[async_trait]
pub trait TaskExecutor: Send + Sync {
    /// Execute a task and record the work.
    async fn execute(&mut self, task: &Task) -> Result<WorkRecord, anyhow::Error>;
}

/// Basic task executor.
pub struct BasicTaskExecutor<S: Storage> {
    storage: std::sync::Arc<tokio::sync::Mutex<S>>,
    tool_executor: std::sync::Arc<dyn devman_tools::ToolExecutor>,
}

impl<S: Storage> BasicTaskExecutor<S> {
    /// Create a new executor.
    pub fn new(
        storage: S,
        tool_executor: std::sync::Arc<dyn devman_tools::ToolExecutor>,
    ) -> Self {
        Self {
            storage: std::sync::Arc::new(tokio::sync::Mutex::new(storage)),
            tool_executor,
        }
    }

    /// Execute task steps.
    async fn execute_steps(
        &self,
        task: &Task,
    ) -> Result<Vec<StepResult>, anyhow::Error> {
        let mut results = Vec::new();

        for step in &task.steps {
            let step_start = std::time::Instant::now();

            // Execute tool call
            use devman_tools::ToolInput;
            let input = ToolInput {
                args: step.tool.args.clone(),
                env: Default::default(),
                stdin: None,
                timeout: Some(std::time::Duration::from_secs(300)),
            };

            let output = self
                .tool_executor
                .execute_tool(&step.tool.tool, input)
                .await?;

            results.push(StepResult {
                step_order: step.order,
                success: output.exit_code == 0,
                output: output.stdout,
                error: output.stderr,
                duration: step_start.elapsed(),
            });

            // Stop if step failed
            if output.exit_code != 0 {
                break;
            }
        }

        Ok(results)
    }
}

#[async_trait::async_trait]
impl<S: Storage> TaskExecutor for BasicTaskExecutor<S> {
    async fn execute(&mut self, task: &Task) -> Result<WorkRecord, anyhow::Error> {
        let started_at = chrono::Utc::now();

        // Execute steps
        let step_results = self.execute_steps(task).await?;

        let all_success = step_results.iter().all(|r| r.success);

        let completed_at = chrono::Utc::now();
        let duration = completed_at - started_at;

        let mut record = WorkRecord {
            id: WorkRecordId::new(),
            task_id: task.id,
            executor: devman_core::Executor::AI {
                model: "basic".to_string(),
            },
            started_at,
            completed_at: Some(completed_at),
            duration: Some(duration),
            events: Vec::new(),
            result: devman_core::WorkResult {
                status: if all_success {
                    devman_core::CompletionStatus::Success
                } else {
                    devman_core::CompletionStatus::Failed
                },
                outputs: step_results
                    .iter()
                    .map(|r| devman_core::Output {
                        name: "stdout".to_string(),
                        value: r.output.clone(),
                    })
                    .collect(),
                metrics: devman_core::WorkMetrics {
                    token_used: None,
                    time_spent: duration.to_std().unwrap_or_default(),
                    tools_invoked: step_results.len(),
                    quality_checks_run: 0,
                    quality_checks_passed: 0,
                },
            },
            artifacts: Vec::new(),
            issues: Vec::new(),
            resolutions: Vec::new(),
        };

        // Save record
        self.storage.lock().await.save_work_record(&record).await?;

        Ok(record)
    }
}
