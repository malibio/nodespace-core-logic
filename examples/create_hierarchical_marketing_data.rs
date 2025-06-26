//! Create Hierarchical Marketing Sample Data
//!
//! This creates realistic marketing data with proper 3-level hierarchy:
//! Date → Parent Content → Child Details/Bullet Points

use nodespace_core_logic::{ServiceContainer, CoreLogic};
use std::error::Error;
use rand::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Creating hierarchical marketing sample data...");
    println!("📊 Structure: Date → Parent Content → Child Details\n");

    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized\n");

    let mut rng = thread_rng();
    let dates = generate_marketing_dates();
    let mut total_entries = 0;

    for (date_idx, date_str) in dates.iter().enumerate() {
        println!("📅 Creating hierarchical content for {} ({}/{})", date_str, date_idx + 1, dates.len());
        
        // Create 2-4 parent content items per date
        let parent_count = rng.gen_range(2..=4);
        
        for _ in 0..parent_count {
            let (parent_content, children) = generate_hierarchical_content(&mut rng);
            
            // Create parent node
            match service_container.create_text_node(&parent_content, date_str).await {
                Ok(parent_id) => {
                    total_entries += 1;
                    println!("  ✅ Parent: {}", parent_content.lines().next().unwrap_or(&parent_content).chars().take(60).collect::<String>());
                    
                    // Create child nodes for this parent
                    for child_content in children {
                        // First create the child node
                        match service_container.create_text_node(&child_content, date_str).await {
                            Ok(child_id) => {
                                // Then establish parent-child relationship
                                match service_container.add_child_node(&parent_id, &child_id).await {
                                    Ok(_) => {
                                        total_entries += 1;
                                        let preview = child_content.chars().take(50).collect::<String>();
                                        println!("    → Child: {}...", preview);
                                        
                                        // Some children might have their own sub-children (bullet points)
                                        if child_content.contains(":") && rng.gen_bool(0.6) {
                                            let sub_children = generate_bullet_points(&mut rng, &child_content);
                                            for sub_child in sub_children {
                                                // Create sub-child node
                                                match service_container.create_text_node(&sub_child, date_str).await {
                                                    Ok(sub_child_id) => {
                                                        // Establish child-subchild relationship
                                                        match service_container.add_child_node(&child_id, &sub_child_id).await {
                                                            Ok(_) => {
                                                                total_entries += 1;
                                                                println!("      • {}", sub_child.chars().take(40).collect::<String>());
                                                            }
                                                            Err(e) => println!("      ❌ Sub-child relationship error: {}", e),
                                                        }
                                                    }
                                                    Err(e) => println!("      ❌ Sub-child creation error: {}", e),
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => println!("    ❌ Child relationship error: {}", e),
                                }
                            }
                            Err(e) => println!("    ❌ Child creation error: {}", e),
                        }
                    }
                }
                Err(e) => println!("  ❌ Parent error: {}", e),
            }
        }
        
        if (date_idx + 1) % 10 == 0 {
            println!("📈 Progress: {} total entries across {} days", total_entries, date_idx + 1);
        }
    }

    println!("\n🎉 Hierarchical marketing data creation completed!");
    println!("✅ Created {} total entries (parents + children + sub-children)", total_entries);
    println!("✅ Proper 3-level hierarchy: Date → Content → Details");
    
    Ok(())
}

fn generate_marketing_dates() -> Vec<String> {
    let mut dates = Vec::new();
    
    // Generate dates over the last 3 months
    for month in 4..=6 { // April, May, June 2025
        for day in [1, 5, 8, 12, 15, 19, 22, 25] { // Realistic business days
            if month == 6 && day > 25 { continue; } // Don't go past today
            let date_str = format!("2025-{:02}-{:02}", month, day);
            dates.push(date_str);
        }
    }
    
    dates
}

fn generate_hierarchical_content(rng: &mut ThreadRng) -> (String, Vec<String>) {
    let content_templates = [
        // Customer Feedback Analysis
        (
            "## Customer Feedback Analysis",
            vec![
                "**Satisfaction Score**: 94%",
                "**Key Insights**:",
                "**Recommended Actions**:",
                "**Response Timeline**: Q3 2025"
            ]
        ),
        
        // Campaign Performance Report
        (
            "## Q2 Campaign Performance Report",
            vec![
                "**Email Marketing**:",
                "**Social Media**:",
                "**Digital Advertising**:",
                "**Overall ROI**: 340%"
            ]
        ),
        
        // Market Research Summary
        (
            "## Market Research Summary",
            vec![
                "**Market Trends**:",
                "**Competitive Analysis**:",
                "**Customer Behavior**:",
                "**Opportunities Identified**:"
            ]
        ),
        
        // Strategy Meeting Notes
        (
            "## Marketing Strategy Meeting",
            vec![
                "**Attendees**: CMO, Digital Lead, Content Manager",
                "**Key Decisions**:",
                "**Action Items**:",
                "**Next Review**: Next Friday"
            ]
        ),
        
        // Product Launch Plan
        (
            "## Product Launch Campaign Plan",
            vec![
                "**Launch Date**: July 15, 2025",
                "**Target Audience**:",
                "**Marketing Channels**:",
                "**Success Metrics**:"
            ]
        ),
        
        // Brand Guidelines Update
        (
            "## Brand Guidelines Update",
            vec![
                "**Voice & Tone**:",
                "**Visual Identity**:",
                "**Content Standards**:",
                "**Implementation Timeline**:"
            ]
        ),
        
        // Budget Analysis
        (
            "## Marketing Budget Analysis",
            vec![
                "**Current Allocation**:",
                "**Performance by Channel**:",
                "**Recommended Adjustments**:",
                "**Q3 Projections**:"
            ]
        ),
        
        // Customer Journey Mapping
        (
            "## Customer Journey Analysis",
            vec![
                "**Awareness Stage**:",
                "**Consideration Stage**:",
                "**Decision Stage**:",
                "**Retention Insights**:"
            ]
        ),
    ];
    
    let (parent, children) = content_templates.choose(rng).unwrap().clone();
    (parent.to_string(), children.iter().map(|s| s.to_string()).collect())
}

fn generate_bullet_points(rng: &mut ThreadRng, parent_content: &str) -> Vec<String> {
    match parent_content {
        content if content.contains("Key Insights") => {
            vec![
                "Product quality exceeds expectations".to_string(),
                "Customer service response time excellent".to_string(),
                "Recommendations for product improvements".to_string(),
            ]
        },
        content if content.contains("Email Marketing") => {
            vec![
                format!("Open rate: {}%", rng.gen_range(22..35)),
                format!("Click rate: {}%", rng.gen_range(4..8)),
                format!("Conversion rate: {}%", rng.gen_range(8..15)),
            ]
        },
        content if content.contains("Social Media") => {
            vec![
                format!("Engagement rate: {}%", rng.gen_range(3..6)),
                format!("Reach: {}k", rng.gen_range(180..350)),
                format!("Follower growth: +{}", rng.gen_range(1200..3500)),
            ]
        },
        content if content.contains("Market Trends") => {
            vec![
                "Sustainable products gaining 23% market share".to_string(),
                "Digital marketing ROI up 31%".to_string(),
                "Customer acquisition cost decreased 15%".to_string(),
            ]
        },
        content if content.contains("Key Decisions") => {
            vec![
                "Increase video content production by 50%".to_string(),
                "Launch influencer partnership program".to_string(),
                "Reallocate budget from print to digital".to_string(),
            ]
        },
        content if content.contains("Target Audience") => {
            vec![
                "Primary: Women 25-40, urban professionals".to_string(),
                "Secondary: Eco-conscious millennials".to_string(),
                "Growth segment: Gen Z early adopters".to_string(),
            ]
        },
        content if content.contains("Action Items") => {
            vec![
                "Finalize creative assets by Friday".to_string(),
                "Schedule influencer outreach calls".to_string(),
                "Update campaign tracking parameters".to_string(),
            ]
        },
        content if content.contains("Voice & Tone") => {
            vec![
                "Authentic and empowering messaging".to_string(),
                "Professional yet approachable tone".to_string(),
                "Sustainability-focused language".to_string(),
            ]
        },
        _ => vec![], // No sub-children for this content type
    }
}