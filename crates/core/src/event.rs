//! Event model - atoms of the timeline.

use crate::id::{EventId, TaskId};
use crate::Time;
use serde::{Deserialize, Serialize};

/// An event is an atomic unit that happened at a specific time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier
    pub id: EventId,

    /// When it happened
    pub timestamp: Time,

    /// Who performed the action
    pub actor: AgentId,

    /// What action was taken
    pub action: String,

    /// What was the result
    pub result: String,

    /// Knowledge gained from this event
    pub delta_knowledge: Vec<KnowledgeUpdate>,

    /// Tasks related to this event
    pub related_tasks: Vec<TaskId>,
}

impl Event {
    /// Create a new event.
    pub fn new(
        actor: AgentId,
        action: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: EventId::new(),
            timestamp: now,
            actor,
            action: action.into(),
            result: result.into(),
            delta_knowledge: Vec::new(),
            related_tasks: Vec::new(),
        }
    }
}

/// Identifier for an agent (could be AI, human, or system).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    /// Create a new agent ID.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// System agent ID
    pub fn system() -> Self {
        Self("system".to_string())
    }

    /// AI agent ID
    pub fn ai() -> Self {
        Self("ai".to_string())
    }

    /// User agent ID
    pub fn user() -> Self {
        Self("user".to_string())
    }
}

/// Knowledge update derived from an event.
use super::knowledge::KnowledgeUpdate;
