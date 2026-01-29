//! Git+JSON storage implementation.
//!
//! Stores data as JSON files in a directory, with Git tracking all changes.

use std::path::Path;
use std::sync::Arc;
use devman_core::{
    Goal, GoalId, Project, ProjectId, Phase, PhaseId, Task, TaskId, TaskFilter,
    Event, EventId, Knowledge, KnowledgeId, QualityCheck, QualityCheckId,
    WorkRecord, WorkRecordId,
};
use super::{Storage, StorageError, Result};
use tokio::fs;

use tokio::sync::Mutex;

/// File-based JSON storage backend.
///
/// This storage stores objects as JSON files under the `.devman/` directory.
/// It maintains a small per-object meta file that records a version marker and
/// timestamp. Full snapshot/version history is expected to be managed by the
/// project's own Git repository; this storage will NOT manage Git or full
/// snapshot archives.
// Compatibility shim: `git_json` historical module name.
// Re-export the canonical implementation from `json_storage`.

pub use crate::json_storage::JsonStorage;

