//! Quality Assurance (Layer 4)
//!
//! Quality checks, gates, and human collaboration.

#![warn(missing_docs)]

pub mod engine;
pub mod checks;
pub mod custom;
pub mod registry;
pub mod gate;
pub mod human;

pub use engine::QualityEngine;
pub use checks::{
    GenericCheckType, QualityCheckType, CustomCheckSpec,
    CommandSpec, ValidationSpec, OutputParser, MetricExtractor,
    HumanReviewSpec, ReviewQuestion, AnswerType, AnswerValue,
};
pub use gate::{QualityGateBuilder, QualityProfileBuilder};
pub use registry::QualityCheckRegistry;
