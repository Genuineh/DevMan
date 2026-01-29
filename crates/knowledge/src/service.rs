//! Knowledge service trait and basic implementation.

use async_trait::async_trait;
use devman_core::{Knowledge, KnowledgeId, Task, TaskContext};
use devman_storage::Storage;

/// Knowledge service for searching and retrieving knowledge.
#[async_trait]
pub trait KnowledgeService: Send + Sync {
    /// Search knowledge by semantic query.
    async fn search_semantic(&self, query: &str, limit: usize) -> Vec<Knowledge>;

    /// Find similar tasks based on context.
    async fn find_similar_tasks(&self, task: &Task) -> Vec<Task>;

    /// Get best practices for a domain.
    async fn get_best_practices(&self, domain: &str) -> Vec<Knowledge>;

    /// Recommend knowledge based on task context.
    async fn recommend_knowledge(&self, context: &TaskContext) -> Vec<Knowledge>;
}

/// Basic knowledge service implementation.
pub struct BasicKnowledgeService<S: Storage> {
    storage: std::sync::Arc<S>,
}

impl<S: Storage> BasicKnowledgeService<S> {
    /// Create a new knowledge service.
    pub fn new(storage: S) -> Self {
        Self {
            storage: std::sync::Arc::new(storage),
        }
    }
}

#[async_trait]
impl<S: Storage + 'static> KnowledgeService for BasicKnowledgeService<S> {
    async fn search_semantic(&self, query: &str, limit: usize) -> Vec<Knowledge> {
        let all = self.storage.list_knowledge().await.unwrap_or_default();
        // Simple text matching for now
        all.into_iter()
            .filter(|k| {
                k.content.summary.to_lowercase().contains(&query.to_lowercase())
                    || k.tags.iter().any(|t| t.to_lowercase().contains(&query.to_lowercase()))
            })
            .take(limit)
            .collect()
    }

    async fn find_similar_tasks(&self, _task: &Task) -> Vec<Task> {
        // TODO: Implement similarity search
        Vec::new()
    }

    async fn get_best_practices(&self, domain: &str) -> Vec<Knowledge> {
        let all = self.storage.list_knowledge().await.unwrap_or_default();
        all.into_iter()
            .filter(|k| k.metadata.domain.iter().any(|d| d == domain))
            .collect()
    }

    async fn recommend_knowledge(&self, context: &TaskContext) -> Vec<Knowledge> {
        let mut results = Vec::new();
        for &id in &context.relevant_knowledge {
            if let Ok(Some(k)) = self.storage.load_knowledge(id).await {
                results.push(k);
            }
        }
        results
    }
}
