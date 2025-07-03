use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, HierarchyComputation, NodeSpaceService};
use nodespace_core_types::{NodeId, NodeSpaceError, NodeSpaceResult};
use nodespace_data_store::NodeType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonNode {
    id: String,
    r#type: String,
    content: Option<String>,
    metadata: Option<serde_json::Value>,
    parent_id: Option<String>,
    root_id: Option<String>,
    before_sibling_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonData {
    nodes: Vec<JsonNode>,
}

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🚀 Simple Population (No AI/Embeddings)");
    println!("=======================================");

    // Initialize service without AI initialization
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    // Skip AI initialization to speed up population
    println!("   ⚡ Skipping AI services for faster population");

    // Read and parse JSON file
    println!("\n📖 Reading JSON template file");
    let json_content = fs::read_to_string(
        "/Users/malibio/nodespace/data/sample-campaign-data.json",
    )
    .map_err(|e| NodeSpaceError::InternalError {
        message: format!("Failed to read JSON file: {}", e),
        service: "populate_simple".to_string(),
    })?;

    let json_data: JsonData =
        serde_json::from_str(&json_content).map_err(|e| NodeSpaceError::InternalError {
            message: format!("Failed to parse JSON: {}", e),
            service: "populate_simple".to_string(),
        })?;

    println!(
        "✅ Successfully loaded {} nodes from JSON",
        json_data.nodes.len()
    );

    // Just create a few key nodes for testing
    let date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();
    let mut created_count = 0;

    // Find nodes with specific content we care about for testing
    let key_nodes: Vec<&JsonNode> = json_data
        .nodes
        .iter()
        .filter(|n| {
            if let Some(content) = &n.content {
                content.contains("500,000 video views")
                    || content.contains("Product Launch Campaign Strategy")
                    || content.contains("Launch Overview")
                    || content.contains("Marketing Channel Strategy")
                    || content.contains("webinar attendees")
            } else {
                false
            }
        })
        .collect();

    println!("\n🎯 Creating {} key nodes for testing", key_nodes.len());

    for json_node in key_nodes {
        println!(
            "🔍 Creating: {}",
            json_node
                .content
                .as_ref()
                .unwrap_or(&json_node.id)
                .chars()
                .take(50)
                .collect::<String>()
        );

        let node_id = NodeId::from_string(json_node.id.clone());
        let node_type = NodeType::Text;
        let parent_id = json_node
            .parent_id
            .as_ref()
            .map(|p| NodeId::from_string(p.clone()));
        let content = json_node.content.as_deref().unwrap_or("");

        // Create node without embeddings
        service
            .create_node_for_date_with_id(
                node_id,
                date,
                content,
                node_type,
                json_node.metadata.clone(),
                parent_id,
                None, // No sibling ordering for speed
            )
            .await?;

        created_count += 1;
    }

    println!("\n✅ Created {} key nodes for testing", created_count);
    println!("🎯 Database ready for basic testing!");

    Ok(())
}
