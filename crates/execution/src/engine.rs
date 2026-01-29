//! The main execution engine - runs the cognitive loop.

use crate::{TaskSelector, DependencyResolver, ResourceScheduler, SelectorStrategy};
use devman_core::{AgentId, Event, Task, TaskStatus};
use devman_storage::Storage;
use tracing::{info, debug, error};

/// Configuration for the execution engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Whether to auto-commit after each cycle
    pub auto_commit: bool,
    /// Max cycles before stopping (None = infinite)
    pub max_cycles: Option<usize>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            auto_commit: true,
            max_cycles: None,
        }
    }
}

/// The main execution engine.
///
/// Runs the cognitive loop:
/// ```text
/// Select Task → Execute → Reflect → Update Memory
/// ```
pub struct ExecutionEngine<S: Storage> {
    storage: S,
    selector: Box<dyn TaskSelector>,
    resolver: DependencyResolver,
    scheduler: ResourceScheduler,
    config: EngineConfig,
    cycles_run: usize,
}

impl<S: Storage> ExecutionEngine<S> {
    /// Create a new execution engine.
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            selector: Box::new(SelectorStrategy::Default(Default::default())),
            resolver: DependencyResolver::default(),
            scheduler: ResourceScheduler::default(),
            config: EngineConfig::default(),
            cycles_run: 0,
        }
    }

    /// Set the task selector strategy.
    pub fn with_selector(mut self, selector: Box<dyn TaskSelector>) -> Self {
        self.selector = selector;
        self
    }

    /// Set the configuration.
    pub fn with_config(mut self, config: EngineConfig) -> Self {
        self.config = config;
        self
    }

    /// Run one execution cycle.
    pub async fn run_cycle(&mut self) -> Result<CycleResult, anyhow::Error> {
        info!("Starting execution cycle {}", self.cycles_run + 1);

        // 1. Select task
        let Some(mut task) = self.selector.select(&self.storage).await else {
            info!("No tasks available to execute");
            return Ok(CycleResult::NoTasks);
        };

        debug!("Selected task: {} - {}", task.id, task.intent);

        // 2. Check dependencies
        match self.resolver.check(&self.storage, &task).await {
            crate::dependency::Resolution::Ready => {
                // Ready to execute
            }
            crate::dependency::Resolution::Blocked(by) => {
                debug!("Task blocked by: {:?}", by);
                task.status = TaskStatus::Blocked;
                self.storage.save_task(&task).await?;
                return Ok(CycleResult::Blocked);
            }
            crate::dependency::Resolution::Circular(path) => {
                error!("Circular dependency detected: {:?}", path);
                return Ok(CycleResult::Error(format!("Circular dependency: {:?}", path)));
            }
        }

        // 3. Execute
        task.status = TaskStatus::Active;
        self.storage.save_task(&task).await?;
        self.scheduler.task_started();

        let event = Event::new(
            AgentId::system(),
            "execute_task",
            format!("Executing task: {}", task.intent),
        );

        self.storage.save_event(&event).await?;

        // TODO: Actual execution hook
        info!("Executing task: {}", task.intent);

        // 4. Move to review
        task.status = TaskStatus::Review;
        self.storage.save_task(&task).await?;
        self.scheduler.task_completed();

        // 5. Commit if enabled
        if self.config.auto_commit {
            self.storage.commit(&format!("Cycle {}: executed task {}", self.cycles_run + 1, task.id)).await?;
        }

        self.cycles_run += 1;

        Ok(CycleResult::Success {
            task_id: task.id,
            event_id: event.id,
        })
    }

    /// Run the engine continuously.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            if let Some(max) = self.config.max_cycles {
                if self.cycles_run >= max {
                    info!("Reached max cycles ({})", max);
                    break;
                }
            }

            match self.run_cycle().await? {
                CycleResult::NoTasks => {
                    info!("No more tasks to execute");
                    break;
                }
                CycleResult::Blocked => {
                    continue;
                }
                CycleResult::Error(e) => {
                    error!("Cycle error: {}", e);
                    continue;
                }
                CycleResult::Success { .. } => {
                    // Continue
                }
            }
        }

        Ok(())
    }

    /// Get cycles run so far.
    pub fn cycles(&self) -> usize {
        self.cycles_run
    }

    /// Get a reference to the storage.
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Get a mutable reference to the storage.
    pub fn storage_mut(&mut self) -> &mut S {
        &mut self.storage
    }
}

/// Result of a single execution cycle.
#[derive(Debug)]
pub enum CycleResult {
    /// Successfully executed a task
    Success { task_id: devman_core::TaskId, event_id: devman_core::EventId },
    /// Task was blocked
    Blocked,
    /// No tasks available
    NoTasks,
    /// Error occurred
    Error(String),
}
