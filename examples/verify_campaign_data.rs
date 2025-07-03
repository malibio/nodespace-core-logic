use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use nodespace_core_types::{NodeId, NodeSpaceResult};

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🔍 Verifying Campaign Data Population");
    println!("=====================================");

    // Initialize service pointing to the database
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    // Initialize AI services (warnings expected)
    match service.initialize().await {
        Ok(_) => println!("   ✅ AI services ready"),
        Err(e) => println!("   ⚠️  AI warning: {} (continuing)", e),
    }

    let today = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();

    // Check how many nodes exist for our date
    println!("\n📊 Checking nodes for date: {}", today);
    let date_nodes = service.get_nodes_for_date(today).await?;
    println!("✅ Found {} nodes for date {}", date_nodes.len(), today);

    // List all nodes with their hierarchical structure
    println!("\n📋 Node Details:");
    for (i, node) in date_nodes.iter().enumerate() {
        let parent_info = match &node.parent_id {
            Some(parent_id) => format!("Parent: {}", parent_id.as_str()),
            None => "Root node".to_string(),
        };

        let content_preview = if let Some(content_str) = node.content.as_str() {
            let preview = content_str.chars().take(100).collect::<String>();
            if content_str.len() > 100 {
                format!("{}...", preview)
            } else {
                preview
            }
        } else {
            "No content".to_string()
        };

        println!("{}. {} - {}", i + 1, node.id.as_str(), parent_info);
        println!("   Content: {}", content_preview);
        println!("   Type: {:?}", node.r#type);
        if let Some(metadata) = &node.metadata {
            println!(
                "   Metadata: {}",
                serde_json::to_string_pretty(metadata).unwrap_or("Error".to_string())
            );
        }
        println!();
    }

    // Check if we can find specific campaign strategy content
    println!("🔍 Searching for campaign strategy content...");
    let search_results = service
        .semantic_search("Product Launch Campaign Strategy", 5)
        .await?;
    println!(
        "✅ Found {} nodes matching 'Product Launch Campaign Strategy'",
        search_results.len()
    );

    for result in search_results.iter() {
        println!(
            "   - {}: {}",
            result.node_id.as_str(),
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

    println!("\n🎯 Database verification complete!");
    Ok(())
}
