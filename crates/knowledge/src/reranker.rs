//! Reranking service for improving search results quality.
//!
//! Uses Qwen3-Reranker-0.6B to re-rank candidate results from vector search.
//! Also implements RRF (Reciprocal Rank Fusion) for combining multiple retrieval methods.

use anyhow::{Context, Result};
use async_trait::async_trait;
use devman_core::{
    Knowledge, RerankerModel, RerankerConfig, RerankedKnowledge,
};
use reqwest::{Client, ClientBuilder};
use serde_json::json;
use tracing::{debug, warn};

/// Ollama Reranker Client.
#[derive(Clone)]
pub struct OllamaRerankerClient {
    /// HTTP client
    client: Client,

    /// Ollama server URL
    url: String,

    /// Model name
    model: String,
}

impl OllamaRerankerClient {
    /// Create a new Ollama reranker client.
    pub fn new(url: String, model: String) -> Self {
        Self {
            client: ClientBuilder::new()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            url,
            model,
        }
    }

    /// Rerank documents given a query using Ollama's rerank API.
    /// Returns relevance scores for each document.
    pub async fn rerank(&self, query: &str, documents: &[String]) -> Result<Vec<f32>> {
        if documents.is_empty() {
            return Ok(vec![]);
        }

        let payload = json!({
            "model": self.model,
            "query": query,
            "documents": documents,
        });

        debug!("Reranking {} documents", documents.len());

        let response = self
            .client
            .post(&format!("{}/api/rerank", self.url))
            .json(&payload)
            .send()
            .await
            .context("Failed to call Ollama rerank API")?;

        // Handle 404 - rerank API not available
        if response.status() == 404 {
            warn!("Ollama /api/rerank endpoint not available, returning neutral scores");
            return Ok(vec![0.5; documents.len()]);
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Ollama rerank API error (status {}): {}",
                status,
                error_text
            );
        }

        #[derive(serde::Deserialize)]
        struct Response {
            results: Vec<RerankResult>,
        }

        #[derive(serde::Deserialize)]
        struct RerankResult {
            relevance_score: f32,
        }

        let response_data: Response = response
            .json()
            .await
            .context("Failed to parse Ollama rerank response")?;

        Ok(response_data
            .results
            .into_iter()
            .map(|r| r.relevance_score)
            .collect())
    }

    /// Check if Ollama server supports rerank API.
    pub async fn is_available(&self) -> bool {
        // Check if /api/rerank endpoint exists
        let rerank_response = self
            .client
            .get(&format!("{}/api/rerank", self.url))
            .send()
            .await;

        // Check if health endpoint also works
        let healthy = self.health_check().await;

        // Rerank API is available if endpoint returns success (not 404)
        if let Ok(resp) = rerank_response {
            if resp.status() == 404 {
                return false; // API endpoint not available
            }
            return healthy;
        }
        false
    }

    /// Check Ollama health.
    pub async fn health_check(&self) -> bool {
        self.client
            .get(&format!("{}/api/version", self.url))
            .send()
            .await
            .is_ok()
            || self
                .client
                .get(&format!("{}/api/tags", self.url))
                .send()
                .await
                .is_ok()
    }
}

/// Reranking service trait.
#[async_trait]
pub trait RerankerService: Send + Sync {
    /// Rerank knowledge items given a query.
    async fn rerank(
        &self,
        query: &str,
        candidates: &[&Knowledge],
    ) -> Result<Vec<RerankedKnowledge>>;

    /// Check if reranking is available.
    async fn is_available(&self) -> bool;
}

/// Reranker service implementation.
#[derive(Clone)]
pub struct RerankerServiceImpl {
    /// Reranker client
    client: Option<OllamaRerankerClient>,

    /// Configuration
    config: RerankerConfig,
}

impl RerankerServiceImpl {
    /// Create a new reranker service.
    pub fn new(config: RerankerConfig) -> Self {
        let client = if config.enabled {
            let model = match &config.model {
                RerankerModel::Qwen3Reranker0_6B => "qwen3-reranker:0.6b".to_string(),
                RerankerModel::OpenAIReranker => "text-embedding-3-small".to_string(),
                RerankerModel::Ollama { name } => name.clone(),
            };
            Some(OllamaRerankerClient::new(config.ollama_url.clone(), model))
        } else {
            None
        };

        Self { client, config }
    }
}

