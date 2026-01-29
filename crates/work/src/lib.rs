//! Work Management (Layer 2)
//!
//! Task execution, work recording, and context management.

#![warn(missing_docs)]

pub mod manager;
pub mod context;
pub mod executor;

pub use manager::{WorkManager, TaskSpec, Executor};
pub use context::WorkManagementContext;
pub use executor::TaskExecutor;
