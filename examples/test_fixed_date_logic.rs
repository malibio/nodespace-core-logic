use nodespace_core_logic::{DateNavigation, NodeSpaceService, CoreLogic};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing Fixed Date Logic - Show All Descendants");

    // Initialize service with production paths
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await.ok(); // Ignore AI init failures

    let today = chrono::Utc::now().date_naive();
    println!("\n1️⃣ Testing get_nodes_for_date({})...", today);

    let nodes = service.get_nodes_for_date(today).await?;
    println!("   Found {} nodes (descendants of date node)", nodes.len());

    if nodes.is_empty() {
        println!("   ❌ Still no nodes found - investigating...");
        
        // Debug: Check if date node exists
        let search_results = service.semantic_search("June 27, 2025", 10).await?;
        println!("   🔍 Searching for date node:");
        for result in search_results {
            if let Some(content) = result.node.content.as_str() {
                println!("      Node {}: \"{}\"", result.node.id, content.trim());
            }
        }
    } else {
        println!("   ✅ SUCCESS! Found descendants:");
        
        // Show all descendants with hierarchy
        for (i, node) in nodes.iter().enumerate() {
            if let Some(content) = node.content.as_str() {
                let preview = content.chars().take(60).collect::<String>();
                
                // Show parent relationship
                let parent_info = if let Some(metadata) = &node.metadata {
                    if let Some(parent_id) = metadata.get("parent_id") {
                        format!(" (parent: {})", parent_id.as_str().unwrap_or("unknown"))
                    } else {
                        " (no parent)".to_string()
                    }
                } else {
                    " (no metadata)".to_string()
                };
                
                println!("      {}. Node {}: \"{}...\" {}", 
                    i + 1, 
                    node.id, 
                    preview,
                    parent_info
                );
            }
        }
    }

    println!("\n2️⃣ Testing hierarchy depth...");
    // Count nodes by hierarchy level
    let mut level_counts = std::collections::HashMap::new();
    
    for node in &nodes {
        let level = calculate_hierarchy_level(&nodes, &node.id);
        *level_counts.entry(level).or_insert(0) += 1;
    }
    
    for (level, count) in level_counts {
        println!("   Level {}: {} nodes", level, count);
    }

    println!("\n🎯 RESULT:");
    if nodes.len() > 1 {
        println!("   ✅ Fixed! Now returning all {} descendants of the date node", nodes.len());
        println!("   ✅ Desktop app will now see the full hierarchy");
    } else if nodes.len() == 1 {
        println!("   ⚠️  Only 1 node found - might be working but limited data");
    } else {
        println!("   ❌ Still not working - needs further investigation");
    }

    Ok(())
}

/// Helper to calculate hierarchy level (0 = direct child of date node)
fn calculate_hierarchy_level(all_nodes: &[nodespace_core_types::Node], node_id: &nodespace_core_types::NodeId) -> usize {
    // Find the node
    if let Some(node) = all_nodes.iter().find(|n| n.id == *node_id) {
        if let Some(metadata) = &node.metadata {
            if let Some(parent_id) = metadata.get("parent_id") {
                if let Some(parent_str) = parent_id.as_str() {
                    // Look for parent in the descendants list
                    if let Some(parent_node_id) = all_nodes.iter()
                        .find(|n| n.id.to_string() == parent_str)
                        .map(|n| &n.id) {
                        return 1 + calculate_hierarchy_level(all_nodes, parent_node_id);
                    }
                }
            }
        }
    }
    0 // This is a direct child of the date node
}