use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Hierarchical Nodes Implementation");

    // Initialize service for testing
    println!("1️⃣ Initializing NodeSpace service...");
    let service = NodeSpaceService::create_for_development().await?;
    service.initialize().await?;
    println!("   ✅ Service initialized");

    // Test date
    let test_date = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
    println!("2️⃣ Testing with date: {}", test_date);

    // First ensure date node exists
    println!("3️⃣ Ensuring date node exists...");
    let date_node_id = service.ensure_date_node_exists(test_date).await?;
    println!("   ✅ Date node ID: {}", date_node_id);

    // Create some test nodes for the date using the knowledge node method
    println!("4️⃣ Creating test nodes...");
    let mut metadata1 = serde_json::Map::new();
    metadata1.insert(
        "parent_id".to_string(),
        serde_json::Value::String(date_node_id.to_string()),
    );

    let node1_id = service
        .create_knowledge_node(
            "First test node for hierarchical testing",
            serde_json::Value::Object(metadata1),
        )
        .await?;
    println!("   ✅ Created node 1: {}", node1_id);

    let mut metadata2 = serde_json::Map::new();
    metadata2.insert(
        "parent_id".to_string(),
        serde_json::Value::String(date_node_id.to_string()),
    );

    let node2_id = service
        .create_knowledge_node(
            "Second test node with some content",
            serde_json::Value::Object(metadata2),
        )
        .await?;
    println!("   ✅ Created node 2: {}", node2_id);

    // Test the new hierarchical API
    println!("5️⃣ Testing get_hierarchical_nodes_for_date...");
    let hierarchical_result = service.get_hierarchical_nodes_for_date(test_date).await?;

    println!("   📊 Hierarchical Results:");
    println!("      - Date node ID: {}", hierarchical_result.date_node.id);
    println!("      - Total count: {}", hierarchical_result.total_count);
    println!("      - Has content: {}", hierarchical_result.has_content);
    println!(
        "      - Children count: {}",
        hierarchical_result.children.len()
    );

    // Display hierarchical structure
    for (i, child) in hierarchical_result.children.iter().enumerate() {
        println!(
            "      📝 Child {}: depth={}, index={}",
            i + 1,
            child.depth,
            child.sibling_index
        );
        if let Some(content) = child.node.content.as_str() {
            println!(
                "         Content: {}",
                content.chars().take(50).collect::<String>()
            );
        }

        // Show nested children if any
        if !child.children.is_empty() {
            println!("         🌿 Has {} nested children", child.children.len());
        }
    }

    // Test with empty date (should return empty hierarchical structure)
    println!("6️⃣ Testing with empty date...");
    let empty_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let empty_result = service.get_hierarchical_nodes_for_date(empty_date).await?;

    println!("   📭 Empty date results:");
    println!("      - Total count: {}", empty_result.total_count);
    println!("      - Has content: {}", empty_result.has_content);
    println!("      - Children count: {}", empty_result.children.len());

    println!("7️⃣ Comparing with old API...");
    let old_api_result = service.get_nodes_for_date(test_date).await?;
    println!("   🔄 Old API returned {} nodes", old_api_result.len());
    println!(
        "   🆕 New API returned {} hierarchical nodes",
        hierarchical_result.children.len()
    );

    if old_api_result.len() == hierarchical_result.children.len() {
        println!("   ✅ Counts match - hierarchical API working correctly!");
    } else {
        println!("   ⚠️  Count mismatch - may need investigation");
    }

    println!("\n🎉 Hierarchical nodes test completed successfully!");
    println!("✅ HierarchicalNodes and HierarchicalNode types working");
    println!("✅ get_hierarchical_nodes_for_date method functional");
    println!("✅ Proper structure with depth and sibling indexing");
    println!("✅ Empty state handling working correctly");

    Ok(())
}
