use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, NodeSpaceService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Checking for duplicate date nodes");

    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await?;

    let test_date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();
    let all_nodes = service.get_nodes_for_date(test_date).await?;

    println!("📊 All nodes for date {}:", test_date);
    let mut date_nodes = Vec::new();

    for node in &all_nodes {
        if node.r#type == "date" {
            date_nodes.push(node);
            println!(
                "📅 Date node found: ID={}, content={:?}",
                node.id, node.content
            );
        }
    }

    println!("\n📈 Summary:");
    println!("   Total nodes: {}", all_nodes.len());
    println!("   Date nodes: {}", date_nodes.len());

    if date_nodes.len() > 1 {
        println!("   ⚠️ DUPLICATE DATE NODES DETECTED!");
        for (i, node) in date_nodes.iter().enumerate() {
            println!(
                "      {}. ID: {}, content: {:?}",
                i + 1,
                node.id,
                node.content
            );
        }
    } else if date_nodes.len() == 1 {
        println!("   ✅ Exactly one date node (correct)");
    } else {
        println!("   ❌ No date nodes found");
    }

    Ok(())
}
