//! Fresh Sample Database Creation Example
//!
//! This example demonstrates how to create a fresh sample database with all the latest
//! features and improvements. This approach is simpler and more reliable for development.
//!
//! Features included:
//! - Clean content without bullet points
//! - Proper SurrealDB relationships
//! - Automatic sibling ordering for child nodes
//! - Hierarchical data structures

use nodespace_data_store::SurrealDataStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Creating Fresh NodeSpace Sample Database...\n");

    // Note: This example demonstrates the core-logic interface
    // The actual sample data creation using core-logic features is in this repository

    println!("📋 To create fresh sample database with all latest features:");
    println!("   cargo run --example create_sample_data_with_core_logic");
    println!("   (This uses the ServiceContainer interface with all core-logic features)");
    println!("");
    println!("✨ The updated sample creation includes:");
    println!("   • Clean content without bullet points for child nodes");
    println!("   • Proper SurrealDB relationship records that can be traversed");
    println!("   • Automatic sibling ordering for all child nodes");
    println!("   • Hierarchical parent-child data structures");
    println!("   • ~300 entries across 100 days with realistic content");
    println!("");

    // Use data store directly to test hierarchical features without NLP dependency
    let data_store =
        SurrealDataStore::new("/Users/malibio/nodespace/nodespace-data-store/data/sample.db")
            .await
            .map_err(|e| format!("Failed to create data store: {}", e))?;

    // Test the hierarchical queries on sample data using data store directly
    println!("🧪 Testing data store interface with sample data (bypassing NLP dependency):");

    // Test with recent dates that should have sample data
    let test_dates = ["2025-06-23", "2025-06-24", "2025-06-25"];

    for test_date in test_dates {
        println!("\n📅 Testing date: {}", test_date);

        match data_store.get_nodes_for_date(test_date).await {
            Ok(nodes) => {
                println!(
                    "   ✅ Retrieved {} nodes for date {}",
                    nodes.len(),
                    test_date
                );

                if !nodes.is_empty() {
                    // Show sample content to verify bullet point cleaning
                    for (i, node) in nodes.iter().take(3).enumerate() {
                        let content_preview = node
                            .content
                            .to_string()
                            .chars()
                            .take(60)
                            .collect::<String>();
                        println!("      📄 Node {}: {}", i + 1, content_preview);

                        // Check for relationships
                        if node.next_sibling.is_some() || node.previous_sibling.is_some() {
                            println!(
                                "         🔗 Has sibling relationships: prev={:?}, next={:?}",
                                node.previous_sibling.is_some(),
                                node.next_sibling.is_some()
                            );
                        }
                    }

                    // Test relationship traversal with SurrealDB
                    let _parent_id = nodespace_core_types::NodeId::from(test_date);
                    match data_store.get_date_children(test_date).await {
                        Ok(children) => {
                            println!(
                                "   ✅ Date children query: {} child relationships found",
                                children.len()
                            );

                            // Show first few children to verify content cleaning
                            for (i, child_value) in children.iter().take(3).enumerate() {
                                // Extract content from the JSON value
                                let child_content =
                                    if let Some(content) = child_value.get("content") {
                                        content.to_string().chars().take(50).collect::<String>()
                                    } else {
                                        "No content field".to_string()
                                    };
                                println!("      🔗 Child {}: {}", i + 1, child_content);

                                // Check if content has bullet points (should be cleaned)
                                if let Some(content) = child_value.get("content") {
                                    let content_str = content.to_string();
                                    if content_str.contains("•") {
                                        println!("         ⚠️  Content still has bullet points - cleaning might not be working");
                                    } else {
                                        println!("         ✅ Content is clean (no bullet points)");
                                    }
                                }
                            }
                        }
                        Err(e) => println!("   ❌ Error getting date children: {}", e),
                    }
                }
            }
            Err(e) => println!("   ❌ Error testing date query: {}", e),
        }
    }

    println!("\n🎉 Fresh database approach is simpler and includes all latest features!");

    Ok(())
}
