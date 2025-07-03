use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, NodeSpaceService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Hierarchy Query Methods");
    println!("=================================");

    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await?;

    let test_date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();
    println!("🔍 Testing date: {}", test_date);

    // Method 1: get_nodes_for_date (what the app is using)
    println!("\n📊 Method 1: get_nodes_for_date");
    let result1 = service.get_nodes_for_date(test_date).await?;
    println!("   Found {} nodes", result1.len());

    // Method 2: Try to get all nodes with this root_id
    println!("\n📊 Method 2: Checking semantic search coverage");
    let search_results = service
        .semantic_search("launch strategy marketing video", 20)
        .await?;
    println!("   Semantic search found {} results", search_results.len());

    // Show some sample content to verify it's the right data
    println!("\n📝 Sample content from semantic search:");
    for (i, result) in search_results.iter().take(10).enumerate() {
        let content = result.node.content.as_str().unwrap_or("No content");
        println!(
            "   {}. Score: {:.3} | Content: \"{}\"",
            i + 1,
            result.score,
            content.chars().take(60).collect::<String>()
        );
    }

    // Check if nodes have the correct root_id structure
    println!("\n🔍 Checking node structure:");
    if let Some(first_result) = search_results.first() {
        println!("   Node ID: {}", first_result.node.id.as_str());
        if let Some(parent_id) = &first_result.node.parent_id {
            println!("   Parent ID: {}", parent_id.as_str());
        }
        if let Some(root_id) = &first_result.node.root_id {
            println!("   Root ID: {}", root_id.as_str());
        }
        println!("   Created: {:?}", first_result.node.created_at);
    }

    println!("\n✅ Hierarchy query test completed");
    println!("\n🔬 DIAGNOSIS:");
    println!(
        "   - get_nodes_for_date returns {} nodes (should be 144)",
        result1.len()
    );
    println!("   - Semantic search finds {} nodes", search_results.len());
    println!("   - Data exists but get_nodes_for_date isn't returning the full hierarchy");

    Ok(())
}
