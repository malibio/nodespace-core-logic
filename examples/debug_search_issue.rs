use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use nodespace_core_types::NodeSpaceResult;

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🔍 Debug Semantic Search Issue");
    println!("===============================");

    // Initialize service with correct model path
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("/Users/malibio/nodespace/models"),
    )
    .await?;

    // Initialize with proper error handling
    match service.initialize().await {
        Ok(_) => println!("✅ AI services ready"),
        Err(e) => {
            println!("❌ AI initialization error: {}", e);
            return Err(e);
        }
    }

    let date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();

    // Check nodes for the specific date
    println!("\n📊 Database contents for {}:", date);
    let date_nodes = service.get_nodes_for_date(date).await?;
    println!("Found {} nodes for {}", date_nodes.len(), date);

    for (i, node) in date_nodes.iter().enumerate() {
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

        println!("{}. Node {}: {}", i + 1, node.id.as_str(), content_preview);

        // Check if this contains our target content
        if let Some(content_str) = node.content.as_str() {
            if content_str.contains("Campaign Management")
                && content_str.contains("40%")
                && content_str.contains("marketing team")
            {
                println!("   🎯 FOUND TARGET CONTENT!");
            }
        }
    }

    // Direct search test
    println!("\n🔍 Testing semantic search:");
    println!("Query: 'How much of the marketing team's resources would we need to support the Product Launch'");

    let results = service.semantic_search(
        "How much of the marketing team's resources would we need to support the Product Launch", 
        10
    ).await?;

    println!("Found {} search results", results.len());

    for (i, result) in results.iter().enumerate() {
        let content_preview = if let Some(content_str) = result.node.content.as_str() {
            let preview = content_str.chars().take(200).collect::<String>();
            if content_str.len() > 200 {
                format!("{}...", preview)
            } else {
                preview
            }
        } else {
            "No content".to_string()
        };

        println!(
            "{}. Score: {:.3}, Content: {}",
            i + 1,
            result.score,
            content_preview
        );
    }

    // Test simpler query
    println!("\n🔍 Testing simpler query:");
    println!("Query: 'marketing team'");

    let simple_results = service.semantic_search("marketing team", 5).await?;
    println!("Found {} results for simple query", simple_results.len());

    for (i, result) in simple_results.iter().enumerate() {
        let content_preview = if let Some(content_str) = result.node.content.as_str() {
            content_str.chars().take(100).collect::<String>()
        } else {
            "No content".to_string()
        };

        println!(
            "{}. Score: {:.3}, Content: {}",
            i + 1,
            result.score,
            content_preview
        );
    }

    // Test query for campaign management specifically
    println!("\n🔍 Testing campaign management query:");
    println!("Query: 'Campaign Management'");

    let campaign_results = service.semantic_search("Campaign Management", 5).await?;
    println!(
        "Found {} results for campaign management",
        campaign_results.len()
    );

    for (i, result) in campaign_results.iter().enumerate() {
        let content_preview = if let Some(content_str) = result.node.content.as_str() {
            content_str.chars().take(150).collect::<String>()
        } else {
            "No content".to_string()
        };

        println!(
            "{}. Score: {:.3}, Content: {}",
            i + 1,
            result.score,
            content_preview
        );
    }

    println!("\n✅ Debug complete!");
    Ok(())
}
