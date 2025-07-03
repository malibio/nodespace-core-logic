use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, HierarchyComputation, NodeSpaceService};
use nodespace_core_types::{NodeId, NodeSpaceResult};

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🔍 Verifying Sibling Order in Database");
    println!("======================================");

    // Initialize service
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    // Initialize AI services
    match service.initialize().await {
        Ok(_) => println!("   ✅ AI services ready"),
        Err(e) => println!("   ⚠️  AI warning: {} (continuing)", e),
    }

    let campaign_date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();

    // Get the date node
    println!("\n📅 Finding date node for {}", campaign_date);
    if let Some(date_node_id) = service.find_date_node(campaign_date).await? {
        println!("✅ Found date node: {}", date_node_id);

        // Get all direct children of the date node
        println!("\n👥 Getting direct children of date node");
        let date_children = service.get_children(&date_node_id).await?;
        println!(
            "📊 Found {} direct children of date node",
            date_children.len()
        );

        // Print each child and check their sibling links
        for (index, child) in date_children.iter().enumerate() {
            println!("\n   📄 Child {}: {}", index + 1, child.id);
            if let Some(content) = child.content.as_str() {
                println!(
                    "      📝 Content: {}",
                    content.chars().take(80).collect::<String>()
                );
            }

            // Check next sibling
            if let Some(next_sibling_id) = &child.next_sibling {
                println!("      ➡️  Next sibling: {}", next_sibling_id);
            } else {
                println!("      ⏹️  No next sibling (end of chain)");
            }

            // Get children of this child to see deeper structure
            let grandchildren = service.get_children(&child.id).await?;
            if !grandchildren.is_empty() {
                println!("      👶 Has {} children", grandchildren.len());

                // Check sibling order within grandchildren
                for (gc_index, grandchild) in grandchildren.iter().take(3).enumerate() {
                    println!("         📄 Grandchild {}: {}", gc_index + 1, grandchild.id);
                    if let Some(gc_content) = grandchild.content.as_str() {
                        println!(
                            "            📝 Content: {}",
                            gc_content.chars().take(60).collect::<String>()
                        );
                    }
                    if let Some(next_gc) = &grandchild.next_sibling {
                        println!("            ➡️  Next: {}", next_gc);
                    }
                }
                if grandchildren.len() > 3 {
                    println!("         ... and {} more", grandchildren.len() - 3);
                }
            }
        }

        // Specific test: Check if "Launch Overview" comes before "Executive Summary"
        println!("\n🧪 Testing specific sibling ordering");
        let children_contents: Vec<String> = date_children
            .iter()
            .filter_map(|child| child.content.as_str())
            .map(|s| s.to_string())
            .collect();

        println!("📋 Order of main sections:");
        for (index, content) in children_contents.iter().enumerate() {
            if content.starts_with("##") || content.starts_with("#") {
                println!("   {}. {}", index + 1, content);
            }
        }

        // Test sibling chain traversal
        println!("\n🔗 Testing sibling chain traversal");
        if let Some(first_child) = date_children.first() {
            let mut current_id = Some(first_child.id.clone());
            let mut chain_count = 0;

            while let Some(node_id) = current_id {
                chain_count += 1;
                // We need to use the public get_children method instead of direct data store access
                // For now, just show the siblings we already have
                if let Some(node) = date_children.iter().find(|n| n.id == node_id) {
                    if let Some(content) = node.content.as_str() {
                        println!(
                            "   {}. {} -> {:?}",
                            chain_count,
                            content.chars().take(50).collect::<String>(),
                            node.next_sibling
                        );
                    }
                    current_id = node.next_sibling.clone();
                } else {
                    break;
                }

                // Safety break
                if chain_count > 20 {
                    println!("   ... (stopping at 20 to avoid potential loops)");
                    break;
                }
            }

            println!("🔗 Sibling chain length: {}", chain_count);
            if chain_count == date_children.len() {
                println!("✅ Sibling chain covers all children!");
            } else {
                println!(
                    "⚠️  Sibling chain mismatch: {} in chain vs {} total children",
                    chain_count,
                    date_children.len()
                );
            }
        }
    } else {
        println!("❌ No date node found for {}", campaign_date);
    }

    println!("\n🎯 Sibling order verification complete!");
    Ok(())
}
