use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use nodespace_data_store::LanceDataStore;
use nodespace_nlp_engine::LocalNLPEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Simple Embedding Test");
    println!("=========================");

    let data_store = LanceDataStore::new("/Users/malibio/nodespace/data/lance_db").await?;
    let nlp_engine = LocalNLPEngine::new();
    let service = NodeSpaceService::new(data_store, nlp_engine);

    // Test if semantic search returns results
    println!("🔍 Testing semantic search for 'EcoSmart'...");

    match service.semantic_search("EcoSmart", 3).await {
        Ok(results) => {
            println!("✅ Semantic search worked! Found {} results", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("   {}. Score: {:.3}", i + 1, result.score);
                let content = result.node.content.as_str().unwrap_or("");
                println!(
                    "      Content: {}",
                    content.chars().take(100).collect::<String>()
                );
            }

            if results.is_empty() {
                println!("❌ No results found - likely means no embeddings exist");
            } else if results.iter().all(|r| r.score == 0.0) {
                println!("⚠️  All scores are 0.0 - likely fallback to text search, no embeddings");
            } else {
                println!("✅ Non-zero scores suggest embeddings are working!");
            }
        }
        Err(e) => {
            println!("❌ Semantic search failed: {}", e);
        }
    }

    Ok(())
}
