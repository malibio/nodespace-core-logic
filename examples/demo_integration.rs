use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, LegacyCoreLogic, NodeSpaceService};
use nodespace_data_store::LanceDataStore;
use nodespace_nlp_engine::LocalNLPEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing Hierarchy Fix for get_nodes_for_date() - BLOCKING DESKTOP APP");

    // Test the hierarchy fix first
    test_hierarchy_fix().await?;

    println!("\nNodeSpace Core Logic - NS-13 Business Logic & Integration Demo");

    // Initialize services using distributed contracts
    let data_store = LanceDataStore::new("memory").await?;
    let nlp_engine = LocalNLPEngine::new();

    // Create the core service that orchestrates everything
    let service = NodeSpaceService::new(data_store, nlp_engine);

    println!("✅ Successfully created NodeSpace service with distributed contracts");
    println!("   - DataStore: imported from nodespace-data-store");
    println!("   - NLPEngine: imported from nodespace-nlp-engine");

    // Demo NS-13 Enhanced Business Logic Features
    println!("\n🎯 NS-13: Demonstrating Enhanced Business Logic...");

    // 1. Knowledge Management - Create knowledge node with AI processing
    println!("\n1️⃣ Knowledge Management:");
    let node_id = service.create_knowledge_node(
        "NodeSpace is an entity-centric, AI-powered knowledge management system that enables users to organize information as interconnected entities with semantic relationships.",
        json!({"type": "knowledge_base", "topic": "NodeSpace", "category": "system_overview"})
    ).await?;
    println!("✅ Created knowledge node with AI processing: {}", node_id);

    // 2. AI Integration - Semantic search with embeddings
    println!("\n2️⃣ AI Integration:");
    let search_results = service
        .semantic_search("knowledge management system", 5)
        .await?;
    println!(
        "✅ Semantic search found {} results with AI-powered scoring",
        search_results.len()
    );
    for result in &search_results {
        println!(
            "   Score: {:.2} - Content: {:?}",
            result.score,
            result
                .node
                .content
                .as_str()
                .unwrap_or("")
                .chars()
                .take(50)
                .collect::<String>()
        );
    }

    // 3. Natural Language Query Processing
    println!("\n3️⃣ Natural Language Query Processing:");
    let response = service
        .process_query("What is NodeSpace and how does it work?")
        .await?;
    println!("✅ Processed natural language query:");
    println!(
        "   Answer: {}",
        response.answer.chars().take(100).collect::<String>()
    );
    println!("   Confidence: {:.1}%", response.confidence * 100.0);
    println!("   Sources: {} nodes", response.sources.len());
    println!("   Related queries: {:?}", response.related_queries);

    // 4. Data Orchestration - Update node and reprocess
    println!("\n4️⃣ Data Orchestration:");
    service.update_node(&node_id, "NodeSpace is an advanced AI-powered knowledge management platform that revolutionizes how users organize, discover, and interact with information through intelligent entity relationships.").await?;
    println!("✅ Updated node content and reprocessed embeddings");

    // 5. Graph Relationship Operations
    println!("\n5️⃣ Graph Operations:");
    let related_nodes = service
        .get_related_nodes(&node_id, vec!["related_to".to_string()])
        .await?;
    println!(
        "✅ Found {} related nodes using graph relationships",
        related_nodes.len()
    );

    // 6. AI-Powered Insight Generation
    println!("\n6️⃣ Insight Generation:");

    // Create additional nodes for insight analysis
    let node_id2 = service.create_knowledge_node(
        "Entity-centric architecture allows for flexible data modeling and natural relationship discovery.",
        json!({"type": "technical_concept", "topic": "architecture"})
    ).await?;

    let node_id3 = service.create_knowledge_node(
        "Semantic search capabilities enable users to find information based on meaning rather than exact keywords.",
        json!({"type": "feature", "topic": "search"})
    ).await?;

    let insights = service
        .generate_insights(vec![node_id.clone(), node_id2, node_id3])
        .await?;
    println!("✅ Generated AI-powered insights from multiple nodes:");
    println!("   {}", insights.chars().take(200).collect::<String>());

    // Legacy API Compatibility Test
    println!("\n🔄 Legacy API Compatibility:");
    let legacy_node = service
        .create_node(
            json!("Testing legacy API compatibility"),
            Some(json!({"legacy": true})),
        )
        .await?;
    println!("✅ Legacy API still functional: {}", legacy_node);

    println!("\n🎉 NS-13 Business Logic & Integration Complete!");
    println!("All core business logic areas implemented:");
    println!("  ✅ Knowledge Management - Document processing, indexing, retrieval");
    println!("  ✅ AI Integration - NLP processing, semantic search, content analysis");
    println!("  ✅ Data Orchestration - Cross-repository data flow and consistency");
    println!("  ✅ Event Handling - System events and state changes");
    println!("  ✅ API Layer - External interfaces and service coordination");
    println!("  ✅ Distributed Contract Usage - Direct service trait imports");

    Ok(())
}

