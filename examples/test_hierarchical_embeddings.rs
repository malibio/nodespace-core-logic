use chrono::NaiveDate;
use nodespace_core_logic::NodeSpaceService;
use nodespace_data_store::NodeType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Hierarchical Embedding Generation");

    // Initialize the service
    let service = NodeSpaceService::create_with_defaults().await?;

    // Create a test date
    let test_date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();

    println!("\n📅 Creating hierarchy for date: {}", test_date);

    // 1. Create root date node (this will be created automatically)
    let root_node_id = service
        .create_node_for_date(test_date, "2025-06-26", NodeType::Date, None)
        .await?;
    println!("✅ Created root node: {}", root_node_id);

    // 2. Create Level 1 - Product Launch Campaign Strategy
    let level1_node_id = service
        .create_node_for_date(
            test_date,
            "# Product Launch Campaign Strategy",
            NodeType::Text,
            None,
        )
        .await?;
    println!("✅ Created Level 1 node: {}", level1_node_id);

    // 3. Create Level 2 - Budget Allocation and Resource Planning
    let level2_node_id = service
        .create_node_for_date(
            test_date,
            "## Budget Allocation and Resource Planning",
            NodeType::Text,
            None,
        )
        .await?;
    println!("✅ Created Level 2 node: {}", level2_node_id);

    // 4. Create Level 3 - Team Resource Allocation
    let level3_node_id = service
        .create_node_for_date(
            test_date,
            "### Team Resource Allocation",
            NodeType::Text,
            None,
        )
        .await?;
    println!("✅ Created Level 3 node: {}", level3_node_id);

    // 5. Create the target node - Campaign Management
    let target_node_id = service
        .create_node_for_date(
            test_date,
            "**Campaign Management**: 40% of marketing team capacity for 12 weeks",
            NodeType::Text,
            None,
        )
        .await?;
    println!("✅ Created target node: {}", target_node_id);

    println!("\n🎯 Expected Hierarchical Context for target node:");
    println!("Root: 2025-06-26");
    println!("Level 1: # Product Launch Campaign Strategy");
    println!("Level 2: ## Budget Allocation and Resource Planning");
    println!("Level 3: ### Team Resource Allocation");
    println!("Current: **Campaign Management**: 40% of marketing team capacity for 12 weeks");

    println!(
        "\n✅ Test completed! Check the logs above to see if hierarchical context was generated."
    );
    println!("🔍 Now try searching for: 'Product Launch marketing team resources'");

    Ok(())
}
