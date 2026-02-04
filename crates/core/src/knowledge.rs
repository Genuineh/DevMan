//! Knowledge model - reusable cognitive assets.

use serde::{Deserialize, Serialize};
use crate::id::{KnowledgeId, WorkRecordId};
use crate::Time;

/// Knowledge is a reusable cognitive asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Knowledge {
    /// Unique identifier
    pub id: KnowledgeId,

    /// Title
    pub title: String,

    /// Knowledge type
    pub knowledge_type: KnowledgeType,

    /// Content
    pub content: KnowledgeContent,

    /// Metadata
    pub metadata: KnowledgeMetadata,

    /// Tags
    pub tags: Vec<String>,

    /// Related knowledge
    pub related_to: Vec<KnowledgeId>,

    /// Derived from work records
    pub derived_from: Vec<WorkRecordId>,

    /// Usage statistics
    pub usage_stats: UsageStats,

    /// Created at
    pub created_at: Time,

    /// Updated at
    pub updated_at: Time,
}

/// Types of knowledge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum KnowledgeType {
    LessonLearned { lesson: String, context: String },
    BestPractice { practice: String, rationale: String },
    CodePattern { pattern: CodeSnippet, usage: String },
    Solution { problem: String, solution: String, verified: bool },
    Template { template: TemplateContent, 适用场景: Vec<String> },
    Decision { decision: String, alternatives: Vec<String>, reasoning: String },
}

/// Knowledge content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeContent {
    /// Summary
    pub summary: String,

    /// Detail
    pub detail: String,

    /// Code examples
    pub examples: Vec<CodeSnippet>,

    /// Reference links
    pub references: Vec<String>,
}

/// Code snippet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    /// Language
    pub language: String,

    /// Code
    pub code: String,

    /// Description
    pub description: String,
}

// Implement PartialEq and Eq for CodeSnippet and TemplateContent
impl PartialEq for CodeSnippet {
    fn eq(&self, other: &Self) -> bool {
        self.language == other.language && self.code == other.code
    }
}

impl Eq for CodeSnippet {}

/// Template content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContent {
    /// Template text
    pub template: String,

    /// Parameters
    pub parameters: Vec<TemplateParameter>,
}

impl PartialEq for TemplateContent {
    fn eq(&self, other: &Self) -> bool {
        self.template == other.template
    }
}

impl Eq for TemplateContent {}

/// Template parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// Parameter name
    pub name: String,

    /// Description
    pub description: String,

    /// Default value
    pub default_value: Option<String>,

    /// Required
    pub required: bool,
}

/// Knowledge metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeMetadata {
    /// Domains/areas
    pub domain: Vec<String>,

    /// Tech stack
    pub tech_stack: Vec<String>,

    /// Applicable scenarios
    pub scenarios: Vec<String>,

    /// Quality score
    pub quality_score: f32,

    /// Verified
    pub verified: bool,
}

/// Usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Times used
    pub times_used: usize,

    /// Last used
    pub last_used: Option<Time>,

    /// Success rate
    pub success_rate: f32,

    /// Feedback
    pub feedback: Vec<Feedback>,
}

/// User feedback on knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    /// Rating (1-5)
    pub rating: i32,

    /// Comment
    pub comment: String,

    /// When
    pub at: Time,

    /// From
    pub from: String,
}

// Export type alias for compatibility
pub type KnowledgeUpdate = ();

/// Embedding model type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbeddingModel {
    /// Qwen3 Embedding (Ollama local)
    Qwen3Embedding0_6B,
    /// OpenAI text-embedding-ada-002
    OpenAIAda002,
    /// Custom model via Ollama
    Ollama { name: String },
}

/// Configuration for vector search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchConfig {
    /// Enable vector search
    pub enabled: bool,

    /// Embedding model to use
    pub model: EmbeddingModel,

    /// Ollama server URL (for local models)
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,

    /// Embedding dimension
    #[serde(default = "default_dimension")]
    pub dimension: usize,

    /// Similarity threshold (0.0 - 1.0)
    #[serde(default = "default_threshold")]
    pub threshold: f32,
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_dimension() -> usize {
    1024 // Qwen3-Embedding-0.6B dimension
}

fn default_threshold() -> f32 {
    0.75
}

impl Default for VectorSearchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            model: EmbeddingModel::Qwen3Embedding0_6B,
            ollama_url: default_ollama_url(),
            dimension: default_dimension(),
            threshold: default_threshold(),
        }
    }
}

/// Knowledge embedding cache - stores pre-computed embeddings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEmbedding {
    /// Knowledge ID
    pub knowledge_id: KnowledgeId,

    /// The embedding vector
    pub embedding: Vec<f32>,

    /// Model used to generate this embedding
    pub model: EmbeddingModel,

    /// When this embedding was generated
    #[serde(default)]
    pub created_at: Time,
}

/// A knowledge item with its similarity score.
#[derive(Debug, Clone)]
pub struct ScoredKnowledge {
    /// The knowledge item
    pub knowledge: Knowledge,
    /// Similarity score (0.0 - 1.0, higher is more similar)
    pub score: f32,
}

/// Reranker model type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RerankerModel {
    /// Qwen3 Reranker (Ollama local)
    Qwen3Reranker0_6B,
    /// OpenAI reranker (if available)
    OpenAIReranker,
    /// Custom model via Ollama
    Ollama { name: String },
}

/// Configuration for reranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankerConfig {
    /// Enable reranking
    pub enabled: bool,

    /// Reranker model to use
    pub model: RerankerModel,

    /// Ollama server URL
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,

    /// Maximum candidates to rerank (after vector search)
    #[serde(default = "default_max_candidates")]
    pub max_candidates: usize,

    /// Final top-k results after reranking
    #[serde(default = "default_final_top_k")]
    pub final_top_k: usize,
}

fn default_max_candidates() -> usize {
    50
}

fn default_final_top_k() -> usize {
    10
}

impl Default for RerankerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            model: RerankerModel::Qwen3Reranker0_6B,
            ollama_url: default_ollama_url(),
            max_candidates: default_max_candidates(),
            final_top_k: default_final_top_k(),
        }
    }
}

/// Result of a reranking operation.
#[derive(Debug, Clone)]
pub struct RerankedKnowledge {
    /// The knowledge item
    pub knowledge: Knowledge,
    /// Reranker score (0.0 - 1.0, higher is more relevant)
    pub rerank_score: f32,
    /// Original vector similarity score
    pub vector_score: Option<f32>,
    /// Combined score (if using fusion)
    pub combined_score: Option<f32>,
}
