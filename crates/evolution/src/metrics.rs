//! System metrics for evolution.

use devman_core::TaskStatus;
use serde::{Deserialize, Serialize};

/// Metrics about task execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskStatistics {
    /// Total tasks by status
    pub by_status: std::collections::HashMap<TaskStatus, usize>,
    /// Average confidence by status
    pub avg_confidence: std::collections::HashMap<TaskStatus, f32>,
    /// Failure rate
    pub failure_rate: f32,
    /// Average execution time (milliseconds)
    pub avg_duration_ms: u64,
}

/// Overall system metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Task statistics
    pub tasks: TaskStatistics,
    /// Reflection success rate
    pub reflection_success_rate: f32,
    /// Most common failure reasons
    pub common_failures: Vec<String>,
}
