use nodespace_data_store::{DataStore, SurrealDataStore};
use nodespace_core_types::{Node, NodeId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Querying datenode:2025-06-23 with children and grandchildren...\n");

    // Connect directly to the database
    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let data_store = SurrealDataStore::new(database_path).await?;
    
    let date_value = "2025-06-23";
    
    // Check if we have any nodes for this date
    println!("📅 Date Node: date:{}", date_value);

    // Get direct children using the DataStore trait method
    println!("\n🔗 Children of date:{}:", date_value);
    match data_store.get_nodes_for_date(date_value).await {
        Ok(children) => {
            if children.is_empty() {
                println!("  No direct children found");
            } else {
                println!("  Found {} direct children:", children.len());
                
                for (i, child_node) in children.iter().enumerate() {
                    println!("\n  📄 Child {} ({}):", i + 1, child_node.id);
                    print_node(child_node, 1);
                    
                    // Get grandchildren using the child's ID as parent date
                    println!("\n    🔗 Grandchildren of {}:", child_node.id);
                    
                    // Try to get nodes that have this child as their parent date
                    let child_id_str = child_node.id.as_str();
                    match data_store.get_nodes_for_date(child_id_str).await {
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

    // Try to get the date node specifically
    println!("\n\n🏗️ Additional information for date:{}:", date_value);
    
    let date_node_id = NodeId::from(format!("date:{}", date_value));
    match data_store.get_node(&date_node_id).await {
        Ok(Some(date_node)) => {
            println!("✅ Date node found:");
            print_node(&date_node, 0);
        }
        Ok(None) => {
            println!("❌ Date node date:{} not found in database", date_value);
        }
        Err(e) => {
            println!("❌ Error fetching date node: {}", e);
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