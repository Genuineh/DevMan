//! Resource scheduling and budget management.

use std::time::Duration;
use std::num::NonZeroUsize;

/// Budget for task execution.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    /// Time budget per task
    pub time_per_task: Duration,
    /// Max concurrent tasks
    pub max_concurrent: NonZeroUsize,
    /// API call limit (for AI operations)
    pub api_calls_per_cycle: u32,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            time_per_task: Duration::from_secs(300), // 5 minutes
            max_concurrent: NonZeroUsize::new(1).unwrap(),
            api_calls_per_cycle: 100,
        }
    }
}

impl Budget {
    /// Create a new budget.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set time budget per task.
    pub fn with_time_per_task(mut self, duration: Duration) -> Self {
        self.time_per_task = duration;
        self
    }

    /// Set max concurrent tasks.
    pub fn with_max_concurrent(mut self, max: NonZeroUsize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Set API call limit.
    pub fn with_api_limit(mut self, limit: u32) -> Self {
        self.api_calls_per_cycle = limit;
        self
    }
}

/// Manages execution resources and scheduling.
pub struct ResourceScheduler {
    budget: Budget,
    active_tasks: usize,
    api_calls_used: u32,
}

impl ResourceScheduler {
    /// Create a new scheduler.
    pub fn new(budget: Budget) -> Self {
        Self {
            budget,
            active_tasks: 0,
            api_calls_used: 0,
        }
    }

    /// Check if we can start a new task.
    pub fn can_start(&self) -> bool {
        self.active_tasks < self.budget.max_concurrent.get()
            && self.api_calls_used < self.budget.api_calls_per_cycle
    }

    /// Register a task start.
    pub fn task_started(&mut self) {
        self.active_tasks += 1;
    }

    /// Register a task completion.
    pub fn task_completed(&mut self) {
        self.active_tasks = self.active_tasks.saturating_sub(1);
    }

    /// Get remaining budget.
    pub fn remaining_api_calls(&self) -> u32 {
        self.budget.api_calls_per_cycle.saturating_sub(self.api_calls_used)
    }

    /// Reset API call budget for new cycle.
    pub fn reset_api_budget(&mut self) {
        self.api_calls_used = 0;
    }
}

impl Default for ResourceScheduler {
    fn default() -> Self {
        Self::new(Budget::default())
    }
}
