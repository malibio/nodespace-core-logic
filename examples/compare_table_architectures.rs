
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Table Architecture Comparison: Separate Tables vs Universal Schema");
    println!("Focus: NLP-based querying complexity");
    
    println!("\n{}", "=".repeat(80));
    println!("SCENARIO: User asks 'Find me content about product launches from last week'");
    println!("{}", "=".repeat(80));
    
    println!("\n🏗️ APPROACH 1: SEPARATE TABLES PER TYPE");
    println!("   Tables: date_nodes, text_nodes, project_nodes, task_nodes, document_nodes");
    
    println!("\n📊 Schema for Each Table:");
    let schemas = vec![
        ("date_nodes", vec!["id", "content", "date", "vector[384]", "created_at"]),
        ("text_nodes", vec!["id", "content", "parent_id", "depth", "vector[384]", "created_at"]),
        ("project_nodes", vec!["id", "content", "status", "priority", "vector[384]", "created_at"]),
        ("task_nodes", vec!["id", "content", "due_date", "assignee", "vector[384]", "created_at"]),
        ("document_nodes", vec!["id", "content", "file_path", "mime_type", "vector[384]", "created_at"]),
    ];
    
    for (table, columns) in schemas {
        println!("   {}: [{}]", table, columns.join(", "));
    }
    
    println!("\n🔍 NLP Query Implementation - SEPARATE TABLES:");
    println!("```rust");
    println!("async fn semantic_search_separate_tables(");
    println!("    query: &str, ");
    println!("    limit: usize");
    println!(") -> Result<Vec<SearchResult>, Error> {{");
    println!("    // Step 1: Generate query embedding");
    println!("    let query_embedding = nlp_engine.generate_embedding(query).await?;");
    println!("    ");
    println!("    // Step 2: Search EACH table separately");
    println!("    let mut all_results = Vec::new();");
    println!("    ");
    println!("    // Search date_nodes");
    println!("    let date_results = date_table.vector_search(");
    println!("        &query_embedding, limit/5");
    println!("    ).await?;");
    println!("    all_results.extend(date_results.into_iter().map(|r| {{");
    println!("        SearchResult {{ node: r.node, score: r.score, table: \"date\" }}");
    println!("    }}));");
    println!("    ");
    println!("    // Search text_nodes");
    println!("    let text_results = text_table.vector_search(");
    println!("        &query_embedding, limit/5");
    println!("    ).await?;");
    println!("    all_results.extend(text_results.into_iter().map(|r| {{");
    println!("        SearchResult {{ node: r.node, score: r.score, table: \"text\" }}");
    println!("    }}));");
    println!("    ");
    println!("    // Search project_nodes");
    println!("    let project_results = project_table.vector_search(");
    println!("        &query_embedding, limit/5");
    println!("    ).await?;");
    println!("    all_results.extend(project_results.into_iter().map(|r| {{");
    println!("        SearchResult {{ node: r.node, score: r.score, table: \"project\" }}");
    println!("    }}));");
    println!("    ");
    println!("    // Search task_nodes");
    println!("    let task_results = task_table.vector_search(");
    println!("        &query_embedding, limit/5");
    println!("    ).await?;");
    println!("    all_results.extend(task_results.into_iter().map(|r| {{");
    println!("        SearchResult {{ node: r.node, score: r.score, table: \"task\" }}");
    println!("    }}));");
    println!("    ");
    println!("    // Search document_nodes");
    println!("    let doc_results = document_table.vector_search(");
    println!("        &query_embedding, limit/5");
    println!("    ).await?;");
    println!("    all_results.extend(doc_results.into_iter().map(|r| {{");
    println!("        SearchResult {{ node: r.node, score: r.score, table: \"document\" }}");
    println!("    }}));");
    println!("    ");
    println!("    // Step 3: Merge and re-rank results across tables");
    println!("    all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());");
    println!("    all_results.truncate(limit);");
    println!("    ");
    println!("    // Step 4: Apply time filter 'last week' - COMPLEX!");
    println!("    let week_ago = Utc::now() - Duration::days(7);");
    println!("    all_results.retain(|r| {{");
    println!("        match r.table {{");
    println!("            \"date\" => r.node.created_at >= week_ago,");
    println!("            \"text\" => {{");
    println!("                // Need to find parent date node - requires JOIN!");
    println!("                if let Some(parent_id) = r.node.parent_id {{");
    println!("                    date_table.get_node(&parent_id).await");
    println!("                        .map(|parent| parent.created_at >= week_ago)");
    println!("                        .unwrap_or(false)");
    println!("                }} else {{ r.node.created_at >= week_ago }}");
    println!("            }},");
    println!("            \"project\" => r.node.created_at >= week_ago,");
    println!("            \"task\" => r.node.created_at >= week_ago,");
    println!("            \"document\" => r.node.created_at >= week_ago,");
    println!("        }}");
    println!("    }});");
    println!("    ");
    println!("    Ok(all_results)");
    println!("}}");
    println!("```");
    
    println!("\n❌ Problems with Separate Tables:");
    println!("   🔸 5 separate vector searches (5x latency)");
    println!("   🔸 5 separate vector indexes to maintain");
    println!("   🔸 Complex result merging and re-ranking");
    println!("   🔸 Time filter requires JOINs across tables");
    println!("   🔸 Inconsistent scoring across different indexes");
    println!("   🔸 Query planning becomes complex");
    println!("   🔸 Adding new node types = more tables to search");
    
    println!("\n🏗️ APPROACH 2: UNIVERSAL SCHEMA (Current)");
    println!("   Single table: universal_nodes");
    println!("   Schema: [id, node_type, content, parent_id, vector[384], created_at, metadata]");
    
    println!("\n🔍 NLP Query Implementation - UNIVERSAL SCHEMA:");
    println!("```rust");
    println!("async fn semantic_search_universal(");
    println!("    query: &str,");
    println!("    limit: usize");
    println!(") -> Result<Vec<SearchResult>, Error> {{");
    println!("    // Step 1: Generate query embedding");
    println!("    let query_embedding = nlp_engine.generate_embedding(query).await?;");
    println!("    ");
    println!("    // Step 2: Single vector search across ALL content");
    println!("    let results = universal_table.vector_search(");
    println!("        &query_embedding,");
    println!("        limit * 2  // Get extra for filtering");
    println!("    ).await?;");
    println!("    ");
    println!("    // Step 3: Apply time filter 'last week' - SIMPLE!");
    println!("    let week_ago = Utc::now() - Duration::days(7);");
    println!("    let filtered_results: Vec<_> = results.into_iter()");
    println!("        .filter(|r| {{");
    println!("            match r.node.node_type.as_str() {{");
    println!("                \"date\" => r.node.created_at >= week_ago,");
    println!("                \"text\" => {{");
    println!("                    // Simple parent lookup in SAME table");
    println!("                    if let Some(parent_id) = &r.node.parent_id {{");
    println!("                        // Parent is in same search space!");
    println!("                        universal_table.get_node(parent_id).await");
    println!("                            .map(|parent| parent.created_at >= week_ago)");
    println!("                            .unwrap_or(false)");
    println!("                    }} else {{ r.node.created_at >= week_ago }}");
    println!("                }},");
    println!("                _ => r.node.created_at >= week_ago,");
    println!("            }}");
    println!("        }})");
    println!("        .take(limit)");
    println!("        .collect();");
    println!("    ");
    println!("    Ok(filtered_results)");
    println!("}}");
    println!("```");
    
    println!("\n✅ Benefits of Universal Schema:");
    println!("   🔸 Single vector search (1x latency)");
    println!("   🔸 Single vector index to maintain");
    println!("   🔸 Consistent scoring across all content");
    println!("   🔸 Simple filtering and relationships");
    println!("   🔸 Cross-type semantic similarity");
    println!("   🔸 Easy to add new node types");
    
    println!("\n🎯 PERFORMANCE COMPARISON:");
    
    println!("\n   Query: 'Find product launch content from last week'");
    println!("   ");
    println!("   Separate Tables:");
    println!("   ├── Vector search date_nodes: ~50ms");
    println!("   ├── Vector search text_nodes: ~50ms");
    println!("   ├── Vector search project_nodes: ~50ms");
    println!("   ├── Vector search task_nodes: ~50ms");
    println!("   ├── Vector search document_nodes: ~50ms");
    println!("   ├── Merge results: ~10ms");
    println!("   ├── Time filtering with JOINs: ~30ms");
    println!("   └── TOTAL: ~290ms");
    println!("   ");
    println!("   Universal Schema:");
    println!("   ├── Single vector search: ~60ms");
    println!("   ├── Time filtering: ~5ms");
    println!("   └── TOTAL: ~65ms");
    println!("   ");
    println!("   🚀 Universal is ~4.5x faster!");
    
    println!("\n🧠 SEMANTIC SEARCH QUALITY:");
    
    println!("\n   Separate Tables:");
    println!("   ❌ 'Product launch' in date_nodes gets different score");
    println!("   ❌ 'Product launch' in text_nodes gets different score");
    println!("   ❌ Can't compare semantically similar content across types");
    println!("   ❌ Artificial score boundaries between tables");
    println!("   ");
    println!("   Universal Schema:");
    println!("   ✅ All 'product launch' content scored in same semantic space");
    println!("   ✅ Best matches rise to top regardless of type");
    println!("   ✅ Natural semantic similarity across content types");
    
    println!("\n🔮 COMPLEX NLP QUERIES:");
    
    println!("\n   Query: 'Show me tasks related to the marketing campaign we discussed yesterday'");
    println!("   ");
    println!("   Separate Tables Approach:");
    println!("   1. Search task_nodes for 'marketing campaign': 20 results");
    println!("   2. Search text_nodes for 'marketing campaign': 45 results");
    println!("   3. Search date_nodes for yesterday's date: 1 result");
    println!("   4. Find relationships between results (complex JOINs)");
    println!("   5. Cross-reference and filter");
    println!("   6. Re-rank merged results");
    println!("   → Complex, slow, inconsistent");
    println!("   ");
    println!("   Universal Schema Approach:");
    println!("   1. Single semantic search for 'marketing campaign tasks yesterday'");
    println!("   2. Filter by date and node_type if needed");
    println!("   3. Relationships preserved in same search space");
    println!("   → Simple, fast, semantically coherent");
    
    println!("\n🎯 RECOMMENDATION:");
    println!("   For NLP-heavy applications like NodeSpace:");
    println!("   ✅ Universal Schema WINS decisively");
    println!("   ");
    println!("   Use separate tables only if:");
    println!("   ❌ You rarely do cross-type searches");
    println!("   ❌ Types have completely different access patterns");
    println!("   ❌ You need type-specific optimizations");
    println!("   ❌ Compliance requires data separation");
    println!("   ");
    println!("   For NodeSpace use case:");
    println!("   ✅ Universal schema is clearly superior");
    println!("   ✅ Simpler code, better performance, better semantics");
    
    Ok(())
}