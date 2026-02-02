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
