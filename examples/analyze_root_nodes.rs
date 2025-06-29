use futures::TryStreamExt;
use lancedb::{
    connect,
    query::{ExecutableQuery, QueryBase},
};
use std::collections::HashSet;

/// Analyze root nodes in the database
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌳 Analyzing Root Nodes in Database\n");

    // Connect to the database
    let db_path = "../data/lance_db/e2e_sample.db";
    let db = connect(db_path).execute().await?;
    let table = db.open_table("universal_nodes").execute().await?;

    // Get all nodes
    let all_query = table.query().limit(1000).execute().await?;
    let all_batches = all_query.try_collect::<Vec<_>>().await?;

    println!("📊 Total batches: {}", all_batches.len());

    let mut total_nodes = 0;
    let mut nodes_with_parents = HashSet::new();
    let mut all_node_ids = HashSet::new();
    let mut parent_child_relationships = Vec::new();
    let mut date_nodes = Vec::new();
    let mut potential_root_nodes = Vec::new();

    // First pass: collect all node IDs and parent relationships
    for batch in &all_batches {
        total_nodes += batch.num_rows();

        for row_idx in 0..batch.num_rows() {
            // Get node ID
            let id = if let Some(id_col) = batch.column_by_name("id") {
                format!("{:?}", id_col.slice(row_idx, 1))
                    .trim_matches('"')
                    .replace("StringArray\n[\n  \"", "")
                    .replace("\",\n]", "")
            } else {
                continue;
            };

            all_node_ids.insert(id.clone());

            // Get parent ID
            let parent_id = if let Some(parent_col) = batch.column_by_name("parent_id") {
                let parent_str = format!("{:?}", parent_col.slice(row_idx, 1));
                if parent_str.contains("null") || parent_str.contains("None") {
                    None
                } else {
                    let cleaned = parent_str
                        .trim_matches('"')
                        .replace("StringArray\n[\n  \"", "")
                        .replace("\",\n]", "");
                    if cleaned.is_empty() || cleaned == "null" {
                        None
                    } else {
                        Some(cleaned)
                    }
                }
            } else {
                None
            };

            // Get content to identify date nodes
            let content = if let Some(content_col) = batch.column_by_name("content") {
                format!("{:?}", content_col.slice(row_idx, 1))
                    .replace("StringArray\n[\n  \"", "")
                    .replace("\",\n]", "")
                    .replace("\\\"", "\"")
            } else {
                String::new()
            };

            // Get node type
            let node_type = if let Some(type_col) = batch.column_by_name("node_type") {
                format!("{:?}", type_col.slice(row_idx, 1))
                    .replace("StringArray\n[\n  \"", "")
                    .replace("\",\n]", "")
            } else {
                String::new()
            };

            // Check if this looks like a date node (YYYY-MM-DD format)
            if content.len() >= 8
                && content.contains("-")
                && (content.matches("-").count() == 2 || content.contains("2025"))
            {
                date_nodes.push((id.clone(), content.clone(), parent_id.clone()));
            }

            // Track parent-child relationships
            if let Some(parent) = &parent_id {
                nodes_with_parents.insert(id.clone());
                parent_child_relationships.push((parent.clone(), id.clone(), content.clone()));
            } else {
                potential_root_nodes.push((id.clone(), content.clone(), node_type.clone()));
            }

            println!(
                "📄 Node: {} | Type: {} | Parent: {:?} | Content: {}",
                id,
                node_type,
                parent_id,
                content.chars().take(60).collect::<String>()
            );
        }
    }

    println!("\n🔍 Analysis Results:");
    println!("   📊 Total nodes: {}", total_nodes);
    println!("   👨‍👩‍👧‍👦 Nodes with parents: {}", nodes_with_parents.len());
    println!(
        "   🌳 Potential root nodes (no parent): {}",
        potential_root_nodes.len()
    );
    println!("   📅 Date-like nodes: {}", date_nodes.len());

    println!("\n📅 Date Nodes Found:");
    for (id, content, parent) in &date_nodes {
        println!(
            "   📅 ID: {} | Content: {} | Parent: {:?}",
            id, content, parent
        );
    }

    println!("\n🌳 Root Nodes (No Parent):");
    for (id, content, node_type) in &potential_root_nodes {
        println!(
            "   🌲 ID: {} | Type: {} | Content: {}",
            id,
            node_type,
            content.chars().take(80).collect::<String>()
        );
    }

    // Find true root nodes (nodes that are not children of any other node)
    let mut true_root_nodes = Vec::new();
    for node_id in &all_node_ids {
        let is_child = parent_child_relationships
            .iter()
            .any(|(_, child_id, _)| child_id == node_id);
        let has_parent = nodes_with_parents.contains(node_id);

        if !has_parent && !is_child {
            // This is a true root - find its content
            for (id, content, node_type) in &potential_root_nodes {
                if id == node_id {
                    true_root_nodes.push((id.clone(), content.clone(), node_type.clone()));
                    break;
                }
            }
        }
    }

    println!("\n🎯 True Root Nodes Analysis:");
    println!(
        "   🌲 Nodes with no parent field set: {}",
        potential_root_nodes.len()
    );
    println!("   🌳 True isolated roots: {}", true_root_nodes.len());

    // Check for date nodes that could be parents of content
    println!("\n🔗 Hierarchy Analysis:");
    for (parent, child, content) in &parent_child_relationships {
        // Check if parent is a date-like node
        let parent_is_date = date_nodes.iter().any(|(date_id, _, _)| date_id == parent);

        if parent_is_date {
            println!(
                "   📅➡️📄 Date parent: {} → Child: {} ({})",
                parent,
                child,
                content.chars().take(50).collect::<String>()
            );
        }
    }

    // Look for nodes that might be direct children of date nodes
    let mut direct_date_children = 0;
    for (parent, child, content) in &parent_child_relationships {
        for (date_id, _, _) in &date_nodes {
            if parent == date_id {
                direct_date_children += 1;
                println!(
                    "   🎯 Direct date child: {} under date {} | Content: {}",
                    child,
                    date_id,
                    content.chars().take(60).collect::<String>()
                );
            }
        }
    }

    println!("\n📊 Final Root Node Count:");
    println!(
        "   🌲 Nodes with no parent_id: {}",
        potential_root_nodes.len()
    );
    println!(
        "   📅 Direct children of date nodes: {}",
        direct_date_children
    );
    println!("   🌳 True orphaned roots: {}", true_root_nodes.len());

    let total_root_like = potential_root_nodes.len() + direct_date_children;
    println!("   🎯 TOTAL ROOT-LEVEL NODES: {}", total_root_like);

    Ok(())
}
