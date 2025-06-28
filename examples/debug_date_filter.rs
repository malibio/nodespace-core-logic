use nodespace_core_logic::{DateNavigation, NodeSpaceService, CoreLogic};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debugging get_nodes_for_date() Filter Logic");

    // Initialize service with production paths
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await.ok(); // Ignore AI init failures

    println!("\n1️⃣ Checking ALL nodes in database...");
    // Use semantic_search with empty query to get all nodes
    let search_results = service.semantic_search("", 1000).await?;
    let all_nodes: Vec<_> = search_results.into_iter().map(|r| r.node).collect();
    println!("   Total nodes in database: {}", all_nodes.len());

    if all_nodes.is_empty() {
        println!("   ❌ Database is empty - this explains the issue!");
        return Ok(());
    }

    println!("\n2️⃣ Analyzing node timestamps and dates...");
    let today = chrono::Utc::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();
    
    for (i, node) in all_nodes.iter().enumerate().take(10) {
        println!("   Node {}: ID={}", i + 1, node.id);
        println!("      created_at: {}", node.created_at);
        
        // Test RFC3339 parsing
        match chrono::DateTime::parse_from_rfc3339(&node.created_at) {
            Ok(parsed_dt) => {
                let node_date = parsed_dt.date_naive();
                println!("      parsed date: {}", node_date);
                println!("      today's date: {}", today);
                println!("      dates match: {}", node_date == today);
            }
            Err(e) => {
                println!("      ❌ RFC3339 parse failed: {}", e);
                // Test fallback logic
                let fallback_match = node.created_at.starts_with(&today_str);
                println!("      fallback match: {}", fallback_match);
            }
        }
        
        // Check parent_id status
        if let Some(metadata) = &node.metadata {
            if let Some(parent_id) = metadata.get("parent_id") {
                println!("      has parent_id: {} (WILL BE FILTERED OUT)", parent_id);
            } else {
                println!("      no parent_id (top-level node)");
            }
        } else {
            println!("      no metadata (top-level node)");
        }
        
        // Show content preview
        if let Some(content) = node.content.as_str() {
            let preview = content.chars().take(50).collect::<String>();
            println!("      content: \"{}...\"", preview);
        }
        println!();
    }

    println!("\n3️⃣ Testing get_nodes_for_date() with today ({})...", today);
    let today_nodes = service.get_nodes_for_date(today).await?;
    println!("   Nodes returned: {}", today_nodes.len());

    if today_nodes.is_empty() {
        println!("   ❌ No nodes returned - investigating why...");
        
        // Manual filter simulation
        println!("\n   🔍 Manual filter simulation:");
        let mut date_matches = 0;
        let mut top_level_count = 0;
        let mut both_match = 0;
        
        for node in &all_nodes {
            // Test date filter
            let matches_date = if let Ok(node_datetime) = chrono::DateTime::parse_from_rfc3339(&node.created_at) {
                node_datetime.date_naive() == today
            } else {
                node.created_at.starts_with(&today_str)
            };
            
            if matches_date {
                date_matches += 1;
            }
            
            // Test top-level filter
            let is_top_level = if let Some(metadata) = &node.metadata {
                metadata.get("parent_id").is_none()
            } else {
                true
            };
            
            if is_top_level {
                top_level_count += 1;
            }
            
            if matches_date && is_top_level {
                both_match += 1;
                println!("      ✅ Node {} matches both filters", node.id);
                if let Some(content) = node.content.as_str() {
                    let preview = content.chars().take(30).collect::<String>();
                    println!("         content: \"{}...\"", preview);
                }
            }
        }
        
        println!("   📊 Filter results:");
        println!("      Date matches: {}", date_matches);
        println!("      Top-level nodes: {}", top_level_count);
        println!("      Both filters match: {}", both_match);
        
        if date_matches == 0 {
            println!("   🎯 ISSUE: No nodes have today's date");
            println!("      Solution: Create nodes with today's date");
        } else if both_match == 0 && top_level_count == 0 {
            println!("   🎯 ISSUE: All nodes have parent_id (no top-level nodes)");
        } else {
            println!("   🎯 ISSUE: Unknown filter problem");
        }
    } else {
        println!("   ✅ Found {} nodes for today", today_nodes.len());
        for node in &today_nodes {
            if let Some(content) = node.content.as_str() {
                let preview = content.chars().take(50).collect::<String>();
                println!("      Node {}: \"{}...\"", node.id, preview);
            }
        }
    }

    println!("\n4️⃣ Testing with different dates...");
    let yesterday = today - chrono::Duration::days(1);
    let yesterday_nodes = service.get_nodes_for_date(yesterday).await?;
    println!("   Yesterday ({}): {} nodes", yesterday, yesterday_nodes.len());

    let tomorrow = today + chrono::Duration::days(1);
    let tomorrow_nodes = service.get_nodes_for_date(tomorrow).await?;
    println!("   Tomorrow ({}): {} nodes", tomorrow, tomorrow_nodes.len());

    println!("\n🎯 DIAGNOSIS COMPLETE");
    println!("   - Check the filter analysis above to identify the issue");
    println!("   - Most likely: No nodes created today OR all nodes have parent_id");

    Ok(())
}