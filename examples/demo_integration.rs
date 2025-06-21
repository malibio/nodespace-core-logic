use nodespace_core_logic::{NodeSpaceService, CoreLogic, LegacyCoreLogic};
use nodespace_data_store::SurrealDataStore;
use nodespace_nlp_engine::LocalNLPEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NodeSpace Core Logic - NS-13 Business Logic & Integration Demo");
    
    // Initialize services using distributed contracts
    let data_store = SurrealDataStore::new("memory").await?;
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
    let search_results = service.semantic_search("knowledge management system", 5).await?;
    println!("✅ Semantic search found {} results with AI-powered scoring", search_results.len());
    for result in &search_results {
        println!("   Score: {:.2} - Content: {:?}", result.score, 
                 result.node.content.as_str().unwrap_or("").chars().take(50).collect::<String>());
    }
    
    // 3. Natural Language Query Processing
    println!("\n3️⃣ Natural Language Query Processing:");
    let response = service.process_query("What is NodeSpace and how does it work?").await?;
    println!("✅ Processed natural language query:");
    println!("   Answer: {}", response.answer.chars().take(100).collect::<String>());
    println!("   Confidence: {:.1}%", response.confidence * 100.0);
    println!("   Sources: {} nodes", response.sources.len());
    println!("   Related queries: {:?}", response.related_queries);
    
    // 4. Data Orchestration - Update node and reprocess
    println!("\n4️⃣ Data Orchestration:");
    service.update_node(&node_id, "NodeSpace is an advanced AI-powered knowledge management platform that revolutionizes how users organize, discover, and interact with information through intelligent entity relationships.").await?;
    println!("✅ Updated node content and reprocessed embeddings");
    
    // 5. Graph Relationship Operations
    println!("\n5️⃣ Graph Operations:");
    let related_nodes = service.get_related_nodes(&node_id, vec!["related_to".to_string()]).await?;
    println!("✅ Found {} related nodes using graph relationships", related_nodes.len());
    
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
    
    let insights = service.generate_insights(vec![node_id.clone(), node_id2, node_id3]).await?;
    println!("✅ Generated AI-powered insights from multiple nodes:");
    println!("   {}", insights.chars().take(200).collect::<String>());
    
    // Legacy API Compatibility Test
    println!("\n🔄 Legacy API Compatibility:");
    let legacy_node = service.create_node(
        json!("Testing legacy API compatibility"),
        Some(json!({"legacy": true}))
    ).await?;
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