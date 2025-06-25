//! Sample data creation for NodeSpace with hierarchical relationships
//!
//! This example creates sample data with all the core features:
//! - Hierarchical parent-child relationships
//! - Bullet point cleaning for child nodes  
//! - Proper sibling ordering
//! - SurrealDB relationship records

use nodespace_core_types::NodeId;
use nodespace_data_store::DataStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Creating NodeSpace sample data...");
    println!("📊 Generating hierarchical data with bullet point cleaning...\n");

    // Initialize the data store
    let data_store = nodespace_data_store::SurrealDataStore::new(
        "/Users/malibio/nodespace/nodespace-data-store/data/sample.db",
    )
    .await?;

    // Create some sample data with hierarchical structure
    let mut total_entries = 0;
    let mut test_parent_id: Option<NodeId> = None; // Capture first parent for relationship testing

    // Create a few days of sample data
    let dates = vec!["2025-06-23", "2025-06-24", "2025-06-25"];

    for date_str in dates {
        println!("📅 Creating entries for {}", date_str);

        // Create date node
        let _date_node = data_store
            .create_or_get_date_node(date_str, Some("Sample marketing day"))
            .await?;

        // Create some entries with bullet points to test hierarchical processing
        let sample_contents = vec![
            "Weekly marketing review completed with strong results:\n• Email campaign achieved 34% open rate\n• Social media engagement up 23%\n• New lead generation exceeded targets\n• Content performance metrics reviewed",
            "Strategic planning session outcomes:\n• Q3 budget allocation finalized\n• New partnership opportunities identified\n• Brand messaging updates approved\n• Team capacity planning completed",
            "Simple entry without bullet points - this should remain as a single node with no children.",
            "Campaign optimization meeting results:\n• A/B testing results analyzed\n• Landing page conversions improved\n• Creative assets updated\n• Performance tracking enhanced",
        ];

        for (i, content) in sample_contents.iter().enumerate() {
            println!(
                "  📝 Processing entry {}: {}",
                i + 1,
                content.lines().next().unwrap_or("")
            );

            // Process manually to show the core-logic approach
            if content.contains("•") {
                // Split into parent and children
                let lines: Vec<&str> = content.split('\n').collect();
                let mut parent_content = String::new();
                let mut children = Vec::new();

                for line in lines {
                    let trimmed = line.trim();
                    if trimmed.starts_with("•") {
                        // Clean bullet point
                        let clean_child = trimmed.trim_start_matches("•").trim().to_string();
                        if !clean_child.is_empty() {
                            children.push(clean_child);
                        }
                    } else if !trimmed.is_empty() {
                        if !parent_content.is_empty() {
                            parent_content.push('\n');
                        }
                        parent_content.push_str(trimmed);
                    }
                }

                if !parent_content.is_empty() && !children.is_empty() {
                    // Create parent node using data store directly
                    let parent_id = data_store
                        .create_text_node(&parent_content, Some(date_str))
                        .await?;

                    // Verify parent node was created successfully
                    match data_store.get_node(&parent_id).await {
                        Ok(Some(_)) => {
                            println!(
                                "    📄 Created parent: '{}' (ID: {})",
                                parent_content.chars().take(50).collect::<String>() + "...",
                                parent_id
                            );
                        }
                        Ok(None) => {
                            println!("    ❌ ERROR: Parent node creation failed - node not found after creation");
                            continue;
                        }
                        Err(e) => {
                            println!("    ❌ ERROR: Failed to verify parent node: {}", e);
                            continue;
                        }
                    }

                    // Capture first parent for testing
                    if test_parent_id.is_none() {
                        test_parent_id = Some(parent_id.clone());
                    }

                    total_entries += 1;

                    // Create children and link them with validation
                    let mut previous_child_id: Option<NodeId> = None;
                    let mut created_child_ids = Vec::new();

                    for (child_idx, child_content) in children.iter().enumerate() {
                        // Create child node with clean content (no bullet points)
                        let child_id = data_store
                            .create_text_node(child_content, Some(date_str))
                            .await?;

                        // Verify child node was created successfully
                        match data_store.get_node(&child_id).await {
                            Ok(Some(_)) => {
                                println!(
                                    "      📝 Created child {}: '{}' (ID: {})",
                                    child_idx + 1,
                                    child_content,
                                    child_id
                                );
                                created_child_ids.push(child_id.clone());
                            }
                            Ok(None) => {
                                println!("      ❌ ERROR: Child node creation failed - node not found after creation");
                                continue;
                            }
                            Err(e) => {
                                println!("      ❌ ERROR: Failed to verify child node: {}", e);
                                continue;
                            }
                        }

                        // Only create relationships if both parent and child nodes exist
                        println!(
                            "      🔗 Creating parent-child relationship: {} -> {}",
                            parent_id, child_id
                        );
                        match data_store
                            .create_relationship(&parent_id, &child_id, "contains")
                            .await
                        {
                            Ok(_) => println!("        ✅ Parent-child relationship created"),
                            Err(e) => println!(
                                "        ❌ Failed to create parent-child relationship: {}",
                                e
                            ),
                        }

                        // Create sibling relationships only if we have valid nodes
                        if let Some(prev_id) = &previous_child_id {
                            println!(
                                "      🔗 Creating sibling relationships: {} <-> {}",
                                prev_id, child_id
                            );

                            // Link to previous sibling
                            match data_store
                                .create_relationship(prev_id, &child_id, "next_sibling")
                                .await
                            {
                                Ok(_) => println!("        ✅ Next sibling relationship created"),
                                Err(e) => println!(
                                    "        ❌ Failed to create next sibling relationship: {}",
                                    e
                                ),
                            }

                            match data_store
                                .create_relationship(&child_id, prev_id, "previous_sibling")
                                .await
                            {
                                Ok(_) => {
                                    println!("        ✅ Previous sibling relationship created")
                                }
                                Err(e) => println!(
                                    "        ❌ Failed to create previous sibling relationship: {}",
                                    e
                                ),
                            }

                            // Update the previous child node's next_sibling pointer
                            if let Ok(Some(mut prev_node)) = data_store.get_node(prev_id).await {
                                prev_node.next_sibling = Some(child_id.clone());
                                prev_node.updated_at = chrono::Utc::now().to_rfc3339();
                                let _ = data_store.store_node(prev_node).await;
                            }
                        }

                        // Update the current child node's previous_sibling pointer
                        if let Ok(Some(mut child_node)) = data_store.get_node(&child_id).await {
                            child_node.previous_sibling = previous_child_id.clone();
                            child_node.updated_at = chrono::Utc::now().to_rfc3339();
                            let _ = data_store.store_node(child_node).await;
                        }

                        previous_child_id = Some(child_id);
                        total_entries += 1;
                    }

                    // Verify all relationships were created correctly
                    println!("    🧪 Verifying parent-child relationships...");
                    let clean_parent_id = parent_id.as_str().replace("-", "_");
                    let verify_query = format!("SELECT * FROM nodes:{}->contains", clean_parent_id);

                    match data_store.query_nodes(&verify_query).await {
                        Ok(relations) => {
                            println!("      ✅ Found {} relationship records", relations.len());

                            // Try to verify each relationship points to an actual child node
                            let mut valid_relationships = 0;
                            for (i, rel) in relations.iter().enumerate() {
                                match data_store.get_node(&rel.id).await {
                                    Ok(Some(_)) => {
                                        valid_relationships += 1;
                                        println!(
                                            "        ✅ Relationship {}: Valid target node {}",
                                            i + 1,
                                            rel.id
                                        );
                                    }
                                    Ok(None) => {
                                        println!("        ❌ Relationship {}: INVALID - target node {} not found", i + 1, rel.id);
                                    }
                                    Err(e) => {
                                        println!("        ❌ Relationship {}: Error checking target node {}: {}", i + 1, rel.id, e);
                                    }
                                }
                            }

                            if valid_relationships == created_child_ids.len() {
                                println!(
                                    "      ✅ All {} relationships are valid",
                                    valid_relationships
                                );
                            } else {
                                println!(
                                    "      ⚠️  Only {}/{} relationships are valid",
                                    valid_relationships,
                                    created_child_ids.len()
                                );
                            }
                        }
                        Err(e) => {
                            println!("      ❌ Failed to verify relationships: {}", e);
                        }
                    }
                } else {
                    // Fallback to simple node
                    let _node_id = data_store.create_text_node(content, Some(date_str)).await?;
                    total_entries += 1;
                    println!("    📄 Created simple node");
                }
            } else {
                // Simple content without bullet points
                let _node_id = data_store.create_text_node(content, Some(date_str)).await?;
                total_entries += 1;
                println!("    📄 Created simple node");
            }
        }
    }

    // Test the hierarchical queries
    println!("\n🧪 Testing hierarchical queries:");

    for date_str in &["2025-06-23", "2025-06-24"] {
        let nodes = data_store.get_nodes_for_date(date_str).await?;
        println!("✅ {} has {} nodes", date_str, nodes.len());

        let children = data_store.get_date_children(date_str).await?;
        println!("✅ {} has {} child relationships", date_str, children.len());
    }

    // Test the actual parent-child relationship query with a real parent ID
    if let Some(parent_id) = test_parent_id {
        println!("\n🔍 Testing actual parent-child relationship query:");
        let clean_id = parent_id.as_str().replace("-", "_");
        let query = format!("SELECT * FROM nodes:{}->contains", clean_id);
        println!("   Query: {}", query);

        match data_store.query_nodes(&query).await {
            Ok(results) => {
                println!(
                    "✅ Parent-child relationship query: {} child nodes found",
                    results.len()
                );

                // Show details of found children
                for (i, child) in results.iter().enumerate() {
                    if let Some(content_str) = child.content.as_str() {
                        println!(
                            "   Child {}: {}",
                            i + 1,
                            content_str.chars().take(50).collect::<String>()
                        );
                    }
                }

                // Check for sibling relationships in the results
                let mut siblings_found = 0;
                for node in &results {
                    if node.next_sibling.is_some() || node.previous_sibling.is_some() {
                        siblings_found += 1;
                    }
                }
                println!(
                    "✅ Found {} child nodes with sibling relationships",
                    siblings_found
                );
            }
            Err(e) => println!("❌ Parent-child relationship query error: {}", e),
        }
    } else {
        println!("⚠️  No test parent ID captured for relationship testing");
    }

    println!("\n🎉 Sample data creation completed!");
    println!("✅ Created {} total entries", total_entries);
    println!("✅ Features demonstrated:");
    println!("   • Hierarchical parent-child relationships");
    println!("   • Bullet point cleaning for child nodes");
    println!("   • Proper sibling ordering with next/previous pointers");
    println!("   • SurrealDB relationship records for traversal");
    println!("\n💡 This data structure will work with:");
    println!("   • get_hierarchical_nodes_for_date()");
    println!("   • get_ordered_children()");
    println!("   • Desktop app hierarchical display");

    Ok(())
}
