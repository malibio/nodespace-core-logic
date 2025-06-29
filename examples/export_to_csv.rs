use futures::TryStreamExt;
use lancedb::{
    connect,
    query::{ExecutableQuery, QueryBase},
};
use std::fs::File;
use std::io::Write;

/// Export all database tables to CSV files
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Exporting Database to CSV Files\n");

    // Connect to the database
    let db_path = "../data/lance_db/e2e_sample.db";
    let db = connect(db_path).execute().await?;
    println!("✅ Connected to database: {}", db_path);

    // Get all table names
    let table_names = db.table_names().execute().await?;
    println!("📋 Found {} table(s):", table_names.len());
    for table_name in &table_names {
        println!("   📊 Table: {}", table_name);
    }

    // Export each table to CSV
    for table_name in &table_names {
        println!("\n🔄 Exporting table: {}", table_name);

        let table = db.open_table(table_name).execute().await?;

        // Get all rows
        let query_result = table.query().limit(10000).execute().await?;
        let batches = query_result.try_collect::<Vec<_>>().await?;

        if batches.is_empty() {
            println!("   ⚠️  Table {} is empty", table_name);
            continue;
        }

        // Create CSV file
        let csv_filename = format!("{}.csv", table_name);
        let mut csv_file = File::create(&csv_filename)?;
        println!("   📄 Creating CSV file: {}", csv_filename);

        // Write CSV header
        let schema = &batches[0].schema();
        let field_names: Vec<String> = schema
            .fields()
            .iter()
            .map(|field| field.name().to_string())
            .collect();

        writeln!(csv_file, "{}", field_names.join(","))?;
        println!("   📋 CSV Header: {}", field_names.join(", "));

        let mut total_rows = 0;

        // Process each batch
        for (batch_idx, batch) in batches.iter().enumerate() {
            println!(
                "   📦 Processing batch {} ({} rows)",
                batch_idx + 1,
                batch.num_rows()
            );

            for row_idx in 0..batch.num_rows() {
                let mut row_values = Vec::new();

                for col_idx in 0..batch.num_columns() {
                    let column = batch.column(col_idx);
                    let schema_ref = batch.schema();
                    let field = schema_ref.field(col_idx);

                    // Extract value based on column type and clean it for CSV
                    let value = match field.data_type() {
                        arrow_schema::DataType::Utf8 => {
                            // String column
                            let array_slice = column.slice(row_idx, 1);
                            let debug_str = format!("{:?}", array_slice);
                            extract_string_from_array_debug(&debug_str)
                        }
                        arrow_schema::DataType::FixedSizeList(_, _) => {
                            // Vector column - represent as JSON array
                            let array_slice = column.slice(row_idx, 1);
                            let debug_str = format!("{:?}", array_slice);
                            extract_vector_from_array_debug(&debug_str)
                        }
                        arrow_schema::DataType::List(_) => {
                            // List column (like children_ids, mentions)
                            let array_slice = column.slice(row_idx, 1);
                            let debug_str = format!("{:?}", array_slice);
                            extract_list_from_array_debug(&debug_str)
                        }
                        _ => {
                            // Other types
                            let array_slice = column.slice(row_idx, 1);
                            let debug_str = format!("{:?}", array_slice);
                            clean_debug_string(&debug_str)
                        }
                    };

                    // Escape CSV value
                    let csv_value = escape_csv_value(&value);
                    row_values.push(csv_value);
                }

                writeln!(csv_file, "{}", row_values.join(","))?;
                total_rows += 1;
            }
        }

        csv_file.flush()?;
        println!("   ✅ Exported {} rows to {}", total_rows, csv_filename);
    }

    println!("\n🎉 Export complete!");
    println!("📁 CSV files created in current directory:");
    for table_name in &table_names {
        println!("   📄 {}.csv", table_name);
    }

    Ok(())
}

fn extract_string_from_array_debug(debug_str: &str) -> String {
    // Extract string from debug format like: StringArray[\n  "content here",\n]
    if let Some(start) = debug_str.find("\"") {
        if let Some(end) = debug_str.rfind("\"") {
            if start < end {
                return debug_str[start + 1..end].to_string();
            }
        }
    }

    // Fallback: clean up the debug string
    debug_str
        .replace("StringArray\n[\n  \"", "")
        .replace("\",\n]", "")
        .replace("\\\"", "\"")
        .replace("\\n", "\n")
        .trim()
        .to_string()
}

fn extract_vector_from_array_debug(debug_str: &str) -> String {
    // Extract float array and convert to JSON-like format
    if debug_str.contains("PrimitiveArray<Float32>") {
        // Try to extract the actual numbers
        if let Some(start) = debug_str.find("[") {
            if let Some(end) = debug_str.rfind("]") {
                let numbers_section = &debug_str[start + 1..end];
                // Clean up and format as JSON array
                let cleaned = numbers_section
                    .replace("\n", "")
                    .replace("  ", " ")
                    .trim()
                    .to_string();
                return format!("[{}]", cleaned);
            }
        }
    }

    // Fallback
    "[vector_data]".to_string()
}

fn extract_list_from_array_debug(debug_str: &str) -> String {
    // Extract list items from debug format
    if debug_str.contains("StringArray\n[\n]") || debug_str.contains("StringArray\n[]") {
        return "[]".to_string();
    }

    // Try to extract actual list items
    if let Some(start) = debug_str.find("StringArray\n[") {
        if let Some(end) = debug_str.rfind("]") {
            let list_content = &debug_str[start + 12..end];
            if list_content.trim().is_empty() {
                return "[]".to_string();
            }
            // Basic parsing - this could be improved
            let items: Vec<&str> = list_content.split(",").collect();
            let cleaned_items: Vec<String> = items
                .iter()
                .map(|item| item.trim().trim_matches('"').to_string())
                .filter(|item| !item.is_empty())
                .collect();
            return format!("[{}]", cleaned_items.join(","));
        }
    }

    "[]".to_string()
}

fn clean_debug_string(debug_str: &str) -> String {
    debug_str
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn escape_csv_value(value: &str) -> String {
    // Escape CSV special characters
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace("\"", "\"\""))
    } else {
        value.to_string()
    }
}
