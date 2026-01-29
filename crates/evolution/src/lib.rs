//! Evolution layer - 自我改进与策略优化。

#![warn(missing_docs, unused_crate_dependencies)]

mod optimizer;
mod metrics;

pub use optimizer::{EvolutionOptimizer, StrategyAdjustment};
pub use metrics::{SystemMetrics, TaskStatistics};
