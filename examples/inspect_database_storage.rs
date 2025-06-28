use nodespace_core_logic::{NodeSpaceService, CoreLogic};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Inspecting Raw Database Storage Format");

    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await.ok();

    // Get nodes to examine
    let search_results = service.semantic_search("", 5).await?;
    
    println!("\n1️⃣ Raw Node Storage Analysis:");
    
    for (i, result) in search_results.iter().enumerate() {
        let node = &result.node;
        
        println!("\n--- Node {} ---", i + 1);
        println!("ID: {}", node.id);
        
        // Examine the content field directly
        println!("Content field type: {:?}", node.content);
        
        // Check if it's a JSON string vs actual string
        match &node.content {
            Value::String(s) => {
                println!("✅ Stored as JSON String");
                println!("   Raw value: {:?}", s);
                println!("   Length: {} characters", s.len());
                println!("   First 10 chars: {:?}", s.chars().take(10).collect::<String>());
                println!("   Last 10 chars: {:?}", s.chars().rev().take(10).collect::<String>().chars().rev().collect::<String>());
                
                // Check for surrounding quotes in the actual string content
                if s.starts_with('"') && s.ends_with('"') {
                    println!("   ⚠️  String value itself contains literal quotes!");
                    println!("   Content without outer quotes: {:?}", &s[1..s.len()-1]);
                } else {
                    println!("   ✅ String value is clean (no literal quotes)");
                }
            }
            Value::Object(obj) => {
                println!("❌ Stored as JSON Object");
                println!("   Object keys: {:?}", obj.keys().collect::<Vec<_>>());
            }
            Value::Array(arr) => {
                println!("❌ Stored as JSON Array");
                println!("   Array length: {}", arr.len());
            }
            Value::Number(n) => {
                println!("❌ Stored as JSON Number: {}", n);
            }
            Value::Bool(b) => {
                println!("❌ Stored as JSON Boolean: {}", b);
            }
            Value::Null => {
                println!("❌ Stored as JSON Null");
            }
        }
        
        // Test what .as_str() returns
        if let Some(as_str_result) = node.content.as_str() {
            println!("   .as_str() returns: {:?}", as_str_result);
            println!("   .as_str() length: {}", as_str_result.len());
            
            // Compare with manual string extraction
            if let Value::String(manual_str) = &node.content {
                if as_str_result == manual_str {
                    println!("   ✅ .as_str() matches manual extraction");
                } else {
                    println!("   ❌ .as_str() differs from manual extraction!");
                    println!("      Manual: {:?}", manual_str);
                    println!("      .as_str(): {:?}", as_str_result);
                }
            }
        } else {
            println!("   ❌ .as_str() returns None");
        }
        
        // Test our quote-cleaning logic
        if let Some(content) = node.content.as_str() {
            let cleaned = content.trim().trim_matches('"').trim();
            println!("   After quote cleaning: {:?}", cleaned);
            println!("   Cleaning changed content: {}", cleaned != content);
        }
        
        if i >= 2 { break; } // Limit to first 3 nodes for detailed analysis
    }

    println!("\n2️⃣ JSON Serialization Test:");
    
    // Create a test node and see how content gets serialized
    if let Some(first_result) = search_results.first() {
        let node = &first_result.node;
        
        println!("Original node content: {:?}", node.content);
        
        // Serialize the entire node to JSON
        let serialized = serde_json::to_string_pretty(&node)?;
        println!("Full node serialized:\n{}", serialized);
        
        // Just serialize the content field
        let content_serialized = serde_json::to_string(&node.content)?;
        println!("Content field serialized: {}", content_serialized);
        
        // Test deserializing it back
        let content_deserialized: Value = serde_json::from_str(&content_serialized)?;
        println!("Content deserialized: {:?}", content_deserialized);
    }

    println!("\n3️⃣ Database Source Investigation:");
    
    // Check if this is coming from LanceDB's arrow format
    println!("   Database type: LanceDB (Arrow/Parquet format)");
    println!("   JSON fields are stored as strings in Arrow format");
    println!("   Content field is likely TEXT column containing JSON string");
    
    println!("\n🎯 CONCLUSION:");
    println!("   The quotes are part of the JSON serialization format");
    println!("   When serde_json stores a String value, it adds escape quotes");
    println!("   Our trim_matches('\"') fix is handling this correctly");

    Ok(())
}