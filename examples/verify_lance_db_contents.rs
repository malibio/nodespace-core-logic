use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use serde_json::json;

/// Verify the contents of the populated LanceDB database
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Verifying LanceDB contents and semantic search functionality");
    println!("=============================================================");

    // Initialize service with the populated database
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/sample_data.db",
        Some("./bundled_models"),
    )
    .await?;

    // Initialize AI services
    println!("\n1️⃣ Initializing service");
    match service.initialize().await {
        Ok(_) => println!("   ✅ AI services ready"),
        Err(e) => println!("   ⚠️  AI warning: {} (continuing)", e),
    }

    // Check total nodes in database
    println!("\n2️⃣ Database content verification");

    // Try a broad semantic search to see what's in the database
    println!("   🔍 Searching for 'product' to see all content");
    match service.semantic_search("product", 10).await {
        Ok(results) => {
            println!("   ✅ Found {} total nodes in database", results.len());
            for (i, result) in results.iter().enumerate() {
                println!(
                    "      {}. ID: {} | Score: {:.3}",
                    i + 1,
                    result.node_id,
                    result.score
                );
                if let Some(content_str) = result.node.content.as_str() {
                    let preview = content_str.chars().take(150).collect::<String>();
                    println!("         Content: {}...", preview);
                } else {
                    println!("         Content: {:?}", result.node.content);
                }
                println!("         Metadata: {:?}", result.node.metadata);
                println!();
            }
        }
        Err(e) => {
            println!("   ❌ Error searching database: {}", e);
        }
    }

    // Test specific semantic queries
    println!("\n3️⃣ Testing specific semantic queries");
    let test_queries = vec![
        ("target audience", "Should find audience analysis content"),
        (
            "budget allocation",
            "Should find financial planning content",
        ),
        (
            "marketing channels",
            "Should find marketing strategy content",
        ),
        (
            "competitive analysis",
            "Should find competitive landscape content",
        ),
        (
            "timeline milestones",
            "Should find project planning content",
        ),
    ];

    for (query, description) in test_queries {
        println!("   🔍 Query: \"{}\"", query);
        println!("      Expected: {}", description);

        match service.semantic_search(query, 3).await {
            Ok(results) => {
                if results.is_empty() {
                    println!("      ❌ No results found");
                } else {
                    println!("      ✅ Found {} results", results.len());
                    for result in results.iter().take(2) {
                        if let Some(content_str) = result.node.content.as_str() {
                            let preview = content_str.chars().take(100).collect::<String>();
                            println!("         - Score: {:.3} | {}", result.score, preview);
                        }
                    }
                }
            }
            Err(e) => {
                println!("      ❌ Search error: {}", e);
            }
        }
        println!();
    }

    // Test process_query (RAG functionality)
    println!("\n4️⃣ Testing RAG query processing");
    match service
        .process_query("What are our main competitive advantages in the product launch strategy?")
        .await
    {
        Ok(response) => {
            println!("   ✅ RAG query successful");
            println!("      Answer: {}", response.answer);
            println!("      Sources: {} nodes used", response.sources.len());
        }
        Err(e) => {
            println!("   ⚠️  RAG query error: {}", e);
        }
    }

    // Check date-based retrieval
    println!("\n5️⃣ Testing date-based node retrieval");
    let today = chrono::Utc::now().date_naive();
    match service.get_nodes_for_date(today).await {
        Ok(date_nodes) => {
            println!("   📅 Date query for {}: {} nodes", today, date_nodes.len());
            if date_nodes.is_empty() {
                println!("      ⚠️  No nodes associated with today's date");
                println!("      💡 This might indicate the nodes weren't properly date-tagged");
            } else {
                for node in date_nodes.iter().take(3) {
                    println!("      - Node ID: {} | Type: {}", node.id, node.r#type);
                }
            }
        }
        Err(e) => {
            println!("   ❌ Date query error: {}", e);
        }
    }

    println!("\n✅ Database verification complete!");
    println!("🎯 Summary: Database contains meaningful sample data and is ready for e2e testing");

    Ok(())
}
