//! Validate Sample Data Structure
//!
//! This example validates that the marketing sample data was created correctly
//! and can be retrieved without serialization issues.

use nodespace_core_logic::{ServiceContainer, CoreLogic};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("📊 Validating Marketing Sample Data Structure\n");

    // Use the same database path as the Tauri app and sample data
    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized\n");

    // Test basic node retrieval for recent dates
    let test_dates = [
        "2025-06-25", "2025-06-24", "2025-06-23",
        "2025-06-20", "2025-06-19", "2025-06-18",
    ];

    let mut total_nodes = 0;

    println!("🔍 Testing basic node retrieval by date:\n");

    for date in test_dates {
        match service_container.get_nodes_for_date(date).await {
            Ok(nodes) => {
                println!("   ✅ {}: {} nodes found", date, nodes.len());
                total_nodes += nodes.len();
                
                // Show first node content preview
                if let Some(first_node) = nodes.first() {
                    let content_preview = first_node.content.to_string()
                        .chars().take(80).collect::<String>();
                    println!("      Sample: {}...", content_preview);
                }
            }
            Err(e) => {
                println!("   ❌ {}: Error - {}", date, e);
            }
        }
    }

    println!("\n📈 Total nodes found: {}", total_nodes);

    // Test hierarchical data structure
    println!("\n🌳 Testing hierarchical node retrieval:\n");

    for date in test_dates.iter().take(3) {
        match service_container.get_hierarchical_nodes_for_date(date).await {
            Ok(hierar_nodes) => {
                println!("   ✅ {}: {} hierarchical nodes", date, hierar_nodes.len());
                
                for (i, hierar_node) in hierar_nodes.iter().take(2).enumerate() {
                    println!("      Node {}: {} children, depth {}", 
                           i + 1, 
                           hierar_node.children.len(),
                           hierar_node.depth_level);
                }
            }
            Err(e) => {
                println!("   ❌ {}: Hierarchical error - {}", date, e);
            }
        }
    }

    // Test individual node creation (to validate the pipeline works)
    println!("\n🧪 Testing node creation pipeline:\n");

    let test_content = "Test marketing insight: Customer feedback shows 73% satisfaction with new onboarding process.";
    let test_date = "2025-06-25";

    match service_container.create_text_node(test_content, test_date).await {
        Ok(node_id) => {
            println!("   ✅ Successfully created test node: {}", node_id);
            
            // Try to retrieve the created node
            match service_container.get_node(&node_id).await {
                Ok(Some(node)) => {
                    println!("   ✅ Successfully retrieved created node");
                    println!("      Content: {}", node.content.to_string().chars().take(60).collect::<String>());
                }
                Ok(None) => {
                    println!("   ⚠️  Node created but not found during retrieval");
                }
                Err(e) => {
                    println!("   ❌ Error retrieving created node: {}", e);
                }
            }
        }
        Err(e) => {
            println!("   ❌ Error creating test node: {}", e);
        }
    }

    println!("\n🎯 Data Validation Summary:");
    if total_nodes > 0 {
        println!("   ✅ Sample data exists and is accessible");
        println!("   ✅ Basic node retrieval works correctly");
        println!("   ✅ Database connection is functioning");
        println!("   ⚠️  Next step: Fix semantic search serialization issue");
    } else {
        println!("   ❌ No sample data found - may need to recreate");
    }

    Ok(())
}