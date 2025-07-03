use chrono::NaiveDate;
use nodespace_core_logic::{CoreLogic, HierarchyComputation, NodeSpaceService};
use nodespace_core_types::NodeSpaceResult;

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("📊 Database Sibling Column Values");
    println!("=================================");

    // Initialize service
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

    let campaign_date = NaiveDate::from_ymd_opt(2025, 6, 26).unwrap();

    // Get the date node
    if let Some(date_node_id) = service.find_date_node(campaign_date).await? {
        println!("\n📅 Date Node: {}", date_node_id);

        // Get main title children
        let main_children = service.get_children(&date_node_id).await?;

        if let Some(main_title) = main_children.first() {
            println!("\n📋 Main Title: {}", main_title.id);
            println!(
                "   Content: {}",
                main_title.content.as_str().unwrap_or("N/A")
            );
            println!("   Next Sibling: {:?}", main_title.next_sibling);

            // Get children of main title (sections)
            let sections = service.get_children(&main_title.id).await?;
            println!("\n📑 Sections under main title ({} total):", sections.len());

            for (index, section) in sections.iter().enumerate() {
                println!("\n   {}. ID: {}", index + 1, section.id);
                println!(
                    "      Content: {}",
                    section
                        .content
                        .as_str()
                        .unwrap_or("N/A")
                        .chars()
                        .take(80)
                        .collect::<String>()
                );
                println!("      Parent ID: {:?}", section.parent_id);
                println!("      Next Sibling: {:?}", section.next_sibling);

                // Show first few children to see deeper structure
                let subsections = service.get_children(&section.id).await?;
                if !subsections.is_empty() {
                    println!("      Children ({}):", subsections.len());
                    for (sub_index, subsection) in subsections.iter().take(3).enumerate() {
                        println!(
                            "         {}.{} ID: {}",
                            index + 1,
                            sub_index + 1,
                            subsection.id
                        );
                        println!(
                            "             Content: {}",
                            subsection
                                .content
                                .as_str()
                                .unwrap_or("N/A")
                                .chars()
                                .take(60)
                                .collect::<String>()
                        );
                        println!("             Next Sibling: {:?}", subsection.next_sibling);
                    }
                    if subsections.len() > 3 {
                        println!("         ... and {} more", subsections.len() - 3);
                    }
                }
            }

            println!("\n🔗 Sibling Chain Analysis:");
            println!("==========================");

            if let Some(first_section) = sections.first() {
                let mut current_id = Some(first_section.id.clone());
                let mut chain_position = 1;

                while let Some(node_id) = current_id {
                    if let Some(node) = sections.iter().find(|n| n.id == node_id) {
                        println!(
                            "{}. {} -> {:?}",
                            chain_position,
                            node.content
                                .as_str()
                                .unwrap_or("N/A")
                                .chars()
                                .take(50)
                                .collect::<String>(),
                            node.next_sibling
                        );
                        current_id = node.next_sibling.clone();
                        chain_position += 1;

                        // Safety break
                        if chain_position > 20 {
                            println!("   ... (stopping to avoid infinite loop)");
                            break;
                        }
                    } else {
                        println!("   ❌ Broken chain at: {}", node_id);
                        break;
                    }
                }

                println!("\n📈 Chain Summary:");
                println!("   Total sections: {}", sections.len());
                println!("   Chain length: {}", chain_position - 1);
                if chain_position - 1 == sections.len() {
                    println!("   ✅ Complete chain!");
                } else {
                    println!("   ⚠️  Incomplete chain - some nodes may not be linked");
                }
            }
        }
    }

    println!("\n🎯 Sibling column display complete!");
    Ok(())
}
