//! Unique identifiers for DevMan entities.

use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Unique identifier for a Task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(Ulid);

impl TaskId {
    /// Generate a new TaskId
    pub fn new() -> Self {
        Self(Ulid::new())
    }

    /// Create from string
    pub fn from_str(s: &str) -> Result<Self, ulid::DecodeError> {
        Ok(Self(s.parse()?))
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for TaskId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

/// Unique identifier for an Event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(Ulid);

impl EventId {
    /// Generate a new EventId
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Unique identifier for a KnowledgeNode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Ulid);

impl NodeId {
    /// Generate a new NodeId
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
