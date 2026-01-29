//! Execution layer - task selection, dependency resolution, and scheduling.

#![warn(missing_docs)]

pub mod selector;
pub mod dependency;
pub mod scheduler;
pub mod engine;

pub use selector::{TaskSelector, SelectorStrategy};
pub use dependency::{DependencyResolver, Resolution};
pub use scheduler::{ResourceScheduler, Budget};
pub use engine::{ExecutionEngine, EngineConfig, CycleResult};
