use nodespace_core_logic::{NodeSpaceService, CoreLogic};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗄️ Raw LanceDB Data - Complete Field Analysis");
    
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db",
        Some("./bundled_models"),
    ).await?;
    
    service.initialize().await.ok();
    
    // Get all nodes to find our target nodes
    let results = service.semantic_search("", 50).await?;
    
    // Find the date node and project node specifically
    let date_node = results.iter()
        .find(|r| {
            if let Some(content) = r.node.content.as_str() {
                let clean = content.trim().trim_matches('"').trim();
                clean == "# June 27, 2025"
            } else { false }
        })
        .map(|r| &r.node);
    
    let project_node = results.iter()
        .find(|r| {
            if let Some(content) = r.node.content.as_str() {
                let clean = content.trim().trim_matches('"').trim();
                clean == "# Product Launch Campaign Strategy"
            } else { false }
        })
        .map(|r| &r.node);
    
    if let Some(date_node) = date_node {
        println!("\n{}", "=".repeat(80));
        println!("NODE 1: DATE NODE - RAW LANCEDB DATA");
        println!("{}", "=".repeat(80));
        
        show_complete_node_data(date_node, "DATE NODE")?;
    }
    
    if let Some(project_node) = project_node {
        println!("\n{}", "=".repeat(80));
        println!("NODE 2: PROJECT NODE - RAW LANCEDB DATA");
        println!("{}", "=".repeat(80));
        
        show_complete_node_data(project_node, "PROJECT NODE")?;
    }
    
    println!("\n🔍 LANCEDB COLUMN MAPPING:");
    println!("   LanceDB stores these as individual Arrow columns:");
    println!("   - id: String column");
    println!("   - node_type: String column (extracted from metadata)");
    println!("   - content: String column (with double-encoded JSON quotes)");
    println!("   - parent_id: String column (extracted from metadata)");
    println!("   - children_ids: List[String] column");
    println!("   - mentions: List[String] column");
    println!("   - created_at: String column (ISO 8601)");
    println!("   - updated_at: String column (ISO 8601)"); 
    println!("   - metadata: String column (full JSON)");
    println!("   - vector: FixedSizeList[Float32] column (384 dimensions)");
    
    Ok(())
}

fn show_complete_node_data(node: &nodespace_core_types::Node, label: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 {} - COMPLETE RAW DATA:", label);
    
    // Show each field as it would appear in LanceDB columns
    println!("\n📊 LanceDB Column Values:");
    
    // ID column
    println!("   [id]: {:?}", node.id.to_string());
    
    // Content column (this has the double-encoding issue)
    if let Some(content) = node.content.as_str() {
        println!("   [content]: {:?}", content);
        println!("      Length: {} characters", content.len());
        println!("      Raw bytes: {:?}", content.as_bytes());
    }
    
    // Extract node_type from metadata for LanceDB column
    let node_type = if let Some(metadata) = &node.metadata {
        if let Value::Object(obj) = metadata {
            obj.get("node_type")
                .and_then(|v| v.as_str())
                .unwrap_or("text")
        } else { "text" }
    } else { "text" };
    println!("   [node_type]: {:?}", node_type);
    
    // Extract parent_id from metadata for LanceDB column
    let parent_id = if let Some(metadata) = &node.metadata {
        if let Value::Object(obj) = metadata {
            obj.get("parent_id")
                .and_then(|v| v.as_str())
        } else { None }
    } else { None };
    println!("   [parent_id]: {:?}", parent_id);
    
    // Children IDs (would be extracted/computed)
    println!("   [children_ids]: [] (empty array - would be computed)");
    
    // Mentions (would be extracted from metadata or content)
    println!("   [mentions]: [] (empty array - would be extracted)");
    
    // Timestamps
    println!("   [created_at]: {:?}", node.created_at);
    println!("   [updated_at]: {:?}", node.updated_at);
    
    // Metadata column (full JSON)
    if let Some(metadata) = &node.metadata {
        println!("   [metadata]: {:?}", serde_json::to_string(metadata)?);
    } else {
        println!("   [metadata]: null");
    }
    
    // Vector column (would be generated/stored)
    println!("   [vector]: [0.0, 0.0, 0.0, ... ] (384 float32 values - embeddings)");
    
    // Sibling relationships (from Node struct, not in LanceDB)
    println!("\n📎 Node Struct Fields (not in LanceDB columns):");
    println!("   next_sibling: {:?}", node.next_sibling);
    println!("   previous_sibling: {:?}", node.previous_sibling);
    
    // Detailed metadata breakdown
    if let Some(metadata) = &node.metadata {
        println!("\n📋 Metadata JSON Breakdown:");
        if let Value::Object(obj) = metadata {
            for (key, value) in obj {
                println!("      \"{}\": {:?}", key, value);
                
                // Show the data type and any special encoding
                match value {
                    Value::String(s) => {
                        println!("         Type: String, Length: {}", s.len());
                        if s.len() < 100 {
                            println!("         Raw: {:?}", s.as_bytes());
                        }
                    }
                    Value::Number(n) => {
                        println!("         Type: Number, Value: {}", n);
                    }
                    _ => {
                        println!("         Type: {:?}", 
                            match value {
                                Value::Bool(_) => "Boolean",
                                Value::Array(_) => "Array", 
                                Value::Object(_) => "Object",
                                Value::Null => "Null",
                                _ => "Unknown"
                            }
                        );
                    }
                }
            }
        }
    }
    
    // Show the complete Node struct serialized
    println!("\n📝 Complete Node Struct (as JSON):");
    println!("{}", serde_json::to_string_pretty(node)?);
    
    Ok(())
}