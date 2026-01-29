//! Blocker detection.

use devman_core::{Blocker, BlockedItem, PhaseId, Task, TaskId, TaskStatus, Severity};
use devman_storage::Storage;
use std::collections::HashMap;

/// Something that is blocking progress.
#[derive(Clone)]
pub struct BlockerDetector {
    storage: std::sync::Arc<dyn Storage>,
}

impl BlockerDetector {
    /// Create a new blocker detector.
    pub fn new(storage: std::sync::Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Detect all current blockers.
    pub async fn detect_blockers(&self) -> Vec<Blocker> {
        let mut blockers = Vec::new();

        // Get all tasks
        let tasks = match self.storage.list_tasks(&Default::default()).await {
            Ok(t) => t,
            Err(_) => return blockers,
        };

        // Build task map
        let task_map: HashMap<TaskId, Task> = tasks
            .into_iter()
            .map(|t| (t.id, t))
            .collect();

        // Check each blocked task
        for (id, task) in &task_map {
            if task.status == TaskStatus::Blocked {
                // Find what's blocking it by checking dependencies
                for dep_id in &task.depends_on {
                    if let Some(dep) = task_map.get(dep_id) {
                        if !matches!(
                            dep.status,
                            TaskStatus::Done | TaskStatus::Abandoned
                        ) {
                            blockers.push(Blocker {
                                id: devman_core::BlockerId::new(),
                                blocked_item: BlockedItem::Task(*id),
                                reason: format!(
                                    "Blocked by task '{}'",
                                    dep.title
                                ),
                                severity: Severity::Error,
                                created_at: task.updated_at,
                                resolved_at: None,
                            });
                        }
                    }
                }
            }
        }

        blockers
    }

    /// Detect blockers for a specific phase.
    pub async fn detect_phase_blockers(&self, phase_id: PhaseId) -> Vec<Blocker> {
        let mut blockers = Vec::new();

        if let Ok(Some(phase)) = self.storage.load_phase(phase_id).await {
            for task_id in &phase.tasks {
                if let Ok(Some(task)) = self.storage.load_task(*task_id).await {
                    if task.status == TaskStatus::Blocked {
                        blockers.push(Blocker {
                            id: devman_core::BlockerId::new(),
                            blocked_item: BlockedItem::Task(task.id),
                            reason: "Task is blocked".to_string(),
                            severity: Severity::Error,
                            created_at: task.updated_at,
                            resolved_at: None,
                        });
                    }
                }
            }
        }

        blockers
    }
}
