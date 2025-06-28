use nodespace_core_logic::{NodeSpaceService, CoreLogic, DateNavigation};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Complete Metadata Analysis - Full Picture");
    
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db", 
        Some("./bundled_models"),
    ).await?;
    
    service.initialize().await.ok();
    
    // Get the date node and project node specifically
    let today = chrono::Utc::now().date_naive();
    let nodes_for_date = service.get_nodes_for_date(today).await?;
    
    // Find the date node first
    let search_results = service.semantic_search("June 27, 2025", 5).await?;
    let date_node = search_results.iter()
        .find(|r| {
            if let Some(content) = r.node.content.as_str() {
                let clean = content.trim().trim_matches('"').trim();
                clean == "# June 27, 2025"
            } else {
                false
            }
        })
        .map(|r| &r.node);
    
    if let Some(date_node) = date_node {
        println!("\n🗓️  DATE NODE - COMPLETE METADATA:");
        println!("ID: {}", date_node.id);
        println!("Content: {:?}", date_node.content.as_str().unwrap().trim().trim_matches('"'));
        println!("Complete Metadata JSON:");
        if let Some(metadata) = &date_node.metadata {
            println!("{}", serde_json::to_string_pretty(metadata)?);
        }
        
        // Find its first child (the project node)
        let project_node = nodes_for_date.iter()
            .find(|node| {
                if let Some(metadata) = &node.metadata {
                    if let Value::Object(obj) = metadata {
                        if let Some(parent_id) = obj.get("parent_id") {
                            return parent_id.as_str() == Some(&date_node.id.to_string());
                        }
                    }
                }
                false
            });
        
        if let Some(project_node) = project_node {
            println!("\n📁 PROJECT NODE - COMPLETE METADATA:");
            println!("ID: {}", project_node.id);
            println!("Content: {:?}", project_node.content.as_str().unwrap().trim().trim_matches('"'));
            println!("Complete Metadata JSON:");
            if let Some(metadata) = &project_node.metadata {
                println!("{}", serde_json::to_string_pretty(metadata)?);
            }
        }
    }
    
    println!("\n🔧 WHAT SHOULD BE THE CORRECT STRUCTURE:");
    println!("According to sample-node-entry.md:");
    println!();
    println!("DateNode: '# June 28, 2025' (type: 'date')");
    println!("└── TextNode: '# Product Launch Campaign Strategy' (type: 'text', depth: 0)");
    println!("    ├── TextNode: 'This comprehensive product...' (type: 'text', depth: 1)");
    println!("    ├── TextNode: '## Launch Overview' (type: 'text', depth: 1)");
    println!("    │   ├── TextNode: '**Product**: EcoSmart...' (type: 'text', depth: 2)");
    println!("    │   ├── TextNode: '**Launch Date**: July...' (type: 'text', depth: 2)");
    println!("    │   └── TextNode: '**Campaign Duration**...' (type: 'text', depth: 2)");
    println!("    ├── TextNode: '## Executive Summary' (type: 'text', depth: 1)");
    println!("    │   └── TextNode: 'The EcoSmart Professional...' (type: 'text', depth: 2)");
    println!("    └── ... (all other markdown hierarchy preserved)");
    println!();
    println!("ALL nodes should be type 'text' except the date header which is 'date'");
    println!("Depth corresponds to hyphen indentation level in markdown");
    
    Ok(())
}