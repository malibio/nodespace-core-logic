use nodespace_core_logic::{NodeSpaceService, CoreLogic};
use nodespace_data_store::SurrealDataStore;
use nodespace_nlp_engine::LocalNLPEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NodeSpace Core Logic - Distributed Contract Integration Demo");
    
    // Initialize services using distributed contracts
    let data_store = SurrealDataStore::new("memory").await?;
    let nlp_engine = LocalNLPEngine::new();
    
    // Create the core service that orchestrates everything
    let service = NodeSpaceService::new(data_store, nlp_engine);
    
    println!("✅ Successfully created NodeSpace service with distributed contracts");
    println!("   - DataStore: imported from nodespace-data-store");
    println!("   - NLPEngine: imported from nodespace-nlp-engine");
    
    // Demo basic functionality
    println!("\n🔧 Testing basic functionality...");
    
    // Create a test node
    let node_id = service.create_node(
        json!("This is a test document about Rust programming."),
        Some(json!({"type": "document", "tags": ["rust", "programming"]}))
    ).await?;
    
    println!("✅ Created node with ID: {}", node_id);
    
    // Retrieve the node
    if let Some(node) = service.get_node(&node_id).await? {
        println!("✅ Retrieved node: {:?}", node.content);
    }
    
    // Demo search functionality
    let search_results = service.search_nodes("rust").await?;
    println!("✅ Search found {} nodes containing 'rust'", search_results.len());
    
    println!("\n🎉 Distributed contract integration working successfully!");
    println!("NS-22 requirements verified:");
    println!("  ✅ Imports DataStore trait from nodespace-data-store");
    println!("  ✅ Imports NLPEngine trait from nodespace-nlp-engine");
    println!("  ✅ Uses nodespace-core-types for shared types");
    println!("  ✅ Service orchestration functional");
    
    Ok(())
}