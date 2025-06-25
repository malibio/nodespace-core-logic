//! Inspect the current database to understand what's actually stored

use nodespace_data_store::{DataStore, SurrealDataStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Inspecting database contents...\n");

    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let data_store = SurrealDataStore::new(database_path).await?;

    // Check what tables exist
    println!("📊 Checking database tables...");

    // Try to query all records from different tables
    let tables_to_check = vec![
        "nodes",
        "text",
        "dates",
        "contains",
        "next_sibling",
        "previous_sibling",
    ];

    for table_name in tables_to_check {
        println!("\n📋 Table: {}", table_name);
        let query = format!("SELECT * FROM {} LIMIT 5", table_name);

        match data_store.query_nodes(&query).await {
            Ok(results) => {
                println!("  ✅ Found {} records", results.len());
                for (i, record) in results.iter().enumerate() {
                    println!("    Record {}: ID={}", i + 1, record.id);
                    if let Some(content) = record.content.as_str() {
                        let preview = if content.len() > 60 {
                            format!("{}...", &content[..57])
                        } else {
                            content.to_string()
                        };
                        println!("      Content: '{}'", preview);
                    } else if !record.content.is_null() {
                        println!("      Content: {:?}", record.content);
                    } else {
                        println!("      Content: NULL");
                    }

                    if let Some(metadata) = &record.metadata {
                        println!("      Metadata: {:?}", metadata);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Error querying {}: {}", table_name, e);
            }
        }
    }

    // Check specific date queries
    println!("\n📅 Testing date-specific queries...");
    let test_dates = ["2025-06-23", "2025-06-24", "2025-06-25"];

    for date in test_dates {
        println!("\n  Date: {}", date);

        // Test get_nodes_for_date
        match data_store.get_nodes_for_date(date).await {
            Ok(nodes) => {
                println!("    ✅ get_nodes_for_date: {} nodes", nodes.len());
                for (i, node) in nodes.iter().take(3).enumerate() {
                    println!(
                        "      Node {}: ID={}, content_len={}",
                        i + 1,
                        node.id,
                        node.content.as_str().map(|s| s.len()).unwrap_or(0)
                    );
                }
            }
            Err(e) => {
                println!("    ❌ get_nodes_for_date failed: {}", e);
            }
        }

        // Test get_date_children
        match data_store.get_date_children(date).await {
            Ok(children) => {
                println!("    ✅ get_date_children: {} children", children.len());
                for (i, child) in children.iter().take(3).enumerate() {
                    if let Some(id) = child.get("id") {
                        println!("      Child {}: ID={}", i + 1, id);
                    } else {
                        println!("      Child {}: {:?}", i + 1, child);
                    }
                }
            }
            Err(e) => {
                println!("    ❌ get_date_children failed: {}", e);
            }
        }
    }

    // Try a generic query to see all data
    println!("\n🌍 Generic queries...");

    // Try to see all records across all tables
    let generic_query = "SELECT * FROM * LIMIT 10";
    match data_store.query_nodes(generic_query).await {
        Ok(results) => {
            println!("  ✅ Generic query found {} records", results.len());
            for (i, record) in results.iter().enumerate() {
                println!(
                    "    Record {}: ID={}, table={}",
                    i + 1,
                    record.id,
                    record.id.as_str().split(':').next().unwrap_or("unknown")
                );
            }
        }
        Err(e) => {
            println!("  ❌ Generic query failed: {}", e);
        }
    }

    println!("\n🏁 Database inspection completed!");
    Ok(())
}
