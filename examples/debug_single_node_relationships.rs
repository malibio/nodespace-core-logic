//! Debug Single Node Relationships
//!
//! Test relationship queries for a specific node to understand the syntax

use nodespace_core_logic::{ServiceContainer, CoreLogic};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Debugging single node relationships...");

    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized\n");

    // Get a sample node to test with
    let nodes = service_container.get_nodes_for_date("2025-06-25").await?;
    println!("📊 Found {} nodes for 2025-06-25", nodes.len());
    
    // Find a parent node (one that should have children)
    let mut test_node = None;
    for node in &nodes {
        if let Some(content_str) = node.content.as_str() {
            if content_str.starts_with("## ") { // Parent nodes start with ##
                test_node = Some(node);
                break;
            }
        }
    }
    
    if let Some(node) = test_node {
        println!("\n🔍 Testing parent node:");
        println!("ID: {}", node.id);
        let content_preview = node.content.as_str()
            .map(|s| s.chars().take(100).collect::<String>())
            .unwrap_or_else(|| "NULL".to_string());
        println!("Content: {}", content_preview);
        
        // Now test the get_child_nodes method directly
        println!("\n🔍 Testing get_child_nodes method...");
        match service_container.get_child_nodes(&node.id).await {
            Ok(children) => {
                println!("✅ get_child_nodes returned {} children", children.len());
                for (i, child) in children.iter().take(3).enumerate() {
                    let child_content = child.content.as_str()
                        .map(|s| s.chars().take(50).collect::<String>())
                        .unwrap_or_else(|| "NULL".to_string());
                    println!("  Child {}: {} | {}", i+1, child.id, child_content);
                }
            }
            Err(e) => {
                println!("❌ get_child_nodes failed: {}", e);
            }
        }
        
        // Let's also manually check if we can find any children that should be related
        println!("\n🔍 Looking for potential children manually...");
        let parent_content = node.content.as_str().unwrap_or("");
        let mut potential_children = 0;
        
        for potential_child in &nodes {
            if let Some(child_content) = potential_child.content.as_str() {
                // Children typically start with ** (bold markdown)
                if child_content.starts_with("**") && !child_content.starts_with("## ") {
                    potential_children += 1;
                    if potential_children <= 3 { // Show first 3
                        let child_preview = child_content.chars().take(50).collect::<String>();
                        println!("  Potential child {}: {} | {}", potential_children, potential_child.id, child_preview);
                    }
                }
            }
        }
        
        println!("📊 Found {} total potential children for this parent", potential_children);
        
        if potential_children > 0 {
            println!("\n💡 We have {} parent nodes and {} potential children, but get_child_nodes finds 0", 
                     nodes.iter().filter(|n| n.content.as_str().map(|s| s.starts_with("## ")).unwrap_or(false)).count(),
                     potential_children);
            println!("   This suggests the relationship traversal is not working correctly");
        }
    } else {
        println!("❌ No parent nodes found (nodes starting with ##)");
    }
    
    Ok(())
}