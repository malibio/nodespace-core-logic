use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use nodespace_core_types::NodeSpaceResult;

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🔍 Searching for Description Text");
    println!("=================================");

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

    // Search for the specific description text
    println!("\n🔍 Searching for description text");
    let results = service
        .semantic_search(
            "comprehensive product launch plan provides strategic framework",
            10,
        )
        .await?;

    println!("📊 Found {} results for description text", results.len());
    for (index, result) in results.iter().enumerate() {
        println!(
            "   {}. Node ID: {} (Score: {:.3})",
            index + 1,
            result.node_id,
            result.score
        );
        if let Some(content) = result.node.content.as_str() {
            println!(
                "      Content: {}",
                content.chars().take(100).collect::<String>()
            );
        }
    }

    // Search for "tactical execution"
    println!("\n🔍 Searching for 'tactical execution'");
    let results2 = service
        .semantic_search("tactical execution details", 5)
        .await?;

    println!(
        "📊 Found {} results for 'tactical execution'",
        results2.len()
    );
    for (index, result) in results2.iter().enumerate() {
        println!(
            "   {}. Node ID: {} (Score: {:.3})",
            index + 1,
            result.node_id,
            result.score
        );
        if let Some(content) = result.node.content.as_str() {
            println!(
                "      Content: {}",
                content.chars().take(100).collect::<String>()
            );
        }
    }

    // Search for "success measurement criteria"
    println!("\n🔍 Searching for 'success measurement criteria'");
    let results3 = service
        .semantic_search("success measurement criteria", 5)
        .await?;

    println!(
        "📊 Found {} results for 'success measurement criteria'",
        results3.len()
    );
    for (index, result) in results3.iter().enumerate() {
        println!(
            "   {}. Node ID: {} (Score: {:.3})",
            index + 1,
            result.node_id,
            result.score
        );
        if let Some(content) = result.node.content.as_str() {
            println!(
                "      Content: {}",
                content.chars().take(100).collect::<String>()
            );
        }
    }

    println!("\n🎯 Description search complete!");
    Ok(())
}
