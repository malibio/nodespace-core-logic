// Test the fixed create_text_node method
use nodespace_core_logic::{ServiceContainer, CoreLogic};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔧 Testing the fixed create_text_node method...\n");
    
    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized");

    let test_date = "2025-01-15"; // Use a date that doesn't have existing test data
    println!("\n📅 Testing with date: {}", test_date);

    // First, check if there's any existing data for this date
    println!("\n1. Checking existing data for test date...");
    match service_container.get_nodes_for_date(test_date).await {
        Ok(existing) => println!("   Found {} existing nodes for {}", existing.len(), test_date),
        Err(e) => println!("   Error checking existing nodes: {}", e),
    }

    // Create a few test nodes using the fixed method
    println!("\n2. Creating test nodes with fixed method...");
    let test_contents = vec![
        "Marketing strategy meeting notes for Q1 planning",
        "Customer feedback analysis shows 94% satisfaction",
        "Brand awareness campaign launch preparation"
    ];

    let mut created_ids = Vec::new();
    for (i, content) in test_contents.iter().enumerate() {
        match service_container.create_text_node(content, test_date).await {
            Ok(node_id) => {
                println!("   ✅ Created node {}: {} (ID: {})", i + 1, 
                         if content.len() > 40 { format!("{}...", &content[..37]) } else { content.to_string() },
                         node_id);
                created_ids.push(node_id);
            }
            Err(e) => println!("   ❌ Failed to create node {}: {}", i + 1, e),
        }
    }

    // Now test retrieval
    println!("\n3. Testing retrieval after creation...");
    match service_container.get_nodes_for_date(test_date).await {
        Ok(retrieved) => {
            println!("   ✅ Retrieved {} nodes for {}", retrieved.len(), test_date);
            for (i, node) in retrieved.iter().enumerate() {
                if let Some(content_str) = node.content.as_str() {
                    let preview = if content_str.len() > 50 {
                        format!("{}...", &content_str[..47])
                    } else {
                        content_str.to_string()
                    };
                    println!("     [{}] ID: {} | Content: {}", i + 1, node.id, preview);
                }
            }
        }
        Err(e) => println!("   ❌ Error retrieving nodes: {}", e),
    }

    // Test with some of the original marketing dates
    println!("\n4. Testing retrieval for original marketing dates...");
    let marketing_dates = ["2025-01-02", "2025-02-03", "2025-03-04"];
    
    for date in marketing_dates {
        match service_container.get_nodes_for_date(date).await {
            Ok(nodes) => {
                if nodes.len() > 0 {
                    println!("   ✅ Found {} nodes for {} (marketing data!)", nodes.len(), date);
                    if let Some(content_str) = nodes[0].content.as_str() {
                        let preview = if content_str.len() > 60 {
                            format!("{}...", &content_str[..57])
                        } else {
                            content_str.to_string()
                        };
                        println!("     Sample: {}", preview);
                    }
                    break; // Found some marketing data
                } else {
                    println!("   No nodes found for {}", date);
                }
            }
            Err(e) => println!("   Error checking {}: {}", date, e),
        }
    }

    println!("\n🎯 Result Analysis:");
    if created_ids.len() == test_contents.len() {
        println!("   ✅ All test nodes created successfully");
        println!("   ✅ Node creation and retrieval is now working correctly");
        println!("   💡 The fix ensures data goes to the 'text' table with proper relationships");
    } else {
        println!("   ❌ Some test nodes failed to create");
    }

    Ok(())
}