// Debug script to identify the data storage/query mismatch issue
use nodespace_core_logic::{ServiceContainer, CoreLogic};
use nodespace_data_store::DataStore;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Debugging data storage/query mismatch issue...\n");
    
    // Use the same database path as the marketing sample data
    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized");

    let test_date = "2025-06-23";
    println!("\n📅 Testing data storage and retrieval for date: {}", test_date);

    // Check what's in the nodes table
    println!("\n1. Checking nodes table...");
    let nodes_query = "SELECT * FROM nodes WHERE metadata.parent_date = $date";
    match service_container.data_store().query_nodes(nodes_query).await {
        Ok(nodes) => {
            println!("   Found {} nodes in 'nodes' table with parent_date metadata", nodes.len());
            for (i, node) in nodes.iter().take(3).enumerate() {
                if let Some(content_str) = node.content.as_str() {
                    let preview = if content_str.len() > 60 {
                        format!("{}...", &content_str[..57])
                    } else {
                        content_str.to_string()
                    };
                    println!("     [{}] ID: {} | Content: {}", i + 1, node.id, preview);
                }
            }
        }
        Err(e) => println!("   Error querying nodes table: {}", e),
    }

    // Check what's in the text table
    println!("\n2. Checking text table...");
    let text_query = "SELECT * FROM text WHERE parent_date = $date";
    match service_container.data_store().query_nodes(text_query).await {
        Ok(text_nodes) => {
            println!("   Found {} nodes in 'text' table with parent_date field", text_nodes.len());
            for (i, node) in text_nodes.iter().take(3).enumerate() {
                if let Some(content_str) = node.content.as_str() {
                    let preview = if content_str.len() > 60 {
                        format!("{}...", &content_str[..57])
                    } else {
                        content_str.to_string()
                    };
                    println!("     [{}] ID: {} | Content: {}", i + 1, node.id, preview);
                }
            }
        }
        Err(e) => println!("   Error querying text table: {}", e),
    }

    // Check date node relationships
    println!("\n3. Checking date node relationships...");
    let date_thing = format!("date:`{}`", test_date);
    let relationship_query = format!("SELECT * FROM {}->contains", date_thing);
    match service_container.data_store().query_nodes(&relationship_query).await {
        Ok(relationships) => {
            println!("   Found {} 'contains' relationships from date node", relationships.len());
            for (i, rel) in relationships.iter().take(3).enumerate() {
                println!("     [{}] Relationship: {:?}", i + 1, rel.content);
            }
        }
        Err(e) => println!("   Error querying date relationships: {}", e),
    }

    // Check what get_nodes_for_date returns
    println!("\n4. Testing get_nodes_for_date method...");
    match service_container.get_nodes_for_date(test_date).await {
        Ok(retrieved_nodes) => {
            println!("   get_nodes_for_date returned {} nodes", retrieved_nodes.len());
            for (i, node) in retrieved_nodes.iter().take(3).enumerate() {
                if let Some(content_str) = node.content.as_str() {
                    let preview = if content_str.len() > 60 {
                        format!("{}...", &content_str[..57])
                    } else {
                        content_str.to_string()
                    };
                    println!("     [{}] ID: {} | Content: {}", i + 1, node.id, preview);
                }
            }
        }
        Err(e) => println!("   Error with get_nodes_for_date: {}", e),
    }

    // Test the data store's direct create_text_node method vs ServiceContainer method
    println!("\n5. Testing data creation methods...");
    
    // Test ServiceContainer method (this goes to nodes table)
    println!("   Creating node via ServiceContainer.create_text_node...");
    match service_container.create_text_node("Test content from ServiceContainer", test_date).await {
        Ok(node_id) => println!("     ✅ Created node: {}", node_id),
        Err(e) => println!("     ❌ Failed: {}", e),
    }

    // Test data store direct method (this goes to text table)  
    println!("   Creating node via DataStore.create_text_node...");
    match service_container.data_store().create_text_node("Test content from DataStore", Some(test_date)).await {
        Ok(node_id) => println!("     ✅ Created node: {}", node_id),
        Err(e) => println!("     ❌ Failed: {}", e),
    }

    // Now test retrieval again
    println!("\n6. Testing retrieval after both creation methods...");
    match service_container.get_nodes_for_date(test_date).await {
        Ok(retrieved_nodes) => {
            println!("   get_nodes_for_date now returns {} nodes", retrieved_nodes.len());
            for (i, node) in retrieved_nodes.iter().take(5).enumerate() {
                if let Some(content_str) = node.content.as_str() {
                    let preview = if content_str.len() > 60 {
                        format!("{}...", &content_str[..57])
                    } else {
                        content_str.to_string()
                    };
                    println!("     [{}] ID: {} | Content: {}", i + 1, node.id, preview);
                }
            }
        }
        Err(e) => println!("   Error with get_nodes_for_date: {}", e),
    }

    println!("\n🔍 Analysis complete!");
    println!("\n💡 Expected finding: ServiceContainer.create_text_node stores in 'nodes' table,");
    println!("   but get_nodes_for_date queries 'text' table via relationships.");
    println!("   The marketing sample data is in 'nodes' table, not 'text' table.");

    Ok(())
}