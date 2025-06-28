use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗄️ LanceDB Structure Investigation");
    
    let db_path = "/Users/malibio/nodespace/data/lance_db/e2e_sample.db";
    
    println!("\n1️⃣ Database Path Structure:");
    println!("   Database path: {}", db_path);
    
    // Check if the path exists and what it contains
    if Path::new(db_path).exists() {
        println!("   ✅ Database path exists");
        
        // List contents of the database directory
        println!("\n2️⃣ Database Directory Contents:");
        let entries = std::fs::read_dir(db_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            
            if metadata.is_dir() {
                println!("   📁 Directory: {}", path.file_name().unwrap().to_string_lossy());
                
                // If it's a directory, list its contents (these would be "tables")
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub_entry in sub_entries {
                        if let Ok(sub_entry) = sub_entry {
                            let sub_path = sub_entry.path();
                            let sub_metadata = sub_entry.metadata().unwrap_or_else(|_| metadata.clone());
                            
                            if sub_metadata.is_file() {
                                println!("      📄 File: {} (size: {} bytes)", 
                                    sub_path.file_name().unwrap().to_string_lossy(),
                                    sub_metadata.len()
                                );
                            } else {
                                println!("      📁 Subdir: {}", 
                                    sub_path.file_name().unwrap().to_string_lossy()
                                );
                            }
                        }
                    }
                }
            } else {
                println!("   📄 File: {} (size: {} bytes)", 
                    path.file_name().unwrap().to_string_lossy(),
                    metadata.len()
                );
            }
        }
    } else {
        println!("   ❌ Database path does not exist");
        return Ok(());
    }
    
    println!("\n3️⃣ LanceDB Conceptual Structure:");
    println!("   In LanceDB:");
    println!("   - Database = Directory (e2e_sample.db/)");
    println!("   - Table = Subdirectory within database");
    println!("   - Data = Parquet/Arrow files within table directory");
    println!("   - Index = Additional files for vector/scalar indexes");
    
    println!("\n4️⃣ What This Means for Our Nodes:");
    println!("   Both nodes are stored in the SAME table/collection");
    println!("   Table name: 'universal_nodes' (single table design)");
    println!("   All NodeSpace entities go into this one table");
    println!("   Differentiated by 'node_type' column, not separate tables");
    
    println!("\n5️⃣ Traditional DB vs LanceDB:");
    println!("   Traditional SQL Database:");
    println!("   ├── Database");
    println!("   │   ├── Table: users");
    println!("   │   ├── Table: posts");
    println!("   │   └── Table: comments");
    println!("   ");
    println!("   LanceDB:");
    println!("   ├── Dataset Directory (e2e_sample.db/)");
    println!("   │   ├── Table Directory: universal_nodes/");
    println!("   │   │   ├── data_0.parquet");
    println!("   │   │   ├── data_1.parquet");
    println!("   │   │   ├── _versions/");
    println!("   │   │   └── _indices/");
    println!("   │   └── (Other table directories if they existed)");
    
    // Try to connect and list actual tables
    println!("\n6️⃣ Attempting to List Actual Tables:");
    
    // We can't directly access LanceDB from here without the right dependencies,
    // but we can infer from the NodeSpace service implementation
    println!("   Based on nodespace-data-store implementation:");
    println!("   - Single table: 'universal_nodes'");
    println!("   - All node types stored together");
    println!("   - Date nodes, text nodes, project nodes all in same table");
    println!("   - Filtered by node_type column when needed");
    
    println!("\n7️⃣ Why Single Table Design?");
    println!("   ✅ Unified schema for all entity types");
    println!("   ✅ Cross-entity vector search");
    println!("   ✅ Simplified relationship queries");
    println!("   ✅ Consistent parent-child hierarchy");
    println!("   ✅ Single vector index across all content");
    
    Ok(())
}