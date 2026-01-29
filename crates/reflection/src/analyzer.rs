//! Analyzes task execution and generates insights.

use devman_core::{Event, KnowledgeNode, Task};

/// An insight from analyzing a task.
#[derive(Debug, Clone)]
pub struct Insight {
    /// Whether the task was successful
    pub success: bool,
    /// Summary of what happened
    pub summary: String,
    /// Confidence adjustment
    pub confidence_adjustment: f32,
    /// Suggested new tasks
    pub suggested_tasks: Vec<Task>,
    /// Knowledge gained
    pub knowledge_gained: Vec<KnowledgeNode>,
}

/// Analyzes task execution results.
pub struct Analyzer;

impl Analyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Analyze a task and its events.
    pub fn analyze(&self, task: &Task, events: &[Event]) -> Insight {
        // Simple analysis: check if task completed
        let success = task.status == devman_core::TaskStatus::Done;

        let mut knowledge_gained = Vec::new();
        let mut suggested_tasks = Vec::new();

        // Extract knowledge from events
        for event in events {
            for update in &event.delta_knowledge {
                let node = if update.confidence_delta > 0.5 {
                    KnowledgeNode::fact(&update.claim)
                } else if update.confidence_delta < -0.3 {
                    let mut node = KnowledgeNode::hypothesis(&update.claim);
                    node.confidence = 0.1;
                    node
                } else {
                    KnowledgeNode::new(&update.claim)
                };
                knowledge_gained.push(node);
            }
        }

        // Generate insight summary
        let summary = if success {
            format!("Task '{}' completed successfully. Confidence increased.", task.intent)
        } else {
            format!("Task '{}' encountered issues. Needs investigation.", task.intent)
        };

        let confidence_adjustment = if success { 0.1 } else { -0.2 };

        // Suggest follow-up tasks based on failure
        if !success && !task.hypothesis.is_empty() {
            let follow_up = Task::new(
                format!("Investigate: {}", task.intent),
                format!("Original hypothesis '{}' may be incorrect", task.hypothesis),
            );
            suggested_tasks.push(follow_up);
        }

        Insight {
            success,
            summary,
            confidence_adjustment,
            suggested_tasks,
            knowledge_gained,
        }
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}
