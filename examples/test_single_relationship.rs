//! Test Single Relationship Creation and Retrieval
//!
//! Create a simple parent-child relationship and try to retrieve it

use nodespace_core_logic::{ServiceContainer, CoreLogic};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Testing single relationship creation and retrieval...");

    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;
    println!("✅ ServiceContainer initialized\n");

    // Step 1: Create a simple parent node
    println!("📝 Creating parent node...");
    let parent_id = service_container.create_text_node(
        "## Test Parent Node", 
        "2025-06-25"
    ).await.map_err(|e| Box::new(e) as Box<dyn Error>)?;
    println!("✅ Created parent node: {}", parent_id);

    // Step 2: Create a simple child node  
    println!("\n📝 Creating child node...");
    let child_id = service_container.create_text_node(
        "**Test Child Details**: This is a child of the parent", 
        "2025-06-25"
    ).await.map_err(|e| Box::new(e) as Box<dyn Error>)?;
    println!("✅ Created child node: {}", child_id);

    // Step 3: Create the relationship
    println!("\n🔗 Creating parent-child relationship...");
    match service_container.add_child_node(&parent_id, &child_id).await {
        Ok(_) => {
            println!("✅ Successfully created relationship: {} -> {}", parent_id, child_id);
        }
        Err(e) => {
            println!("❌ Failed to create relationship: {}", e);
            return Err(Box::new(e) as Box<dyn Error>);
        }
    }

    // Step 4: Try to retrieve the child
    println!("\n🔍 Testing get_child_nodes...");
    match service_container.get_child_nodes(&parent_id).await {
        Ok(children) => {
            println!("✅ get_child_nodes returned {} children", children.len());
            for (i, child) in children.iter().enumerate() {
                println!("  Child {}: {} | Content: {:?}", i+1, child.id, child.content);
            }
            
            if children.is_empty() {
                println!("❌ PROBLEM: No children found even though we just created a relationship!");
            } else {
                println!("✅ SUCCESS: Found the child we created!");
                
                // Check if it's the right child
                if children.iter().any(|c| c.id == child_id) {
                    println!("✅ PERFECT: Found the exact child we created!");
                } else {
                    println!("⚠️  WARNING: Found children, but not the one we created");
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get children: {}", e);
        }
    }

    // Step 5: Test hierarchical view
    println!("\n🔍 Testing hierarchical view...");
    let hierarchical_nodes = service_container.get_hierarchical_nodes_for_date("2025-06-25").await
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;
    
    let mut found_parent = false;
    let mut found_child = false;
    let mut parent_has_children = false;
    
    for hn in &hierarchical_nodes {
        if hn.node.id == parent_id {
            found_parent = true;
            if !hn.children.is_empty() {
                parent_has_children = true;
                println!("✅ Parent node in hierarchical view has {} children", hn.children.len());
            }
        }
        if hn.node.id == child_id {
            found_child = true;
            if hn.parent.is_some() {
                println!("✅ Child node in hierarchical view has parent: {:?}", hn.parent);
            }
        }
    }
    
    println!("\n📊 Hierarchical Summary:");
    println!("  Found parent in hierarchical view: {}", found_parent);
    println!("  Found child in hierarchical view: {}", found_child);
    println!("  Parent has children in hierarchical view: {}", parent_has_children);
    
    if found_parent && found_child && parent_has_children {
        println!("✅ SUCCESS: Simple relationship test passed!");
    } else {
        println!("❌ FAILURE: Simple relationship test failed!");
    }
    
    Ok(())
}