//! Debug Raw Relationships
//!
//! This checks what relationship records actually exist in SurrealDB

use nodespace_core_logic::{ServiceContainer, CoreLogic};
use nodespace_data_store::DataStore;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Debugging raw relationships in SurrealDB...");

    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized\n");

    // Skip database info for now - we'll focus on the relationships

    // Try to find relationships using raw SurrealDB syntax
    println!("\n🔍 Trying raw relationship queries...");
    
    // Get a sample text node ID to test with
    let nodes = service_container.get_nodes_for_date("2025-06-25").await?;
    if let Some(sample_node) = nodes.first() {
        let node_id_clean = sample_node.id.as_str().replace("-", "_");
        println!("Testing with node ID: {} (clean: {})", sample_node.id, node_id_clean);
        
        // Try different query formats
        let queries = vec![
            format!("SELECT * FROM text:{}->contains", node_id_clean),
            format!("SELECT * FROM contains WHERE in = text:{}", node_id_clean),
            format!("SELECT out FROM text:{}->contains", node_id_clean),
        ];
        
        for (i, query) in queries.iter().enumerate() {
            println!("\n  Query {}: {}", i+1, query);
            // We can't use query_nodes directly, so let's just check the logic
            match service_container.data_store().query_nodes(query).await {
                Ok(results) => {
                    if results.is_empty() {
                        println!("    ❌ No results");
                    } else {
                        println!("    ✅ Found {} results:", results.len());
                        for (j, result) in results.iter().take(2).enumerate() {
                            println!("      Result {}: ID={}", j+1, result.id);
                            println!("        Content: {:?}", result.content);
                        }
                    }
                }
                Err(e) => println!("    ❌ Error: {}", e),
            }
        }
    }
    
    Ok(())
}