//! Blocker detection and resolution.
//!
//! This module provides advanced blocker detection capabilities:
//! - Dependency-based blocker detection
//! - Circular dependency detection
//! - Auto-resolution suggestions
//! - Blocker statistics and reporting

use devman_core::{
    Blocker, BlockedItem, PhaseId, Task, TaskId, TaskStatus, Severity, GoalId, Phase,
};
use devman_storage::Storage;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Blocker resolution suggestion.
#[derive(Debug, Clone)]
pub struct ResolutionSuggestion {
    /// The type of resolution
    pub action: ResolutionAction,
    /// Description of the suggested action
    pub description: String,
    /// Priority of this resolution (lower = higher priority)
    pub priority: u32,
}

/// Actions that can resolve a blocker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionAction {
    /// Complete the blocking task
    CompleteTask,
    /// Abandon the blocking task
    AbandonTask,
    /// Skip the blocked task
    SkipTask,
    /// Modify dependencies
    ModifyDependencies,
    /// Manual intervention required
    ManualReview,
    /// Wait for external factor
    Wait,
}

impl ResolutionAction {
    /// Get string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            ResolutionAction::CompleteTask => "CompleteTask",
            ResolutionAction::AbandonTask => "AbandonTask",
            ResolutionAction::SkipTask => "SkipTask",
            ResolutionAction::ModifyDependencies => "ModifyDependencies",
            ResolutionAction::ManualReview => "ManualReview",
            ResolutionAction::Wait => "Wait",
        }
    }
}

/// Blocker statistics.
#[derive(Debug, Clone, Default)]
pub struct BlockerStats {
    /// Total blockers detected
    pub total_blockers: usize,
    /// Blockers by severity
    pub by_severity: HashMap<Severity, usize>,
    /// Blockers by item type
    pub by_item_type: HashMap<String, usize>,
    /// Average blocker age in hours
    pub average_age_hours: Option<f32>,
    /// Circular dependency count
    pub circular_dependencies: usize,
}

/// Result of blocker analysis.
#[derive(Debug, Clone)]
pub struct BlockerAnalysis {
    /// All detected blockers
    pub blockers: Vec<Blocker>,
    /// Suggested resolutions
    pub suggestions: Vec<ResolutionSuggestion>,
    /// Blocker statistics
    pub stats: BlockerStats,
    /// Circular dependency chains
    pub circular_chains: Vec<Vec<TaskId>>,
}

/// Something that is blocking progress.
#[derive(Clone)]
pub struct BlockerDetector {
    storage: Arc<dyn Storage>,
}

impl BlockerDetector {
    /// Create a new blocker detector.
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Detect all current blockers with full analysis.
    pub async fn detect_and_analyze(&self) -> BlockerAnalysis {
        let tasks = match self.storage.list_tasks(&Default::default()).await {
            Ok(t) => t,
            Err(_) => return BlockerAnalysis::default(),
        };

        // Build task map
        let task_map: HashMap<TaskId, Task> = tasks
            .into_iter()
            .map(|t| (t.id, t.clone()))
            .collect();

        // Detect dependency-based blockers
        let dependency_blockers = self.detect_dependency_blockers(&task_map);

        // Detect circular dependencies
        let (circular_chains, circular_blockers) = self.detect_circular_dependencies(&task_map);

        // Combine all blockers
        let mut all_blockers = dependency_blockers;
        all_blockers.extend(circular_blockers);

        // Generate resolution suggestions
        let suggestions = self.generate_suggestions(&task_map, &all_blockers);

        // Calculate statistics
        let stats = self.calculate_stats(&all_blockers);

        BlockerAnalysis {
            blockers: all_blockers,
            suggestions,
            stats,
            circular_chains,
        }
    }

