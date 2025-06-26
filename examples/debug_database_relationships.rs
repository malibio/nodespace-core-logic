//! Debug Database Relationships
//!
//! This directly queries the database to see what relationships exist

use nodespace_core_logic::{ServiceContainer, CoreLogic};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Debugging database relationships directly...");

    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized\n");

    // Get a sample of nodes to understand the structure
    let nodes = service_container.get_nodes_for_date("2025-06-25").await?;
    println!("📊 Found {} nodes for 2025-06-25", nodes.len());
    
    if let Some(first_node) = nodes.first() {
        println!("\n🔍 Testing with first node:");
        println!("ID: {}", first_node.id);
        let content_preview = first_node.content.as_str()
            .map(|s| s.chars().take(100).collect::<String>())
            .unwrap_or_else(|| "NULL".to_string());
        println!("Content: {}...", content_preview);
        
        // Check if this node has metadata that might indicate parent/child relationships
        println!("Metadata: {:?}", first_node.metadata);
        
        // Try to see what relationships exist in the contains table
        println!("\n🔍 Checking contains relationships...");
        let children = service_container.get_child_nodes(&first_node.id).await?;
        println!("Children found: {}", children.len());
        
        for (i, child) in children.iter().take(3).enumerate() {
            let child_content = child.content.as_str()
                .map(|s| s.chars().take(50).collect::<String>())
                .unwrap_or_else(|| "NULL".to_string());
            println!("  Child {}: {} | {}", i+1, child.id, child_content);
        }
    }
    
    // Let's also check if we have any date nodes
    println!("\n🔍 Let's check if hierarchical structure is working...");
    let hierarchical_nodes = service_container.get_hierarchical_nodes_for_date("2025-06-25").await?;
    
    println!("Hierarchical nodes returned: {}", hierarchical_nodes.len());
    for (i, hn) in hierarchical_nodes.iter().take(5).enumerate() {
        let content_preview = hn.node.content.as_str()
            .map(|s| s.chars().take(60).collect::<String>())
            .unwrap_or_else(|| "NULL".to_string());
        println!("  HNode {}: {} | Children: {} | Parent: {:?} | Depth: {}", 
                 i+1, hn.node.id, hn.children.len(), hn.parent, hn.depth_level);
        println!("    Content: {}", content_preview);
        println!("    Children IDs: {:?}", hn.children);
    }
    
    Ok(())
}