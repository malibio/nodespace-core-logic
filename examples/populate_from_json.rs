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
    before_sibling_id: Option<String>, // Support for sibling ordering
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonData {
    nodes: Vec<JsonNode>,
}

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🚀 Populating NodeSpace from JSON Template");
    println!("==========================================");

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
        service: "populate_from_json".to_string(),
    })?;

    let json_data: JsonData =
        serde_json::from_str(&json_content).map_err(|e| NodeSpaceError::InternalError {
            message: format!("Failed to parse JSON: {}", e),
            service: "populate_from_json".to_string(),
        })?;

    println!(
        "✅ Successfully loaded {} nodes from JSON",
        json_data.nodes.len()
    );

    // Create a mapping to track created nodes for parent relationship resolution
    let mut created_nodes: HashMap<String, bool> = HashMap::new();
    // Track the last sibling created for each parent (for automatic sibling ordering)
    let mut last_sibling_by_parent: HashMap<Option<String>, String> = HashMap::new();
    // Clone the nodes to avoid moving the original data
    let mut nodes_to_create = json_data.nodes.clone();

    // Sort nodes to ensure parents are created before children, but preserve sibling order
    nodes_to_create.sort_by(|a, b| {
        // Date nodes first (no parent_id)
        match (&a.parent_id, &b.parent_id) {
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            // For nodes with same parent status, preserve original JSON order
            _ => std::cmp::Ordering::Equal,
        }
    });

    // Extract date from the JSON template - use the date node's ID for all nodes
    let date = if let Some(date_node) = json_data.nodes.iter().find(|n| n.r#type == "date") {
        // Parse the date from the date node's ID (format: YYYY-MM-DD)
        let date_parts: Vec<&str> = date_node.id.split('-').collect();
        if date_parts.len() == 3 {
            let year = date_parts[0].parse::<i32>().unwrap_or(2025);
            let month = date_parts[1].parse::<u32>().unwrap_or(6);
            let day = date_parts[2].parse::<u32>().unwrap_or(26);
            NaiveDate::from_ymd_opt(year, month, day)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(2025, 6, 26).unwrap())
        } else {
            NaiveDate::from_ymd_opt(2025, 6, 26).unwrap()
        }
    } else {
        NaiveDate::from_ymd_opt(2025, 6, 26).unwrap()
    };

    // Pre-mark the date node as available since it will be auto-created by ensure_date_node_exists
    let date_str = date.format("%Y-%m-%d").to_string();
    created_nodes.insert(date_str, true);

    let total_nodes = nodes_to_create.len();
    let mut created_count = 0;
    let mut retry_queue = Vec::new();

    // First pass: create nodes whose parents are already available
    println!("\n🏗️  Creating nodes in dependency order");
    for json_node in nodes_to_create {
        println!(
            "🔍 Processing node: id='{}', type='{}', parent_id={:?}",
            json_node.id, json_node.r#type, json_node.parent_id
        );

        let can_create = match &json_node.parent_id {
            None => {
                println!("  ✅ Can create (no parent required)");
                true
            }
            Some(parent_id) => {
                let has_parent = created_nodes.contains_key(parent_id);
                println!("  📋 Parent check: {} -> {}", parent_id, has_parent);
                has_parent
            }
        };

        if can_create {
            println!("  🚀 Creating node...");
            create_node_from_json(&service, &json_node, &mut last_sibling_by_parent, date).await?;
            created_nodes.insert(json_node.id.clone(), true);
            created_count += 1;

            if created_count % 10 == 0 {
                println!("   📊 Created {}/{} nodes", created_count, total_nodes);
            }
        } else {
            retry_queue.push(json_node);
        }
    }

    // Retry pass: handle any nodes that couldn't be created in first pass
    let mut max_retries = 10;
    while !retry_queue.is_empty() && max_retries > 0 {
        let mut remaining_queue = Vec::new();
        let queue_size_before = retry_queue.len();

        for json_node in retry_queue {
            let can_create = match &json_node.parent_id {
                None => true,
                Some(parent_id) => created_nodes.contains_key(parent_id),
            };

            if can_create {
                create_node_from_json(&service, &json_node, &mut last_sibling_by_parent, date)
                    .await?;
                created_nodes.insert(json_node.id.clone(), true);
                created_count += 1;

                if created_count % 10 == 0 {
                    println!("   📊 Created {}/{} nodes", created_count, total_nodes);
                }
            } else {
                remaining_queue.push(json_node);
            }
        }

        // If we didn't make progress, break to avoid infinite loop
        if remaining_queue.len() == queue_size_before {
            println!(
                "   ⚠️  {} nodes couldn't be created due to missing parents",
                remaining_queue.len()
            );
            break;
        }

        retry_queue = remaining_queue;
        max_retries -= 1;
    }

    // Verify the data
    println!("\n🔍 Verifying JSON-based data population");
    let all_nodes = service.get_nodes_for_date(date).await?;

    println!("✅ Successfully populated database from JSON!");
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

    println!("\n💬 Database ready for comprehensive RAG queries!");
    println!("🎯 JSON-based population complete!");

    Ok(())
}

#[allow(clippy::implied_bounds_in_impls)]
async fn create_node_from_json(
    service: &NodeSpaceService<
        impl nodespace_data_store::DataStore + Send + Sync,
        impl nodespace_nlp_engine::NLPEngine + Send + Sync,
    >,
    json_node: &JsonNode,
    last_sibling_by_parent: &mut HashMap<Option<String>, String>,
    date: NaiveDate,
) -> NodeSpaceResult<()> {
    let node_id = NodeId::from_string(json_node.id.clone());
    let node_type = match json_node.r#type.as_str() {
        "date" => NodeType::Date,
        "text" => NodeType::Text,
        _ => {
            return Err(NodeSpaceError::InternalError {
                message: format!("Unknown node type: {}", json_node.r#type),
                service: "populate_from_json".to_string(),
            });
        }
    };

    let parent_id = json_node
        .parent_id
        .as_ref()
        .map(|p| NodeId::from_string(p.clone()));

    // Automatic sibling ordering: use the last sibling created for this parent
    let before_sibling_id = if let Some(explicit_before) = &json_node.before_sibling_id {
        // If JSON explicitly specifies before_sibling_id, use it
        Some(NodeId::from_string(explicit_before.clone()))
    } else {
        // Otherwise, automatically position after the last sibling for this parent
        last_sibling_by_parent
            .get(&json_node.parent_id)
            .map(|s| NodeId::from_string(s.clone()))
    };

    let content = json_node.content.as_deref().unwrap_or("");

    // Check if this is a date node by type string rather than enum comparison
    match json_node.r#type.as_str() {
        "date" => {
            service
                .create_node_for_date_with_id(
                    node_id,
                    date,
                    content,
                    node_type,
                    json_node.metadata.clone(),
                    parent_id,
                    before_sibling_id,
                )
                .await?;
        }
        "text" => {
            service
                .create_node_for_date_with_id(
                    node_id,
                    date,
                    content,
                    node_type,
                    json_node.metadata.clone(),
                    parent_id,
                    before_sibling_id,
                )
                .await?;
        }
        _ => {
            return Err(NodeSpaceError::InternalError {
                message: format!("Unknown node type: {}", json_node.r#type),
                service: "populate_from_json".to_string(),
            });
        }
    }

    // Update the last sibling tracker for this parent
    if json_node.r#type != "date" {
        // Don't track date nodes as siblings
        last_sibling_by_parent.insert(json_node.parent_id.clone(), json_node.id.clone());
    }

    Ok(())
}
