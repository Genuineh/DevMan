//! Reflection engine - generates insights and new tasks.

use crate::Analyzer;
use devman_core::{Event, KnowledgeNode, Task, TaskId, Time};
use devman_storage::Storage;
use tracing::{info, debug};

/// Configuration for the reflection engine.
#[derive(Debug, Clone)]
pub struct ReflectionConfig {
    /// Whether to auto-generate new tasks
    pub auto_generate_tasks: bool,
    /// Minimum confidence to accept insights
    pub min_insight_confidence: f32,
}

impl Default for ReflectionConfig {
    fn default() -> Self {
        Self {
            auto_generate_tasks: true,
            min_insight_confidence: 0.6,
        }
    }
}

/// Generates reflections from completed tasks.
pub struct ReflectionEngine<S: Storage> {
    storage: S,
    analyzer: Analyzer,
    config: ReflectionConfig,
}

impl<S: Storage> ReflectionEngine<S> {
    /// Create a new reflection engine.
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            analyzer: Analyzer::new(),
            config: ReflectionConfig::default(),
        }
    }

    /// Set the configuration.
    pub fn with_config(mut self, config: ReflectionConfig) -> Self {
        self.config = config;
        self
    }

    /// Reflect on a completed task.
    pub async fn reflect_on(&mut self, task_id: TaskId) -> Result<ReflectionReport, anyhow::Error> {
        info!("Reflecting on task {}", task_id);

        let task = self.storage.load_task(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;

        // Get related events
        let mut events = Vec::new();
        for &id in &task.logs {
            if let Ok(Some(event)) = self.storage.load_event(id).await {
                events.push(event);
            }
        }

        // Analyze
        let insight = self.analyzer.analyze(&task, &events);

        // Generate report
        let report = ReflectionReport {
            task_id,
            success: insight.success,
            insight: insight.summary,
            confidence_delta: insight.confidence_adjustment,
            derived_tasks: if self.config.auto_generate_tasks {
                insight.suggested_tasks
            } else {
                Vec::new()
            },
            knowledge_updates: insight.knowledge_gained,
            generated_at: chrono::Utc::now(),
        };

        // Save knowledge updates
        for node in &report.knowledge_updates {
            self.storage.save_knowledge(node).await?;
            debug!("Saved knowledge: {}", node.claim);
        }

        Ok(report)
    }

    /// Reflect on all tasks in Review status.
    pub async fn reflect_all(&mut self) -> Result<Vec<ReflectionReport>, anyhow::Error> {
        let tasks = self.storage.list_tasks(
            &devman_core::TaskFilter {
                status: Some(vec![devman_core::TaskStatus::Review]),
                ..Default::default()
            }
        ).await?;

        let mut reports = Vec::new();
        for task in tasks {
            let report = self.reflect_on(task.id).await?;
            reports.push(report);
        }

        Ok(reports)
    }
}

/// Report generated from reflecting on a task.
#[derive(Debug, Clone)]
pub struct ReflectionReport {
    /// The task that was reflected on
    pub task_id: TaskId,
    /// Whether the task was successful
    pub success: bool,
    /// Key insight from the reflection
    pub insight: String,
    /// Adjustment to confidence (-1.0 to 1.0)
    pub confidence_delta: f32,
    /// New tasks derived from this reflection
    pub derived_tasks: Vec<Task>,
    /// New knowledge gained
    pub knowledge_updates: Vec<KnowledgeNode>,
    /// When the report was generated
    pub generated_at: Time,
}
