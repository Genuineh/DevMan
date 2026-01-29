//! Knowledge model - cognitive graph nodes.

use crate::{EventId, NodeId, Time};
use serde::{Deserialize, Serialize};

/// A node in the knowledge graph representing a claim with confidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    /// Unique identifier
    pub id: NodeId,

    /// The claim or fact
    pub claim: String,

    /// Confidence level (0-1)
    pub confidence: f32,

    /// What this was derived from
    pub derived_from: Vec<NodeId>,

    /// Supporting evidence
    pub evidence: Vec<EventId>,

    /// When created
    pub created_at: Time,
}

impl KnowledgeNode {
    /// Create a new knowledge node.
    pub fn new(claim: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: NodeId::new(),
            claim: claim.into(),
            confidence: 0.5,
            derived_from: Vec::new(),
            evidence: Vec::new(),
            created_at: now,
        }
    }

    /// Create a fact with high confidence.
    pub fn fact(claim: impl Into<String>) -> Self {
        let mut node = Self::new(claim);
        node.confidence = 0.95;
        node
    }

    /// Create a hypothesis with low confidence.
    pub fn hypothesis(claim: impl Into<String>) -> Self {
        let mut node = Self::new(claim);
        node.confidence = 0.3;
        node
    }
}

/// An update to knowledge (delta).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeUpdate {
    /// The claim to add/update
    pub claim: String,

    /// Confidence adjustment (positive or negative delta)
    pub confidence_delta: f32,

    /// Source event
    pub evidence: EventId,
}
