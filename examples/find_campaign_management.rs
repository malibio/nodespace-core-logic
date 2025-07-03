use nodespace_core_logic::{CoreLogic, NodeSpaceService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Finding the specific Campaign Management node");

    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await?;

    // Search specifically for the campaign management node
    let results = service
        .semantic_search("Campaign Management 40% marketing team capacity", 10)
        .await?;

    println!("\n🔍 Search Results for Campaign Management:");
    for (i, result) in results.iter().enumerate() {
        let content = result.node.content.as_str().unwrap_or("No content");
        println!(
            "{}. Score: {:.3} | Content: \"{}\"",
            i + 1,
            result.score,
            content
        );

        if content.contains("Campaign Management") && content.contains("40%") {
            println!("   🎯 FOUND THE TARGET NODE!");
        }
    }

    Ok(())
}
