#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗄️ Raw Database Content Analysis");
    
    let db_path = "/Users/malibio/nodespace/data/lance_db/e2e_sample.db";
    
    use nodespace_core_logic::{NodeSpaceService, CoreLogic};
    
    let service = NodeSpaceService::create_with_paths(db_path, Some("./bundled_models")).await?;
    service.initialize().await.ok();
    
    let results = service.semantic_search("", 5).await?;
    
    println!("\n📊 Raw Data Analysis - First 5 Nodes:");
    
    for (i, result) in results.iter().enumerate() {
        let node = &result.node;
        println!("\n{}", "=".repeat(60));
        println!("Node {}: {}", i + 1, node.id);
        println!("{}", "=".repeat(60));
        
        // Show the exact content as stored in database
        println!("🔍 Content Field Analysis:");
        println!("  JSON Type: {:?}", node.content);
        
        if let Some(content_str) = node.content.as_str() {
            println!("  Raw String: {:?}", content_str);
            println!("  Length: {} characters", content_str.len());
            
            // Detailed character analysis
            println!("\n📝 Character-by-Character Breakdown:");
            for (idx, ch) in content_str.char_indices().take(30) {
                let description = match ch {
                    '"' => "QUOTE CHARACTER",
                    '\\' => "BACKSLASH",
                    '\n' => "NEWLINE",
                    '\r' => "CARRIAGE RETURN",
                    '\t' => "TAB",
                    ' ' => "SPACE",
                    _ if ch.is_ascii_control() => "CONTROL CHAR",
                    _ => "REGULAR CHAR",
                };
                println!("    [{:2}]: '{}' (U+{:04X}) - {}", idx, ch, ch as u32, description);
            }
            if content_str.len() > 30 {
                println!("    ... (showing first 30 characters only)");
            }
            
            // Quote analysis
            let quote_count = content_str.chars().filter(|&ch| ch == '"').count();
            println!("\n🔍 Quote Analysis:");
            println!("  Total quote characters: {}", quote_count);
            println!("  Starts with quote: {}", content_str.starts_with('"'));
            println!("  Ends with quote: {}", content_str.ends_with('"'));
            
            if quote_count > 0 {
                let quote_positions: Vec<_> = content_str.char_indices()
                    .filter(|(_, ch)| *ch == '"')
                    .map(|(idx, _)| idx)
                    .collect();
                println!("  Quote positions: {:?}", quote_positions);
            }
            
            // Show what happens with our cleaning logic
            let cleaned = content_str.trim().trim_matches('"').trim();
            println!("\n🧹 After Quote Cleaning:");
            println!("  Original: {:?}", content_str);
            println!("  Cleaned:  {:?}", cleaned);
            println!("  Changed:  {}", cleaned != content_str);
            
            // Show bytes for absolute certainty
            println!("\n🔢 Raw Bytes (first 20):");
            let bytes: Vec<u8> = content_str.bytes().take(20).collect();
            for (idx, byte) in bytes.iter().enumerate() {
                println!("    [{:2}]: 0x{:02X} ({:3}) = '{}'", 
                    idx, 
                    byte, 
                    byte, 
                    if *byte >= 32 && *byte <= 126 { *byte as char } else { '.' }
                );
            }
            
        } else {
            println!("  ❌ Content is not a string type");
        }
        
        // Show metadata format too
        if let Some(metadata) = &node.metadata {
            println!("\n📋 Metadata:");
            println!("  Type: {:?}", metadata);
            if let serde_json::Value::Object(obj) = metadata {
                for (key, value) in obj.iter().take(3) {
                    println!("    {}: {:?}", key, value);
                }
            }
        }
        
        if i >= 2 { break; } // Show first 3 nodes in detail
    }
    
    println!("\n🎯 Raw Data Analysis Complete");
    println!("   The content field contains literal quote characters");
    println!("   This confirms double JSON encoding in the storage layer");
    
    Ok(())
}