    /// Detect blockers based on task dependencies.
    fn detect_dependency_blockers(&self, task_map: &HashMap<TaskId, Task>) -> Vec<Blocker> {
        let mut blockers = Vec::new();

        for (id, task) in task_map {
            if task.status == TaskStatus::Blocked {
                // Check if blocked by dependency
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
                                    "Blocked by task '{}' (status: {:?})",
                                    dep.title, dep.status
                                ),
                                severity: Severity::Error,
                                created_at: task.updated_at,
                                resolved_at: None,
                            });
                        }
                    } else {
                        // Dependency task not found
                        blockers.push(Blocker {
                            id: devman_core::BlockerId::new(),
                            blocked_item: BlockedItem::Task(*id),
                            reason: format!(
                                "Blocked by missing or deleted dependency: {}",
                                dep_id
                            ),
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

    /// Detect circular dependencies in task graph.
    fn detect_circular_dependencies(
        &self,
        task_map: &HashMap<TaskId, Task>,
    ) -> (Vec<Vec<TaskId>>, Vec<Blocker>) {
        let mut cycles = Vec::new();
        let mut blockers = Vec::new();
        let mut visited: HashSet<TaskId> = HashSet::new();
        let mut recursion_stack: HashSet<TaskId> = HashSet::new();

        for start_id in task_map.keys() {
            if !visited.contains(start_id) {
                if let Some(cycle) =
                    self.find_cycle(start_id, task_map, &mut visited, &mut recursion_stack, &mut Vec::new())
                {
                    cycles.push(cycle.clone());

                    // Create blocker for each task in the cycle
                    for task_id in &cycle {
                        if let Some(task) = task_map.get(task_id) {
                            blockers.push(Blocker {
                                id: devman_core::BlockerId::new(),
                                blocked_item: BlockedItem::Task(*task_id),
                                reason: format!(
                                    "Circular dependency detected: task is part of a dependency cycle"
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

        (cycles, blockers)
    }

    /// Find a cycle starting from a node using DFS.
    fn find_cycle(
        &self,
        node: &TaskId,
        task_map: &HashMap<TaskId, Task>,
        visited: &mut HashSet<TaskId>,
        recursion_stack: &mut HashSet<TaskId>,
        path: &mut Vec<TaskId>,
    ) -> Option<Vec<TaskId>> {
        visited.insert(*node);
        recursion_stack.insert(*node);
        path.push(*node);

        if let Some(task) = task_map.get(node) {
            for dep_id in &task.depends_on {
                if !visited.contains(dep_id) {
                    if let Some(cycle) = self.find_cycle(
                        dep_id,
                        task_map,
                        visited,
                        recursion_stack,
                        path,
                    ) {
                        return Some(cycle);
                    }
                } else if recursion_stack.contains(dep_id) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|id| id == dep_id).unwrap();
                    return Some(path[cycle_start..].to_vec());
                }
            }
        }

        path.pop();
        recursion_stack.remove(node);
        None
    }

    /// Generate resolution suggestions for blockers.
    fn generate_suggestions(
        &self,
        task_map: &HashMap<TaskId, Task>,
        blockers: &[Blocker],
    ) -> Vec<ResolutionSuggestion> {
        let mut suggestions = Vec::new();

        for blocker in blockers {
            if let BlockedItem::Task(task_id) = &blocker.blocked_item {
                if let Some(task) = task_map.get(task_id) {
                    // Check what the dependencies are
                    let mut blocking_tasks: Vec<&Task> = Vec::new();
                    for dep_id in &task.depends_on {
                        if let Some(dep) = task_map.get(dep_id) {
                            if !matches!(
                                dep.status,
                                TaskStatus::Done | TaskStatus::Abandoned
                            ) {
                                blocking_tasks.push(dep);
                            }
                        }
                    }

                    if !blocking_tasks.is_empty() {
                        // Suggest completing or abandoning blocking tasks
                        for blocking_task in &blocking_tasks {
                            if blocking_task.progress.percentage > 50.0 {
                                suggestions.push(ResolutionSuggestion {
                                    action: ResolutionAction::CompleteTask,
                                    description: format!(
                                        "Complete blocking task '{}' to unblock '{}'",
                                        blocking_task.title, task.title
                                    ),
                                    priority: 1,
                                });
                            } else {
                                suggestions.push(ResolutionSuggestion {
                                    action: ResolutionAction::AbandonTask,
                                    description: format!(
                                        "Consider abandoning blocking task '{}' or unblocking '{}'",
                                        blocking_task.title, task.title
                                    ),
                                    priority: 2,
                                });
                            }
                        }
                    } else if task.depends_on.is_empty() && task.status == TaskStatus::Blocked {
                        // Blocked without dependencies - needs manual review
                        suggestions.push(ResolutionSuggestion {
                            action: ResolutionAction::ManualReview,
                            description: format!(
                                "Task '{}' is blocked but has no dependencies - manual review required",
                                task.title
                            ),
                            priority: 1,
                        });
                    }
                }
            }
        }

        suggestions.sort_by(|a, b| a.priority.cmp(&b.priority));
        suggestions
    }

    /// Calculate blocker statistics.
    fn calculate_stats(&self, blockers: &[Blocker]) -> BlockerStats {
        let mut stats = BlockerStats::default();
        stats.total_blockers = blockers.len();

        let now = chrono::Utc::now();
        let mut total_age_hours = 0.0f32;
        let mut count = 0;

        for blocker in blockers {
            *stats.by_severity.entry(blocker.severity).or_insert(0) += 1;

            let item_type = match blocker.blocked_item {
                BlockedItem::Task(_) => "task",
                BlockedItem::Phase(_) => "phase",
                BlockedItem::Goal(_) => "goal",
            };
            *stats.by_item_type.entry(item_type.to_string()).or_insert(0) += 1;

            // Calculate age
            let age = now.signed_duration_since(blocker.created_at);
            total_age_hours += age.num_hours() as f32;
            count += 1;
        }

        if count > 0 {
            stats.average_age_hours = Some(total_age_hours / count as f32);
        }

        // Count circular dependencies
        stats.circular_dependencies = blockers
            .iter()
            .filter(|b| b.reason.contains("Circular dependency"))
            .count();

        stats
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

    /// Detect blockers for a specific goal.
    pub async fn detect_goal_blockers(&self, goal_id: GoalId) -> Vec<Blocker> {
        let mut blockers = Vec::new();

        if let Ok(Some(goal)) = self.storage.load_goal(goal_id).await {
            if let Ok(Some(project)) = self.storage.load_project(goal.project_id).await {
                for phase_id in &project.phases {
                    let phase_blockers = self.detect_phase_blockers(*phase_id).await;
                    blockers.extend(phase_blockers);
                }
            }
        }

        blockers
    }
}

impl Default for BlockerAnalysis {
    fn default() -> Self {
        BlockerAnalysis {
            blockers: Vec::new(),
            suggestions: Vec::new(),
            stats: BlockerStats::default(),
            circular_chains: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devman_core::{GoalId, PhaseId};
    use chrono::Utc;
    use devman_storage::Storage;

    struct MockStorage;
    #[async_trait::async_trait]
    impl Storage for MockStorage {
        async fn load_goal(&self, _id: devman_core::GoalId) -> devman_storage::Result<Option<devman_core::Goal>> { Ok(None) }
        async fn load_project(&self, _id: devman_core::ProjectId) -> devman_storage::Result<Option<devman_core::Project>> { Ok(None) }
        async fn load_phase(&self, _id: devman_core::PhaseId) -> devman_storage::Result<Option<devman_core::Phase>> { Ok(None) }
        async fn load_task(&self, _id: devman_core::TaskId) -> devman_storage::Result<Option<devman_core::Task>> { Ok(None) }
        async fn load_work_record(&self, _id: devman_core::WorkRecordId) -> devman_storage::Result<Option<devman_core::WorkRecord>> { Ok(None) }
        async fn load_knowledge(&self, _id: devman_core::KnowledgeId) -> devman_storage::Result<Option<devman_core::Knowledge>> { Ok(None) }
        async fn load_quality_check(&self, _id: devman_core::QualityCheckId) -> devman_storage::Result<Option<devman_core::QualityCheck>> { Ok(None) }
        async fn load_event(&self, _id: devman_core::EventId) -> devman_storage::Result<Option<devman_core::Event>> { Ok(None) }
        async fn save_goal(&mut self, _goal: &devman_core::Goal) -> devman_storage::Result<()> { Ok(()) }
        async fn save_project(&mut self, _project: &devman_core::Project) -> devman_storage::Result<()> { Ok(()) }
        async fn save_phase(&mut self, _phase: &devman_core::Phase) -> devman_storage::Result<()> { Ok(()) }
        async fn save_task(&mut self, _task: &devman_core::Task) -> devman_storage::Result<()> { Ok(()) }
        async fn save_work_record(&mut self, _record: &devman_core::WorkRecord) -> devman_storage::Result<()> { Ok(()) }
        async fn save_knowledge(&mut self, _knowledge: &devman_core::Knowledge) -> devman_storage::Result<()> { Ok(()) }
        async fn save_quality_check(&mut self, _check: &devman_core::QualityCheck) -> devman_storage::Result<()> { Ok(()) }
        async fn save_event(&mut self, _event: &devman_core::Event) -> devman_storage::Result<()> { Ok(()) }
        async fn save_vector_embedding(&mut self, _embedding: &devman_core::KnowledgeEmbedding) -> devman_storage::Result<()> { Ok(()) }
        async fn load_vector_embedding(&self, _knowledge_id: &str) -> devman_storage::Result<Option<devman_core::KnowledgeEmbedding>> { Ok(None) }
        async fn list_vector_embeddings(&self) -> devman_storage::Result<Vec<devman_core::KnowledgeEmbedding>> { Ok(vec![]) }
        async fn list_goals(&self) -> devman_storage::Result<Vec<devman_core::Goal>> { Ok(vec![]) }
        async fn list_tasks(&self, _filter: &devman_core::TaskFilter) -> devman_storage::Result<Vec<devman_core::Task>> { Ok(vec![]) }
        async fn list_events(&self) -> devman_storage::Result<Vec<devman_core::Event>> { Ok(vec![]) }
        async fn list_work_records(&self, _task_id: devman_core::TaskId) -> devman_storage::Result<Vec<devman_core::WorkRecord>> { Ok(vec![]) }
        async fn list_knowledge(&self) -> devman_storage::Result<Vec<devman_core::Knowledge>> { Ok(vec![]) }
        async fn list_quality_checks(&self) -> devman_storage::Result<Vec<devman_core::QualityCheck>> { Ok(vec![]) }
        async fn delete_task(&mut self, _id: devman_core::TaskId) -> devman_storage::Result<()> { Ok(()) }
        async fn commit(&mut self, _message: &str) -> devman_storage::Result<()> { Ok(()) }
        async fn rollback(&mut self) -> devman_storage::Result<()> { Ok(()) }
    }

    fn create_test_task(id: TaskId, title: &str, status: TaskStatus) -> Task {
        Task {
            id,
            title: title.to_string(),
            description: "Test task".to_string(),
            intent: devman_core::TaskIntent {
                natural_language: format!("Test task: {}", title),
                context: devman_core::TaskContext {
                    relevant_knowledge: vec![],
                    similar_tasks: vec![],
                    affected_files: vec![],
                },
                success_criteria: vec![],
            },
            status,
            depends_on: Vec::new(),
            steps: Vec::new(),
            inputs: vec![],
            expected_outputs: vec![],
            quality_gates: vec![],
            progress: devman_core::TaskProgress::default(),
            phase_id: PhaseId::new(),
            blocks: Vec::new(),
            work_records: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_blocker_stats_default() {
        let stats = BlockerStats::default();
        assert_eq!(stats.total_blockers, 0);
        assert_eq!(stats.circular_dependencies, 0);
        assert!(stats.average_age_hours.is_none());
    }

    #[test]
    fn test_resolution_suggestion_ordering() {
        let mut suggestions = vec![
            ResolutionSuggestion {
                action: ResolutionAction::ManualReview,
                description: "Manual review needed".to_string(),
                priority: 3,
            },
            ResolutionSuggestion {
                action: ResolutionAction::CompleteTask,
                description: "Complete task".to_string(),
                priority: 1,
            },
            ResolutionSuggestion {
                action: ResolutionAction::AbandonTask,
                description: "Abandon task".to_string(),
                priority: 2,
            },
        ];

        suggestions.sort_by(|a, b| a.priority.cmp(&b.priority));

        assert_eq!(suggestions[0].priority, 1);
        assert_eq!(suggestions[1].priority, 2);
        assert_eq!(suggestions[2].priority, 3);
    }

    #[tokio::test]
    async fn test_detect_phase_blockers_empty() {
        use devman_storage::Storage;

        struct MockStorage;
        #[async_trait::async_trait]
        impl Storage for MockStorage {
            async fn load_goal(&self, _id: devman_core::GoalId) -> devman_storage::Result<Option<devman_core::Goal>> { Ok(None) }
            async fn load_project(&self, _id: devman_core::ProjectId) -> devman_storage::Result<Option<devman_core::Project>> { Ok(None) }
            async fn load_phase(&self, _id: devman_core::PhaseId) -> devman_storage::Result<Option<devman_core::Phase>> { Ok(None) }
            async fn load_task(&self, _id: devman_core::TaskId) -> devman_storage::Result<Option<devman_core::Task>> { Ok(None) }
            async fn load_work_record(&self, _id: devman_core::WorkRecordId) -> devman_storage::Result<Option<devman_core::WorkRecord>> { Ok(None) }
            async fn load_knowledge(&self, _id: devman_core::KnowledgeId) -> devman_storage::Result<Option<devman_core::Knowledge>> { Ok(None) }
            async fn load_quality_check(&self, _id: devman_core::QualityCheckId) -> devman_storage::Result<Option<devman_core::QualityCheck>> { Ok(None) }
            async fn load_event(&self, _id: devman_core::EventId) -> devman_storage::Result<Option<devman_core::Event>> { Ok(None) }
            async fn save_goal(&mut self, _goal: &devman_core::Goal) -> devman_storage::Result<()> { Ok(()) }
            async fn save_project(&mut self, _project: &devman_core::Project) -> devman_storage::Result<()> { Ok(()) }
            async fn save_phase(&mut self, _phase: &devman_core::Phase) -> devman_storage::Result<()> { Ok(()) }
            async fn save_task(&mut self, _task: &devman_core::Task) -> devman_storage::Result<()> { Ok(()) }
            async fn save_work_record(&mut self, _record: &devman_core::WorkRecord) -> devman_storage::Result<()> { Ok(()) }
            async fn save_knowledge(&mut self, _knowledge: &devman_core::Knowledge) -> devman_storage::Result<()> { Ok(()) }
            async fn save_quality_check(&mut self, _check: &devman_core::QualityCheck) -> devman_storage::Result<()> { Ok(()) }
            async fn save_event(&mut self, _event: &devman_core::Event) -> devman_storage::Result<()> { Ok(()) }
            async fn save_vector_embedding(&mut self, _embedding: &devman_core::KnowledgeEmbedding) -> devman_storage::Result<()> { Ok(()) }
            async fn load_vector_embedding(&self, _knowledge_id: &str) -> devman_storage::Result<Option<devman_core::KnowledgeEmbedding>> { Ok(None) }
            async fn list_vector_embeddings(&self) -> devman_storage::Result<Vec<devman_core::KnowledgeEmbedding>> { Ok(vec![]) }
            async fn list_goals(&self) -> devman_storage::Result<Vec<devman_core::Goal>> { Ok(vec![]) }
            async fn list_tasks(&self, _filter: &devman_core::TaskFilter) -> devman_storage::Result<Vec<devman_core::Task>> { Ok(vec![]) }
            async fn list_events(&self) -> devman_storage::Result<Vec<devman_core::Event>> { Ok(vec![]) }
            async fn list_work_records(&self, _task_id: devman_core::TaskId) -> devman_storage::Result<Vec<devman_core::WorkRecord>> { Ok(vec![]) }
            async fn list_knowledge(&self) -> devman_storage::Result<Vec<devman_core::Knowledge>> { Ok(vec![]) }
            async fn list_quality_checks(&self) -> devman_storage::Result<Vec<devman_core::QualityCheck>> { Ok(vec![]) }
            async fn delete_task(&mut self, _id: devman_core::TaskId) -> devman_storage::Result<()> { Ok(()) }
            async fn commit(&mut self, _message: &str) -> devman_storage::Result<()> { Ok(()) }
            async fn rollback(&mut self) -> devman_storage::Result<()> { Ok(()) }
        }

        let storage: Arc<dyn Storage> = Arc::new(MockStorage);
        let detector = BlockerDetector::new(storage);

        let blockers = detector.detect_phase_blockers(PhaseId::new()).await;
        assert!(blockers.is_empty());
    }

    #[test]
    fn test_blocker_analysis_default() {
        let analysis = BlockerAnalysis::default();
        assert!(analysis.blockers.is_empty());
        assert!(analysis.suggestions.is_empty());
        assert_eq!(analysis.stats.total_blockers, 0);
        assert!(analysis.circular_chains.is_empty());
    }

    #[test]
    fn test_resolution_action_variants() {
        assert_eq!(ResolutionAction::CompleteTask.as_str(), "CompleteTask");
        assert_eq!(ResolutionAction::AbandonTask.as_str(), "AbandonTask");
        assert_eq!(ResolutionAction::SkipTask.as_str(), "SkipTask");
        assert_eq!(ResolutionAction::ModifyDependencies.as_str(), "ModifyDependencies");
        assert_eq!(ResolutionAction::ManualReview.as_str(), "ManualReview");
        assert_eq!(ResolutionAction::Wait.as_str(), "Wait");
    }

    #[test]
    fn test_detect_dependency_blockers_empty() {
        let empty_map: HashMap<TaskId, Task> = HashMap::new();
        let detector = BlockerDetector::new(Arc::new(MockStorage {}));

        let blockers = detector.detect_dependency_blockers(&empty_map);
        assert!(blockers.is_empty());
    }

    #[test]
    fn test_detect_circular_dependencies_empty() {
        let empty_map: HashMap<TaskId, Task> = HashMap::new();
        let detector = BlockerDetector::new(Arc::new(MockStorage {}));

        let (cycles, blockers) = detector.detect_circular_dependencies(&empty_map);
        assert!(cycles.is_empty());
        assert!(blockers.is_empty());
    }

    #[test]
    fn test_generate_suggestions_empty() {
        let empty_map: HashMap<TaskId, Task> = HashMap::new();
        let detector = BlockerDetector::new(Arc::new(MockStorage {}));

        let suggestions = detector.generate_suggestions(&empty_map, &[]);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_calculate_stats_empty() {
        let detector = BlockerDetector::new(Arc::new(MockStorage {}));

        let stats = detector.calculate_stats(&[]);
        assert_eq!(stats.total_blockers, 0);
        assert_eq!(stats.circular_dependencies, 0);
    }
}
