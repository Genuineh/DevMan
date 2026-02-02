//! Knowledge classification enhancement with vector embeddings.
//!
//! This module provides advanced knowledge classification capabilities:
//! - Vector embedding storage and similarity search
//! - Automatic experience extraction from work records
//! - Code pattern recognition and indexing
//! - Semantic solution indexing

use anyhow::Result;
use async_trait::async_trait;
use devman_core::{Knowledge, KnowledgeId, KnowledgeType, WorkRecord};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Vector embedding type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Embedding(pub Vec<f32>);

/// Similarity search result.
#[derive(Debug, Clone)]
pub struct SimilarityResult {
    /// Knowledge ID
    pub id: KnowledgeId,
    /// Similarity score (0.0 to 1.0)
    pub score: f32,
}

/// Extracted pattern from code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPattern {
    /// Pattern name
    pub name: String,
    /// Pattern description
    pub description: String,
    /// Programming language
    pub language: String,
    /// Code snippet
    pub code: String,
    /// Use cases
    pub use_cases: Vec<String>,
    /// Related knowledge ID
    pub related_knowledge_id: Option<KnowledgeId>,
}

/// Extracted experience from work record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedExperience {
    /// Title of the experience
    pub title: String,
    /// What was learned
    pub lesson: String,
    /// Context where it was learned
    pub context: String,
    /// Tags derived from the experience
    pub tags: Vec<String>,
    /// Related domain
    pub domain: String,
    /// Quality score
    pub quality_score: f32,
    /// Source work record ID
    pub source_work_record_id: String,
}

/// Vector store for embedding storage and retrieval.
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Store an embedding.
    async fn store(&self, id: &KnowledgeId, embedding: &Embedding) -> Result<()>;

    /// Find similar embeddings.
    async fn find_similar(&self, embedding: &Embedding, limit: usize) -> Vec<SimilarityResult>;

    /// Delete an embedding.
    async fn delete(&self, id: &KnowledgeId) -> Result<()>;

    /// Clear all embeddings.
    async fn clear(&self) -> Result<()>;
}

/// In-memory vector store implementation.
#[derive(Clone)]
pub struct InMemoryVectorStore {
    embeddings: Arc<Mutex<HashMap<KnowledgeId, Embedding>>>,
}

