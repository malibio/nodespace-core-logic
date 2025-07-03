use nodespace_core_logic::{CoreLogic, NodeSpaceService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Checking all nodes in database");

    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await?;

    // Try to get all nodes by searching for a broad query
    let results = service.semantic_search("", 200).await?;

    println!("📊 Found {} nodes total", results.len());

    let mut type_counts = std::collections::HashMap::new();
    let mut date_nodes = Vec::new();

    for result in &results {
        let node = &result.node;
        *type_counts.entry(node.r#type.clone()).or_insert(0) += 1;

        if node.r#type == "date" {
            date_nodes.push(node);
        }

        // Show first few nodes for inspection
        if results.len() <= 10 {
            println!(
                "   Node: ID={}, type='{}', content={:?}",
                node.id,
                node.r#type,
                node.content
                    .as_str()
                    .map(|s| s.chars().take(50).collect::<String>())
            );
        }
    }

    println!("\n📈 Node types breakdown:");
    for (node_type, count) in type_counts {
        println!("   {}: {} nodes", node_type, count);
    }

    if !date_nodes.is_empty() {
        println!("\n📅 Date nodes found:");
        for (i, node) in date_nodes.iter().enumerate() {
            println!("   {}. ID: {}, content: {:?}", i + 1, node.id, node.content);
        }
    }

    Ok(())
}
