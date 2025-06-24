use chrono::NaiveDate;
use nodespace_core_logic::{DateNavigation, ServiceContainer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing database connection and data retrieval...");

    // Initialize ServiceContainer
    let container = ServiceContainer::new().await?;
    println!("✓ ServiceContainer initialized successfully");

    // Test date: 2025-04-15 (from sample data)
    let test_date = NaiveDate::from_ymd_opt(2025, 4, 15).unwrap();

    // Test get_nodes_for_date
    println!("\nTesting get_nodes_for_date for {}...", test_date);
    match container.get_nodes_for_date(test_date).await {
        Ok(nodes) => {
            println!(
                "✓ Successfully retrieved {} nodes for date {}",
                nodes.len(),
                test_date
            );
            for (i, node) in nodes.iter().enumerate() {
                if let Some(content) = node.content.as_str() {
                    println!(
                        "  Node {}: {}",
                        i + 1,
                        content.chars().take(100).collect::<String>()
                    );
                }
            }
        }
        Err(e) => {
            println!("✗ Error retrieving nodes: {}", e);
        }
    }

    // Test navigate_to_date
    println!("\nTesting navigate_to_date for {}...", test_date);
    match container.navigate_to_date(test_date).await {
        Ok(result) => {
            println!("✓ Navigation successful:");
            println!("  Date: {}", result.date);
            println!("  Nodes: {}", result.nodes.len());
            println!("  Has previous: {}", result.has_previous);
            println!("  Has next: {}", result.has_next);
        }
        Err(e) => {
            println!("✗ Navigation error: {}", e);
        }
    }

    // Test get_previous_day
    println!("\nTesting get_previous_day from {}...", test_date);
    match container.get_previous_day(test_date).await {
        Ok(Some(prev_date)) => {
            println!("✓ Previous day with content: {}", prev_date);
        }
        Ok(None) => {
            println!("✓ No previous day with content found");
        }
        Err(e) => {
            println!("✗ Error finding previous day: {}", e);
        }
    }

    // Test get_next_day
    println!("\nTesting get_next_day from {}...", test_date);
    match container.get_next_day(test_date).await {
        Ok(Some(next_date)) => {
            println!("✓ Next day with content: {}", next_date);
        }
        Ok(None) => {
            println!("✓ No next day with content found");
        }
        Err(e) => {
            println!("✗ Error finding next day: {}", e);
        }
    }

    // Test create_or_get_date_node
    println!("\nTesting create_or_get_date_node for {}...", test_date);
    match container.create_or_get_date_node(test_date).await {
        Ok(date_node) => {
            println!("✓ Date node found/created:");
            println!("  ID: {}", date_node.id);
            println!("  Date: {}", date_node.date);
            println!("  Description: {:?}", date_node.description);
            println!("  Child count: {}", date_node.child_count);
        }
        Err(e) => {
            println!("✗ Error with date node: {}", e);
        }
    }

    println!("\nDatabase test completed!");
    Ok(())
}
