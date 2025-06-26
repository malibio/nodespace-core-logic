use nodespace_data_store::{DataStore, SurrealDataStore};
use nodespace_core_types::Node;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Generating summary for dates 2025-06-24 and 2025-06-25...\n");

    // Connect directly to the database
    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let data_store = SurrealDataStore::new(database_path).await?;
    
    let dates = vec!["2025-06-24", "2025-06-25"];
    let mut markdown_content = String::new();
    
    markdown_content.push_str("# NodeSpace Database Content - June 24-25, 2025\n\n");
    markdown_content.push_str("This document contains all the content stored in the NodeSpace database for June 24th and 25th, 2025.\n\n");
    
    for date_value in dates {
        markdown_content.push_str(&format!("## 📅 Date: {}\n\n", date_value));
        
        // Get all nodes for this date
        match data_store.get_nodes_for_date(date_value).await {
            Ok(nodes) => {
                if nodes.is_empty() {
                    markdown_content.push_str(&format!("*No nodes found for date: {}*\n\n", date_value));
                } else {
                    markdown_content.push_str(&format!("**Found {} nodes for this date:**\n\n", nodes.len()));
                    
                    for (i, node) in nodes.iter().enumerate() {
                        markdown_content.push_str(&format!("### {}. Node: `{}`\n\n", i + 1, node.id));
                        
                        // Add content
                        match &node.content {
                            serde_json::Value::String(s) => {
                                // If it's already markdown-like, preserve formatting
                                if s.contains("##") || s.contains("**") || s.contains("*") {
                                    markdown_content.push_str(s);
                                } else {
                                    markdown_content.push_str(&format!("```\n{}\n```", s));
                                }
                            }
                            other => {
                                markdown_content.push_str(&format!("```json\n{}\n```", serde_json::to_string_pretty(other)?));
                            }
                        }
                        
                        markdown_content.push_str("\n\n");
                        
                        // Add metadata
                        markdown_content.push_str(&format!("**Metadata:**\n"));
                        markdown_content.push_str(&format!("- Created: {}\n", node.created_at));
                        markdown_content.push_str(&format!("- Updated: {}\n", node.updated_at));
                        if let Some(metadata) = &node.metadata {
                            markdown_content.push_str(&format!("- Metadata: {}\n", metadata));
                        }
                        if let Some(next_sibling) = &node.next_sibling {
                            markdown_content.push_str(&format!("- Next Sibling: {}\n", next_sibling));
                        }
                        if let Some(prev_sibling) = &node.previous_sibling {
                            markdown_content.push_str(&format!("- Previous Sibling: {}\n", prev_sibling));
                        }
                        
                        markdown_content.push_str("\n---\n\n");
                    }
                }
            }
            Err(e) => {
                markdown_content.push_str(&format!("*Error fetching nodes for {}: {}*\n\n", date_value, e));
            }
        }
    }
    
    // Write to file
    let output_path = "/Users/malibio/nodespace/nodespace-core-logic/nodespace_content_june_24_25.md";
    let mut file = File::create(output_path)?;
    file.write_all(markdown_content.as_bytes())?;
    
    println!("✅ Markdown file created at: {}", output_path);
    println!("📊 Summary:");
    
    // Print summary
    for date_value in vec!["2025-06-24", "2025-06-25"] {
        match data_store.get_nodes_for_date(date_value).await {
            Ok(nodes) => {
                println!("  - {}: {} nodes", date_value, nodes.len());
            }
            Err(e) => {
                println!("  - {}: Error - {}", date_value, e);
            }
        }
    }
    
    Ok(())
}