#[async_trait]
impl RerankerService for RerankerServiceImpl {
    async fn rerank(
        &self,
        query: &str,
        candidates: &[&Knowledge],
    ) -> Result<Vec<RerankedKnowledge>> {
        if candidates.is_empty() {
            return Ok(vec![]);
        }

        // If reranker is disabled, return candidates with neutral scores
        if self.client.is_none() {
            warn!("Reranker is disabled, returning candidates without reranking");
            return Ok(candidates
                .iter()
                .map(|k| RerankedKnowledge {
                    knowledge: (*k).clone(),
                    rerank_score: 0.5,
                    vector_score: None,
                    combined_score: None,
                })
                .collect());
        }

        let client = self.client.as_ref().unwrap();

        // Prepare documents for reranking
        let documents: Vec<String> = candidates
            .iter()
            .map(|k| format!("{}: {}", k.title, k.content.summary))
            .collect();

        // Call rerank API
        let scores = client.rerank(query, &documents).await?;

        // Combine results
        let mut results: Vec<_> = candidates
            .iter()
            .zip(scores.into_iter())
            .map(|(&k, score)| RerankedKnowledge {
                knowledge: (*k).clone(),
                rerank_score: score,
                vector_score: None,
                combined_score: None,
            })
            .collect();

        // Sort by rerank score (descending)
        results.sort_by(|a, b| b.rerank_score.partial_cmp(&a.rerank_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    async fn is_available(&self) -> bool {
        if let Some(ref client) = self.client {
            client.is_available().await
        } else {
            false
        }
    }
}

/// Reciprocal Rank Fusion (RRF) for combining multiple retrieval results.
///
/// RRF combines results from different retrieval methods (e.g., vector + keyword)
/// using the formula: RRF(d) = 1 / (k + rank(d))
#[derive(Debug, Clone)]
pub struct RRFusion {
    /// Constant to prevent division by zero and smooth rankings
    k: u32,
}

impl Default for RRFusion {
    fn default() -> Self {
        Self { k: 60 }
    }
}

impl RRFusion {
    /// Create a new RRF combiner.
    pub fn new(k: u32) -> Self {
        Self { k }
    }

    /// Fuse results from multiple retrievers using RRF.
    ///
    /// `results_lists` is a slice of result lists from different retrievers.
    /// Each inner list contains knowledge IDs in order of relevance.
    pub fn fuse(&self, results_lists: &[Vec<String>]) -> Vec<(String, f32)> {
        if results_lists.is_empty() {
            return vec![];
        }

        // Collect all unique IDs
        let mut all_ids = std::collections::HashSet::new();
        for list in results_lists {
            for id in list {
                all_ids.insert(id.clone());
            }
        }

        // Calculate RRF score for each ID
        let mut scores: Vec<_> = all_ids
            .into_iter()
            .map(|id| {
                let rrf_score = self.calculate_rrf(&id, results_lists);
                (id, rrf_score)
            })
            .collect();

        // Sort by RRF score (descending)
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scores
    }

    /// Calculate RRF score for a single document.
    fn calculate_rrf(&self, id: &str, results_lists: &[Vec<String>]) -> f32 {
        let mut total_score = 0.0f32;

        for list in results_lists {
            if let Some(rank) = list.iter().position(|r| r == id) {
                // Rank is 0-indexed, so add 1
                total_score += 1.0 / ((self.k + (rank as u32 + 1)) as f32);
            }
        }

        total_score
    }
}

/// Hybrid search result combining multiple signals.
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    /// The knowledge item
    pub knowledge: Knowledge,

    /// Vector similarity score (if used)
    pub vector_score: Option<f32>,

    /// Reranker score (if used)
    pub rerank_score: Option<f32>,

    /// RRF combined score (if combining multiple methods)
    pub combined_score: Option<f32>,
}

impl HybridSearchResult {
    /// Calculate final score with weighted combination.
    pub fn calculate_final_score(
        &self,
        vector_weight: f32,
        rerank_weight: f32,
    ) -> f32 {
        let vector = self.vector_score.unwrap_or(0.0);
        let rerank = self.rerank_score.unwrap_or(0.0);

        vector * vector_weight + rerank * rerank_weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_fusion_single_list() {
        let rrf = RRFusion::default();
        let lists = vec![
            vec!["doc1".to_string(), "doc2".to_string(), "doc3".to_string()],
        ];

        let scores = rrf.fuse(&lists);

        assert_eq!(scores.len(), 3);
        // doc1 should have highest score (rank 1)
        assert_eq!(scores[0].0, "doc1");
        assert!(scores[0].1 > scores[1].1);
        assert!(scores[1].1 > scores[2].1);
    }

    #[test]
    fn test_rrf_fusion_multiple_lists() {
        let rrf = RRFusion::default();
        let lists = vec![
            vec!["doc1".to_string(), "doc2".to_string(), "doc3".to_string()],
            vec!["doc2".to_string(), "doc1".to_string(), "doc3".to_string()],
        ];

        let scores = rrf.fuse(&lists);

        assert_eq!(scores.len(), 3);
        // doc1 and doc2 have higher score (at rank 0 or 1)
        // doc3 has lowest score (rank 2 in both lists)
        // Due to HashSet iteration order, either doc1 or doc2 comes first
        assert!(scores[0].1 > scores[2].1); // First has higher score than last
        assert!(scores[1].1 > scores[2].1);  // Middle has higher score than last
        // doc3 must be last (lowest score)
        assert_eq!(scores[2].0, "doc3");
    }

    #[test]
    fn test_rrf_fusion_empty() {
        let rrf = RRFusion::default();
        let scores = rrf.fuse(&[]);
        assert!(scores.is_empty());
    }
}
