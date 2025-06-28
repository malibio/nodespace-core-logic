use nodespace_core_logic::{DateNavigation, NodeSpaceService, CoreLogic};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debugging Date Node Matching");

    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await.ok();

    let today = chrono::Utc::now().date_naive();
    
    // Test all the date format matching logic
    println!("\n1️⃣ Date format testing:");
    let date_str = today.format("%Y-%m-%d").to_string();
    let date_header = format!("# {}", today.format("%B %-d, %Y"));
    
    println!("   date_str: '{}'", date_str);
    println!("   date_header: '{}'", date_header);
    println!("   Alternative format: '# {}'", date_str);

    // Get all nodes and manually test the matching
    let search_results = service.semantic_search("", 100).await?;
    let all_nodes: Vec<_> = search_results.into_iter().map(|r| r.node).collect();
    
    println!("\n2️⃣ Testing against actual node content:");
    let mut found_date_node = None;
    
    for node in &all_nodes {
        if let Some(content) = node.content.as_str() {
            // Apply same logic as the fixed function - remove quotes and trim
            let clean_content = content.trim().trim_matches('"').trim();
            
            // Test all matching conditions
            let match1 = clean_content == date_header;
            let match2 = clean_content.starts_with(&format!("# {}", date_str));
            let match3 = clean_content == format!("# {}", date_str);
            
            if match1 || match2 || match3 {
                println!("   ✅ FOUND DATE NODE: {}", node.id);
                println!("      Content: '{}'", clean_content);
                println!("      match1 (full format): {}", match1);
                println!("      match2 (starts with): {}", match2);
                println!("      match3 (exact date): {}", match3);
                found_date_node = Some(&node.id);
                break;
            } else if clean_content.contains("June") || clean_content.contains("2025") {
                println!("   📅 Date-related node: {} -> '{}'", node.id, clean_content);
                println!("      match1: {} (expected: '{}')", match1, date_header);
                println!("      match2: {} (expected starts with: '# {}')", match2, date_str);
                println!("      match3: {} (expected: '# {}')", match3, date_str);
            }
        }
    }

    if let Some(date_node_id) = found_date_node {
        println!("\n3️⃣ Looking for children of date node {}:", date_node_id);
        
        let mut children_found = 0;
        for node in &all_nodes {
            if let Some(metadata) = &node.metadata {
                if let Some(parent_id) = metadata.get("parent_id") {
                    if let Some(parent_str) = parent_id.as_str() {
                        if parent_str == date_node_id.to_string() {
                            children_found += 1;
                            if let Some(content) = node.content.as_str() {
                                let preview = content.chars().take(50).collect::<String>();
                                println!("   ✅ Child {}: {} -> '{}'", children_found, node.id, preview);
                            }
                        }
                    }
                }
            }
        }
        
        println!("   Total direct children: {}", children_found);
        
        if children_found == 0 {
            println!("   ❌ No direct children found!");
            println!("   🔍 Let's check ALL parent_id values:");
            
            let mut all_parents = std::collections::HashSet::new();
            for node in &all_nodes {
                if let Some(metadata) = &node.metadata {
                    if let Some(parent_id) = metadata.get("parent_id") {
                        if let Some(parent_str) = parent_id.as_str() {
                            all_parents.insert(parent_str.to_string());
                        }
                    }
                }
            }
            
            println!("   All unique parent_ids in database:");
            for parent_id in &all_parents {
                println!("      - {}", parent_id);
            }
            
            println!("   Looking for: {}", date_node_id.to_string());
            println!("   Match found in parent_ids: {}", all_parents.contains(&date_node_id.to_string()));
        }
    } else {
        println!("\n❌ DATE NODE NOT FOUND!");
        
        // Show what we're looking for vs what exists
        println!("   Looking for any of:");
        println!("      - '{}'", date_header);
        println!("      - content starting with '# {}'", date_str);
        println!("      - exact match '# {}'", date_str);
        
        println!("   Found these potential matches:");
        for node in all_nodes.iter().take(5) {
            if let Some(content) = node.content.as_str() {
                println!("      {} -> '{}'", node.id, content.trim());
            }
        }
    }

    // Test the actual function
    println!("\n4️⃣ Testing actual get_nodes_for_date function:");
    let result_nodes = service.get_nodes_for_date(today).await?;
    println!("   Function returned: {} nodes", result_nodes.len());

    Ok(())
}