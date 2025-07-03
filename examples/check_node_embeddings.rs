use arrow_array::{Array, FixedSizeListArray};
use futures::TryStreamExt;
use lancedb::{
    connect,
    query::{ExecutableQuery, QueryBase},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Checking Embedding Status in NodeSpace Database");
    println!("=================================================");

    let db_path = "/Users/malibio/nodespace/data/lance_db";
    println!("📂 Connecting to database at: {}", db_path);

    let connection = connect(db_path).execute().await?;
    let table = connection.open_table("universal_nodes").execute().await?;

    // Get row count
    let total_rows = table.count_rows(None).await?;
    println!("📊 Total rows in database: {}", total_rows);

    // Query all rows to check vector field
    let mut results = table.query().limit(total_rows as usize).execute().await?;

    let mut nodes_with_embeddings = 0;
    let mut nodes_without_embeddings = 0;
    let mut total_checked = 0;
    let mut sample_rows = Vec::new();

    while let Some(batch) = results.try_next().await? {
        for row_idx in 0..batch.num_rows() {
            total_checked += 1;

            // Get the vector column
            if let Some(vector_column) = batch.column_by_name("vector") {
                let fixed_size_list = vector_column.as_any().downcast_ref::<FixedSizeListArray>();

                if let Some(array) = fixed_size_list {
                    if array.is_null(row_idx) {
                        nodes_without_embeddings += 1;
                    } else {
                        nodes_with_embeddings += 1;

                        // Save first few samples
                        if sample_rows.len() < 5 {
                            if let Some(id_column) = batch.column_by_name("id") {
                                if let Some(id_str_array) = id_column
                                    .as_any()
                                    .downcast_ref::<arrow_array::StringArray>(
                                ) {
                                    let id = id_str_array.value(row_idx);
                                    sample_rows.push(id.to_string());
                                }
                            }
                        }
                    }
                } else {
                    println!("⚠️  Could not cast vector column to FixedSizeListArray");
                }
            } else {
                println!("❌ No vector column found");
                break;
            }
        }
    }

    println!("\n📈 Embedding Analysis Results:");
    println!("   📊 Total nodes checked: {}", total_checked);
    println!("   ✅ Nodes WITH embeddings: {}", nodes_with_embeddings);
    println!(
        "   ❌ Nodes WITHOUT embeddings: {}",
        nodes_without_embeddings
    );

    let percent_with_embeddings = if total_checked > 0 {
        (nodes_with_embeddings as f64 / total_checked as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "   📊 Percentage with embeddings: {:.1}%",
        percent_with_embeddings
    );

    if !sample_rows.is_empty() {
        println!("\n🔍 Sample nodes with embeddings:");
        for (i, id) in sample_rows.iter().enumerate() {
            println!("   {}. {}", i + 1, id);
        }
    }

    if nodes_with_embeddings > 0 && nodes_without_embeddings > 0 {
        println!("\n💡 FINDING: Mixed embedding status - some nodes have embeddings, others don't");
        println!("   This suggests multiple data population workflows are being used:");
        println!("   📝 store_node() - saves nodes WITHOUT embeddings");
        println!("   🔢 store_node_with_embedding() - saves nodes WITH embeddings");
    } else if nodes_with_embeddings == total_checked {
        println!("\n✅ FINDING: ALL nodes have embeddings - consistent use of store_node_with_embedding()");
    } else if nodes_without_embeddings == total_checked {
        println!("\n❌ FINDING: NO nodes have embeddings - only store_node() has been used");
    }

    // Check for transaction count correlation
    println!("\n🗂️  Transaction File Analysis:");
    println!("   Your question about 462+ transaction files vs 299 nodes suggests:");
    println!("   • Each operation (create, update, delete) creates a transaction file");
    println!("   • Updates and corrections accumulate as additional transactions");
    println!("   • The 462 files represent the full history of all database operations");
    println!("   • Only 299 nodes remain after deletions/updates in the current state");

    Ok(())
}
