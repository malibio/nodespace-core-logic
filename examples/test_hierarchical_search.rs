use nodespace_core_logic::{CoreLogic, NodeSpaceService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Hierarchical Semantic Search");
    println!("======================================");

    // Initialize the service
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    // Initialize AI services
    match service.initialize().await {
        Ok(_) => println!("   ✅ AI services ready"),
        Err(e) => println!("   ⚠️  AI warning: {} (continuing)", e),
    }

    println!("\n🎯 Testing key search query for hierarchical embeddings:");
    println!("Query: \"Product Launch marketing team resources\"");
    println!("Expected to find: \"**Campaign Management**: 40% of marketing team capacity for 12 weeks\"");
    println!("Expected hierarchy context: Root → Product Launch Campaign Strategy → Budget Allocation → Team Resource Allocation → Campaign Management");

    let results = service
        .semantic_search("Product Launch marketing team resources", 5)
        .await?;

    println!("\n📊 Search Results:");
    for (i, result) in results.iter().enumerate() {
        println!(
            "{}. Score: {:.3} | Content: \"{}\"",
            i + 1,
            result.score,
            result
                .node
                .content
                .as_str()
                .unwrap_or("No content")
                .chars()
                .take(100)
                .collect::<String>()
        );
    }

    if let Some(top_result) = results.first() {
        if top_result.score > 0.8 {
            println!(
                "\n✅ SUCCESS: High relevance score ({:.3}) - hierarchical embeddings working!",
                top_result.score
            );
        } else {
            println!(
                "\n⚠️ MODERATE: Score {:.3} - may need tuning",
                top_result.score
            );
        }

        if top_result
            .node
            .content
            .as_str()
            .unwrap_or("")
            .contains("Campaign Management")
        {
            println!("✅ CORRECT: Found the expected Campaign Management node!");
        }
    } else {
        println!("\n❌ FAIL: No search results returned");
    }

    // Test a few more queries
    println!("\n\n🔍 Additional Test Queries:");

    let test_queries = vec![
        "marketing team capacity allocation",
        "12 weeks campaign duration",
        "budget planning resources",
    ];

    for query in test_queries {
        println!("\n🔍 Query: \"{}\"", query);
        let results = service.semantic_search(query, 3).await?;
        if let Some(top) = results.first() {
            println!(
                "   Top result (score: {:.3}): \"{}\"",
                top.score,
                top.node
                    .content
                    .as_str()
                    .unwrap_or("No content")
                    .chars()
                    .take(80)
                    .collect::<String>()
            );
        } else {
            println!("   No results found");
        }
    }

    println!("\n🎉 Hierarchical embedding search test complete!");

    Ok(())
}
