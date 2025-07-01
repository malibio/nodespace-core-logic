use nodespace_core_types::NodeId;
use nodespace_data_store::{DataStore, LanceDataStore};
use serde_json::Value;

/// Examine the raw data in the desktop app's LanceDB database
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Examining Desktop App's LanceDB Database\n");

    // Connect to the desktop app's database
    let db_path = "../data/lance_db/e2e_sample.db";
    let data_store = LanceDataStore::new(db_path).await?;
    println!("✅ Connected to database: {}", db_path);

    // Check for the 2025-06-28 date node (from the screenshot)
    let date_node_id = NodeId::from_string("2025-06-28".to_string());

    println!("\n📅 Looking for 2025-06-28 date node...");
    if let Some(date_node) = data_store.get_node(&date_node_id).await? {
        println!("   ✅ Found date node: {}", date_node.id);
        println!("   📄 Content: {:?}", date_node.content);
        println!("   🏷️  Metadata: {:?}", date_node.metadata);
        println!("   ⏰ Created: {}", date_node.created_at);
        println!("   🔄 Updated: {}", date_node.updated_at);
        if let Some(next) = &date_node.next_sibling {
            println!("   ➡️  Next sibling: {}", next);
        }
        // Note: previous_sibling field removed in NS-125
    } else {
        println!("   ❌ Date node not found");
    }

    // Get all nodes to examine the raw structure
    println!("\n📊 Examining all nodes in database...");

    // Use search to get all nodes (empty vector will return all)
    let all_nodes = data_store
        .search_multimodal(
            vec![0.0; 384], // dummy embedding
            vec![],         // all node types
        )
        .await?;

    println!("   📈 Total nodes found: {}", all_nodes.len());

    // Look for Product Launch Campaign Strategy nodes
    println!("\n🎯 Looking for 'Product Launch Campaign Strategy' nodes...");
    let mut campaign_nodes = Vec::new();
    let mut date_nodes = Vec::new();
    let mut other_nodes = Vec::new();

    for node in &all_nodes {
        let content_str = match &node.content {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };

        if content_str.contains("Product Launch Campaign Strategy")
            || content_str.contains("# Product Launch Campaign Strategy")
        {
            campaign_nodes.push((node, content_str));
        } else if content_str.len() == 10 && content_str.contains("-") {
            // Likely a date node (YYYY-MM-DD format)
            date_nodes.push((node, content_str));
        } else {
            other_nodes.push((node, content_str));
        }
    }

    // Display campaign strategy nodes
    if !campaign_nodes.is_empty() {
        println!(
            "   🚀 Found {} campaign strategy node(s):",
            campaign_nodes.len()
        );
        for (i, (node, content)) in campaign_nodes.iter().enumerate() {
            println!("      Node {}: ID = {}", i + 1, node.id);
            println!(
                "         Content: {}",
                content.chars().take(100).collect::<String>()
            );
            if let Some(metadata) = &node.metadata {
                println!("         Metadata: {}", metadata);
            }
            println!("         Created: {}", node.created_at);
            println!("         Next sibling: {:?}", node.next_sibling);
            // Note: previous_sibling field removed in NS-125
            println!();
        }
    }

    // Display date nodes
    if !date_nodes.is_empty() {
        println!("   📅 Found {} date node(s):", date_nodes.len());
        for (i, (node, content)) in date_nodes.iter().enumerate() {
            println!("      Date {}: ID = {} ({})", i + 1, node.id, content);
            if let Some(metadata) = &node.metadata {
                println!("         Metadata: {}", metadata);
            }
            println!("         Next sibling: {:?}", node.next_sibling);
            // Note: previous_sibling field removed in NS-125
        }
        println!();
    }

    // Show first 10 other nodes for context
    println!("   📋 Other nodes (showing first 10):");
    for (i, (node, content)) in other_nodes.iter().take(10).enumerate() {
        println!("      Node {}: ID = {}", i + 1, node.id);
        println!(
            "         Content: {}",
            content.chars().take(80).collect::<String>()
        );
        if let Some(metadata) = &node.metadata {
            println!("         Metadata: {}", metadata);
        }
        println!();
    }

    // Show database schema information
    println!("\n🏗️  Database Schema Information:");
    println!("   📊 Total nodes: {}", all_nodes.len());
    println!("   🚀 Campaign nodes: {}", campaign_nodes.len());
    println!("   📅 Date nodes: {}", date_nodes.len());
    println!("   📋 Other nodes: {}", other_nodes.len());

    // Look for the specific hierarchy from the screenshot
    println!("\n🌳 Looking for node hierarchy related to 2025-06-28...");

    // Check if any nodes have metadata indicating parent-child relationships
    for node in &all_nodes {
        if let Some(metadata) = &node.metadata {
            if metadata.to_string().contains("2025-06-28")
                || metadata.to_string().contains("parent")
                || metadata.to_string().contains("child")
            {
                println!("   🔗 Hierarchical node found:");
                println!("      ID: {}", node.id);
                println!(
                    "      Content: {}",
                    node.content
                        .to_string()
                        .chars()
                        .take(60)
                        .collect::<String>()
                );
                println!("      Metadata: {}", metadata);
                println!();
            }
        }
    }

    println!("\n✅ Database examination complete!");
    println!("\n📋 Raw Data Summary:");
    println!("   Database Path: {}", db_path);
    println!("   Total Transactions: 189+ (from file listing)");
    println!("   Total Nodes Retrieved: {}", all_nodes.len());
    println!("   Campaign Strategy Nodes: {}", campaign_nodes.len());
    println!("   Date Nodes: {}", date_nodes.len());

    Ok(())
}
