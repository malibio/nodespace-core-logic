use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use nodespace_core_types::{NodeId, NodeSpaceResult};

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🔍 Finding Specific Node");
    println!("========================");

    // Initialize service
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("/Users/malibio/nodespace/models"),
    )
    .await?;

    service.initialize().await?;

    // Look for the specific node with our target content
    let target_node_id = NodeId::from_string("9f3693a9-42ea-478e-8105-15ea2915b434".to_string());

    println!("🔍 Looking for node: {}", target_node_id.as_str());

    // Use data store directly since get_node isn't available in CoreLogic
    match service.data_store.get_node(&target_node_id).await? {
        Some(node) => {
            println!("✅ FOUND the target node!");
            println!("Content: {:?}", node.content);
            println!("Type: {:?}", node.r#type);
            println!("Parent: {:?}", node.parent_id);

            // Check if it contains our target text
            if let Some(content_str) = node.content.as_str() {
                if content_str.contains("Campaign Management") && content_str.contains("40%") {
                    println!("🎯 YES! This contains the target content!");
                } else {
                    println!("❌ This node exists but doesn't contain target content");
                    println!("Actual content: {}", content_str);
                }
            }
        }
        None => {
            println!("❌ Node not found in database");

            // Let's do a broader search for any content containing "Campaign Management"
            println!("\n🔍 Searching for any content with 'Campaign Management':");
            let search_results = service.semantic_search("Campaign Management", 20).await?;

            for result in search_results {
                if let Some(content_str) = result.node.content.as_str() {
                    if content_str.contains("Campaign Management") {
                        println!("Found: {} - {}", result.node_id, content_str);
                    }
                }
            }
        }
    }

    // Also check all nodes for any containing "40%" and "marketing team"
    println!("\n🔍 Searching for content with '40%' and 'marketing team':");
    let marketing_results = service
        .semantic_search("40% marketing team capacity", 20)
        .await?;

    for result in marketing_results {
        if let Some(content_str) = result.node.content.as_str() {
            if content_str.contains("40%")
                || (content_str.contains("marketing") && content_str.contains("team"))
            {
                println!("Possible match: {} - {}", result.node_id, content_str);
            }
        }
    }

    println!("\n✅ Search complete!");
    Ok(())
}
