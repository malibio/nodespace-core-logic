use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use nodespace_core_types::NodeSpaceResult;

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🔍 Searching for Marketing Team Content");
    println!("=====================================");

    // Initialize service
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

    // Search for marketing team specifically
    let queries = vec![
        "marketing team capacity",
        "40% marketing team",
        "Campaign Management",
        "Team Resource Allocation",
        "marketing team resources",
        "40 percent marketing",
    ];

    for query in queries {
        println!("\n🔍 Searching for: '{}'", query);
        let results = service.semantic_search(query, 15).await?;

        println!("📊 Found {} results", results.len());
        for (index, result) in results.iter().enumerate() {
            if let Some(content) = result.node.content.as_str() {
                if content.to_lowercase().contains("marketing")
                    || content.to_lowercase().contains("40%")
                    || content.to_lowercase().contains("campaign management")
                {
                    println!(
                        "   🎯 {}. Node ID: {} (Score: {:.3})",
                        index + 1,
                        result.node_id,
                        result.score
                    );
                    println!(
                        "      Content: {}",
                        content.chars().take(200).collect::<String>()
                    );
                }
            }
        }
    }

    // Also try broader search with more results
    println!("\n🔍 Broad search for 'marketing' with 50 results");
    let results = service.semantic_search("marketing", 50).await?;

    println!("📊 Found {} results", results.len());
    let mut found_target = false;
    for (index, result) in results.iter().enumerate() {
        if let Some(content) = result.node.content.as_str() {
            if content.contains("40%") && content.contains("marketing team") {
                println!(
                    "   🎯 FOUND TARGET! {}. Node ID: {} (Score: {:.3})",
                    index + 1,
                    result.node_id,
                    result.score
                );
                println!("      Content: {}", content);
                found_target = true;
            }
        }
    }

    if !found_target {
        println!(
            "   ❌ Target content '40% marketing team capacity' not found in {} results",
            results.len()
        );
    }

    println!("\n🎯 Marketing team search complete!");
    Ok(())
}
