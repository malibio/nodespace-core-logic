use chrono::NaiveDate;
use nodespace_core_logic::{DateNavigation, LegacyCoreLogic, NodeSpaceService};
use nodespace_data_store::LanceDataStore;
use nodespace_nlp_engine::LocalNLPEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Date Navigation - Debugging Empty Database Issue");

    // Initialize services
    let data_store = LanceDataStore::new("memory").await?;
    let nlp_engine = LocalNLPEngine::new();
    let service = NodeSpaceService::new(data_store, nlp_engine);

    // Test 1: Check if database is empty
    println!("\n1️⃣ Checking if database is empty...");
    let today = chrono::Utc::now().date_naive();
    let nodes_today = service.get_nodes_for_date(today).await?;
    println!(
        "   Nodes found for today ({}): {}",
        today,
        nodes_today.len()
    );

    // Test 2: Create some sample nodes for testing
    println!("\n2️⃣ Creating sample nodes for testing...");

    // Create a node with today's date
    let node_id1 = service
        .create_node(
            json!("This is a test node created today for debugging date navigation"),
            Some(json!({"type": "test", "purpose": "date_navigation_debug"})),
        )
        .await?;
    println!("   ✅ Created test node: {}", node_id1);

    // Create another node
    let node_id2 = service
        .create_node(
            json!("Another test node to verify date filtering works correctly"),
            Some(json!({"type": "test", "category": "verification"})),
        )
        .await?;
    println!("   ✅ Created second test node: {}", node_id2);

    // Test 3: Query for today's nodes again
    println!("\n3️⃣ Re-checking nodes for today...");
    let nodes_today_after = service.get_nodes_for_date(today).await?;
    println!(
        "   Nodes found for today after creation: {}",
        nodes_today_after.len()
    );

    for (i, node) in nodes_today_after.iter().enumerate() {
        println!(
            "   Node {}: ID={}, created_at={}",
            i + 1,
            node.id,
            node.created_at
        );
        if let Some(content) = node.content.as_str() {
            println!(
                "            Content: {}",
                content.chars().take(50).collect::<String>()
            );
        }
    }

    // Test 4: Try navigation
    println!("\n4️⃣ Testing date navigation...");
    let nav_result = service.navigate_to_date(today).await?;
    println!("   Navigation result:");
    println!("     Date: {}", nav_result.date);
    println!("     Nodes: {}", nav_result.nodes.len());
    println!("     Has previous: {}", nav_result.has_previous);
    println!("     Has next: {}", nav_result.has_next);

    // Test 5: Test with yesterday (should be empty)
    let yesterday = today - chrono::Duration::days(1);
    println!("\n5️⃣ Testing with yesterday ({})...", yesterday);
    let nodes_yesterday = service.get_nodes_for_date(yesterday).await?;
    println!("   Nodes found for yesterday: {}", nodes_yesterday.len());

    // Test 6: Debug timestamp format
    println!("\n6️⃣ Debugging timestamp format...");
    if let Ok(Some(node)) = service.get_node(&node_id1).await {
        println!("   Sample node timestamp: {}", node.created_at);
        if let Ok(parsed_dt) = chrono::DateTime::parse_from_rfc3339(&node.created_at) {
            println!("   Parsed date: {}", parsed_dt.date_naive());
            println!("   Today's date: {}", today);
            println!("   Dates match: {}", parsed_dt.date_naive() == today);
        } else {
            println!("   ❌ Failed to parse timestamp as RFC3339");
        }
    }

    println!("\n🎯 Summary:");
    if nodes_today_after.len() > 0 {
        println!(
            "   ✅ Date navigation is working - found {} nodes for today",
            nodes_today_after.len()
        );
        println!("   ➡️  The original issue was likely an empty database");
    } else {
        println!("   ❌ Date navigation still not working - deeper investigation needed");
    }

    Ok(())
}
