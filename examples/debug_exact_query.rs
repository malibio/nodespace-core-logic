use nodespace_core_logic::{CoreLogic, NodeSpaceService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing exact query from desktop app logs");

    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await?;

    // Test the exact query from the logs
    let query =
        "How much of the marketing team's resources would we need to support the Product Launch";
    println!("🎯 Query: \"{}\"", query);

    // Get more results to see if Campaign Management is just outside top 5
    let results = service.semantic_search(query, 15).await?;

    println!("\n📊 All {} results:", results.len());
    for (i, result) in results.iter().enumerate() {
        let content = result.node.content.as_str().unwrap_or("No content");
        println!(
            "{}. Score: {:.3} | Content: \"{}\"",
            i + 1,
            result.score,
            content
        );

        if content.contains("Campaign Management") && content.contains("40%") {
            println!(
                "   🎯 FOUND CAMPAIGN MANAGEMENT NODE AT POSITION {}!",
                i + 1
            );
        }
    }

    // Also test a more direct query
    println!("\n\n🔍 Testing more direct query:");
    let direct_query = "marketing team capacity Product Launch";
    let direct_results = service.semantic_search(direct_query, 10).await?;

    for (i, result) in direct_results.iter().enumerate() {
        let content = result.node.content.as_str().unwrap_or("No content");
        if content.contains("Campaign Management") {
            println!(
                "{}. Score: {:.3} | Content: \"{}\"",
                i + 1,
                result.score,
                content
            );
            println!("   🎯 FOUND IT WITH DIRECT QUERY!");
            break;
        }
    }

    Ok(())
}
