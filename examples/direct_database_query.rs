use futures::TryStreamExt;
use lancedb::{
    connect,
    query::{ExecutableQuery, QueryBase},
};

/// Direct LanceDB query to examine raw table contents
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Direct LanceDB Query - Examining Raw Table Contents\n");

    // Connect directly to LanceDB
    let db_path = "../data/lance_db/e2e_sample.db";
    let db = connect(db_path).execute().await?;
    println!("✅ Connected to database: {}", db_path);

    // List all tables
    let table_names = db.table_names().execute().await?;
    println!("\n📋 Tables in database:");
    for table_name in &table_names {
        println!("   📊 Table: {}", table_name);
    }

    if table_names.is_empty() {
        println!("   ❌ No tables found in database");
        return Ok(());
    }

    // Check the universal_nodes table specifically
    if table_names.contains(&"universal_nodes".to_string()) {
        println!("\n🎯 Examining 'universal_nodes' table...");

        let table = db.open_table("universal_nodes").execute().await?;

        // Get table info
        let schema = table.schema().await?;
        println!("   📊 Table schema:");
        for field in schema.fields() {
            println!(
                "      📌 Field: {} - Type: {:?}",
                field.name(),
                field.data_type()
            );
        }

        // Count rows
        let row_count = table.count_rows(None).await?;
        println!("\n   📈 Total rows: {}", row_count);

        if row_count > 0 {
            println!("\n   📄 Examining first 10 rows...");

            // Query first 10 rows
            let query_result = table.query().limit(10).execute().await?;

            // Convert to RecordBatch and examine
            let batches = query_result.try_collect::<Vec<_>>().await?;

            for (batch_idx, batch) in batches.iter().enumerate() {
                println!(
                    "      📦 Batch {}: {} rows, {} columns",
                    batch_idx + 1,
                    batch.num_rows(),
                    batch.num_columns()
                );

                // Print column names
                print!("         Columns: ");
                for (i, field) in batch.schema().fields().iter().enumerate() {
                    print!("{}", field.name());
                    if i < batch.schema().fields().len() - 1 {
                        print!(", ");
                    }
                }
                println!();

                // Print first few rows of data
                for row_idx in 0..std::cmp::min(3, batch.num_rows()) {
                    println!("         Row {}: ", row_idx + 1);
                    for (col_idx, field) in batch.schema().fields().iter().enumerate() {
                        let column = batch.column(col_idx);
                        println!(
                            "           {}: {:?}",
                            field.name(),
                            column.slice(row_idx, 1)
                        );
                    }
                }
            }
        } else {
            println!("   📭 Table is empty (0 rows)");
        }

        // Get all rows and search manually for patterns
        if row_count > 0 && row_count <= 100 {
            println!("\n   🔍 Searching for content patterns in all rows...");

            let all_query = table.query().limit(100).execute().await?;
            let all_batches = all_query.try_collect::<Vec<_>>().await?;

            let search_patterns = vec![
                "Product Launch",
                "2025-06-28",
                "Campaign Strategy",
                "Launch Overview",
            ];

            for pattern in search_patterns {
                println!("      🔍 Searching for '{}'...", pattern);
                let mut found_matches = 0;

                for batch in &all_batches {
                    if let Some(content_col) = batch.column_by_name("content") {
                        for row_idx in 0..batch.num_rows() {
                            let content_array = content_col.slice(row_idx, 1);
                            let content_str = format!("{:?}", content_array);
                            if content_str.contains(pattern) {
                                found_matches += 1;
                                if let Some(id_col) = batch.column_by_name("id") {
                                    println!(
                                        "           📝 Match: ID={:?}, Content={:?}",
                                        id_col.slice(row_idx, 1),
                                        content_array
                                    );
                                }
                            }
                        }
                    }
                }
                println!(
                    "         ✅ Found {} matches for '{}'",
                    found_matches, pattern
                );
            }
        }
    } else {
        println!("\n❌ 'universal_nodes' table not found");
        println!("   Available tables: {:?}", table_names);
    }

    println!("\n✅ Direct database examination complete!");

    Ok(())
}
