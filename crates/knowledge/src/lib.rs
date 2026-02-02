//! Knowledge Service (Layer 5)
//!
//! Knowledge storage, retrieval, and template management.

#![warn(missing_docs)]

pub mod service;
pub mod template;
pub mod classification;

pub use service::KnowledgeService;
