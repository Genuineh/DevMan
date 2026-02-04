//! Knowledge Service (Layer 5)
//!
//! Knowledge storage, retrieval, and template management.

#![warn(missing_docs)]

pub mod service;
pub mod template;
pub mod classification;
pub mod vector;

pub use service::{KnowledgeService, BasicKnowledgeService};
pub use vector::{VectorKnowledgeService, VectorKnowledgeServiceImpl, OllamaEmbeddingClient};
