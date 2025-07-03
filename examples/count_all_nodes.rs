use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use nodespace_core_types::{NodeId, NodeSpaceResult};

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🔍 Counting All Nodes in Database");
    println!("==================================");

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

    let campaign_date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();

    // Method 1: Check nodes for date (direct children only)
    println!("\n📊 Method 1: get_nodes_for_date()");
    let date_nodes = service.get_nodes_for_date(campaign_date).await?;
    println!(
        "   Found {} nodes via get_nodes_for_date()",
        date_nodes.len()
    );

    // Method 2: Search for various terms to find more nodes
    println!("\n📊 Method 2: Semantic search for various terms");

    let search_terms = vec![
        "Product Launch",
        "EcoSmart",
        "Campaign",
        "Budget",
        "Marketing",
        "Target Audience",
        "Demographics",
        "Sustainability",
        "Launch Date",
        "Professional",
    ];

    let mut found_node_ids = std::collections::HashSet::new();

    for term in search_terms {
        let results = service.semantic_search(term, 50).await?;
        println!("   '{}': {} results", term, results.len());

        for result in results {
            found_node_ids.insert(result.node_id.to_string());
        }
    }

    println!(
        "\n📈 Total unique nodes found via search: {}",
        found_node_ids.len()
    );

    // Method 3: Try to search for specific content we know exists
    println!("\n📊 Method 3: Search for specific known content");

    let specific_searches = vec![
        "Age: 28-45 years",
        "Income: 75,000-150,000",
        "Education: College degree",
        "Digital Advertising: 65,000",
        "Video production: 45,000",
    ];

    for search_term in specific_searches {
        let results = service.semantic_search(search_term, 10).await?;
        println!("   '{}': {} results", search_term, results.len());
    }

    println!("\n🎯 Node counting complete!");
    Ok(())
}
