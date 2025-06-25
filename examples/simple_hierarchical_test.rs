// Simple test of hierarchical logic with sample data
use nodespace_core_logic::HierarchicalNode;
use nodespace_core_types::{Node, NodeId};
use serde_json::json;

fn main() {
    println!("🔍 Testing hierarchical logic with sample data...\n");

    // Simulate the nodes we get from get_nodes_for_date()
    let nodes = vec![
        Node::with_id(
            NodeId::from_string("text1".to_string()),
            json!("Try creating your own notes and exploring the features."),
        )
        .with_metadata(json!({"parent_date": "2025-06-25"})),
        Node::with_id(
            NodeId::from_string("text2".to_string()),
            json!("The SurrealDB 2.x upgrade is working perfectly!"),
        )
        .with_metadata(json!({"parent_date": "2025-06-25"})),
        Node::with_id(
            NodeId::from_string("text3".to_string()),
            json!("Welcome to NodeSpace! This is a sample note for today."),
        )
        .with_metadata(json!({"parent_date": "2025-06-25"})),
    ];

    println!("📄 Input nodes:");
    for (i, node) in nodes.iter().enumerate() {
        println!(
            "   {}. {} (parent_date: {:?})",
            i + 1,
            node.id,
            node.metadata
                .as_ref()
                .and_then(|m| m.get("parent_date"))
                .and_then(|v| v.as_str())
        );
    }

    // Simulate the hierarchical processing logic
    let mut hierarchical_nodes = Vec::new();

    for node in nodes {
        if let Some(metadata) = &node.metadata {
            if let Some(parent_date) = metadata.get("parent_date").and_then(|v| v.as_str()) {
                println!(
                    "\n🔗 Processing node {} with parent_date: {}",
                    node.id, parent_date
                );

                let hierarchical_node = HierarchicalNode {
                    node: node.clone(),
                    children: Vec::new(), // Text nodes typically don't have children
                    parent: Some(NodeId::from(parent_date)),
                    depth_level: 1, // Child of date node
                    order_in_parent: hierarchical_nodes.len() as u32,
                    relationship_type: Some("contains".to_string()),
                };

                hierarchical_nodes.push(hierarchical_node);
            }
        }
    }

    println!(
        "\n📊 Result: {} hierarchical nodes",
        hierarchical_nodes.len()
    );
    for (i, h_node) in hierarchical_nodes.iter().enumerate() {
        println!(
            "   {}. {} (parent: {:?}, depth: {})",
            i + 1,
            h_node.node.id,
            h_node.parent,
            h_node.depth_level
        );
    }

    if hierarchical_nodes.is_empty() {
        println!("\n❌ Something is wrong with the hierarchical processing logic!");
    } else {
        println!("\n✅ Hierarchical processing should work!");
    }
}
