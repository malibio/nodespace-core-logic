use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, NodeSpaceService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Date Query Issue");
    println!("===========================");

    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await?;

    // Test the exact date from the logs
    let test_date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();
    println!("🔍 Testing date: {}", test_date);

    // Try to get nodes for this date
    println!("\n📊 Attempting to get nodes for date...");
    let result = service.get_nodes_for_date(test_date).await;

    match result {
        Ok(nodes) => {
            println!(
                "✅ SUCCESS: Found {} nodes for date {}",
                nodes.len(),
                test_date
            );
            for (i, node) in nodes.iter().take(5).enumerate() {
                println!(
                    "   {}. Node ID: {} | Content: \"{}\"",
                    i + 1,
                    node.id.as_str(),
                    node.content
                        .as_str()
                        .unwrap_or("No content")
                        .chars()
                        .take(50)
                        .collect::<String>()
                );
            }
            if nodes.len() > 5 {
                println!("   ... and {} more nodes", nodes.len() - 5);
            }
        }
        Err(e) => {
            println!("❌ ERROR: Failed to get nodes: {}", e);
        }
    }

    // Also test a semantic search to see if data is accessible that way
    println!("\n🔍 Testing semantic search...");
    let search_result = service.semantic_search("video views", 5).await;

    match search_result {
        Ok(results) => {
            println!("✅ Semantic search found {} results", results.len());
            for (i, result) in results.iter().enumerate() {
                println!(
                    "   {}. Score: {:.3} | Content: \"{}\"",
                    i + 1,
                    result.score,
                    result
                        .node
                        .content
                        .as_str()
                        .unwrap_or("No content")
                        .chars()
                        .take(50)
                        .collect::<String>()
                );
            }
        }
        Err(e) => {
            println!("❌ Semantic search failed: {}", e);
        }
    }

    println!("\n✅ Date query test completed");

    Ok(())
}
