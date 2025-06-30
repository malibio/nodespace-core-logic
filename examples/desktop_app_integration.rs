use nodespace_core_logic::{CoreLogic, LegacyCoreLogic, NodeSpaceService};
use nodespace_data_store::LanceDataStore;
use nodespace_nlp_engine::LocalNLPEngine;
use serde_json::json;

/// Example integration pattern for desktop apps showing proper initialization,
/// empty database handling, and model configuration
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Desktop App Integration Example - Comprehensive Setup");

    // Step 1: Configure NLP Engine with smart model path resolution
    println!("\n1️⃣ Configuring NLP Engine...");

    // Option 1: Use environment variable (for production deployment)
    // std::env::set_var("NODESPACE_MODELS_DIR", "/path/to/bundled/models");
    // let nlp_engine = LocalNLPEngine::new();

    // Option 2: Explicit model directory (for development)
    // let nlp_engine = LocalNLPEngine::with_model_directory("../models");

    // Option 3: Use defaults with smart path resolution (recommended for MVP)
    let nlp_engine = LocalNLPEngine::new();
    println!("   ✅ NLP Engine configured with smart path resolution");
    println!("      - Environment: NODESPACE_MODELS_DIR");
    println!("      - Development: ../models/");
    println!("      - Fallback: ~/.cache/nodespace/models/");

    // Step 2: Initialize data store
    println!("\n2️⃣ Initializing Data Store...");
    let data_store = LanceDataStore::new("../data/lance_db/desktop_app.db").await?;
    println!("   ✅ Data store initialized: ../data/lance_db/desktop_app.db");

    // Step 3: Create and initialize NodeSpace service
    println!("\n3️⃣ Creating NodeSpace Service...");
    let service = NodeSpaceService::new(data_store, nlp_engine);
    println!("   ✅ Service created with distributed contract architecture");

    // Initialize the service (loads models)
    println!("   🔧 Initializing AI services...");
    match service.initialize().await {
        Ok(_) => println!("   ✅ Service initialization complete"),
        Err(e) => {
            println!("   ⚠️  Service initialization failed: {}", e);
            println!("   ➡️  Continuing in offline mode for demonstration");
        }
    }

    // Step 4: Check for empty database and handle gracefully
    println!("\n4️⃣ Database Initialization Check...");
    let today = chrono::Utc::now().date_naive();
    let existing_nodes = service.get_nodes_for_date(today).await?;

    println!(
        "   📊 Database status: {} nodes found for today ({})",
        existing_nodes.len(),
        today
    );

    if existing_nodes.is_empty() {
        println!("   🔧 Empty database detected - creating welcome content...");

        // Create welcome node
        let welcome_node_id = service.create_node(
            json!("Welcome to NodeSpace! This is your first knowledge node. You can create new notes, search existing content, and ask natural language questions about your information."),
            Some(json!({
                "type": "welcome",
                "category": "system",
                "priority": "high",
                "tags": ["welcome", "getting-started"]
            }))
        ).await?;
        println!("     ✅ Created welcome node: {}", welcome_node_id);

        // Create example knowledge node
        let example_node_id = service.create_node(
            json!("NodeSpace uses AI-powered semantic search to help you find information based on meaning, not just keywords. Try asking questions like 'What did I learn about AI today?' or 'Show me notes about productivity.'"),
            Some(json!({
                "type": "example",
                "category": "tutorial", 
                "topic": "semantic_search",
                "tags": ["tutorial", "search", "ai"]
            }))
        ).await?;
        println!("     ✅ Created example node: {}", example_node_id);

        // Create sample date node for navigation  
        let date_node_id = service.ensure_date_node_exists(today).await?;
        println!(
            "     ✅ Created date node for today: {} ({})",
            date_node_id, today
        );

        println!("   🎯 Database initialized with {} sample nodes", 2);
    } else {
        println!("   ✅ Database contains existing data - no initialization needed");
    }

    // Step 5: Demonstrate date navigation functionality
    println!("\n5️⃣ Testing Date Navigation...");
    let nav_result = service.navigate_to_date(today).await?;
    println!("   📅 Navigation to {}:", nav_result.date);
    println!("      - Nodes found: {}", nav_result.nodes.len());
    println!("      - Has previous day: {}", nav_result.has_previous);
    println!("      - Has next day: {}", nav_result.has_next);

    for (i, node) in nav_result.nodes.iter().enumerate() {
        if let Some(content) = node.content.as_str() {
            println!(
                "      📝 Node {}: {}",
                i + 1,
                content.chars().take(60).collect::<String>()
            );
        }
    }

    // Step 6: Demonstrate AI-powered query processing
    println!("\n6️⃣ Testing AI Query Processing...");
    let query_response = service
        .process_query("What can NodeSpace help me with?")
        .await?;
    println!("   🤖 Query: 'What can NodeSpace help me with?'");
    println!(
        "      Answer: {}",
        query_response.answer.chars().take(100).collect::<String>()
    );
    println!(
        "      Confidence: {:.1}%",
        query_response.confidence * 100.0
    );
    println!("      Sources: {} nodes", query_response.sources.len());

    // Step 7: Test semantic search
    println!("\n7️⃣ Testing Semantic Search...");
    let search_results = service
        .semantic_search("getting started tutorial", 3)
        .await?;
    println!("   🔍 Search: 'getting started tutorial'");
    println!("      Results: {} matches", search_results.len());

    for (i, result) in search_results.iter().enumerate() {
        println!("      📄 Match {}: Score {:.2}", i + 1, result.score);
        if let Some(content) = result.node.content.as_str() {
            println!(
                "         Content: {}",
                content.chars().take(50).collect::<String>()
            );
        }
    }

    // Step 8: Show empty state handling pattern
    println!("\n8️⃣ Empty State Handling Pattern...");
    let yesterday = today - chrono::Duration::days(1);
    let yesterday_nodes = service.get_nodes_for_date(yesterday).await?;

    if yesterday_nodes.is_empty() {
        println!(
            "   📭 No content found for {} - showing empty state",
            yesterday
        );
        println!("      UI should display: 'No notes for this date. Create your first note?'");
    } else {
        println!(
            "   📁 Found {} nodes for {}",
            yesterday_nodes.len(),
            yesterday
        );
    }

    println!("\n🎉 Desktop App Integration Complete!");
    println!("\n📋 Key Patterns Demonstrated:");
    println!("  ✅ Smart NLP model path resolution");
    println!("  ✅ Empty database detection and initialization");
    println!("  ✅ Welcome content creation for new users");
    println!("  ✅ Date navigation with empty state handling");
    println!("  ✅ AI-powered query processing");
    println!("  ✅ Semantic search functionality");

    println!("\n🔧 Integration Checklist for Desktop App:");
    println!("  □ Set NODESPACE_MODELS_DIR environment variable in production");
    println!("  □ Implement UI empty states for dates with no content");
    println!("  □ Add first-time user onboarding flow");
    println!("  □ Handle AI service initialization errors gracefully");
    println!("  □ Implement proper loading states for async operations");

    Ok(())
}
