use nodespace_core_logic::{CoreLogic, NodeSpaceService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Quick RAG Test - Real ONNX Inference");
    println!("=====================================");

    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await?;

    // Test a simple query
    let query = "How much of the marketing team's resources would we need?";
    println!("🔍 Query: \"{}\"", query);

    // Test semantic search
    println!("\n📊 Top 5 semantic search results:");
    let results = service.semantic_search(query, 5).await?;

    for (i, result) in results.iter().enumerate() {
        let content = result.node.content.as_str().unwrap_or("No content");
        println!(
            "{}. Score: {:.3} | Content: \"{}\"",
            i + 1,
            result.score,
            content.chars().take(60).collect::<String>()
        );
    }

    // Test RAG with shorter response
    println!("\n🤖 Testing RAG pipeline:");
    let rag_response = service.process_query(query).await?;

    println!("📝 Generated Answer: \"{}\"", rag_response.answer);
    println!("📚 Sources used: {}", rag_response.sources.len());
    println!("🎯 Confidence: {:.2}", rag_response.confidence);

    println!("\n✅ ONNX inference test completed!");

    Ok(())
}
