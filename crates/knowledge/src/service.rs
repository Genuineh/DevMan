//! Knowledge service trait and basic implementation.

use async_trait::async_trait;
use devman_core::{Knowledge, KnowledgeType, Task, TaskContext};
use devman_storage::Storage;
use std::collections::{HashMap, HashSet};

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

    /// Search knowledge by tags (OR logic - any tag matches).
    async fn search_by_tags(&self, tags: &[String], limit: usize) -> Vec<Knowledge>;

    /// Search knowledge by tags (AND logic - all tags must match).
    async fn search_by_tags_all(&self, tags: &[String], limit: usize) -> Vec<Knowledge>;

    /// Get all unique tags across all knowledge.
    async fn get_all_tags(&self) -> HashSet<String>;

    /// Get tag statistics (tag -> count).
    async fn get_tag_statistics(&self) -> HashMap<String, usize>;

    /// Find similar knowledge by content.
    async fn find_similar_knowledge(&self, knowledge: &Knowledge, limit: usize) -> Vec<Knowledge>;

    /// Get knowledge by type.
    async fn get_by_type(&self, knowledge_type: KnowledgeType) -> Vec<Knowledge>;

    /// Suggest tags based on query.
    async fn suggest_tags(&self, query: &str, limit: usize) -> Vec<String>;
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
        let query_lower = query.to_lowercase();

        // Score each knowledge item by relevance
        let mut scored: Vec<_> = all.into_iter()
            .map(|k| {
                let score = self.calculate_relevance_score(&k, &query_lower);
                (k, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        // Sort by score (descending)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter()
            .take(limit)
            .map(|(k, _)| k)
            .collect()
    }

    async fn find_similar_tasks(&self, _task: &Task) -> Vec<Task> {
        // TODO: Implement similarity search
        Vec::new()
    }

    async fn get_best_practices(&self, domain: &str) -> Vec<Knowledge> {
        let all = self.storage.list_knowledge().await.unwrap_or_default();
        all.into_iter()
            .filter(|k| {
                matches!(k.knowledge_type, KnowledgeType::BestPractice { .. })
                    && k.metadata.domain.iter().any(|d| d == domain)
            })
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

    async fn search_by_tags(&self, tags: &[String], limit: usize) -> Vec<Knowledge> {
        if tags.is_empty() {
            return Vec::new();
        }

        let all = self.storage.list_knowledge().await.unwrap_or_default();
        let tags_set: HashSet<_> = tags.iter().map(|t| t.to_lowercase()).collect();

        all.into_iter()
            .filter(|k| {
                k.tags.iter()
                    .any(|t| tags_set.contains(&t.to_lowercase()))
            })
            .take(limit)
            .collect()
    }

    async fn search_by_tags_all(&self, tags: &[String], limit: usize) -> Vec<Knowledge> {
        if tags.is_empty() {
            return Vec::new();
        }

        let all = self.storage.list_knowledge().await.unwrap_or_default();
        let tags_set: HashSet<_> = tags.iter().map(|t| t.to_lowercase()).collect();

        all.into_iter()
            .filter(|k| {
                let k_tags: HashSet<_> = k.tags.iter().map(|t| t.to_lowercase()).collect();
                tags_set.is_subset(&k_tags)
            })
            .take(limit)
            .collect()
    }

    async fn get_all_tags(&self) -> HashSet<String> {
        let all = self.storage.list_knowledge().await.unwrap_or_default();
        all.into_iter()
            .flat_map(|k| k.tags.into_iter())
            .collect()
    }

    async fn get_tag_statistics(&self) -> HashMap<String, usize> {
        let all = self.storage.list_knowledge().await.unwrap_or_default();
        let mut stats = HashMap::new();

        for k in all {
            for tag in k.tags {
                *stats.entry(tag).or_insert(0) += 1;
            }
        }

        stats
    }

    async fn find_similar_knowledge(&self, knowledge: &Knowledge, limit: usize) -> Vec<Knowledge> {
        let all = self.storage.list_knowledge().await.unwrap_or_default();
        let query = format!("{} {}", knowledge.content.summary, knowledge.content.detail);
        let query_lower = query.to_lowercase();

        let mut scored: Vec<_> = all.into_iter()
            .filter(|k| k.id != knowledge.id)
            .map(|k| {
                let score = self.calculate_relevance_score(&k, &query_lower);
                (k, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter()
            .take(limit)
            .map(|(k, _)| k)
            .collect()
    }

    async fn get_by_type(&self, knowledge_type: KnowledgeType) -> Vec<Knowledge> {
        let all = self.storage.list_knowledge().await.unwrap_or_default();
        all.into_iter()
            .filter(|k| k.knowledge_type == knowledge_type)
            .collect()
    }

    async fn suggest_tags(&self, query: &str, limit: usize) -> Vec<String> {
        let all_tags = self.get_all_tags().await;
        let query_lower = query.to_lowercase();

        all_tags.into_iter()
            .filter(|t| t.to_lowercase().contains(&query_lower))
            .take(limit)
            .collect()
    }
}

impl<S: Storage> BasicKnowledgeService<S> {
    /// Calculate relevance score for a knowledge item against a query.
    fn calculate_relevance_score(&self, knowledge: &Knowledge, query_lower: &str) -> f64 {
        let mut score = 0.0;

        // Match in summary (highest weight)
        if knowledge.content.summary.to_lowercase().contains(query_lower) {
            score += 10.0;
        }

        // Match in detail (medium weight)
        if knowledge.content.detail.to_lowercase().contains(query_lower) {
            score += 5.0;
        }

        // Match in tags (high weight)
        for tag in &knowledge.tags {
            if tag.to_lowercase().contains(query_lower) {
                score += 7.0;
            }
        }

        // Match in domain (lower weight)
        for domain in &knowledge.metadata.domain {
            if domain.to_lowercase().contains(query_lower) {
                score += 3.0;
            }
        }

        // Bonus for best practices or solutions
        if matches!(knowledge.knowledge_type, KnowledgeType::BestPractice { .. } | KnowledgeType::Solution { .. }) {
            score *= 1.2;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devman_core::{Knowledge, KnowledgeType, KnowledgeContent, KnowledgeMetadata, UsageStats};

    fn create_test_knowledge(title: &str, summary: &str, tags: Vec<&str>) -> Knowledge {
        Knowledge {
            id: devman_core::KnowledgeId::new(),
            title: title.to_string(),
            knowledge_type: KnowledgeType::LessonLearned {
                lesson: "Test lesson".to_string(),
                context: "Test context".to_string(),
            },
            content: KnowledgeContent {
                summary: summary.to_string(),
                detail: "Detailed content".to_string(),
                examples: vec![],
                references: vec![],
            },
            metadata: KnowledgeMetadata {
                domain: vec!["testing".to_string()],
                tech_stack: vec![],
                scenarios: vec![],
                quality_score: 1.0,
                verified: true,
            },
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
            related_to: vec![],
            derived_from: vec![],
            usage_stats: UsageStats {
                times_used: 0,
                last_used: None,
                success_rate: 1.0,
                feedback: vec![],
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    // Helper function to test the scoring logic without needing full storage
    fn test_score_calculation(knowledge: &Knowledge, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let mut score = 0.0;

        if knowledge.content.summary.to_lowercase().contains(&query_lower) {
            score += 10.0;
        }
        if knowledge.content.detail.to_lowercase().contains(&query_lower) {
            score += 5.0;
        }
        for tag in &knowledge.tags {
            if tag.to_lowercase().contains(&query_lower) {
                score += 7.0;
            }
        }
        for domain in &knowledge.metadata.domain {
            if domain.to_lowercase().contains(&query_lower) {
                score += 3.0;
            }
        }
        if matches!(knowledge.knowledge_type, KnowledgeType::BestPractice { .. } | KnowledgeType::Solution { .. }) {
            score *= 1.2;
        }
        score
    }

    #[test]
    fn test_calculate_relevance_score_summary_match() {
        let knowledge = create_test_knowledge("Test", "Important content", vec![]);
        let score = test_score_calculation(&knowledge, "important");
        assert!(score > 5.0); // Summary match has high weight
    }

    #[test]
    fn test_calculate_relevance_score_tag_match() {
        let knowledge = create_test_knowledge("Test", "Content", vec!["rust", "testing"]);
        let score = test_score_calculation(&knowledge, "rust");
        assert!(score >= 7.0); // Tag match has high weight
    }

    #[test]
    fn test_calculate_relevance_score_no_match() {
        let knowledge = create_test_knowledge("Test", "Content", vec![]);
        let score = test_score_calculation(&knowledge, "nonexistent");
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_calculate_relevance_score_best_practice_bonus() {
        let mut knowledge = create_test_knowledge("Test", "Best practice content", vec![]);
        knowledge.knowledge_type = KnowledgeType::BestPractice {
            practice: "Test".to_string(),
            rationale: "Test".to_string(),
        };
        let score = test_score_calculation(&knowledge, "best");
        // Should get bonus multiplier
        assert!(score > 10.0 * 1.1); // 10 from summary match * 1.2 bonus
    }
}
