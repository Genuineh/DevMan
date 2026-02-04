//! Comprehensive E2E test for vector search with Ollama

use std::sync::Arc;
use tokio::sync::Mutex;
use devman_knowledge::VectorKnowledgeService;
use devman_storage::Storage;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    println!("=== DevMan Vector Search E2E Test ===\n");

    // Create storage
    let storage_path = "/tmp/devman-test/.devman";
    let storage = Arc::new(Mutex::new(
        devman_storage::JsonStorage::new(storage_path).await?,
    ));
    println!("[OK] Storage initialized at: {}\n", storage_path);

    // Create vector service config
    let config = devman_core::VectorSearchConfig {
        enabled: true,
        model: devman_core::EmbeddingModel::Qwen3Embedding0_6B,
        ollama_url: "http://localhost:11434".to_string(),
        dimension: 1024,
        threshold: 0.5,
    };
    println!("[OK] Config: model={:?}, threshold={}\n", config.model, config.threshold);

    let vector_service = devman_knowledge::VectorKnowledgeServiceImpl::new(storage.clone(), config);
    println!("[OK] Vector service created\n");

    // Check Ollama
    let is_available = vector_service.is_available().await;
    if !is_available {
        println!("[FAIL] Ollama is not available\n");
        return Err(anyhow::anyhow!("Ollama not available"));
    }
    println!("[OK] Ollama is available\n");

    // Initialize
    vector_service.initialize().await?;
    println!("[OK] Vector index initialized\n");

    // Create test knowledge
    println!("--- Creating Test Knowledge ---");

    let knowledge_items = vec![
        ("Rust Error Handling", "How to properly handle errors in Rust using Result<T, E> and the ? operator"),
        ("Python Exception Handling", "Guide to handling exceptions in Python using try except blocks"),
        ("Database ACID Transactions", "Managing database transactions with ACID properties"),
        ("JavaScript Async Await", "Understanding asynchronous programming with async and await in JavaScript"),
        ("Go Error Handling", "Error handling patterns in Go using error types and returns"),
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
                detail: "".to_string(),
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

    // Test semantic searches
    println!("--- Semantic Search Tests ---\n");

    let tests = vec![
        ("how to handle errors in code", vec!["Rust Error Handling", "Python Exception Handling", "Go Error Handling"]),
        ("ACID database transactions", vec!["Database ACID Transactions"]),
        ("async programming patterns", vec!["JavaScript Async Await"]),
        ("exception management in software", vec!["Python Exception Handling", "Rust Error Handling"]),
    ];

    let mut passed = 0;
    let mut total = 0;

    for (query, expected_titles) in tests {
        println!("Query: '{}'", query);
        println!("Expected: {:?}", expected_titles);

        let results = vector_service.search_by_vector(query, 5, 0.4).await?;
        let result_titles: Vec<_> = results.iter().map(|r| r.knowledge.title.clone()).collect();
        println!("Results: {:?}", result_titles);
        println!("Scores: {:?}", results.iter().map(|r| r.score).collect::<Vec<_>>());

        // Check if at least one expected result is found
        let found = expected_titles.iter().any(|e| result_titles.iter().any(|rt| rt == *e));
        if found {
            println!("[PASS] Found at least one expected result\n");
            passed += 1;
        } else {
            println!("[FAIL] None of expected results found\n");
        }
        total += 1;
    }

    // Summary
    println!("=== Test Summary ===");
    println!("Passed: {}/{}\n", passed, total);

    if passed == total {
        println!("All semantic search tests passed!");
        println!("\nVector search with Qwen3-Embedding-0.6B is working correctly on CPU.");
    } else {
        println!("Some tests failed.");
    }

    Ok(())
}
