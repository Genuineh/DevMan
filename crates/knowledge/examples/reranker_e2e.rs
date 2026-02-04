//! E2E test for hybrid search with reranking using Ollama

use std::sync::Arc;
use tokio::sync::Mutex;
use devman_knowledge::{RerankerService, RerankerServiceImpl, VectorKnowledgeService};
use devman_core::{RerankerConfig, RerankerModel, VectorSearchConfig, EmbeddingModel};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    println!("=== DevMan Hybrid Search with Reranking E2E Test ===\n");

    // Create storage
    let storage_path = "/tmp/devman-test/.devman";
    let storage = Arc::new(Mutex::new(
        devman_storage::JsonStorage::new(storage_path).await?,
    ));
    println!("[OK] Storage initialized at: {}\n", storage_path);

    // Create reranker config
    let reranker_config = RerankerConfig {
        enabled: true,
        model: RerankerModel::Qwen3Reranker0_6B,
        ollama_url: "http://localhost:11434".to_string(),
        max_candidates: 10,
        final_top_k: 5,
    };
    println!("[OK] Reranker config: model={:?}\n", reranker_config.model);

    let reranker = RerankerServiceImpl::new(reranker_config);

    // Check Ollama rerank availability
    let is_available = reranker.is_available().await;
    if !is_available {
        println!("[WARN] Ollama rerank API not available, testing with disabled reranker\n");
    } else {
        println!("[OK] Ollama rerank API is available\n");
    }

    // Create vector service for comparison
    let vector_config = VectorSearchConfig {
        enabled: true,
        model: EmbeddingModel::Qwen3Embedding0_6B,
        ollama_url: "http://localhost:11434".to_string(),
        dimension: 1024,
        threshold: 0.3,
    };

    let vector_service = devman_knowledge::VectorKnowledgeServiceImpl::new(storage.clone(), vector_config);

    // Check vector availability
    let vector_available = vector_service.is_available().await;
    if !vector_available {
        println!("[FAIL] Ollama embedding not available\n");
        return Err(anyhow::anyhow!("Ollama not available"));
    }
    println!("[OK] Ollama embedding is available\n");

    // Initialize vector service
    vector_service.initialize().await?;
    println!("[OK] Vector index initialized\n");

    // Create test knowledge with similar content
    println!("--- Creating Test Knowledge ---");

    let knowledge_items = vec![
        ("Rust Error Handling", "How to properly handle errors in Rust using Result<T, E> and the ? operator for ergonomic error propagation"),
        ("Rust Error Handling Advanced", "Advanced Rust error handling patterns with custom error types, thiserror crate, and context macros"),
        ("Python Exception Handling", "Guide to handling exceptions in Python using try except blocks with multiple exception types"),
        ("Python Error Best Practices", "Python error handling best practices including logging, custom exceptions, and error recovery"),
        ("JavaScript Error Handling", "JavaScript error handling with try-catch, error boundaries, and async/await error management"),
        ("Database Transaction Management", "Managing database transactions with ACID properties, rollbacks, and concurrency control"),
        ("Go Error Handling Patterns", "Error handling patterns in Go using error types, sentinel errors, and error wrapping"),
    ];

    for (title, summary) in knowledge_items {
        let knowledge = devman_core::Knowledge {
            id: devman_core::KnowledgeId::new(),
            title: title.to_string(),
            knowledge_type: devman_core::KnowledgeType::LessonLearned {
                lesson: "Test knowledge".to_string(),
                context: "Test".to_string(),
            },
            content: devman_core::KnowledgeContent {
                summary: summary.to_string(),
                detail: format!("Detailed information about {}", title.to_lowercase()),
                examples: vec![],
                references: vec![],
            },
            metadata: devman_core::KnowledgeMetadata {
                domain: vec![],
                tech_stack: vec![],
                scenarios: vec![],
                quality_score: 1.0,
                verified: true,
            },
            tags: vec![],
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
        vector_service.save_with_embedding(&knowledge).await?;
        println!("[OK] Created: {}", title);
    }
    println!();

    // Test RRF fusion
    println!("--- RRF Fusion Test ---\n");

    use devman_knowledge::RRFusion;

    let rrf = RRFusion::default();

    // Simulate two different retrieval methods
    let vector_results = vec![
        "Rust Error Handling".to_string(),
        "Python Exception Handling".to_string(),
        "Database Transaction Management".to_string(),
    ];

    let keyword_results = vec![
        "Python Error Best Practices".to_string(),
        "Rust Error Handling Advanced".to_string(),
        "Go Error Handling Patterns".to_string(),
    ];

    let fused = rrf.fuse(&[vector_results.clone(), keyword_results.clone()]);

    println!("Vector results: {:?}", vector_results);
    println!("Keyword results: {:?}", keyword_results);
    println!("RRF fused results:");
    for (i, (id, score)) in fused.iter().enumerate() {
        println!("  {}. {} (score: {:.4})", i + 1, id, score);
    }
    println!();

    // Test hybrid search with reranking
    println!("--- Hybrid Search Test ---\n");

    let query = "error handling patterns in programming";

    // First get vector search results
    let vector_results = vector_service.search_by_vector(query, 10, 0.2).await?;
    println!("Query: '{}'", query);
    println!("Vector search found {} results:", vector_results.len());

    let candidates: Vec<_> = vector_results.iter().map(|r| &r.knowledge).collect();

    for (i, r) in vector_results.iter().enumerate() {
        println!("  {}. {} (vector score: {:.4})", i + 1, r.knowledge.title, r.score);
    }
    println!();

    // Now rerank the results
    if is_available {
        let reranked = reranker.rerank(query, &candidates).await?;
        println!("Reranked results:");
        for (i, r) in reranked.iter().enumerate() {
            println!("  {}. {} (rerank score: {:.4})", i + 1, r.knowledge.title, r.rerank_score);
        }
        println!();
    } else {
        println!("[SKIP] Reranking not available (Ollama rerank API required)\n");
    }

    println!("=== Reranker E2E Test Complete ===");
    Ok(())
}
