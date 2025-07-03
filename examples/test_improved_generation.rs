use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, NodeSpaceService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Improved ONNX Text Generation");
    println!("========================================");

    // Initialize service
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    // Initialize AI services
    match service.initialize().await {
        Ok(_) => println!("✅ AI services initialized successfully"),
        Err(e) => {
            println!("⚠️ AI initialization warning: {}", e);
            println!("Continuing with test...");
        }
    }

    // Test with the query that worked well before
    let test_query = "What is the target income for campaign participants?";

    println!("\n🔍 Testing Query: \"{}\"", test_query);
    println!("🎯 Expected to find: Income range 75,000-150,000 annually");

    // Execute the RAG query
    println!("\n🚀 Executing RAG query with enhanced parameters...");
    match service.process_query(test_query).await {
        Ok(response) => {
            println!("✅ RAG Query SUCCESS!");
            println!("📊 Query Execution Report:");
            println!("   • Sources found: {}", response.sources.len());
            println!("   • Confidence: {:.1}%", response.confidence * 100.0);
            println!("   • Answer: \"{}\"", response.answer);

            // Check if the answer contains the expected information
            let answer_lower = response.answer.to_lowercase();
            if answer_lower.contains("75") && answer_lower.contains("150") {
                println!("🎯 SUCCESS: Answer contains expected income range!");
            } else if answer_lower.contains("income") {
                println!("💡 PARTIAL: Answer mentions income but may not have specific range");
            } else {
                println!("⚠️ ISSUE: Answer doesn't seem to contain expected income information");
            }

            // Show sources for verification
            println!("\n📚 Source Analysis:");
            for (i, source) in response.sources.iter().take(3).enumerate() {
                println!("   {}. Source ID: {}", i + 1, source.as_str());
            }
        }
        Err(e) => {
            println!("❌ RAG Query FAILED: {}", e);
            return Err(e.into());
        }
    }

    println!("\n🔬 ENHANCED GENERATION TEST SUMMARY:");
    println!("   • Temperature: 1.0 (increased from 0.7)");
    println!("   • Max tokens: 100 (limited for focused answers)");
    println!("   • top_p: 0.9 (hard-coded in NLP engine, team wants 0.95)");
    println!("   • Prompt format: Simplified from verbose instructions");
    println!("   • Debug logging: Enhanced tracing enabled");

    println!("\n✅ Improved generation test completed!");

    Ok(())
}
