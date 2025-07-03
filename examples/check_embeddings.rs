use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use nodespace_data_store::LanceDataStore;
use nodespace_nlp_engine::LocalNLPEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Checking Vector Embeddings in Database");
    println!("=========================================");

    // Initialize services
    let data_store = LanceDataStore::new("/Users/malibio/nodespace/data/lance_db").await?;

    let nlp_engine = LocalNLPEngine::new();
    let service = NodeSpaceService::new(data_store, nlp_engine);

    // Check nodes from the campaign date (2025-06-26)
    let campaign_date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();
    println!("\n📅 Checking nodes for date: {}", campaign_date);

    let nodes = service.get_nodes_for_date(campaign_date).await?;
    println!("   Found {} nodes for this date", nodes.len());

    // Check a few different dates
    let test_dates = vec![
        NaiveDate::from_ymd_opt(2025, 4, 15).unwrap(), // HR
        NaiveDate::from_ymd_opt(2025, 5, 8).unwrap(),  // Marketing
        NaiveDate::from_ymd_opt(2025, 6, 10).unwrap(), // Operations
    ];

    for date in test_dates {
        let nodes = service.get_nodes_for_date(date).await?;
        println!("   Date {}: {} nodes", date, nodes.len());
    }

    // Try to access the raw data store to check vector fields
    println!("\n🔬 Direct Database Inspection");
    println!("This would require accessing LanceDB directly to check vector fields");
    println!("The vector field should contain a Vec<f32> with 384 dimensions if embeddings exist");

    // Try a semantic search to see if embeddings work
    println!("\n🔍 Testing Semantic Search");
    let search_results = service
        .semantic_search("product launch strategy", 5)
        .await?;
    println!(
        "   Search for 'product launch strategy': {} results",
        search_results.len()
    );

    for (i, result) in search_results.iter().take(3).enumerate() {
        println!(
            "   {}. Score: {:.3}, Content: '{}'",
            i + 1,
            result.score,
            result
                .node
                .content
                .as_str()
                .unwrap_or("")
                .chars()
                .take(60)
                .collect::<String>()
        );
    }

    Ok(())
}
