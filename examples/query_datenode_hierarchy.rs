use nodespace_core_logic::{CoreLogic, ServiceContainer};
use nodespace_core_types::{Node, NodeId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Querying datenode:2025-06-23 with children and grandchildren...\n");

    // Initialize service container
    let mut service_container = ServiceContainer::new().await?;
    
    let date_node_id = NodeId::from("date:2025-06-23");
    
    // First, get the date node itself
    println!("📅 Date Node: {}", date_node_id);
    match service_container.get_node(&date_node_id).await {
        Ok(Some(date_node)) => {
            print_node(&date_node, 0);
        }
        Ok(None) => {
            println!("❌ Date node {} not found", date_node_id);
            return Ok(());
        }
        Err(e) => {
            println!("❌ Error fetching date node: {}", e);
            return Ok(());
        }
    }

    // Get direct children of the date node
    println!("\n🔗 Children of {}:", date_node_id);
    match service_container.get_ordered_children(&date_node_id).await {
        Ok(children) => {
            if children.is_empty() {
                println!("  No direct children found");
            } else {
                println!("  Found {} direct children:", children.len());
                
                for (i, child_node) in children.iter().enumerate() {
                    println!("\n  📄 Child {} ({}):", i + 1, child_node.id);
                    
                    // Print the child node
                    print_node(child_node, 1);
                    
                    // Get grandchildren
                    println!("\n    🔗 Grandchildren of {}:", child_node.id);
                    match service_container.get_ordered_children(&child_node.id).await {
                        Ok(grandchildren) => {
                            if grandchildren.is_empty() {
                                println!("      No grandchildren found");
                            } else {
                                println!("      Found {} grandchildren:", grandchildren.len());
                                
                                for (j, grandchild_node) in grandchildren.iter().enumerate() {
                                    println!("\n      📝 Grandchild {} ({}):", j + 1, grandchild_node.id);
                                    print_node(grandchild_node, 2);
                                }
                            }
                        }
                        Err(e) => {
                            println!("      ❌ Error fetching grandchildren: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Error fetching children: {}", e);
        }
    }

    // Also try to get hierarchical data using the core logic method
    println!("\n\n🏗️ Hierarchical structure for {}:", date_node_id);
    match service_container.get_hierarchical_node(&date_node_id).await {
        Ok(Some(hierarchical_node)) => {
            println!("📊 Hierarchical Node Details:");
            println!("  - Depth Level: {}", hierarchical_node.depth_level);
            println!("  - Order in Parent: {}", hierarchical_node.order_in_parent);
            println!("  - Relationship Type: {:?}", hierarchical_node.relationship_type);
            println!("  - Children Count: {}", hierarchical_node.children.len());
            println!("  - Parent: {:?}", hierarchical_node.parent);
            
            if !hierarchical_node.children.is_empty() {
                println!("\n  📋 Child IDs:");
                for (i, child_id) in hierarchical_node.children.iter().enumerate() {
                    println!("    {}. {}", i + 1, child_id);
                }
            }
        }
        Ok(None) => {
            println!("❌ Hierarchical node data not found");
        }
        Err(e) => {
            println!("❌ Error fetching hierarchical node: {}", e);
        }
    }

    println!("\n✅ Query completed!");
    Ok(())
}

fn print_node(node: &Node, indent_level: usize) {
    let indent = "  ".repeat(indent_level);
    
    println!("{}📋 Node ID: {}", indent, node.id);
    println!("{}🔤 Content: {}", indent, format_content(&node.content));
    
    if let Some(metadata) = &node.metadata {
        println!("{}📝 Metadata: {}", indent, metadata);
    }
    
    println!("{}⏰ Created: {}", indent, node.created_at);
    println!("{}🔄 Updated: {}", indent, node.updated_at);
    
    if let Some(next_sibling) = &node.next_sibling {
        println!("{}➡️  Next Sibling: {}", indent, next_sibling);
    }
    
    if let Some(prev_sibling) = &node.previous_sibling {
        println!("{}⬅️  Previous Sibling: {}", indent, prev_sibling);
    }
}

fn format_content(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => {
            if s.len() > 100 {
                format!("{}...", &s[..100])
            } else {
                s.clone()
            }
        }
        other => {
            let formatted = serde_json::to_string_pretty(other).unwrap_or_else(|_| "Invalid JSON".to_string());
            if formatted.len() > 200 {
                format!("{}...", &formatted[..200])
            } else {
                formatted
            }
        }
    }
}