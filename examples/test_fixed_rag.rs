use nodespace_core_logic::{CoreLogic, NodeSpaceService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Testing Fixed RAG Pipeline");
    println!("=============================");

    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await?;

    // Test the exact query that was failing in the desktop app
    let query =
        "How much of the marketing team's resources would we need to support the Product Launch";
    println!("🔍 Query: \"{}\"", query);

    // Test semantic search with new limit
    println!("\n📊 Semantic search results (top 10):");
    let results = service.semantic_search(query, 10).await?;

    let mut found_campaign_management = false;
    for (i, result) in results.iter().enumerate() {
        let content = result.node.content.as_str().unwrap_or("No content");
        println!(
            "{}. Score: {:.3} | Content: \"{}\"",
            i + 1,
            result.score,
            content.chars().take(80).collect::<String>()
        );

        if content.contains("Campaign Management") && content.contains("40%") {
            found_campaign_management = true;
            println!("   🎯 ← FOUND THE PERFECT ANSWER!");
        }
    }

    if found_campaign_management {
        println!("\n✅ SUCCESS: Campaign Management node found in top 10!");
    } else {
        println!("\n❌ FAIL: Campaign Management node still not in top 10");
    }

    // Test full RAG pipeline
    println!("\n🤖 Testing full RAG pipeline:");
    let rag_response = service.process_query(query).await?;

    println!("📝 Generated Answer: \"{}\"", rag_response.answer);
    println!("📚 Sources used: {}", rag_response.sources.len());
    println!("🎯 Confidence: {:.2}", rag_response.confidence);

    // Show source IDs
    println!("📋 Source IDs:");
    for (i, source_id) in rag_response.sources.iter().enumerate() {
        println!("   {}. {}", i + 1, source_id.as_str());
    }

    // The RAG system should have found relevant sources including Campaign Management
    println!(
        "✅ RAG pipeline completed with {} sources",
        rag_response.sources.len()
    );

    Ok(())
}
