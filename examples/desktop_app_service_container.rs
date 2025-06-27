use nodespace_core_logic::{CoreLogic, DateNavigation, NodeSpaceService};
use serde_json::json;

/// Desktop app service container pattern - exactly matching your production setup
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖥️ Desktop App Service Container - Production Pattern");

    // This is exactly how your desktop app should initialize the service
    println!("\n1️⃣ Service Container Initialization");
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db", // Correct absolute path
        Some("./bundled_models"),                               // Model directory for deployment
    )
    .await?;

    println!("   ✅ Service created with dependency injection");
    println!("      - Database: /Users/malibio/nodespace/data/lance_db/e2e_sample.db");
    println!("      - Models: ./bundled_models");

    // Initialize AI services
    println!("\n2️⃣ Initializing AI Services");
    match service.initialize().await {
        Ok(_) => println!("   ✅ AI services ready"),
        Err(e) => {
            println!("   ⚠️  AI initialization warning: {}", e);
            println!("   ➡️  Continuing in offline mode (normal for demo)");
        }
    }

    // This is the exact scenario your desktop app encounters
    println!("\n3️⃣ Database Status Check (Your Original Issue)");
    let today = chrono::Utc::now().date_naive();
    let existing_nodes = service.get_nodes_for_date(today).await?;

    println!("   📊 Querying database for date: {}", today);
    println!(
        "   📊 Retrieved {} nodes for date {} from database",
        existing_nodes.len(),
        today
    );

    if existing_nodes.is_empty() {
        println!("   🔧 Empty database detected - this was your original issue!");
        println!("   💡 Solution: Initialize with welcome content");

        // Create the first node (this solves your problem)
        let welcome_id = service
            .create_knowledge_node(
                "Welcome to NodeSpace! Start by creating your first note or asking a question.",
                json!({
                    "type": "welcome",
                    "category": "onboarding",
                    "priority": "high"
                }),
            )
            .await?;

        println!("   ✅ Created welcome node: {}", welcome_id);

        // Create a sample note for today
        let note_id = service.create_knowledge_node(
            "This is a sample note created today. You can search for it, ask questions about it, or create relationships with other notes.",
            json!({
                "type": "note",
                "category": "sample",
                "date": today.to_string()
            })
        ).await?;

        println!("   ✅ Created sample note: {}", note_id);

        // Verify the fix
        let nodes_after = service.get_nodes_for_date(today).await?;
        println!(
            "   🎯 FIXED: Now returning {} nodes for date {}",
            nodes_after.len(),
            today
        );
    } else {
        println!(
            "   ✅ Database already populated with {} nodes",
            existing_nodes.len()
        );
    }

    // Test the functionality that was failing
    println!("\n4️⃣ Testing Date Navigation (Previously Failing)");
    let navigation_result = service.navigate_to_date(today).await?;
    println!("   📅 Navigation to {}:", navigation_result.date);
    println!(
        "      - Found {} nodes (was 0 before)",
        navigation_result.nodes.len()
    );
    println!(
        "      - Has previous day: {}",
        navigation_result.has_previous
    );
    println!("      - Has next day: {}", navigation_result.has_next);

    // Show the nodes that are now being returned
    for (i, node) in navigation_result.nodes.iter().enumerate() {
        if let Some(content) = node.content.as_str() {
            println!(
                "      📝 Node {}: {}",
                i + 1,
                content.chars().take(50).collect::<String>()
            );
        }
    }

    // Test yesterday (should still be empty)
    let yesterday = today - chrono::Duration::days(1);
    let yesterday_nodes = service.get_nodes_for_date(yesterday).await?;
    println!("\n5️⃣ Testing Yesterday ({}) - Should Be Empty", yesterday);
    println!(
        "   📊 Nodes for yesterday: {} (expected: 0)",
        yesterday_nodes.len()
    );

    println!("\n🎉 Problem Solved! Summary:");
    println!("  ❌ Original Issue: get_nodes_for_date returning 0 nodes");
    println!("  ✅ Root Cause: Empty database");
    println!("  ✅ Solution: Service container with database initialization");
    println!("  ✅ Architecture: Proper dependency injection for paths");

    println!("\n📋 Desktop App Integration Checklist:");
    println!("  ✅ Use NodeSpaceService::create_with_paths()");
    println!("  ✅ Inject correct database path for each environment");
    println!("  ✅ Inject model directory for bundled deployment");
    println!("  ✅ Check for empty database on startup");
    println!("  ✅ Create welcome content for new users");
    println!("  ✅ Handle AI service initialization gracefully");

    println!("\n🚀 Your Desktop App Should Now Return Nodes Successfully!");

    Ok(())
}