impl InMemoryVectorStore {
    /// Create a new in-memory vector store.
    pub fn new() -> Self {
        Self {
            embeddings: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn store(&self, id: &KnowledgeId, embedding: &Embedding) -> Result<()> {
        let mut embeddings = self.embeddings.lock().await;
        embeddings.insert(id.clone(), embedding.clone());
        Ok(())
    }

    async fn find_similar(&self, embedding: &Embedding, limit: usize) -> Vec<SimilarityResult> {
        let embeddings = self.embeddings.lock().await;
        let mut results: Vec<_> = embeddings
            .iter()
            .map(|(id, stored)| {
                let score = cosine_similarity(embedding, stored);
                SimilarityResult {
                    id: id.clone(),
                    score,
                }
            })
            .filter(|r| r.score > 0.0)
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }

    async fn delete(&self, id: &KnowledgeId) -> Result<()> {
        let mut embeddings = self.embeddings.lock().await;
        embeddings.remove(id);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut embeddings = self.embeddings.lock().await;
        embeddings.clear();
        Ok(())
    }
}

/// Embedding generator trait.
/// Implement this to integrate with external embedding APIs.
#[async_trait]
pub trait EmbeddingGenerator: Send + Sync {
    /// Generate embedding for text.
    async fn generate(&self, text: &str) -> Result<Embedding>;
}

/// Simple embedding generator (keyword-based, no external dependencies).
#[derive(Clone)]
pub struct KeywordEmbeddingGenerator {
    vocabulary: Arc<Mutex<HashMap<String, usize>>>,
}

impl KeywordEmbeddingGenerator {
    /// Create a new keyword embedding generator.
    pub fn new() -> Self {
        Self {
            vocabulary: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }
}

#[async_trait]
impl EmbeddingGenerator for KeywordEmbeddingGenerator {
    async fn generate(&self, text: &str) -> Result<Embedding> {
        let tokens = Self::tokenize(text);
        let vocab = self.vocabulary.lock().await;

        // Create simple one-hot encoding based on vocabulary
        let dim = vocab.len().max(1);
        let mut embedding = vec![0.0f32; dim];

        for token in &tokens {
            if let Some(idx) = vocab.get(token) {
                if *idx < dim {
                    embedding[*idx] = 1.0;
                }
            }
        }

        Ok(Embedding(embedding))
    }
}

/// Calculate cosine similarity between two embeddings.
pub fn cosine_similarity(a: &Embedding, b: &Embedding) -> f32 {
    if a.0.is_empty() || b.0.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.0.iter().zip(b.0.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.0.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.0.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Knowledge classifier for automatic categorization.
#[derive(Debug, Clone, Default)]
pub struct KnowledgeClassifier {
    /// Category to keywords mapping
    pub category_keywords: HashMap<String, Vec<String>>,
}

impl KnowledgeClassifier {
    /// Create a new knowledge classifier.
    pub fn new() -> Self {
        let mut category_keywords: HashMap<String, Vec<String>> = HashMap::new();

        // Performance keywords
        category_keywords.insert("performance".to_string(), vec![
            "optimization".to_string(), "performance".to_string(), "speed".to_string(),
            "efficient".to_string(), "cache".to_string(), "profiling".to_string()
        ]);

        // Error handling keywords
        category_keywords.insert("error_handling".to_string(), vec![
            "error".to_string(), "exception".to_string(), "panic".to_string(),
            "fallback".to_string(), "retry".to_string(), "recovery".to_string()
        ]);

        // Concurrency keywords
        category_keywords.insert("concurrency".to_string(), vec![
            "async".to_string(), "parallel".to_string(), "thread".to_string(),
            "concurrent".to_string(), "mutex".to_string(), "lock".to_string(),
            "atomic".to_string()
        ]);

        // Testing keywords
        category_keywords.insert("testing".to_string(), vec![
            "test".to_string(), "mock".to_string(), "unit".to_string(),
            "integration".to_string(), "property".to_string(), "fuzz".to_string(),
            "assert".to_string()
        ]);

        // Security keywords
        category_keywords.insert("security".to_string(), vec![
            "security".to_string(), "auth".to_string(), "encryption".to_string(),
            "validate".to_string(), "sanitize".to_string(), "permission".to_string()
        ]);

        // API design keywords
        category_keywords.insert("api_design".to_string(), vec![
            "api".to_string(), "endpoint".to_string(), "rest".to_string(),
            "grpc".to_string(), "interface".to_string(), "protocol".to_string(),
            "contract".to_string()
        ]);

        Self { category_keywords }
    }

    /// Classify knowledge based on content.
    pub fn classify(&self, knowledge: &Knowledge) -> Vec<String> {
        let text = format!(
            "{} {} {} {}",
            knowledge.title,
            knowledge.content.summary,
            knowledge.content.detail,
            knowledge.tags.join(" ")
        );
        let text_lower = text.to_lowercase();

        let mut categories = Vec::new();

        for (category, keywords) in &self.category_keywords {
            let matches: usize = keywords
                .iter()
                .filter(|kw| text_lower.contains(&kw.as_str()))
                .count();

            if matches >= 2 {
                categories.push(category.clone());
            }
        }

        categories
    }

    /// Extract tech stack from knowledge.
    pub fn extract_tech_stack(&self, knowledge: &Knowledge) -> Vec<String> {
        let text = format!(
            "{} {} {}",
            knowledge.content.summary,
            knowledge.content.detail,
            knowledge.tags.join(" ")
        );
        let text_lower = text.to_lowercase();

        let mut detected = Vec::new();

        // Check for common tech stacks
        if text_lower.contains("rust") || text_lower.contains("cargo") || text_lower.contains("tokio") {
            detected.push("rust".to_string());
        }
        if text_lower.contains("python") || text_lower.contains("pip") {
            detected.push("python".to_string());
        }
        if text_lower.contains("javascript") || text_lower.contains("node") {
            detected.push("javascript".to_string());
        }
        if text_lower.contains("typescript") {
            detected.push("typescript".to_string());
        }
        if text_lower.contains("docker") || text_lower.contains("container") {
            detected.push("docker".to_string());
        }
        if text_lower.contains("kubernetes") || text_lower.contains("k8s") {
            detected.push("kubernetes".to_string());
        }
        if text_lower.contains("sql") || text_lower.contains("database") {
            detected.push("database".to_string());
        }

        detected
    }
}

/// Experience extractor for automatic lesson learned creation.
#[derive(Debug, Clone, Default)]
pub struct ExperienceExtractor;

impl ExperienceExtractor {
    /// Extract experience from a work record.
    pub fn extract(&self, work_record: &WorkRecord) -> Option<ExtractedExperience> {
        // Only extract from successful work
        if work_record.result.status != devman_core::CompletionStatus::Success {
            return None;
        }

        // Extract description from events
        let description = work_record
            .events
            .iter()
            .filter(|e| e.event_type == devman_core::WorkEventType::StepCompleted)
            .last()
            .map(|e| e.description.clone())
            .unwrap_or_else(|| "Work completed".to_string());

        let summary = format!("Work completed: {}", description);

        // Extract potential tags from artifacts
        let mut tags = Vec::new();
        for artifact in &work_record.artifacts {
            if artifact.artifact_type == "code" || artifact.name.ends_with(".rs") {
                tags.push("rust".to_string());
            } else if artifact.name.ends_with(".py") {
                tags.push("python".to_string());
            }
        }

        Some(ExtractedExperience {
            title: "Experience from work record".to_string(),
            lesson: summary.clone(),
            context: format!("Work record ID: {}", work_record.id),
            tags,
            domain: self.detect_domain(&summary),
            quality_score: 0.9,
            source_work_record_id: format!("{}", work_record.id),
        })
    }

    fn detect_domain(&self, text: &str) -> String {
        let text_lower = text.to_lowercase();

        if text_lower.contains("api") || text_lower.contains("endpoint") {
            "api".to_string()
        } else if text_lower.contains("database") || text_lower.contains("query") {
            "database".to_string()
        } else if text_lower.contains("test") || text_lower.contains("testing") {
            "testing".to_string()
        } else if text_lower.contains("deploy") || text_lower.contains("production") {
            "devops".to_string()
        } else {
            "general".to_string()
        }
    }
}

/// Code pattern recognizer.
#[derive(Debug, Clone, Default)]
pub struct CodePatternRecognizer;

impl CodePatternRecognizer {
    /// Recognize patterns in code snippets.
    pub fn recognize(&self, code: &str, language: &str) -> Vec<ExtractedPattern> {
        let mut patterns = Vec::new();

        // Common patterns to recognize
        let patterns_config: Vec<(&str, Vec<&str>)> = vec![
            ("Error Handling", vec!["match", "Result", "?"]),
            ("Async/Await", vec!["async", "await", "tokio"]),
            ("Builder Pattern", vec!["build", "self."]),
            ("Iterator", vec!["iter", "map", "filter", "collect"]),
        ];

        for (name, keywords) in &patterns_config {
            let matches: usize = keywords
                .iter()
                .filter(|kw| code.contains(*kw))
                .count();

            if matches >= 2 {
                patterns.push(ExtractedPattern {
                    name: name.to_string(),
                    description: format!("Detected {} pattern in {}", name, language),
                    language: language.to_string(),
                    code: code.to_string(),
                    use_cases: vec![format!("Use for {}", name.to_lowercase())],
                    related_knowledge_id: None,
                });
            }
        }

        patterns
    }
}

/// Enhanced knowledge classifier with vector embeddings.
#[derive(Clone)]
pub struct EnhancedKnowledgeClassifier {
    classifier: KnowledgeClassifier,
    extractor: ExperienceExtractor,
    recognizer: CodePatternRecognizer,
    vector_store: Arc<dyn VectorStore>,
    embedding_generator: Arc<dyn EmbeddingGenerator>,
}

impl EnhancedKnowledgeClassifier {
    /// Create a new enhanced knowledge classifier.
    pub fn new(
        vector_store: Arc<dyn VectorStore>,
        embedding_generator: Arc<dyn EmbeddingGenerator>,
    ) -> Self {
        Self {
            classifier: KnowledgeClassifier::new(),
            extractor: ExperienceExtractor,
            recognizer: CodePatternRecognizer,
            vector_store,
            embedding_generator,
        }
    }

    /// Index a knowledge item for similarity search.
    pub async fn index_knowledge(&self, knowledge: &Knowledge) -> Result<()> {
        let text = format!(
            "{} {} {}",
            knowledge.title,
            knowledge.content.summary,
            knowledge.content.detail
        );

        let embedding = self.embedding_generator.generate(&text).await?;
        self.vector_store.store(&knowledge.id, &embedding).await?;

        Ok(())
    }

    /// Find similar knowledge using vector similarity.
    pub async fn find_similar(
        &self,
        knowledge: &Knowledge,
        limit: usize,
    ) -> Result<Vec<SimilarityResult>> {
        let text = format!(
            "{} {} {}",
            knowledge.title,
            knowledge.content.summary,
            knowledge.content.detail
        );

        let embedding = self.embedding_generator.generate(&text).await?;
        let results = self.vector_store.find_similar(&embedding, limit).await;

        // Filter out the original knowledge item
        let results: Vec<_> = results
            .into_iter()
            .filter(|r| r.id != knowledge.id)
            .collect();

        Ok(results)
    }

    /// Auto-classify knowledge and suggest categories.
    pub fn classify(&self, knowledge: &Knowledge) -> Vec<String> {
        self.classifier.classify(knowledge)
    }

    /// Extract tech stack from knowledge.
    pub fn extract_tech_stack(&self, knowledge: &Knowledge) -> Vec<String> {
        self.classifier.extract_tech_stack(knowledge)
    }

    /// Extract experience from work record.
    pub fn extract_experience(
        &self,
        work_record: &WorkRecord,
    ) -> Option<ExtractedExperience> {
        self.extractor.extract(work_record)
    }

    /// Recognize code patterns in knowledge content.
    pub fn recognize_patterns(&self, knowledge: &Knowledge) -> Vec<ExtractedPattern> {
        let mut all_patterns = Vec::new();

        for example in &knowledge.content.examples {
            let patterns = self.recognizer.recognize(&example.code, &example.language);
            all_patterns.extend(patterns);
        }

        all_patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = Embedding(vec![1.0, 0.0, 0.0]);
        let b = Embedding(vec![1.0, 0.0, 0.0]);
        assert_eq!(cosine_similarity(&a, &b), 1.0);

        let c = Embedding(vec![0.0, 1.0, 0.0]);
        assert_eq!(cosine_similarity(&a, &c), 0.0);

        let d = Embedding(vec![0.5, 0.5, 0.0]);
        let similarity = cosine_similarity(&a, &d);
        assert!(similarity > 0.6 && similarity < 0.8);
    }

    #[tokio::test]
    async fn test_in_memory_vector_store() {
        let store = Arc::new(InMemoryVectorStore::new());
        let id = KnowledgeId::new();
        let embedding = Embedding(vec![1.0, 0.0, 0.0]);

        store.store(&id, &embedding).await.unwrap();
        let results = store.find_similar(&embedding, 10).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id);
        assert!((results[0].score - 1.0).abs() < 0.001);

        store.delete(&id).await.unwrap();
        let results = store.find_similar(&embedding, 10).await;
        assert!(results.is_empty());
    }

    #[test]
    fn test_knowledge_classifier_classify() {
        let classifier = KnowledgeClassifier::new();
        let knowledge = Knowledge {
            id: KnowledgeId::new(),
            title: "Performance Optimization Guide".to_string(),
            knowledge_type: KnowledgeType::BestPractice {
                practice: "Use caching".to_string(),
                rationale: "Improves performance".to_string(),
            },
            content: devman_core::KnowledgeContent {
                summary: "How to optimize performance".to_string(),
                detail: "Use caching, profiling, and efficient algorithms".to_string(),
                examples: vec![],
                references: vec![],
            },
            metadata: devman_core::KnowledgeMetadata {
                domain: vec!["performance".to_string()],
                tech_stack: vec![],
                scenarios: vec![],
                quality_score: 0.9,
                verified: true,
            },
            tags: vec!["optimization".to_string(), "cache".to_string()],
            related_to: vec![],
            derived_from: vec![],
            usage_stats: devman_core::UsageStats {
                times_used: 0,
                last_used: None,
                success_rate: 1.0,
                feedback: vec![],
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let categories = classifier.classify(&knowledge);
        assert!(categories.contains(&"performance".to_string()));
    }

    #[test]
    fn test_knowledge_classifier_extract_tech_stack() {
        let classifier = KnowledgeClassifier::new();
        let knowledge = Knowledge {
            id: KnowledgeId::new(),
            title: "Rust Error Handling".to_string(),
            knowledge_type: KnowledgeType::BestPractice {
                practice: "Use anyhow".to_string(),
                rationale: "Simplifies error handling".to_string(),
            },
            content: devman_core::KnowledgeContent {
                summary: "Error handling in Rust".to_string(),
                detail: "Use anyhow and thiserror for error handling in Rust".to_string(),
                examples: vec![],
                references: vec![],
            },
            metadata: devman_core::KnowledgeMetadata {
                domain: vec!["error-handling".to_string()],
                tech_stack: vec![],
                scenarios: vec![],
                quality_score: 0.9,
                verified: true,
            },
            tags: vec!["rust".to_string(), "error".to_string()],
            related_to: vec![],
            derived_from: vec![],
            usage_stats: devman_core::UsageStats {
                times_used: 0,
                last_used: None,
                success_rate: 1.0,
                feedback: vec![],
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let tech_stack = classifier.extract_tech_stack(&knowledge);
        assert!(tech_stack.contains(&"rust".to_string()));
    }

    #[test]
    fn test_code_pattern_recognizer() {
        let recognizer = CodePatternRecognizer;

        let code = r#"
async fn fetch_data() -> Result<String, Error> {
    let response = reqwest::get(url).await?;
    Ok(response.text().await?)
}
"#;

        let patterns = recognizer.recognize(code, "rust");
        assert!(patterns.iter().any(|p| p.name.contains("Async")));
        assert!(patterns.iter().any(|p| p.name.contains("Error")));
    }

    #[test]
    fn test_experience_extractor() {
        let extractor = ExperienceExtractor;

        let work_record = WorkRecord {
            id: devman_core::WorkRecordId::new(),
            task_id: devman_core::TaskId::new(),
            executor: devman_core::Executor::AI {
                model: "claude".to_string(),
            },
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            duration: Some(chrono::Duration::seconds(60)),
            events: vec![
                devman_core::WorkEvent {
                    timestamp: chrono::Utc::now(),
                    event_type: devman_core::WorkEventType::StepCompleted,
                    description: "Fixed performance issue".to_string(),
                    data: serde_json::json!({}),
                }
            ],
            result: devman_core::WorkResult {
                status: devman_core::CompletionStatus::Success,
                outputs: vec![],
                metrics: devman_core::WorkMetrics {
                    token_used: Some(1000),
                    time_spent: std::time::Duration::from_secs(60),
                    tools_invoked: 5,
                    quality_checks_run: 3,
                    quality_checks_passed: 3,
                },
            },
            artifacts: vec![devman_core::Artifact {
                name: "src/db.rs".to_string(),
                artifact_type: "code".to_string(),
                location: "src/db.rs".to_string(),
            }],
            issues: vec![],
            resolutions: vec![],
        };

        let experience = extractor.extract(&work_record);
        assert!(experience.is_some());
    }

    #[tokio::test]
    async fn test_keyword_embedding_generator() {
        let generator = Arc::new(KeywordEmbeddingGenerator::new());
        let embedding = generator.generate("test query").await.unwrap();
        assert!(!embedding.0.is_empty());
    }
}