/// Test the hierarchy fix for get_nodes_for_date - only returns top-level nodes
async fn test_hierarchy_fix() -> Result<(), Box<dyn std::error::Error>> {
    println!("1️⃣ Creating service with pure LanceDB (no HashMap fallback)");

    // Create service using pure LanceDB
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/hierarchy_test.db",
        None,
    )
    .await?;
    service.initialize().await?;

    let today = chrono::Utc::now().date_naive();

    println!("2️⃣ Creating test hierarchy for {}", today);

    // Create a top-level parent node (no parent_id)
    let parent_id = service
        .create_node(
            json!("PARENT: This should appear in get_nodes_for_date"),
            Some(json!({
                "type": "parent",
                "category": "top_level"
                // No parent_id = top-level
            })),
        )
        .await?;

    // Create a child node (has parent_id)
    let _child_id = service
        .create_node(
            json!("CHILD: This should NOT appear in get_nodes_for_date"),
            Some(json!({
                "type": "child",
                "category": "nested",
                "parent_id": parent_id.to_string() // Has parent_id = child level
            })),
        )
        .await?;

    // Create another top-level node
    let _sibling_id = service
        .create_node(
            json!("SIBLING: This should appear in get_nodes_for_date"),
            Some(json!({
                "type": "sibling",
                "category": "top_level"
                // No parent_id = top-level
            })),
        )
        .await?;

    println!("3️⃣ Testing get_nodes_for_date() hierarchy fix");

    let nodes_for_date = service.get_nodes_for_date(today).await?;

    println!("   📊 Nodes returned: {}", nodes_for_date.len());
    println!("   🎯 Expected: 2 (parent + sibling only)");

    let mut found_parent = false;
    let mut found_sibling = false;
    let mut found_child = false;

    for (i, node) in nodes_for_date.iter().enumerate() {
        if let Some(content) = node.content.as_str() {
            println!(
                "   📝 Node {}: {}",
                i + 1,
                content.chars().take(30).collect::<String>()
            );

            if content.contains("PARENT:") {
                found_parent = true;
            }
            if content.contains("SIBLING:") {
                found_sibling = true;
            }
            if content.contains("CHILD:") {
                found_child = true;
            }
        }

        // Check for hierarchy violations
        if let Some(metadata) = &node.metadata {
            if metadata.get("parent_id").is_some() {
                println!("      ❌ ERROR: Node with parent_id returned!");
            }
        }
    }

    println!("4️⃣ Hierarchy Fix Results");

    if found_parent && found_sibling && !found_child && nodes_for_date.len() == 2 {
        println!("   ✅ SUCCESS: Hierarchy fix working!");
        println!("   ✅ Only top-level nodes returned");
        println!("   ✅ Child nodes properly filtered out");
        println!("   ✅ Desktop app should now show proper hierarchy");
    } else {
        println!("   ❌ FAILURE: Hierarchy fix broken");
        println!("      Found parent: {}", found_parent);
        println!("      Found sibling: {}", found_sibling);
        println!("      Found child: {} (should be false)", found_child);
        println!("      Node count: {} (should be 2)", nodes_for_date.len());
    }

    println!("🔧 Hierarchy test complete - proceeding with main demo...\n");

    Ok(())
}
