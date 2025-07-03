use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use nodespace_data_store::NodeType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Single Date Node Creation with Hierarchical Embeddings");

    // Initialize the service
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

    // Create a test date
    let test_date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();
    println!("📅 Creating date node for: {}", test_date);

    // Try to create just the date node itself
    let date_node_id = service
        .create_node_for_date(
            test_date,
            "", // Empty content for date nodes
            NodeType::Date,
            None,
        )
        .await?;

    println!("✅ Successfully created date node: {}", date_node_id);

    Ok(())
}
