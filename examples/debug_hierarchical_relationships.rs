//! Debug Hierarchical Relationships
//!
//! This checks what relationships actually exist in the database

use nodespace_core_logic::{ServiceContainer, CoreLogic};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Debugging hierarchical relationships...");

    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized\n");

    // Get nodes for a specific date
    let test_date = "2025-06-25";
    println!("📅 Getting nodes for date: {}", test_date);
    
    let nodes = service_container.get_nodes_for_date(test_date).await?;
    println!("📊 Found {} nodes for {}", nodes.len(), test_date);
    
    // Show first few nodes and their content
    for (i, node) in nodes.iter().take(5).enumerate() {
        let content_preview = node.content.as_str()
            .map(|s| s.chars().take(80).collect::<String>())
            .unwrap_or_else(|| "NULL".to_string());
        println!("  {}. ID: {} | Content: {}...", i+1, node.id, content_preview);
    }
    
    println!("\n🔗 Checking parent-child relationships...");
    
    // For each node, try to find its children using different approaches
    for (i, node) in nodes.iter().take(3).enumerate() {
        println!("\n--- Node {} ---", i+1);
        println!("ID: {}", node.id);
        let content_preview = node.content.as_str()
            .map(|s| s.chars().take(60).collect::<String>())
            .unwrap_or_else(|| "NULL".to_string());
        println!("Content: {}...", content_preview);
        
        // Try to get children using the service method
        match service_container.get_child_nodes(&node.id).await {
            Ok(children) => {
                if children.is_empty() {
                    println!("❌ No children found via get_child_nodes");
                } else {
                    println!("✅ Found {} children via get_child_nodes:", children.len());
                    for (j, child) in children.iter().take(3).enumerate() {
                        let child_preview = child.content.as_str()
                            .map(|s| s.chars().take(40).collect::<String>())
                            .unwrap_or_else(|| "NULL".to_string());
                        println!("  Child {}: {} | {}", j+1, child.id, child_preview);
                    }
                }
            }
            Err(e) => println!("❌ Error getting children: {}", e),
        }
    }
    
    Ok(())
}