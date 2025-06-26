// Check table counts using DataStore trait methods
use nodespace_data_store::{SurrealDataStore, DataStore};
use std::error::Error;

#[tokio::main] 
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Checking table counts using DataStore trait methods...\n");
    
    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let data_store = SurrealDataStore::new(database_path).await?;
    println!("✅ Data store connection established");

    // Check counts using query_nodes method
    println!("\n1. Checking table counts...");
    
    // Use simpler count queries that don't require GROUP ALL
    let simple_nodes_query = "SELECT * FROM nodes";
    match data_store.query_nodes(simple_nodes_query).await {
        Ok(nodes) => println!("   'nodes' table: {} records", nodes.len()),
        Err(e) => println!("   'nodes' table error: {}", e),
    }

    let simple_text_query = "SELECT * FROM text";
    match data_store.query_nodes(simple_text_query).await {
        Ok(text_nodes) => println!("   'text' table: {} records", text_nodes.len()),
        Err(e) => println!("   'text' table error: {}", e),
    }

    let simple_date_query = "SELECT * FROM date";
    match data_store.query_nodes(simple_date_query).await {
        Ok(date_nodes) => println!("   'date' table: {} records", date_nodes.len()),
        Err(e) => println!("   'date' table error: {}", e),
    }

    // Check for marketing data on specific dates
    println!("\n2. Checking for data on specific dates...");
    
    let test_dates = ["2025-01-02", "2025-02-03", "2025-06-23"];
    
    for test_date in test_dates {
        println!("   Testing date: {}", test_date);
        
        // Test the get_nodes_for_date method
        match data_store.get_nodes_for_date(test_date).await {
            Ok(nodes) => {
                println!("     get_nodes_for_date returned: {} nodes", nodes.len());
                if nodes.len() > 0 {
                    if let Some(content_str) = nodes[0].content.as_str() {
                        let preview = if content_str.len() > 50 {
                            format!("{}...", &content_str[..47])
                        } else {
                            content_str.to_string()
                        };
                        println!("     Sample content: {}", preview);
                    }
                    break; // Found data, no need to check more dates
                }
            }
            Err(e) => println!("     get_nodes_for_date error: {}", e),
        }
    }

    // Check if there are any contains relationships
    println!("\n3. Checking relationship table...");
    let contains_query = "SELECT * FROM contains LIMIT 5";
    match data_store.query_nodes(contains_query).await {
        Ok(relationships) => {
            println!("   'contains' relationships: {} found", relationships.len());
            for (i, rel) in relationships.iter().take(3).enumerate() {
                println!("     [{}] Relationship: {:?}", i + 1, rel.id);
            }
        }
        Err(e) => println!("   'contains' relationships error: {}", e),
    }

    // Test creating a relationship and see if it works
    println!("\n4. Testing relationship creation...");
    
    // First ensure we have a date node for testing
    let test_date = "2025-12-31";
    match data_store.create_or_get_date_node(test_date, Some("Test date")).await {
        Ok(date_node_id) => {
            println!("   Created/got date node: {}", date_node_id);
            
            // Now create a text node
            match data_store.create_text_node("Test content for relationship", Some(test_date)).await {
                Ok(text_node_id) => {
                    println!("   Created text node: {}", text_node_id);
                    
                    // Test if we can retrieve it
                    match data_store.get_nodes_for_date(test_date).await {
                        Ok(retrieved) => {
                            println!("   Retrieved {} nodes for test date", retrieved.len());
                            if retrieved.len() > 0 {
                                println!("   ✅ Relationship creation and retrieval working!");
                            } else {
                                println!("   ❌ Relationship not working - no nodes retrieved");
                            }
                        }
                        Err(e) => println!("   Error retrieving test nodes: {}", e),
                    }
                }
                Err(e) => println!("   Error creating text node: {}", e),
            }
        }
        Err(e) => println!("   Error creating date node: {}", e),
    }

    println!("\n💡 Summary: This should show us the current state of the database");
    println!("   and whether the relationship system is working correctly.");

    Ok(())
}