//! Unique identifiers for DevMan entities.

use serde::{Deserialize, Serialize};
use ulid::Ulid;

// === IDs ===

/// Unique identifier for a Goal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoalId(pub Ulid);

impl GoalId {
    /// Create a new unique goal ID.
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for GoalId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for GoalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for GoalId {
    type Err = ulid::DecodeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

/// Unique identifier for a Project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub Ulid);

impl ProjectId {
    /// Create a new unique project ID.
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Unique identifier for a Phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhaseId(pub Ulid);

impl PhaseId {
    /// Create a new unique phase ID.
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for PhaseId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PhaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Unique identifier for a Task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Ulid);

impl TaskId {
    /// Create a new unique task ID.
    pub fn new() -> Self {
        Self(Ulid::new())
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
pub struct EventId(pub Ulid);

impl EventId {
    /// Create a new unique event ID.
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

/// Unique identifier for a WorkRecord
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkRecordId(pub Ulid);

impl WorkRecordId {
    /// Create a new unique work record ID.
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for WorkRecordId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WorkRecordId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Unique identifier for Knowledge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KnowledgeId(pub Ulid);

impl KnowledgeId {
    /// Create a new unique knowledge ID.
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for KnowledgeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for KnowledgeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Unique identifier for a QualityCheck
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QualityCheckId(pub Ulid);

impl QualityCheckId {
    /// Create a new unique quality check ID.
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for QualityCheckId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for QualityCheckId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Unique identifier for a Blocker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockerId(pub Ulid);

impl BlockerId {
    /// Create a new unique blocker ID.
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for BlockerId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for an Issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IssueId(pub Ulid);

impl IssueId {
    /// Create a new unique issue ID.
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for IssueId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a Criterion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CriterionId(pub Ulid);

impl CriterionId {
    /// Create a new unique criterion ID.
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for CriterionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a QualityProfile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QualityProfileId(pub Ulid);

impl QualityProfileId {
    /// Create a new unique quality profile ID.
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for QualityProfileId {
    fn default() -> Self {
        Self::new()
    }
}

// === Legacy compatibility ===

/// Alias for KnowledgeId (for backward compatibility)
pub type NodeId = KnowledgeId;
