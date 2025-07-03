use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use nodespace_core_types::{NodeId, NodeSpaceResult};

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🔍 Debugging Sibling Order in Database");
    println!("=====================================");

    // Initialize service
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    match service.initialize().await {
        Ok(_) => println!("   ✅ AI services ready"),
        Err(e) => println!("   ⚠️  AI warning: {} (continuing)", e),
    }

    // Get nodes for the campaign date
    let campaign_date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();
    let all_nodes = service.get_nodes_for_date(campaign_date).await?;

    println!(
        "\n📊 Found {} nodes for date {}",
        all_nodes.len(),
        campaign_date
    );

    // Debug: Show all nodes returned
    println!("\n🔍 All nodes returned by get_nodes_for_date:");
    for (i, node) in all_nodes.iter().enumerate() {
        let content = node.content.as_str().unwrap_or("");
        println!(
            "  {}. 🆔 {} - Type: {} - Content: \"{}\"",
            i + 1,
            node.id,
            node.r#type,
            content.chars().take(100).collect::<String>()
        );
        println!(
            "     👨‍👩‍👧‍👦 Parent: {:?}, 🌳 Root: {:?}",
            node.parent_id, node.root_id
        );
        println!("     🔗 before_sibling: {:?}", node.before_sibling);
    }

    // Since all nodes are direct children of date node, let's examine their sibling order
    let date_node_children: Vec<_> = all_nodes
        .iter()
        .filter(|n| n.parent_id.as_ref() == Some(&NodeId::from_string("2025-06-26".to_string())))
        .filter(|n| n.id.as_str() != "2025-06-26") // Exclude the date node itself
        .collect();

    println!(
        "\n👨‍👩‍👧‍👦 Date Node Children ({} found):",
        date_node_children.len()
    );
    for (i, child) in date_node_children.iter().enumerate() {
        let content = child.content.as_str().unwrap_or("");
        let title = if content.starts_with('#') {
            content
                .lines()
                .next()
                .unwrap_or(content)
                .trim_start_matches('#')
                .trim()
                .to_string()
        } else {
            content.chars().take(50).collect::<String>()
        };

        println!("  {}. 🆔 {} - \"{}\"", i + 1, child.id, title);
        println!("     🔗 before_sibling: {:?}", child.before_sibling);
        println!("     📅 created_at: {}", child.created_at);
        println!();
    }

    Ok(())
}
