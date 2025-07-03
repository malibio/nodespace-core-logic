use arrow_array::RecordBatch;
use futures::TryStreamExt;
use lancedb::connect;
use lancedb::query::{ExecutableQuery, QueryBase};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Direct LanceDB Vector Inspection");
    println!("====================================");

    let db_path = "/Users/malibio/nodespace/data/lance_db";

    println!("📂 Opening LanceDB database at: {}", db_path);

    // Connect to LanceDB
    let db = connect(db_path).execute().await?;
    let table_names = db.table_names().execute().await?;

    println!("📋 Available tables: {:?}", table_names);

    // Try to open the main table (assuming it's called "universal_nodes")
    if table_names.is_empty() {
        println!("❌ No tables found in database!");
        return Ok(());
    }

    let table_name = &table_names[0]; // Use first table
    println!("\n📊 Inspecting table: {}", table_name);

    let table = db.open_table(table_name).execute().await?;
    let schema = table.schema().await?;

    println!("📊 Table schema:");
    for field in schema.fields() {
        println!("   - {}: {:?}", field.name(), field.data_type());
    }

    println!("\n📈 Table statistics:");
    println!("   - Total rows: {}", table.count_rows(None).await?);

    // Take a small sample to inspect the vector field
    println!("\n🔬 Inspecting first 3 rows for vector field:");

    let mut stream = table.query().limit(3).execute().await?;
    let batches: Vec<RecordBatch> = stream.try_collect().await?;

    if let Some(record_batch) = batches.first() {
        // Look for vector column
        if let Some(vector_column) = record_batch.column_by_name("vector") {
            println!("✅ Found 'vector' column!");
            println!("   - Data type: {:?}", vector_column.data_type());
            println!("   - Length: {}", vector_column.len());

            // Check if vectors are null or have data
            let null_count = vector_column.null_count();
            println!("   - Null count: {}/{}", null_count, vector_column.len());

            if null_count == vector_column.len() {
                println!("❌ ALL vectors are NULL - no embeddings exist!");
            } else if null_count > 0 {
                println!("⚠️  Some vectors are NULL - partial embeddings");
            } else {
                println!("✅ All vectors have data - embeddings exist!");

                // Try to print first vector dimensions if possible
                println!("   - Trying to inspect vector dimensions...");
            }
        } else {
            println!("❌ No 'vector' column found in dataset!");
            println!("Available columns:");
            for (i, column) in record_batch.columns().iter().enumerate() {
                println!(
                    "   {}: {} ({:?})",
                    i,
                    record_batch.schema().field(i).name(),
                    column.data_type()
                );
            }
        }

        // Also check content field to verify data exists
        if let Some(content_column) = record_batch.column_by_name("content") {
            println!("\n📝 Content column found:");
            println!("   - Type: {:?}", content_column.data_type());
            println!(
                "   - Non-null entries: {}/{}",
                content_column.len() - content_column.null_count(),
                content_column.len()
            );
        }
    } else {
        println!("❌ No data found in dataset!");
    }

    Ok(())
}
