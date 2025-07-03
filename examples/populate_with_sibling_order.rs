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
    r#type: String, // Using r#type because 'type' is a Rust keyword
    content: Option<String>,
    metadata: Option<serde_json::Value>,
    parent_id: Option<String>,
    root_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonData {
    nodes: Vec<JsonNode>,
}

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🚀 Populating NodeSpace with Proper Sibling Ordering");
    println!("==================================================");

    // Initialize service pointing to the existing database
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

    // Read and parse JSON file
    println!("\n📖 Reading JSON template file");
    let json_content = fs::read_to_string(
        "/Users/malibio/nodespace/data/sample-campaign-data.json",
    )
    .map_err(|e| NodeSpaceError::InternalError {
        message: format!("Failed to read JSON file: {}", e),
        service: "populate_with_sibling_order".to_string(),
    })?;

    let json_data: JsonData =
        serde_json::from_str(&json_content).map_err(|e| NodeSpaceError::InternalError {
            message: format!("Failed to parse JSON: {}", e),
            service: "populate_with_sibling_order".to_string(),
        })?;

    println!(
        "✅ Successfully loaded {} nodes from JSON",
        json_data.nodes.len()
    );

    // Group nodes by parent_id to maintain sibling order within each group
    println!("\n🏗️  Organizing nodes by parent for sibling ordering");
    let mut nodes_by_parent: HashMap<Option<String>, Vec<&JsonNode>> = HashMap::new();
    for node in &json_data.nodes {
        nodes_by_parent
            .entry(node.parent_id.clone())
            .or_insert_with(Vec::new)
            .push(node);
    }

    // Track created nodes
    let mut created_nodes: HashMap<String, NodeId> = HashMap::new();
    let total_nodes = json_data.nodes.len();
    let mut created_count = 0;

    // Process in levels: first root nodes, then their children, etc.
    let mut current_parents = vec![None]; // Start with root nodes (no parent)

    println!("🏗️  Creating nodes level by level with sibling ordering");

    while !current_parents.is_empty() && created_count < total_nodes {
        let mut next_level_parents = Vec::new();

        for parent_id_opt in current_parents {
            if let Some(siblings) = nodes_by_parent.get(&parent_id_opt) {
                // Verify parent exists (unless it's root level)
                let parent_exists = parent_id_opt.is_none()
                    || parent_id_opt
                        .as_ref()
                        .map_or(false, |pid| created_nodes.contains_key(pid));

                if !parent_exists && parent_id_opt.is_some() {
                    // Parent doesn't exist yet, skip this group
                    continue;
                }

                println!(
                    "   📁 Processing {} siblings under parent: {:?}",
                    siblings.len(),
                    parent_id_opt.as_deref().unwrap_or("ROOT")
                );

                // Create all siblings in order (sibling linking happens automatically in NodeSpace)
                for sibling in siblings.iter() {
                    // Skip if already created
                    if created_nodes.contains_key(&sibling.id) {
                        continue;
                    }

                    // Create the node (NodeSpace handles sibling ordering automatically)
                    create_node_from_json(&service, sibling).await?;
                    let current_node_id = NodeId::from_string(sibling.id.clone());
                    created_nodes.insert(sibling.id.clone(), current_node_id.clone());
                    created_count += 1;

                    next_level_parents.push(Some(sibling.id.clone()));

                    if created_count % 10 == 0 {
                        println!("   📊 Created {}/{} nodes", created_count, total_nodes);
                    }
                }
            }
        }

        current_parents = next_level_parents;
    }

    // Final verification
    println!("\n🔍 Verifying sibling-ordered data population");
    let campaign_date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();
    let all_nodes = service.get_nodes_for_date(campaign_date).await?;

    println!("✅ Successfully populated database with proper sibling ordering!");
    println!(
        "📊 Created {}/{} nodes successfully",
        created_count, total_nodes
    );
    println!("📊 Found {} nodes in date hierarchy", all_nodes.len());

    // Count nodes by type from JSON data
    let mut type_counts = HashMap::new();
    for node in &json_data.nodes {
        *type_counts.entry(node.r#type.clone()).or_insert(0) += 1;
    }

    println!("\n📈 Node breakdown from JSON template:");
    for (node_type, count) in type_counts.iter() {
        println!("   • {}: {} nodes", node_type, count);
    }

    println!("\n💬 Database ready with proper sibling ordering for RAG queries!");
    println!("🎯 Sibling-ordered population complete!");

    Ok(())
}

async fn create_node_from_json(
    service: &NodeSpaceService<
        impl nodespace_data_store::DataStore + Send + Sync,
        impl nodespace_nlp_engine::NLPEngine + Send + Sync,
    >,
    json_node: &JsonNode,
) -> NodeSpaceResult<()> {
    let node_id = NodeId::from_string(json_node.id.clone());
    let node_type = match json_node.r#type.as_str() {
        "date" => NodeType::Date,
        "text" => NodeType::Text,
        _ => {
            return Err(NodeSpaceError::InternalError {
                message: format!("Unknown node type: {}", json_node.r#type),
                service: "populate_with_sibling_order".to_string(),
            });
        }
    };

    let parent_id = json_node
        .parent_id
        .as_ref()
        .map(|p| NodeId::from_string(p.clone()));
    let content = json_node.content.as_deref().unwrap_or("");

    // Use the same date for all nodes (from the JSON template)
    let date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();

    service
        .create_node_for_date_with_id(
            node_id,
            date,
            content,
            node_type,
            json_node.metadata.clone(),
            parent_id,
        )
        .await?;

    Ok(())
}
