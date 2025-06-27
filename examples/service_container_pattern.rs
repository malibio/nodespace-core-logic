use nodespace_core_logic::{CoreLogic, DateNavigation, NodeSpaceService};
use serde_json::json;

/// Demonstrates the service container pattern with dependency injection
/// for database paths and model directories across different environments
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️ Service Container Pattern - Dependency Injection Demo");

    // Environment 1: Development
    println!("\n1️⃣ Development Environment Setup");
    let dev_service = NodeSpaceService::create_for_development().await?;
    dev_service.initialize().await?;
    println!("   ✅ Development service initialized");
    println!("      - Database: ../data/lance_db/development.db");
    println!("      - Models: ../models/ (workspace relative)");

    // Environment 2: Testing
    println!("\n2️⃣ Testing Environment Setup");
    let test_service = NodeSpaceService::create_for_testing().await?;
    test_service.initialize().await?;
    println!("   ✅ Testing service initialized");
    println!("      - Database: memory (no persistence)");
    println!("      - Models: Smart path resolution");

    // Environment 3: Production (with explicit paths)
    println!("\n3️⃣ Production Environment Setup");
    let prod_service = NodeSpaceService::create_for_production(
        "../data/lance_db/production_demo.db", // Demo path, would be /var/lib/nodespace in real production
        "../models", // Demo path, would be /usr/share/nodespace/models in real production
    )
    .await?;
    prod_service.initialize().await?;
    println!("   ✅ Production service initialized");
    println!("      - Database: ../data/lance_db/production_demo.db (demo path)");
    println!("      - Models: ../models (demo path)");

    // Environment 4: Custom (Desktop App Pattern)
    println!("\n4️⃣ Desktop App Service Container Pattern");
    let desktop_service = NodeSpaceService::create_with_paths(
        "../../data/lance_db/e2e_sample.db", // Your desktop app's database
        Some("./bundled_models"),            // Bundled with app
    )
    .await?;
    desktop_service.initialize().await?;
    println!("   ✅ Desktop app service initialized");
    println!("      - Database: ../../data/lance_db/e2e_sample.db");
    println!("      - Models: ./bundled_models");

    // Test functionality with the desktop service
    println!("\n5️⃣ Testing Desktop App Service");

    // Check if database needs initialization
    let today = chrono::Utc::now().date_naive();
    let existing_nodes = desktop_service.get_nodes_for_date(today).await?;

    if existing_nodes.is_empty() {
        println!("   🔧 Empty database detected - initializing with sample data");

        // Create welcome content
        let welcome_id = desktop_service.create_knowledge_node(
            "Welcome to NodeSpace! This is your first knowledge node created via the service container pattern.",
            json!({
                "type": "welcome",
                "environment": "desktop_app",
                "created_via": "service_container"
            })
        ).await?;
        println!("      ✅ Created welcome node: {}", welcome_id);

        // Verify it was stored
        let nodes_after = desktop_service.get_nodes_for_date(today).await?;
        println!(
            "      📊 Nodes in database after initialization: {}",
            nodes_after.len()
        );
    } else {
        println!(
            "   ✅ Database already contains {} nodes for today",
            existing_nodes.len()
        );
    }

    // Test AI functionality
    println!("\n6️⃣ Testing AI Integration");
    let search_results = desktop_service.semantic_search("welcome", 3).await?;
    println!(
        "   🔍 Semantic search for 'welcome': {} results",
        search_results.len()
    );

    let query_response = desktop_service.process_query("What is NodeSpace?").await?;
    println!(
        "   🤖 AI Query response confidence: {:.1}%",
        query_response.confidence * 100.0
    );

    println!("\n🎯 Service Container Benefits Demonstrated:");
    println!("  ✅ Environment-specific configuration");
    println!("  ✅ Database path injection");
    println!("  ✅ Model directory injection");
    println!("  ✅ Factory methods for each environment");
    println!("  ✅ Centralized dependency management");

    println!("\n📋 Desktop App Integration Pattern:");
    println!("```rust");
    println!("// In your desktop app's service initialization");
    println!("let service = NodeSpaceService::create_with_paths(");
    println!("    \"../../data/lance_db/e2e_sample.db\",");
    println!("    Some(\"./bundled_models\")");
    println!(").await?;");
    println!("service.initialize().await?;");
    println!("```");

    Ok(())
}
