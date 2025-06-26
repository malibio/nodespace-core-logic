// Check which tables contain the marketing sample data
use nodespace_core_logic::ServiceContainer;
use nodespace_data_store::DataStore;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Checking which tables contain the marketing sample data...\n");
    
    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized");

    // Check nodes table
    println!("\n1. Checking 'nodes' table...");
    let nodes_query = "SELECT * FROM nodes LIMIT 5";
    match service_container.data_store().query_nodes(nodes_query).await {
        Ok(nodes) => {
            println!("   Found {} nodes in 'nodes' table", nodes.len());
            for (i, node) in nodes.iter().take(3).enumerate() {
                if let Some(content_str) = node.content.as_str() {
                    let preview = if content_str.len() > 80 {
                        format!("{}...", &content_str[..77])
                    } else {
                        content_str.to_string()
                    };
                    println!("     [{}] Content: {}", i + 1, preview);
                    
                    if let Some(metadata) = &node.metadata {
                        if let Some(parent_date) = metadata.get("parent_date") {
                            println!("         Parent date: {}", parent_date);
                        }
                    }
                }
            }
        }
        Err(e) => println!("   Error querying nodes table: {}", e),
    }

    // Check text table  
    println!("\n2. Checking 'text' table...");
    let text_query = "SELECT * FROM text LIMIT 5";
    match service_container.data_store().query_nodes(text_query).await {
        Ok(text_nodes) => {
            println!("   Found {} nodes in 'text' table", text_nodes.len());
            for (i, node) in text_nodes.iter().take(3).enumerate() {
                if let Some(content_str) = node.content.as_str() {
                    let preview = if content_str.len() > 80 {
                        format!("{}...", &content_str[..77])
                    } else {
                        content_str.to_string()
                    };
                    println!("     [{}] Content: {}", i + 1, preview);
                }
            }
        }
        Err(e) => println!("   Error querying text table: {}", e),
    }

    // Check for marketing content specifically
    println!("\n3. Searching for marketing content patterns...");
    
    // Search in nodes table for marketing content
    let marketing_nodes_query = "SELECT * FROM nodes WHERE content CONTAINS 'marketing' OR content CONTAINS 'campaign' LIMIT 3";
    match service_container.data_store().query_nodes(marketing_nodes_query).await {
        Ok(marketing_nodes) => {
            println!("   Found {} marketing-related nodes in 'nodes' table", marketing_nodes.len());
            for (i, node) in marketing_nodes.iter().enumerate() {
                if let Some(content_str) = node.content.as_str() {
                    let preview = if content_str.len() > 60 {
                        format!("{}...", &content_str[..57])
                    } else {
                        content_str.to_string()
                    };
                    println!("     [{}] {}", i + 1, preview);
                }
            }
        }
        Err(e) => println!("   Error searching marketing nodes: {}", e),
    }

    // Search in text table for marketing content
    let marketing_text_query = "SELECT * FROM text WHERE content CONTAINS 'marketing' OR content CONTAINS 'campaign' LIMIT 3";
    match service_container.data_store().query_nodes(marketing_text_query).await {
        Ok(marketing_text) => {
            println!("   Found {} marketing-related nodes in 'text' table", marketing_text.len());
            for (i, node) in marketing_text.iter().enumerate() {
                if let Some(content_str) = node.content.as_str() {
                    let preview = if content_str.len() > 60 {
                        format!("{}...", &content_str[..57])
                    } else {
                        content_str.to_string()
                    };
                    println!("     [{}] {}", i + 1, preview);
                }
            }
        }
        Err(e) => println!("   Error searching marketing text: {}", e),
    }

    // Count total records in each table
    println!("\n4. Counting total records...");
    
    let count_nodes_query = "SELECT count() FROM nodes GROUP ALL";
    match service_container.data_store().query_nodes(count_nodes_query).await {
        Ok(count_result) => {
            println!("   Total nodes in 'nodes' table: {}", count_result.len());
        }
        Err(e) => println!("   Error counting nodes: {}", e),
    }

    let count_text_query = "SELECT count() FROM text GROUP ALL";  
    match service_container.data_store().query_nodes(count_text_query).await {
        Ok(count_result) => {
            println!("   Total nodes in 'text' table: {}", count_result.len());
        }
        Err(e) => println!("   Error counting text: {}", e),
    }

    println!("\n💡 Conclusion: The marketing sample data is likely in the 'nodes' table,");
    println!("   but get_nodes_for_date queries the 'text' table via relationships.");

    Ok(())
}