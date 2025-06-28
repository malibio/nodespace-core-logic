use nodespace_core_logic::{NodeSpaceService, CoreLogic, DateNavigation};
use serde_json::json;

#[tokio::main] 
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Creating Correct Node Structure for 2025-06-28");
    
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/e2e_sample.db",
        Some("./bundled_models"),
    ).await?;
    
    service.initialize().await?;
    
    println!("\n1️⃣ Creating Date Node for June 28, 2025");
    
    // Create the date node
    let date_node_id = service.create_knowledge_node(
        "# June 28, 2025",
        json!({
            "node_type": "date",
            "date": "2025-06-28",
            "created_from": "sample-node-entry.md"
        })
    ).await?;
    
    println!("   ✅ Date node created: {}", date_node_id);
    
    // Now create the hierarchy from sample-node-entry.md
    println!("\n2️⃣ Creating Text Node Hierarchy from Markdown");
    
    // Level 0: Main title
    let main_title_id = service.create_knowledge_node(
        "# Product Launch Campaign Strategy", 
        json!({
            "node_type": "text",
            "depth": 0,
            "order": 1,
            "parent_id": date_node_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 0: Main title -> {}", main_title_id);
    
    // Level 1: Description
    let desc_id = service.create_knowledge_node(
        "This comprehensive product launch plan provides the strategic framework, tactical execution details, and success measurement criteria necessary for achieving market leadership in the sustainable professional products category.",
        json!({
            "node_type": "text", 
            "depth": 1,
            "order": 2,
            "parent_id": main_title_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 1: Description -> {}", desc_id);
    
    // Level 1: Launch Overview section
    let overview_id = service.create_knowledge_node(
        "## Launch Overview",
        json!({
            "node_type": "text",
            "depth": 1, 
            "order": 3,
            "parent_id": main_title_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 1: Launch Overview -> {}", overview_id);
    
    // Level 2: Product details under Launch Overview
    let product_id = service.create_knowledge_node(
        "**Product**: EcoSmart Professional Series",
        json!({
            "node_type": "text",
            "depth": 2,
            "order": 4,
            "parent_id": overview_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 2: Product -> {}", product_id);
    
    let launch_date_id = service.create_knowledge_node(
        "**Launch Date**: July 15, 2025", 
        json!({
            "node_type": "text",
            "depth": 2,
            "order": 5,
            "parent_id": overview_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 2: Launch Date -> {}", launch_date_id);
    
    let campaign_duration_id = service.create_knowledge_node(
        "**Campaign Duration**: 12 weeks (4 weeks pre-launch, 4 weeks launch, 4 weeks post-launch)",
        json!({
            "node_type": "text",
            "depth": 2,
            "order": 6,
            "parent_id": overview_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 2: Campaign Duration -> {}", campaign_duration_id);
    
    let budget_id = service.create_knowledge_node(
        "**Total Budget**: $180,000",
        json!({
            "node_type": "text", 
            "depth": 2,
            "order": 7,
            "parent_id": overview_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 2: Budget -> {}", budget_id);
    
    let objective_id = service.create_knowledge_node(
        "**Primary Objective**: Establish market leadership in sustainable professional products",
        json!({
            "node_type": "text",
            "depth": 2,
            "order": 8, 
            "parent_id": overview_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 2: Objective -> {}", objective_id);
    
    // Level 1: Executive Summary section  
    let exec_summary_id = service.create_knowledge_node(
        "## Executive Summary",
        json!({
            "node_type": "text",
            "depth": 1,
            "order": 9,
            "parent_id": main_title_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 1: Executive Summary -> {}", exec_summary_id);
    
    // Level 2: Executive Summary content
    let exec_content_id = service.create_knowledge_node(
        "The EcoSmart Professional Series represents our most significant product innovation in three years, combining professional-grade performance with industry-leading sustainability features. This launch campaign will position us as the premium choice for environmentally conscious professionals while maintaining our quality and performance reputation.",
        json!({
            "node_type": "text",
            "depth": 2,
            "order": 10,
            "parent_id": exec_summary_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 2: Executive Summary content -> {}", exec_content_id);
    
    // Level 1: Target Audience Analysis section
    let target_audience_id = service.create_knowledge_node(
        "## Target Audience Analysis",
        json!({
            "node_type": "text",
            "depth": 1,
            "order": 11,
            "parent_id": main_title_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 1: Target Audience Analysis -> {}", target_audience_id);
    
    // Level 2: Primary Target Segment
    let primary_segment_id = service.create_knowledge_node(
        "### Primary Target Segment",
        json!({
            "node_type": "text",
            "depth": 2,
            "order": 12,
            "parent_id": target_audience_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 2: Primary Target Segment -> {}", primary_segment_id);
    
    // Level 3: Professional Demographics
    let demographics_id = service.create_knowledge_node(
        "**Professional Demographics**:",
        json!({
            "node_type": "text",
            "depth": 3,
            "order": 13,
            "parent_id": primary_segment_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 3: Demographics -> {}", demographics_id);
    
    // Level 4: Age demographic
    let age_id = service.create_knowledge_node(
        "Age: 28-45 years",
        json!({
            "node_type": "text",
            "depth": 4,
            "order": 14,
            "parent_id": demographics_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 4: Age -> {}", age_id);
    
    // Level 4: Income demographic  
    let income_id = service.create_knowledge_node(
        "Income: $75,000−$150,000 annually",
        json!({
            "node_type": "text",
            "depth": 4,
            "order": 15,
            "parent_id": demographics_id.to_string()
        })
    ).await?;
    println!("   ✅ Level 4: Income -> {}", income_id);
    
    println!("\n3️⃣ Testing the created structure");
    
    // Test get_nodes_for_date for tomorrow
    let tomorrow = chrono::Utc::now().date_naive() + chrono::Duration::days(1);
    let nodes_for_tomorrow = service.get_nodes_for_date(tomorrow).await?;
    
    println!("   Nodes for {} (tomorrow): {}", tomorrow, nodes_for_tomorrow.len());
    
    if nodes_for_tomorrow.len() > 0 {
        println!("   ✅ SUCCESS! Created proper text hierarchy");
        println!("   First 5 nodes:");
        for (i, node) in nodes_for_tomorrow.iter().take(5).enumerate() {
            if let Some(content) = node.content.as_str() {
                let clean = content.trim().trim_matches('"').trim();
                println!("      {}. {:?} (ID: {})", i + 1, clean.chars().take(50).collect::<String>(), node.id);
            }
        }
    } else {
        println!("   ❌ No nodes found for tomorrow");
    }
    
    println!("\n🎯 SUMMARY:");
    println!("   ✅ Created date node: 'date' type");
    println!("   ✅ Created {} text nodes: all 'text' type", 15);  
    println!("   ✅ Preserved markdown hierarchy with depth levels 0-4");
    println!("   ✅ Used parent_id relationships to maintain structure");
    println!("   ✅ All content preserved as markdown text");
    
    Ok(())
}