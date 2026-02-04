//! Vector-based knowledge service using embeddings.
//!
//! This module provides semantic search capability for knowledge items
//! using Ollama's embedding API with Qwen3-Embedding-0.6B model.

use anyhow::{Context, Result};
use async_trait::async_trait;
use devman_core::{
    EmbeddingModel, Knowledge, KnowledgeEmbedding, ScoredKnowledge,
    VectorSearchConfig,
};
use reqwest::{Client, ClientBuilder};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, warn};

/// Ollama Embedding Client.
#[derive(Clone)]
pub struct OllamaEmbeddingClient {
    /// HTTP client
    client: Client,

    /// Ollama server URL
    url: String,

    /// Model name
    model: String,
}

impl OllamaEmbeddingClient {
    /// Create a new Ollama embedding client.
    pub fn new(url: String, model: String) -> Self {
        Self {
            client: ClientBuilder::new()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
            url,
            model,
        }
    }

    /// Generate embedding for a single text.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let payload = json!({
            "model": self.model,
            "prompt": text,
            "options": {
                "num_thread": 4
            }
        });

        debug!("Generating embedding for text ({} chars)", text.len());

        let response = self
            .client
            .post(&format!("{}/api/embeddings", self.url))
            .json(&payload)
            .send()
            .await
            .context("Failed to call Ollama embeddings API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Ollama API error (status {}): {}",
                status,
                error_text
            );
        }

        #[derive(serde::Deserialize)]
        struct Response {
            embedding: Vec<f32>,
        }

        let response_data: Response = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(response_data.embedding)
    }

    /// Generate embeddings for multiple texts in batch.
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            match self.embed(text).await {
                Ok(embedding) => results.push(embedding),
                Err(e) => {
                    warn!("Failed to embed text: {}", e);
                    // Return zeros for failed embeddings
                    results.push(vec![0.0; 1024]); // Default dimension
                }
            }
        }

        Ok(results)
    }

    /// Check if Ollama server is available.
    pub async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(&format!("{}/api/version", self.url))
            .send()
            .await
            .context("Failed to check Ollama health")?;

        Ok(response.status().is_success())
    }
}

/// In-memory vector index for small-scale semantic search.
#[derive(Clone, Default)]
pub struct LocalVectorIndex {
    /// Knowledge embeddings
    embeddings: Vec<KnowledgeEmbedding>,

    /// Dimension of embeddings
    dimension: usize,
}

impl LocalVectorIndex {
    /// Create a new vector index.
    pub fn new(dimension: usize) -> Self {
        Self {
            embeddings: Vec::new(),
            dimension,
        }
    }

    /// Add an embedding to the index.
    pub fn add(&mut self, embedding: KnowledgeEmbedding) {
        self.embeddings.push(embedding);
    }

    /// Remove an embedding by knowledge ID.
    pub fn remove(&mut self, knowledge_id: &str) {
        self.embeddings
            .retain(|e| e.knowledge_id.to_string() != knowledge_id);
    }

    /// Search for similar embeddings using cosine similarity.
    pub fn search(&self, query: &[f32], limit: usize, threshold: f32) -> Vec<(String, f32)> {
        let mut scored: Vec<_> = self
            .embeddings
            .iter()
            .map(|e| {
                let similarity = cosine_similarity(query, &e.embedding);
                (e.knowledge_id.to_string(), similarity)
            })
            .filter(|(_, score)| *score >= threshold)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(limit).collect()
    }

    /// Get the number of embeddings in the index.
    pub fn len(&self) -> usize {
        self.embeddings.len()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
    }
}

/// Calculate cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Vector knowledge service trait.
#[async_trait]
pub trait VectorKnowledgeService: Send + Sync {
    /// Generate embedding for text.
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;

    /// Save knowledge with its embedding.
    async fn save_with_embedding(&self, knowledge: &Knowledge) -> Result<()>;

    /// Search knowledge by vector similarity.
    async fn search_by_vector(
        &self,
        query: &str,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<ScoredKnowledge>>;

    /// Search with both keyword and vector similarity (hybrid).
    async fn search_hybrid(&self, query: &str, limit: usize) -> Result<Vec<ScoredKnowledge>>;

    /// Re-index all knowledge items.
    async fn reindex_all(&self) -> Result<usize>;

    /// Check if vector search is available.
    async fn is_available(&self) -> bool;
}

/// Implementation of vector knowledge service.
#[derive(Clone)]
pub struct VectorKnowledgeServiceImpl<S: devman_storage::Storage> {
    /// Storage backend (wrapped in mutex for interior mutability)
    storage: Arc<tokio::sync::Mutex<S>>,

    /// Ollama client
    ollama: OllamaEmbeddingClient,

    /// Vector index
    index: Arc<tokio::sync::Mutex<LocalVectorIndex>>,

