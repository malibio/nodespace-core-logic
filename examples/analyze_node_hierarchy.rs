use nodespace_core_logic::{NodeSpaceService, CoreLogic, DateNavigation};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Node Hierarchy and Type Analysis");
    
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db",
        Some("./bundled_models"),
    ).await?;
    
    service.initialize().await.ok();
    
    // Get all nodes to analyze the full hierarchy
    let results = service.semantic_search("", 50).await?;
    
    println!("\n📊 Full Database Node Analysis:");
    println!("Total nodes in database: {}", results.len());
    
    // Find all date nodes first
    println!("\n🗓️  DATE NODES ANALYSIS:");
    let mut date_nodes = Vec::new();
    
    for result in &results {
        let node = &result.node;
        let clean_content = if let Some(content) = node.content.as_str() {
            content.trim().trim_matches('"').trim()
        } else {
            continue;
        };
        
        // Check if this looks like a date node
        let is_date_node = clean_content.starts_with("# ") && 
            (clean_content.contains("2025") || clean_content.contains("June") || clean_content.contains("27"));
        
        if is_date_node {
            date_nodes.push(node);
            println!("\n✅ FOUND DATE NODE:");
            println!("   ID: {}", node.id);
            println!("   Content: {:?}", clean_content);
            
            // Check metadata for node type
            if let Some(metadata) = &node.metadata {
                println!("   Metadata: {}", serde_json::to_string_pretty(metadata)?);
                
                if let Value::Object(obj) = metadata {
                    if let Some(node_type) = obj.get("node_type") {
                        println!("   Type: {:?}", node_type);
                    }
                    if let Some(date) = obj.get("date") {
                        println!("   Date field: {:?}", date);
                    }
                    if let Some(parent_id) = obj.get("parent_id") {
                        println!("   Parent ID: {:?}", parent_id);
                    } else {
                        println!("   ✅ NO PARENT (top-level date node)");
                    }
                }
            }
        }
    }
    
    if date_nodes.is_empty() {
        println!("❌ NO DATE NODES FOUND!");
        return Ok(());
    }
    
    println!("\n📈 HIERARCHY ANALYSIS for each date node:");
    
    for date_node in &date_nodes {
        let date_node_id = &date_node.id;
        println!("\n🎯 Analyzing children of date node: {}", date_node_id);
        
        // Find direct children
        let mut direct_children = Vec::new();
        let mut all_descendants = Vec::new();
        
        for result in &results {
            let node = &result.node;
            
            if let Some(metadata) = &node.metadata {
                if let Value::Object(obj) = metadata {
                    if let Some(parent_id) = obj.get("parent_id") {
                        if let Some(parent_str) = parent_id.as_str() {
                            if parent_str == date_node_id.to_string() {
                                direct_children.push(node);
                                all_descendants.push(node);
                            }
                        }
                    }
                }
            }
        }
        
        println!("   Direct children: {}", direct_children.len());
        
        for (i, child) in direct_children.iter().enumerate() {
            let clean_content = if let Some(content) = child.content.as_str() {
                content.trim().trim_matches('"').trim()
            } else {
                "NO CONTENT"
            };
            
            println!("   Child {}: {} -> {:?}", 
                i + 1, 
                child.id, 
                clean_content.chars().take(50).collect::<String>()
            );
            
            // Check child's metadata
            if let Some(metadata) = &child.metadata {
                if let Value::Object(obj) = metadata {
                    if let Some(node_type) = obj.get("node_type") {
                        println!("      Type: {:?}", node_type);
                    }
                    if let Some(depth) = obj.get("depth") {
                        println!("      Depth: {:?}", depth);
                    }
                }
            }
        }
        
        // Now find ALL descendants (recursive)
        let mut to_process = direct_children.clone();
        while let Some(current_node) = to_process.pop() {
            // Find children of this node
            for result in &results {
                let node = &result.node;
                
                if let Some(metadata) = &node.metadata {
                    if let Value::Object(obj) = metadata {
                        if let Some(parent_id) = obj.get("parent_id") {
                            if let Some(parent_str) = parent_id.as_str() {
                                if parent_str == current_node.id.to_string() {
                                    if !all_descendants.iter().any(|n| n.id == node.id) {
                                        all_descendants.push(node);
                                        to_process.push(node);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        println!("   Total descendants (all levels): {}", all_descendants.len());
    }
    
    // Now test what get_nodes_for_date returns
    println!("\n🧪 TESTING get_nodes_for_date() function:");
    let today = chrono::Utc::now().date_naive();
    let nodes_for_date = service.get_nodes_for_date(today).await?;
    
    println!("   Date queried: {}", today);
    println!("   Nodes returned by get_nodes_for_date(): {}", nodes_for_date.len());
    
    // Analyze what was returned
    println!("\n🔍 ANALYSIS OF RETURNED NODES:");
    for (i, node) in nodes_for_date.iter().enumerate().take(5) {
        let clean_content = if let Some(content) = node.content.as_str() {
            content.trim().trim_matches('"').trim()
        } else {
            "NO CONTENT"
        };
        
        println!("   Node {}: {}", i + 1, node.id);
        println!("      Content: {:?}", clean_content.chars().take(60).collect::<String>());
        
        if let Some(metadata) = &node.metadata {
            if let Value::Object(obj) = metadata {
                if let Some(node_type) = obj.get("node_type") {
                    println!("      Type: {:?}", node_type);
                }
                if let Some(parent_id) = obj.get("parent_id") {
                    println!("      Parent: {:?}", parent_id);
                } else {
                    println!("      Parent: NONE (top-level)");
                }
                if let Some(depth) = obj.get("depth") {
                    println!("      Depth: {:?}", depth);
                }
            }
        }
    }
    
    println!("\n🎯 SUMMARY:");
    println!("   - Date nodes found: {}", date_nodes.len());
    println!("   - get_nodes_for_date() returns: {} nodes", nodes_for_date.len());
    println!("   - This should include ALL descendants of the date node for today");
    
    if nodes_for_date.len() > 0 {
        println!("   ✅ Function is working - returning descendants");
    } else {
        println!("   ❌ Function not working - no descendants returned");
    }
    
    Ok(())
}