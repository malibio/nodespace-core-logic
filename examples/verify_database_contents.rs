//! Verify Database Contents
//!
//! This script directly queries the database to see what data actually exists

use nodespace_core_logic::{ServiceContainer, CoreLogic};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Verifying Database Contents\n");

    // Use the exact same database path as both desktop app and sample creation
    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized with desktop app database path\n");

    // Check a range of dates to see what data exists
    let test_dates = [
        "2025-06-25", "2025-06-24", "2025-06-23", "2025-06-22", "2025-06-21", "2025-06-20",
        "2025-06-19", "2025-06-18", "2025-06-17", "2025-06-16", "2025-06-15",
        "2025-06-10", "2025-06-05", "2025-06-01",
        "2025-05-20", "2025-05-15", "2025-05-10", "2025-05-05", "2025-05-01",
        "2025-04-20", "2025-04-15", "2025-04-10", "2025-04-05", "2025-04-01",
        "2025-03-20", "2025-03-15", "2025-03-10",
    ];

    println!("📊 Database Content Analysis:\n");

    let mut total_nodes = 0;
    let mut dates_with_data = 0;

    for date in test_dates {
        match service_container.get_nodes_for_date(date).await {
            Ok(nodes) => {
                if nodes.len() > 0 {
                    dates_with_data += 1;
                    total_nodes += nodes.len();
                    println!("   ✅ {}: {} nodes", date, nodes.len());
                    
                    // Show sample content from first node
                    if let Some(first_node) = nodes.first() {
                        let content_preview = first_node.content.to_string()
                            .chars().take(80).collect::<String>();
                        println!("      Sample: {}...", content_preview);
                    }
                } else {
                    println!("   ⭕ {}: No data", date);
                }
            }
            Err(e) => {
                println!("   ❌ {}: Error - {}", date, e);
            }
        }
    }

    println!("\n📈 Database Summary:");
    println!("   • Total nodes found: {}", total_nodes);
    println!("   • Dates with data: {}/{}", dates_with_data, test_dates.len());
    
    if total_nodes == 0 {
        println!("   ⚠️  No sample data found - database appears empty");
        println!("   💡 Suggestion: Run marketing sample data creation script");
    } else if dates_with_data <= 1 {
        println!("   ⚠️  Very limited data - mostly test entries");
        println!("   💡 Suggestion: Re-run comprehensive marketing sample data creation");
    } else {
        println!("   ✅ Good data coverage across multiple dates");
    }

    // Test hierarchical queries as well (what desktop app uses)
    println!("\n🌳 Testing Hierarchical Queries (Desktop App Method):\n");

    let hierarchical_test_dates = ["2025-06-25", "2025-06-24", "2025-06-23"];
    
    for date in hierarchical_test_dates {
        match service_container.get_hierarchical_nodes_for_date(date).await {
            Ok(hierarchical_nodes) => {
                println!("   ✅ {}: {} hierarchical nodes", date, hierarchical_nodes.len());
                
                for (i, h_node) in hierarchical_nodes.iter().take(2).enumerate() {
                    let content_preview = h_node.node.content.to_string()
                        .chars().take(60).collect::<String>();
                    println!("      {}. Depth {}: {}...", i + 1, h_node.depth_level, content_preview);
                }
            }
            Err(e) => {
                println!("   ❌ {}: Hierarchical error - {}", date, e);
            }
        }
    }

    Ok(())
}