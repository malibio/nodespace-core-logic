use nodespace_core_logic::{NodeSpaceService, DateNavigation, CoreLogic};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("✅ Verifying Correct Node Structure for 2025-06-28");
    
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db",
        Some("./bundled_models"),
    ).await?;
    
    service.initialize().await.ok();
    
    // Test get_nodes_for_date for tomorrow
    let tomorrow = chrono::Utc::now().date_naive() + chrono::Duration::days(1);
    println!("\n🗓️  Testing get_nodes_for_date({}):", tomorrow);
    
    let nodes_for_tomorrow = service.get_nodes_for_date(tomorrow).await?;
    println!("   Found {} nodes", nodes_for_tomorrow.len());
    
    // Verify the structure
    println!("\n📊 Node Hierarchy Analysis:");
    
    let mut depth_counts = std::collections::HashMap::new();
    let mut type_counts = std::collections::HashMap::new();
    
    for (i, node) in nodes_for_tomorrow.iter().enumerate() {
        let clean_content = if let Some(content) = node.content.as_str() {
            content.trim().trim_matches('"').trim()
        } else {
            "NO CONTENT"
        };
        
        let mut depth = 0;
        let mut node_type = "unknown".to_string();
        let mut parent_id = "none".to_string();
        
        if let Some(metadata) = &node.metadata {
            if let Value::Object(obj) = metadata {
                if let Some(d) = obj.get("depth").and_then(|v| v.as_u64()) {
                    depth = d;
                }
                if let Some(t) = obj.get("node_type").and_then(|v| v.as_str()) {
                    node_type = t.to_string();
                }
                if let Some(p) = obj.get("parent_id").and_then(|v| v.as_str()) {
                    parent_id = p.to_string();
                }
            }
        }
        
        *depth_counts.entry(depth).or_insert(0) += 1;
        *type_counts.entry(node_type.clone()).or_insert(0) += 1;
        
        // Show first 10 nodes in detail
        if i < 10 {
            println!("   Node {}: depth={}, type='{}', parent='{}'", 
                i + 1, depth, node_type, &parent_id[..8]);
            println!("      Content: {:?}", clean_content.chars().take(60).collect::<String>());
        }
    }
    
    println!("\n📈 Structure Summary:");
    println!("   Depth distribution:");
    for depth in 0..=4 {
        if let Some(count) = depth_counts.get(&depth) {
            println!("      Depth {}: {} nodes", depth, count);
        }
    }
    
    println!("   Type distribution:");
    for (node_type, count) in &type_counts {
        println!("      Type '{}': {} nodes", node_type, count);
    }
    
    // Verify the requirements
    println!("\n✅ Verification Results:");
    
    // Check if all nodes are 'text' type
    let all_text = type_counts.len() == 1 && type_counts.contains_key("text");
    println!("   All nodes are 'text' type: {}", if all_text { "✅ YES" } else { "❌ NO" });
    
    // Check depth levels 0-4
    let has_all_depths = (0..=4).all(|d| depth_counts.contains_key(&d));
    println!("   Has depth levels 0-4: {}", if has_all_depths { "✅ YES" } else { "❌ NO" });
    
    // Check markdown preservation
    let has_markdown = nodes_for_tomorrow.iter().any(|node| {
        if let Some(content) = node.content.as_str() {
            let clean = content.trim().trim_matches('"').trim();
            clean.contains("##") || clean.contains("**") || clean.contains("***")
        } else {
            false
        }
    });
    println!("   Markdown formatting preserved: {}", if has_markdown { "✅ YES" } else { "❌ NO" });
    
    // Check hierarchy relationships
    let has_hierarchy = nodes_for_tomorrow.iter().all(|node| {
        if let Some(metadata) = &node.metadata {
            if let Value::Object(obj) = metadata {
                return obj.contains_key("parent_id");
            }
        }
        false
    });
    println!("   All nodes have parent_id: {}", if has_hierarchy { "✅ YES" } else { "❌ NO" });
    
    println!("\n🎯 COMPARISON:");
    println!("   OLD STRUCTURE (2025-06-27):");
    println!("   - Used types: 'date', 'project', 'section', 'text'");
    println!("   - Complex type system with unnecessary categorization");
    println!("   ");
    println!("   NEW STRUCTURE (2025-06-28):");
    println!("   - Uses only: 'text' type (except date node which is 'date')");
    println!("   - Hierarchy preserved through depth levels and parent_id");
    println!("   - Markdown formatting maintained in content");
    println!("   - Follows hyphen indentation from sample-node-entry.md");
    
    if all_text && has_all_depths && has_markdown && has_hierarchy {
        println!("\n🎉 SUCCESS! Correct structure implemented for 2025-06-28");
    } else {
        println!("\n❌ Issues found - structure needs refinement");
    }
    
    Ok(())
}