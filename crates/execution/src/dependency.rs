//! Dependency resolution for tasks.

use devman_core::{Task, TaskId, TaskStatus, LinkKind};
use devman_storage::Storage;
use std::collections::{HashMap, HashSet};

/// Result of dependency resolution.
#[derive(Debug, Clone, PartialEq)]
pub enum Resolution {
    /// All dependencies satisfied, ready to execute
    Ready,
    /// Blocked by these tasks
    Blocked(Vec<TaskId>),
    /// Circular dependency detected
    Circular(Vec<TaskId>),
}

/// Resolves task dependencies.
pub struct DependencyResolver;

impl DependencyResolver {
    /// Create a new resolver.
    pub fn new() -> Self {
        Self
    }

    /// Check if a task's dependencies are satisfied.
    pub async fn check(
        &self,
        storage: &dyn Storage,
        task: &Task,
    ) -> Resolution {
        // Find all DependsOn links
        let dependencies: Vec<_> = task.links
            .iter()
            .filter(|l| l.kind == LinkKind::DependsOn)
            .map(|l| l.to)
            .collect();

        if dependencies.is_empty() {
            return Resolution::Ready;
        }

        let mut blocked = Vec::new();
        let mut visited = HashSet::new();
        visited.insert(task.id);

        for dep_id in dependencies {
            if let Resolution::Blocked(_) = self.check_recursive(storage, dep_id, &mut visited).await {
                blocked.push(dep_id);
            }
        }

        if blocked.is_empty() {
            Resolution::Ready
        } else {
            Resolution::Blocked(blocked)
        }
    }

    /// Check dependencies recursively, detecting cycles.
    async fn check_recursive(
        &self,
        storage: &dyn Storage,
        task_id: TaskId,
        visited: &mut HashSet<TaskId>,
    ) -> Resolution {
        if !visited.insert(task_id) {
            // Cycle detected
            return Resolution::Circular(visited.iter().copied().collect());
        }

        let Some(task) = storage.load_task(task_id).await.ok().flatten() else {
            // Task doesn't exist, consider it blocking
            return Resolution::Blocked(vec![task_id]);
        };

        match task.status {
            TaskStatus::Done => Resolution::Ready,
            TaskStatus::Abandoned => Resolution::Ready, // Abandoned deps don't block
            _ => Resolution::Blocked(vec![task_id]),
        }
    }

    /// Build a dependency graph of all tasks.
    pub async fn build_graph(&self, storage: &dyn Storage) -> Result<DepGraph, anyhow::Error> {
        let all_tasks = storage.list_tasks(&Default::default()).await?;

        let mut graph: HashMap<TaskId, Vec<TaskId>> = HashMap::new();
        let mut reverse: HashMap<TaskId, Vec<TaskId>> = HashMap::new();

        for task in &all_tasks {
            for link in &task.links {
                if link.kind == LinkKind::DependsOn {
                    graph.entry(task.id).or_default().push(link.to);
                    reverse.entry(link.to).or_default().push(task.id);
                }
            }
        }

        Ok(DepGraph { graph, reverse, tasks: all_tasks })
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// A dependency graph.
pub struct DepGraph {
    /// task -> [dependencies]
    graph: HashMap<TaskId, Vec<TaskId>>,
    /// task -> [dependents]
    reverse: HashMap<TaskId, Vec<TaskId>>,
    /// All tasks
    tasks: Vec<Task>,
}

impl DepGraph {
    /// Get tasks in topological order (ready to execute first).
    pub fn topological_sort(&self) -> Vec<Task> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();

        // Find tasks with no dependencies
        let task_ids: HashSet<_> = self.tasks.iter().map(|t| t.id).collect();
        let has_deps: HashSet<_> = self.graph.keys().copied().collect();
        let mut ready: Vec<_> = task_ids.difference(&has_deps).copied().collect();

        while let Some(id) = ready.pop() {
            if !visited.insert(id) {
                continue;
            }

            if let Some(task) = self.tasks.iter().find(|t| t.id == id) {
                sorted.push(task.clone());
            }

            // Add dependents that are now ready
            if let Some(dependents) = self.reverse.get(&id) {
                for dep in dependents {
                    let deps = self.graph.get(dep).map(|v| v.as_slice()).unwrap_or(&[]);
                    if deps.iter().all(|d| visited.contains(d)) {
                        ready.push(*dep);
                    }
                }
            }
        }

        sorted
    }
}
