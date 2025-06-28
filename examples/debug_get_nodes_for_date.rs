use nodespace_core_logic::{NodeSpaceService, DateNavigation, LegacyCoreLogic};
use chrono::NaiveDate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐛 Debugging get_nodes_for_date() - Why is it returning flat results?");
    
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db",
        Some("./bundled_models"),
    ).await?;
    
    service.initialize().await.ok();
    
    // Test the specific date
    let test_date = NaiveDate::from_ymd_opt(2025, 6, 27).unwrap();
    println!("\n🔍 Testing date: {}", test_date);
    
    // Get results from get_nodes_for_date
    let date_nodes = service.get_nodes_for_date(test_date).await?;
    println!("\n📊 get_nodes_for_date() returned {} nodes", date_nodes.len());
    
    if date_nodes.is_empty() {
        println!("❌ No nodes returned - investigating why...");
        
        // Let's manually debug the logic
        let all_nodes = service.search_nodes("").await?;
        println!("\n📋 Total nodes in database: {}", all_nodes.len());
        
        // Look for the date node
        let date_str = test_date.format("%Y-%m-%d").to_string();
        let date_header = format!("# {}", test_date.format("%B %-d, %Y"));
        
        println!("🔍 Looking for date node with:");
        println!("   date_str: '{}'", date_str);
        println!("   date_header: '{}'", date_header);
        
        let mut found_date_node = false;
        for (i, node) in all_nodes.iter().enumerate() {
            if let Some(content) = node.content.as_str() {
                let clean_content = content.trim().trim_matches('"').trim();
                
                if clean_content == date_header || 
                   clean_content.starts_with(&format!("# {}", date_str)) ||
                   clean_content == format!("# {}", date_str) {
                    
                    println!("\n✅ Found date node at index {}:", i);
                    println!("   ID: {}", node.id);
                    println!("   Content: '{}'", clean_content);
                    println!("   Raw content: {:?}", content);
                    found_date_node = true;
                    
                    // Now look for its children
                    let mut children_found = 0;
                    for child_node in &all_nodes {
                        if let Some(metadata) = &child_node.metadata {
                            if let Some(parent_id) = metadata.get("parent_id") {
                                if let Some(parent_str) = parent_id.as_str() {
                                    if parent_str == node.id.to_string() {
                                        children_found += 1;
                                        let child_content = if let Some(content) = child_node.content.as_str() {
                                            content.trim().trim_matches('"').trim()
                                        } else {
                                            "No content"
                                        };
                                        println!("   Child {}: {} - '{}'", 
                                               children_found, 
                                               child_node.id,
                                               child_content);
                                    }
                                }
                            }
                        }
                    }
                    
                    if children_found == 0 {
                        println!("   ⚠️  No direct children found for this date node");
                    } else {
                        println!("   📊 Found {} direct children", children_found);
                    }
                    break;
                }
            }
        }
        
        if !found_date_node {
            println!("❌ Date node not found at all!");
            
            // Show all nodes that might be date-related
            println!("\n🔍 Nodes that contain date-like content:");
            for (i, node) in all_nodes.iter().enumerate() {
                let content_str = if let Some(content) = node.content.as_str() {
                    content
                } else {
                    "No content"
                };
                let clean_content = content_str.trim().trim_matches('"').trim();
                if clean_content.contains("2025") || clean_content.contains("June") || clean_content.contains("#") {
                    println!("   {}: '{}' (raw: {:?})", i, clean_content, content_str);
                }
            }
        }
        
    } else {
        println!("\n📋 Nodes returned by get_nodes_for_date():");
        for (i, node) in date_nodes.iter().enumerate() {
            let content = node.content.as_str().unwrap_or("").trim().trim_matches('"').trim();
            let parent_info = if let Some(metadata) = &node.metadata {
                if let Some(parent_id) = metadata.get("parent_id") {
                    format!(" (parent: {})", parent_id.as_str().unwrap_or("unknown"))
                } else {
                    " (no parent)".to_string()
                }
            } else {
                " (no metadata)".to_string()
            };
            
            println!("   {}: '{}'{}", i + 1, content, parent_info);
        }
        
        // Check if we're missing any hierarchy
        println!("\n🏗️ Checking hierarchy structure:");
        let mut hierarchy_levels = std::collections::HashMap::new();
        
        for node in &date_nodes {
            let level = if let Some(metadata) = &node.metadata {
                if let Some(parent_id) = metadata.get("parent_id") {
                    // Count how deep this node is
                    let mut depth = 1;
                    let mut current_parent = parent_id.as_str().unwrap_or("");
                    
                    // Traverse up the hierarchy
                    while !current_parent.is_empty() {
                        if let Some(parent_node) = date_nodes.iter().find(|n| n.id.to_string() == current_parent) {
                            if let Some(parent_metadata) = &parent_node.metadata {
                                if let Some(grandparent_id) = parent_metadata.get("parent_id") {
                                    current_parent = grandparent_id.as_str().unwrap_or("");
                                    depth += 1;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    depth
                } else {
                    0 // Root level
                }
            } else {
                0
            };
            
            *hierarchy_levels.entry(level).or_insert(0) += 1;
        }
        
        println!("   Hierarchy levels found:");
        for (level, count) in hierarchy_levels {
            println!("     Level {}: {} nodes", level, count);
        }
    }
    
    Ok(())
}