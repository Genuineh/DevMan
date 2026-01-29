//! Task selection strategies.

use devman_core::{Task, TaskFilter, TaskStatus};
use devman_storage::Storage;
use async_trait::async_trait;

/// Strategy for selecting the next task to execute.
#[async_trait]
pub trait TaskSelector: Send + Sync {
    /// Select the next task from storage.
    async fn select(&self, storage: &dyn Storage) -> Option<Task>;
}

/// Default selector using priority and status.
pub struct DefaultSelector {
    /// Minimum priority threshold
    min_priority: u8,
}

impl DefaultSelector {
    /// Create a new default selector.
    pub fn new() -> Self {
        Self { min_priority: 0 }
    }

    /// Set minimum priority.
    pub fn with_min_priority(mut self, min_priority: u8) -> Self {
        self.min_priority = min_priority;
        self
    }
}

impl Default for DefaultSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskSelector for DefaultSelector {
    async fn select(&self, storage: &dyn Storage) -> Option<Task> {
        let filter = TaskFilter {
            status: Some(vec![TaskStatus::Queued]),
            min_priority: Some(self.min_priority),
            ..Default::default()
        };

        let mut tasks = storage.list_tasks(&filter).await.ok()?;

        // Sort by priority (descending), then by created_at
        tasks.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        tasks.into_iter().next()
    }
}

/// Selector strategies available.
pub enum SelectorStrategy {
    /// Default priority-based selector
    Default(DefaultSelector),
}

#[async_trait]
impl TaskSelector for SelectorStrategy {
    async fn select(&self, storage: &dyn Storage) -> Option<Task> {
        match self {
            Self::Default(s) => s.select(storage).await,
        }
    }
}
