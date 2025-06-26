//! Debug Relationship Table
//!
//! Direct inspection of what's in the contains relationship table

use nodespace_core_logic::{ServiceContainer, CoreLogic};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Debugging relationship table structure...");

    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized\n");

    // Let's check if our hierarchical structure is working with relationships
    println!("🔍 Testing hierarchical node retrieval...");
    
    // Test the hierarchical functionality directly
    println!("\n🔍 Testing get_hierarchical_nodes_for_date...");
    let hierarchical_nodes = service_container.get_hierarchical_nodes_for_date("2025-06-25").await?;
    
    println!("📊 Found {} hierarchical nodes", hierarchical_nodes.len());
    
    let mut nodes_with_children = 0;
    let mut nodes_with_parents = 0;
    let mut max_depth = 0;
    
    for (i, hn) in hierarchical_nodes.iter().take(10).enumerate() {
        let content_preview = hn.node.content.as_str()
            .map(|s| s.chars().take(60).collect::<String>())
            .unwrap_or_else(|| "NULL".to_string());
        
        if !hn.children.is_empty() {
            nodes_with_children += 1;
        }
        if hn.parent.is_some() {
            nodes_with_parents += 1;
        }
        if hn.depth_level > max_depth {
            max_depth = hn.depth_level;
        }
        
        println!("  Node {}: Depth={} | Children={} | Parent={:?}", 
                 i+1, hn.depth_level, hn.children.len(), 
                 hn.parent.as_ref().map(|p| p.to_string()).unwrap_or_else(|| "None".to_string()));
        println!("    Content: {}", content_preview);
        if !hn.children.is_empty() {
            println!("    Children IDs: {:?}", hn.children);
        }
    }
    
    println!("\n📈 Summary:");
    println!("  Total nodes: {}", hierarchical_nodes.len());
    println!("  Nodes with children: {}", nodes_with_children);
    println!("  Nodes with parents: {}", nodes_with_parents);
    println!("  Maximum depth: {}", max_depth);
    
    if nodes_with_children == 0 && nodes_with_parents == 0 {
        println!("\n❌ PROBLEM: No hierarchical relationships found!");
        println!("   This confirms the issue - all nodes are flat (no parent-child relationships)");
    } else {
        println!("\n✅ SUCCESS: Found hierarchical relationships!");
    }
    
    Ok(())
}