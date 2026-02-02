//! Progress Tracking (Layer 3)
//!
//! Goal progress, phase milestones, and blocker detection.

#![warn(missing_docs)]

pub mod tracker;
pub mod blocker;
pub mod estimator;

pub use tracker::{ProgressTracker, ProgressSnapshot};
pub use blocker::{
    BlockerDetector, BlockerAnalysis, BlockerStats, ResolutionSuggestion, ResolutionAction,
};
pub use estimator::CompletionEstimator;
