#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 LanceDB Data Types & Storage Analysis");
    
    println!("\n{}", "=".repeat(80));
    println!("1️⃣ NODE_TYPE COLUMN: String vs Enum vs Numbers");
    println!("{}", "=".repeat(80));
    
    println!("\n📊 Option 1: STRING (Current Implementation)");
    println!("   node_type: DataType::Utf8");
    println!("   Values: 'text', 'date', 'task', 'aichat', 'image', 'audio'");
    println!("   ");
    println!("   ✅ Pros:");
    println!("      - Human readable in queries and debugging");
    println!("      - Self-documenting code");
    println!("      - Easy to add new types without schema changes");
    println!("      - JSON interoperability");
    println!("      - No enum maintenance overhead");
    println!("   ");
    println!("   ❌ Cons:");
    println!("      - Slightly larger storage (4-8 bytes vs 1-2 bytes)");
    println!("      - String comparison slightly slower than integer");
    println!("      - Possible typos in code");
    
    println!("\n📊 Option 2: ENUM-LIKE INTEGERS");
    println!("   node_type: DataType::UInt8");
    println!("   Values: 0=text, 1=date, 2=task, 3=aichat, 4=image, 5=audio");
    println!("   ");
    println!("   ✅ Pros:");
    println!("      - Compact storage (1 byte per value)");
    println!("      - Faster integer comparisons");
    println!("      - Memory efficient");
    println!("   ");
    println!("   ❌ Cons:");
    println!("      - Need constant mapping: TYPE_TEXT = 0");
    println!("      - Queries become cryptic: WHERE node_type = 2");
    println!("      - Schema evolution requires careful enum management");
    println!("      - Debugging shows numbers instead of names");
    println!("      - JSON APIs need translation layer");
    
    println!("\n📊 Option 3: ARROW DICTIONARY (Best of Both)");
    println!("   node_type: DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))");
    println!("   Storage: Strings stored once, references are integers");
    println!("   ");
    println!("   ✅ Pros:");
    println!("      - Compact storage like integers");
    println!("      - Human readable like strings");
    println!("      - Fast comparisons");
    println!("      - Automatic deduplication");
    println!("   ");
    println!("   ❌ Cons:");
    println!("      - Slightly more complex Arrow schema");
    println!("      - Dictionary needs to be maintained");
    
    println!("\n🎯 PERFORMANCE COMPARISON (1M records):");
    
    let performance_data = vec![
        ("String (Utf8)", "~8MB", "String comparison", "Easy"),
        ("UInt8", "~1MB", "Integer comparison", "Enum mapping needed"),
        ("Dictionary", "~1.1MB", "Integer comparison", "Automatic mapping"),
    ];
    
    for (approach, storage, speed, complexity) in performance_data {
        println!("   {:<15} | {:<8} | {:<18} | {}", approach, storage, speed, complexity);
    }
    
    println!("\n{}", "=".repeat(80));
    println!("2️⃣ BINARY DATA STORAGE: Images, Audio, Documents");
    println!("{}", "=".repeat(80));
    
    println!("\n📊 Option 1: STORE BINARY IN LANCEDB (Possible!)");
    println!("   DataType::Binary or DataType::FixedSizeBinary[N]");
    println!("   ");
    println!("   Schema Example:");
    println!("   Field::new('image_data', DataType::Binary, true),");
    println!("   Field::new('audio_data', DataType::Binary, true),");
    println!("   Field::new('document_data', DataType::Binary, true),");
    println!("   ");
    println!("   ✅ Pros:");
    println!("      - Everything in one place");
    println!("      - Atomic operations");
    println!("      - Vector embeddings + raw data together");
    println!("      - Simplified backup/restore");
    println!("   ");
    println!("   ❌ Cons:");
    println!("      - Large .lance files (images can be MB each)");
    println!("      - Memory usage when scanning");
    println!("      - Not optimized for streaming large files");
    println!("      - Query performance impact");
    
    println!("\n📊 Option 2: HYBRID STORAGE (Recommended)");
    println!("   Binary data: File system");
    println!("   Metadata + embeddings: LanceDB");
    println!("   ");
    println!("   Schema Example:");
    println!("   Field::new('file_path', DataType::Utf8, true),");
    println!("   Field::new('file_hash', DataType::FixedSizeBinary[32], true),");
    println!("   Field::new('file_size', DataType::UInt64, true),");
    println!("   Field::new('mime_type', DataType::Utf8, true),");
    println!("   Field::new('image_embedding', FixedSizeList[Float32, 512], true),");
    println!("   ");
    println!("   ✅ Pros:");
    println!("      - Optimal performance for each data type");
    println!("      - File system handles large files efficiently");
    println!("      - LanceDB optimized for vectors + metadata");
    println!("      - Can use CDN/object storage for files");
    println!("      - Better caching strategies");
    
    println!("\n📊 STORAGE SIZE ANALYSIS:");
    
    println!("\n   For 1000 nodes with mixed content:");
    println!("   ");
    println!("   Text nodes (800):");
    println!("   ├── Content: ~100 bytes avg");
    println!("   ├── Text embedding: 384 × 4 = 1,536 bytes");
    println!("   └── Total per node: ~1.6KB");
    println!("   ");
    println!("   Image nodes (150):");
    println!("   ├── OPTION 1 (in LanceDB): 2MB avg per image");
    println!("   │   └── Total: 150 × 2MB = 300MB in LanceDB");
    println!("   ├── OPTION 2 (hybrid): File path + hash");
    println!("   │   └── Total: 150 × 100 bytes = 15KB in LanceDB");
    println!("   │   └── Files: 300MB on file system");
    println!("   ");
    println!("   Audio nodes (50):");
    println!("   ├── OPTION 1 (in LanceDB): 5MB avg per audio");
    println!("   │   └── Total: 50 × 5MB = 250MB in LanceDB");
    println!("   ├── OPTION 2 (hybrid): File path + hash");
    println!("   │   └── Total: 50 × 100 bytes = 5KB in LanceDB");
    println!("   │   └── Files: 250MB on file system");
    
    println!("\n   TOTAL DATABASE SIZE:");
    println!("   Option 1 (all in LanceDB): ~551MB");
    println!("   Option 2 (hybrid): ~1.3MB + 550MB files");
    
    println!("\n🚀 QUERY PERFORMANCE IMPACT:");
    
    println!("\n   Vector search query: 'Find similar images'");
    println!("   ");
    println!("   Option 1 (binary in LanceDB):");
    println!("   ├── Must scan through 551MB of data");
    println!("   ├── Memory usage: High");
    println!("   └── Query time: ~500ms");
    println!("   ");
    println!("   Option 2 (hybrid storage):");
    println!("   ├── Scan only 1.3MB of vectors + metadata");
    println!("   ├── Memory usage: Low");
    println!("   ├── Query time: ~50ms");
    println!("   └── Load files only when needed");
    
    println!("\n🏗️ RECOMMENDED SCHEMA:");
    
    println!("   Schema::new([");
    println!("     // Core fields");
    println!("     Field::new('id', DataType::Utf8, false),");
    println!("     Field::new('node_type', DataType::Utf8, false),  // ✅ Keep as string");
    println!("     Field::new('content', DataType::Utf8, false),");
    println!("     ");
    println!("     // Vector embeddings (multiple types)");
    println!("     Field::new('text_embedding', FixedSizeList[Float32, 384], true),");
    println!("     Field::new('image_embedding', FixedSizeList[Float32, 512], true),");
    println!("     Field::new('audio_embedding', FixedSizeList[Float32, 256], true),");
    println!("     ");
    println!("     // File references (not raw binary)");
    println!("     Field::new('file_path', DataType::Utf8, true),");
    println!("     Field::new('file_hash', DataType::FixedSizeBinary[32], true),");
    println!("     Field::new('file_size', DataType::UInt64, true),");
    println!("     Field::new('mime_type', DataType::Utf8, true),");
    println!("     ");
    println!("     // Standard fields");
    println!("     Field::new('parent_id', DataType::Utf8, true),");
    println!("     Field::new('created_at', DataType::Utf8, false),");
    println!("     Field::new('metadata', DataType::Utf8, true),");
    println!("   ])");
    
    println!("\n🎯 FINAL RECOMMENDATIONS:");
    
    println!("\n   1️⃣ Node Type: Keep as STRING (Utf8)");
    println!("      - Readability and maintainability > micro-optimizations");
    println!("      - Easy debugging and development");
    println!("      - Future-proof for new types");
    println!("   ");
    println!("   2️⃣ Binary Storage: HYBRID approach");
    println!("      - Store file paths/hashes in LanceDB");
    println!("      - Store actual files on file system");
    println!("      - Best performance for both vectors and files");
    println!("   ");
    println!("   3️⃣ Multiple Embeddings: YES, absolutely!");
    println!("      - text_embedding: For text content");
    println!("      - image_embedding: For visual similarity");
    println!("      - audio_embedding: For audio content");
    println!("      - Cross-modal search becomes possible");
    
    println!("\n   This gives you maximum flexibility with optimal performance!");
    
    Ok(())
}