use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, HierarchyComputation, NodeSpaceService};
use nodespace_core_types::{NodeId, NodeSpaceResult};
use nodespace_data_store::NodeType;
use serde_json::json;

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🧪 Testing Simple Hierarchy Creation");
    println!("====================================");

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

    let test_date = NaiveDate::from_ymd_opt(2025, 7, 3).unwrap(); // Different date for this test

    // Step 1: Create date node
    println!("\n📅 Creating date node");
    let date_result = service
        .create_node_for_date_with_id(
            NodeId::from_string("2025-07-03".to_string()),
            test_date,
            "Test Date Node - July 3, 2025",
            NodeType::Date,
            Some(json!({"test": "date_node"})),
            None,
        )
        .await;

    match date_result {
        Ok(_) => println!("   ✅ Date node created successfully"),
        Err(e) => println!("   ❌ Date node creation failed: {}", e),
    }

    // Step 2: Create first child
    println!("\n📋 Creating first child node");
    let child1_id = NodeId::new();
    let child1_result = service
        .create_node_for_date_with_id(
            child1_id.clone(),
            test_date,
            "First Child Node Content",
            NodeType::Text,
            Some(json!({"test": "child1"})),
            Some(NodeId::from_string("2025-07-03".to_string())),
        )
        .await;

    match child1_result {
        Ok(_) => println!(
            "   ✅ First child created successfully: {}",
            child1_id.as_str()
        ),
        Err(e) => println!("   ❌ First child creation failed: {}", e),
    }

    // Step 3: Create second child
    println!("\n📋 Creating second child node");
    let child2_id = NodeId::new();
    let child2_result = service
        .create_node_for_date_with_id(
            child2_id.clone(),
            test_date,
            "Second Child Node Content",
            NodeType::Text,
            Some(json!({"test": "child2"})),
            Some(NodeId::from_string("2025-07-03".to_string())),
        )
        .await;

    match child2_result {
        Ok(_) => println!(
            "   ✅ Second child created successfully: {}",
            child2_id.as_str()
        ),
        Err(e) => println!("   ❌ Second child creation failed: {}", e),
    }

    // Step 4: Create grandchild
    println!("\n👶 Creating grandchild node");
    let grandchild_id = NodeId::new();
    let grandchild_result = service
        .create_node_for_date_with_id(
            grandchild_id.clone(),
            test_date,
            "Grandchild Node Content",
            NodeType::Text,
            Some(json!({"test": "grandchild"})),
            Some(child1_id.clone()),
        )
        .await;

    match grandchild_result {
        Ok(_) => println!(
            "   ✅ Grandchild created successfully: {}",
            grandchild_id.as_str()
        ),
        Err(e) => println!("   ❌ Grandchild creation failed: {}", e),
    }

    // Step 5: Verify all nodes
    println!("\n🔍 Verifying hierarchy");
    let all_nodes = service.get_nodes_for_date(test_date).await?;
    println!(
        "✅ Found {} total nodes for date {}",
        all_nodes.len(),
        test_date
    );

    for (i, node) in all_nodes.iter().enumerate() {
        let parent_info = match &node.parent_id {
            Some(parent_id) => format!("Parent: {}", parent_id.as_str()),
            None => "Root".to_string(),
        };
        println!(
            "{}. {} - {} - {}",
            i + 1,
            node.id.as_str(),
            parent_info,
            node.content.as_str().unwrap_or("No content")
        );
    }

    println!("\n🎯 Simple hierarchy test complete!");
    Ok(())
}