    /// Configuration
    config: VectorSearchConfig,
}

impl<S: devman_storage::Storage> VectorKnowledgeServiceImpl<S> {
    /// Create a new vector knowledge service.
    pub fn new(storage: Arc<tokio::sync::Mutex<S>>, config: VectorSearchConfig) -> Self {
        let (url, model) = match &config.model {
            EmbeddingModel::Qwen3Embedding0_6B => (config.ollama_url.clone(), "qwen3-embedding:0.6b".to_string()),
            EmbeddingModel::OpenAIAda002 => (config.ollama_url.clone(), "text-embedding-ada-002".to_string()),
            EmbeddingModel::Ollama { name } => (config.ollama_url.clone(), name.clone()),
        };

        let ollama = OllamaEmbeddingClient::new(url, model);

        Self {
            storage,
            ollama,
            index: Arc::new(tokio::sync::Mutex::new(LocalVectorIndex::new(config.dimension))),
            config,
        }
    }

    /// Initialize the index from storage.
    pub async fn initialize(&self) -> Result<()> {
        let all_embeddings = self
            .storage
            .lock()
            .await
            .list_vector_embeddings()
            .await
            .unwrap_or_default();

        let mut index = self.index.lock().await;
        for embedding in all_embeddings {
            index.add(embedding);
        }

        debug!("Initialized vector index with {} embeddings", index.len());
        Ok(())
    }
}

#[async_trait]
impl<S: devman_storage::Storage + 'static> VectorKnowledgeService for VectorKnowledgeServiceImpl<S> {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.ollama.embed(text).await
    }

    async fn save_with_embedding(&self, knowledge: &Knowledge) -> Result<()> {
        // Generate embedding
        let text_to_embed = format!("{}: {}", knowledge.title, knowledge.content.summary);
        let embedding = self.generate_embedding(&text_to_embed).await?;

        // Create embedding record
        let knowledge_embedding = KnowledgeEmbedding {
            knowledge_id: knowledge.id,
            embedding,
            model: self.config.model.clone(),
            created_at: chrono::Utc::now(),
        };

        // Save knowledge to storage
        self.storage
            .lock()
            .await
            .save_knowledge(knowledge)
            .await
            .context("Failed to save knowledge")?;

        // Save embedding to storage
        self.storage
            .lock()
            .await
            .save_vector_embedding(&knowledge_embedding)
            .await
            .context("Failed to save vector embedding")?;

        // Add to index
        let mut index = self.index.lock().await;
        index.add(knowledge_embedding);

        Ok(())
    }

    async fn search_by_vector(
        &self,
        query: &str,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<ScoredKnowledge>> {
        // Generate query embedding
        let query_embedding = self.generate_embedding(query).await?;

        // Search index
        let index = self.index.lock().await;
        let results = index.search(&query_embedding, limit, threshold);

        // Load knowledge for each result
        let storage = self.storage.lock().await;
        let mut scored_knowledge = Vec::new();
        for (knowledge_id_str, score) in results {
            if let Ok(knowledge_id) = knowledge_id_str.parse() {
                if let Ok(Some(knowledge)) = storage.load_knowledge(knowledge_id).await {
                    scored_knowledge.push(ScoredKnowledge {
                        knowledge,
                        score,
                    });
                }
            }
        }

        Ok(scored_knowledge)
    }

    async fn search_hybrid(&self, query: &str, limit: usize) -> Result<Vec<ScoredKnowledge>> {
        // For now, just use vector search
        // TODO: Implement RRF fusion with keyword search
        self.search_by_vector(query, limit, self.config.threshold).await
    }

    async fn reindex_all(&self) -> Result<usize> {
        let all_knowledge = self
            .storage
            .lock()
            .await
            .list_knowledge()
            .await
            .context("Failed to list knowledge")?;

        let mut count = 0;
        for knowledge in &all_knowledge {
            if let Err(e) = self.save_with_embedding(knowledge).await {
                warn!("Failed to reindex knowledge {}: {}", knowledge.id, e);
            } else {
                count += 1;
            }
        }

        Ok(count)
    }

    async fn is_available(&self) -> bool {
        self.ollama.health_check().await.unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_embedding() -> KnowledgeEmbedding {
        KnowledgeEmbedding {
            knowledge_id: devman_core::KnowledgeId::new(),
            embedding: vec![1.0, 0.0, 0.0],
            model: EmbeddingModel::Qwen3Embedding0_6B,
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 1.0);

        let c = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &c), 0.0);

        let d = vec![0.5, 0.5, 0.0];
        let similarity = cosine_similarity(&a, &d);
        assert!(similarity > 0.7 && similarity < 0.71);
    }

    #[test]
    fn test_vector_index_search() {
        let mut index = LocalVectorIndex::new(3);

        // Add some embeddings
        let embedding1 = create_test_embedding();
        let id1_str = embedding1.knowledge_id.to_string();
        index.add(embedding1);

        let embedding2 = {
            KnowledgeEmbedding {
                knowledge_id: devman_core::KnowledgeId::new(),
                embedding: vec![0.0, 1.0, 0.0],
                model: EmbeddingModel::Qwen3Embedding0_6B,
                created_at: chrono::Utc::now(),
            }
        };
        index.add(embedding2);

        // Search for something similar to [1, 0, 0]
        let results = index.search(&vec![1.0, 0.0, 0.0], 10, 0.5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, id1_str);
        assert!((results[0].1 - 1.0).abs() < 0.001);
    }
}
