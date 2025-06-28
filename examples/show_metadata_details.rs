use nodespace_core_logic::{NodeSpaceService, CoreLogic};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Detailed Metadata Analysis - Where 'type' comes from");
    
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db",
        Some("./bundled_models"),
    ).await?;
    
    service.initialize().await.ok();
    
    let results = service.semantic_search("", 10).await?;
    
    println!("\n📊 Raw Metadata Inspection:");
    
    for (i, result) in results.iter().enumerate().take(5) {
        let node = &result.node;
        let clean_content = if let Some(content) = node.content.as_str() {
            content.trim().trim_matches('"').trim()
        } else {
            "NO CONTENT"
        };
        
        println!("\n{}", "=".repeat(70));
        println!("Node {}: {}", i + 1, node.id);
        println!("Content: {:?}", clean_content.chars().take(50).collect::<String>());
        println!("{}", "=".repeat(70));
        
        // Show the EXACT metadata structure
        println!("🔍 Raw Metadata Field:");
        println!("  Type: {:?}", node.metadata);
        
        if let Some(metadata) = &node.metadata {
            println!("\n📋 Metadata Breakdown:");
            
            // Show the complete JSON structure
            println!("  Complete JSON: {}", serde_json::to_string_pretty(metadata)?);
            
            // Parse and show each field
            if let Value::Object(obj) = metadata {
                println!("\n🔎 Individual Fields:");
                for (key, value) in obj {
                    println!("    '{}': {:?} (JSON type: {})", 
                        key, 
                        value,
                        match value {
                            Value::String(_) => "String",
                            Value::Number(_) => "Number", 
                            Value::Bool(_) => "Boolean",
                            Value::Array(_) => "Array",
                            Value::Object(_) => "Object",
                            Value::Null => "Null",
                        }
                    );
                    
                    // Special focus on node_type field
                    if key == "node_type" {
                        println!("      ⭐ THIS IS THE 'TYPE' I REFERENCED!");
                        if let Value::String(type_str) = value {
                            println!("      ⭐ Value: '{}'", type_str);
                        }
                    }
                }
            } else {
                println!("  ❌ Metadata is not a JSON object!");
            }
        } else {
            println!("  ❌ No metadata found for this node");
        }
        
        println!("\n💡 Summary for this node:");
        if let Some(metadata) = &node.metadata {
            if let Value::Object(obj) = metadata {
                if let Some(node_type) = obj.get("node_type") {
                    if let Value::String(type_str) = node_type {
                        println!("    ✅ Has 'node_type': '{}'", type_str);
                        println!("    ✅ This is stored as metadata.node_type in the database");
                    }
                } else {
                    println!("    ❌ No 'node_type' field in metadata");
                }
                
                if let Some(parent_id) = obj.get("parent_id") {
                    println!("    ✅ Has parent_id: {:?}", parent_id);
                } else {
                    println!("    ⭐ No parent_id (top-level node)");
                }
                
                if let Some(depth) = obj.get("depth") {
                    println!("    ✅ Has depth: {:?}", depth);
                }
                
                if let Some(order) = obj.get("order") {
                    println!("    ✅ Has order: {:?}", order);
                }
            }
        }
    }
    
    println!("\n🎯 CONCLUSION:");
    println!("   The 'type' information comes from the 'node_type' field in metadata");
    println!("   This is actual data stored in the database, not inferred");
    println!("   Each node has structured metadata with fields like:");
    println!("   - node_type: 'date', 'project', 'section', 'text', etc.");
    println!("   - parent_id: Links to parent node");
    println!("   - depth: Hierarchy level");
    println!("   - order: Position among siblings");
    println!("   - Other custom fields depending on the node");
    
    Ok(())
}