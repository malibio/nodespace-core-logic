use nodespace_core_logic::{NodeSpaceService, CoreLogic};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Column Usage Analysis - Universal Schema vs Type-Specific Data");
    
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db",
        Some("./bundled_models"),
    ).await?;
    
    service.initialize().await.ok();
    
    let results = service.semantic_search("", 10).await?;
    
    println!("\n📊 Column Usage by Node Type:");
    
    let mut type_analysis = std::collections::HashMap::new();
    
    for result in &results {
        let node = &result.node;
        
        // Extract node type
        let node_type = if let Some(metadata) = &node.metadata {
            if let Value::Object(obj) = metadata {
                obj.get("node_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
            } else { "unknown" }
        } else { "unknown" };
        
        // Analyze what "columns" this node actually uses
        let mut used_columns = Vec::new();
        let mut unused_columns = Vec::new();
        
        // Always used columns
        used_columns.push("id");
        used_columns.push("content");
        used_columns.push("created_at");
        used_columns.push("updated_at");
        used_columns.push("node_type");
        
        // Conditionally used columns
        let has_parent = if let Some(metadata) = &node.metadata {
            if let Value::Object(obj) = metadata {
                obj.contains_key("parent_id")
            } else { false }
        } else { false };
        
        if has_parent {
            used_columns.push("parent_id");
        } else {
            unused_columns.push("parent_id");
        }
        
        // These are currently unused in NodeSpace but exist in schema
        unused_columns.push("children_ids");
        unused_columns.push("mentions");
        unused_columns.push("vector"); // (populated but not shown here)
        
        let analysis = type_analysis.entry(node_type.to_string()).or_insert_with(|| {
            (Vec::new(), Vec::new(), 0)
        });
        
        analysis.0 = used_columns.clone();
        analysis.1 = unused_columns.clone();
        analysis.2 += 1;
    }
    
    for (node_type, (used, unused, count)) in &type_analysis {
        println!("\n🏷️  Node Type: '{}' ({} nodes)", node_type, count);
        println!("   ✅ Used columns: {:?}", used);
        println!("   ❌ Unused columns: {:?}", unused);
        
        // Show what goes in metadata for this type
        let sample_node = results.iter()
            .find(|r| {
                if let Some(metadata) = &r.node.metadata {
                    if let Value::Object(obj) = metadata {
                        obj.get("node_type")
                            .and_then(|v| v.as_str()) == Some(node_type)
                    } else { false }
                } else { false }
            })
            .map(|r| &r.node);
        
        if let Some(node) = sample_node {
            if let Some(metadata) = &node.metadata {
                println!("   📋 Metadata fields for this type:");
                if let Value::Object(obj) = metadata {
                    for (key, value) in obj {
                        println!("      '{}': {:?}", key, value);
                    }
                }
            }
        }
    }
    
    println!("\n🏗️  Universal Schema Design:");
    println!("   LanceDB uses a FIXED schema for ALL node types:");
    println!("   ");
    println!("   Schema::new([");
    println!("       Field::new(\"id\", Utf8, false),           // ✅ All types use");
    println!("       Field::new(\"node_type\", Utf8, false),    // ✅ All types use");
    println!("       Field::new(\"content\", Utf8, false),      // ✅ All types use");
    println!("       Field::new(\"parent_id\", Utf8, true),     // ❓ Only child nodes use");
    println!("       Field::new(\"children_ids\", List, true),  // ❌ Currently unused");
    println!("       Field::new(\"mentions\", List, true),      // ❌ Currently unused");
    println!("       Field::new(\"vector\", FixedSizeList, false), // ✅ All types use");
    println!("       Field::new(\"created_at\", Utf8, false),   // ✅ All types use");
    println!("       Field::new(\"updated_at\", Utf8, false),   // ✅ All types use");
    println!("       Field::new(\"metadata\", Utf8, true),      // ✅ All types use");
    println!("   ])");
    
    println!("\n❓ Does It Matter If Columns Are Unused?");
    println!("   ✅ NO - Columnar storage handles this efficiently:");
    println!("   ");
    println!("   🔸 NULL columns: Take minimal space (null bitmap only)");
    println!("   🔸 Arrow format: Compressed unused columns");
    println!("   🔸 Memory mapping: Only load needed columns");
    println!("   🔸 Query optimization: Skip unused columns in scans");
    
    println!("\n🆚 Alternative Approaches:");
    println!("   📊 Option 1: UNIVERSAL SCHEMA (current)");
    println!("      ✅ All types share same table");
    println!("      ✅ Cross-type queries and joins");
    println!("      ✅ Single vector index");
    println!("      ❌ Some columns unused per type");
    println!("   ");
    println!("   📊 Option 2: TYPE-SPECIFIC TABLES");
    println!("      ✅ Each type has only needed columns");
    println!("      ❌ Complex cross-type queries");
    println!("      ❌ Multiple vector indexes");
    println!("      ❌ Schema management overhead");
    
    println!("\n🎯 Why Universal Schema Wins:");
    println!("   1. Type-specific data goes in 'metadata' JSON column");
    println!("   2. Core relationships use fixed columns (parent_id)");
    println!("   3. Vector search works across ALL content types");
    println!("   4. Unused columns cost almost nothing in Arrow format");
    println!("   5. Future node types don't need schema changes");
    
    // Show specific example
    println!("\n📋 Concrete Example:");
    if let Some(date_node) = results.iter().find(|r| {
        if let Some(content) = r.node.content.as_str() {
            content.contains("June 27, 2025")
        } else { false }
    }) {
        println!("   Date Node Columns:");
        println!("      id: ✅ Used (unique identifier)");
        println!("      content: ✅ Used ('# June 27, 2025')");
        println!("      parent_id: ❌ NULL (top-level node)");
        println!("      children_ids: ❌ NULL (not populated)");
        println!("      metadata: ✅ Used ({{\"node_type\": \"date\", \"date\": \"2025-06-27\"}})");
        println!("      vector: ✅ Used (embeddings for semantic search)");
    }
    
    Ok(())
}