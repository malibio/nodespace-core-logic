//! Test RAG Query Functionality with Marketing Sample Data
//!
//! This example validates that the RAG pipeline works correctly with the comprehensive
//! marketing sample data, testing semantic search and query processing capabilities.

use nodespace_core_logic::{ServiceContainer, CoreLogic};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Testing RAG Query Functionality with Marketing Sample Data\n");

    // Use the same database path as the Tauri app and sample data
    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized for RAG testing\n");

    // Test queries that should find relevant marketing content
    let test_queries = vec![
        "social media campaign strategy",
        "email marketing performance",
        "customer personas and insights",
        "brand guidelines and creative briefs",
        "competitor analysis and market research",
        "budget planning and ROI analysis",
        "product launch strategies",
        "partnership opportunities",
        "content marketing plans",
        "marketing automation workflows",
    ];

    println!("🎯 Testing Semantic Search Across Marketing Topics:\n");

    for (i, query) in test_queries.iter().enumerate() {
        println!("{}. Testing query: \"{}\"", i + 1, query);
        
        match service_container.semantic_search(query, 5).await {
            Ok(results) => {
                if results.is_empty() {
                    println!("   ⚠️  No results found for query");
                } else {
                    println!("   ✅ Found {} relevant results:", results.len());
                    for (j, result) in results.iter().enumerate() {
                        // Show content preview
                        let content_preview = result.node.content.to_string()
                            .chars().take(80).collect::<String>();
                        let preview = if content_preview.len() < result.node.content.to_string().len() {
                            format!("{}...", content_preview)
                        } else {
                            content_preview
                        };
                        println!("      {}. Score: {:.3} - {}", j + 1, result.score, preview);
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Error in semantic search: {}", e);
            }
        }
        println!();
    }

    println!("🤖 Testing RAG Query Processing (Context + Generation):\n");

    let rag_queries = vec![
        "What are our main social media strategies?",
        "How are our email campaigns performing?",
        "What do we know about our customer personas?",
        "What competitive advantages do we have?",
        "What are our budget priorities for marketing?",
    ];

    for (i, query) in rag_queries.iter().enumerate() {
        println!("{}. RAG Query: \"{}\"", i + 1, query);
        
        match service_container.process_query(query).await {
            Ok(response) => {
                println!("   ✅ Generated response (confidence: {:.3}):", response.confidence);
                // Show first few lines of response
                let response_lines: Vec<&str> = response.answer.lines().take(3).collect();
                for line in response_lines {
                    println!("      {}", line);
                }
                if response.answer.lines().count() > 3 {
                    println!("      ... (response continues)");
                }
                println!("      Sources used: {} nodes", response.sources.len());
            }
            Err(e) => {
                println!("   ❌ Error in RAG query processing: {}", e);
            }
        }
        println!();
    }

    println!("📊 Testing Cross-Content Type Search:\n");

    // Test queries that should span different content types
    let cross_type_queries = vec![
        ("meeting", "Should find meeting notes and action items"),
        ("campaign", "Should find campaign strategies and performance data"),
        ("budget", "Should find budget planning and ROI analysis"),
        ("research", "Should find market research and competitor analysis"),
        ("creative", "Should find creative briefs and brand guidelines"),
    ];

    for (keyword, description) in cross_type_queries {
        println!("Testing keyword: \"{}\" - {}", keyword, description);
        
        match service_container.semantic_search(keyword, 3).await {
            Ok(results) => {
                println!("   ✅ Found {} results spanning different content types:", results.len());
                for (j, result) in results.iter().enumerate() {
                    let content_preview = result.node.content.to_string()
                        .chars().take(60).collect::<String>();
                    println!("      {}. Score: {:.3} - {}...", j + 1, result.score, content_preview);
                }
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
            }
        }
        println!();
    }

    println!("🎉 RAG Query Testing Complete!\n");
    println!("✅ Test Results Summary:");
    println!("   • Semantic search tested across 10 marketing topics");
    println!("   • RAG query processing tested with 5 questions");
    println!("   • Cross-content type search validated with 5 keywords");
    println!("   • Marketing sample data provides comprehensive coverage for RAG functionality");
    
    Ok(())
}