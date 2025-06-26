use nodespace_data_store::{DataStore, SurrealDataStore};
use nodespace_core_types::Node;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Querying multiple dates: 2025-06-24 and 2025-06-25...\n");

    // Connect directly to the database
    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let data_store = SurrealDataStore::new(database_path).await?;
    
    let dates = vec!["2025-06-24", "2025-06-25"];
    
    for date_value in dates {
        println!("📅 Date: {}", date_value);
        println!("{}", "=".repeat(50));
        
        // Get all nodes for this date
        match data_store.get_nodes_for_date(date_value).await {
            Ok(nodes) => {
                if nodes.is_empty() {
                    println!("  No nodes found for date: {}\n", date_value);
                } else {
                    println!("  Found {} nodes for date: {}", nodes.len(), date_value);
                    
                    for (i, node) in nodes.iter().enumerate() {
                        println!("\n  📄 Node {} ({}):", i + 1, node.id);
                        print_node_detailed(node, 1);
                        
                        // Check for children of this node
                        let node_id_str = node.id.as_str();
                        match data_store.get_nodes_for_date(node_id_str).await {
                            Ok(children) => {
                                if !children.is_empty() {
                                    println!("\n    🔗 Children ({}):", children.len());
                                    for (j, child) in children.iter().enumerate() {
                                        println!("\n      📝 Child {} ({}):", j + 1, child.id);
                                        print_node_detailed(child, 2);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("      ❌ Error checking children: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Error fetching nodes for {}: {}", date_value, e);
            }
        }
        
        println!("\n{}\n", "=".repeat(50));
    }

    println!("✅ Query completed!");
    Ok(())
}

fn print_node_detailed(node: &Node, indent_level: usize) {
    let indent = "  ".repeat(indent_level);
    
    println!("{}📋 ID: {}", indent, node.id);
    
    // Print full content
    match &node.content {
        serde_json::Value::String(s) => {
            println!("{}🔤 Content:", indent);
            for line in s.lines() {
                println!("{}   {}", indent, line);
            }
        }
        other => {
            println!("{}🔤 Content: {}", indent, serde_json::to_string_pretty(other).unwrap_or_else(|_| "Invalid JSON".to_string()));
        }
    }
    
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