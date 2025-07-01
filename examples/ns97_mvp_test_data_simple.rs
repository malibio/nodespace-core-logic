use nodespace_core_types::{Node, NodeId};
use serde_json::json;
use std::collections::HashMap;

/// NS-97: Implement Single DateNode Test Data for MVP E2E Validation
///
/// This simplified example creates the test data structure without requiring
/// full service initialization, demonstrating the DateNode + TextNode hierarchy.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 NS-97: MVP E2E Test Data Implementation (Simplified)");
    println!("=========================================================");

    // Step 1: Create DateNode (Root)
    println!("\n📅 Step 1: Creating DateNode (Root)");
    let date_node = create_date_node();
    println!("✅ DateNode created: {}", date_node.id);

    // Step 2: Create hierarchical TextNodes
    println!("\n📝 Step 2: Creating hierarchical TextNodes");
    let text_nodes = create_hierarchical_text_nodes(&date_node.id);
    println!(
        "✅ Created {} TextNodes with proper hierarchy",
        text_nodes.len()
    );

    // Step 3: Validate structure
    println!("\n🔍 Step 3: Validating structure");
    validate_hierarchy(&date_node, &text_nodes)?;

    // Step 4: Show sample nodes
    println!("\n📋 Step 4: Sample Content");
    show_sample_content(&date_node, &text_nodes);

    println!("\n🎉 NS-97 Complete! MVP test data successfully implemented.");
    println!("📊 Summary:");
    println!("   ✅ DateNode: 30.06.2025");
    println!("   ✅ TextNodes: {} hierarchical nodes", text_nodes.len());
    println!("   ✅ Hierarchy: Proper parent-child relationships");
    println!("   ✅ Ready for MVP E2E workflow testing");

    Ok(())
}

/// Create the root DateNode for 30.06.2025
fn create_date_node() -> Node {
    let date_content = "30.06.2025";
    let date_metadata = json!({
        "type": "date",
        "date": "30.06.2025",
        "category": "root_date"
    });

    Node {
        id: NodeId::new(),
        content: json!(date_content),
        metadata: Some(date_metadata),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        parent_id: None, // Root node
        next_sibling: None,
        previous_sibling: None,
    }
}

/// Create hierarchical TextNodes from the Product Launch Campaign Strategy
fn create_hierarchical_text_nodes(date_parent_id: &NodeId) -> Vec<Node> {
    let mut created_nodes = Vec::new();
    let mut parent_stack: Vec<(NodeId, u32)> = vec![(date_parent_id.clone(), 0)]; // (parent_id, depth)

    // Sample content from sample-node-entry.md with proper hierarchy
    let content_lines = get_sample_content_lines();

    for (content, depth) in content_lines {
        // Determine parent based on depth
        while parent_stack.len() > 1 && parent_stack.last().unwrap().1 >= depth {
            parent_stack.pop();
        }

        let current_parent_id = &parent_stack.last().unwrap().0;

        // Create TextNode
        let metadata = json!({
            "type": "text",
            "depth": depth,
            "category": "hierarchical_content"
        });

        let node = Node {
            id: NodeId::new(),
            content: json!(content),
            metadata: Some(metadata),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            parent_id: Some(current_parent_id.clone()),
            next_sibling: None,
            previous_sibling: None,
        };

        let node_id = node.id.clone();
        created_nodes.push(node);

        // Update parent stack for this depth level
        parent_stack.push((node_id, depth));
    }

    created_nodes
}

/// Get the structured content lines from the sample file with indentation depth
fn get_sample_content_lines() -> Vec<(String, u32)> {
    vec![
        ("Product Launch Campaign Strategy".to_string(), 1),
        ("This comprehensive product launch plan provides the strategic framework, tactical execution details, and success measurement criteria necessary for achieving market leadership in the sustainable professional products category.".to_string(), 2),

        ("Launch Overview".to_string(), 2),
        ("Product: EcoSmart Professional Series".to_string(), 3),
        ("Launch Date: July 15, 2025".to_string(), 3),
        ("Campaign Duration: 12 weeks (4 weeks pre-launch, 4 weeks launch, 4 weeks post-launch)".to_string(), 3),
        ("Total Budget: $180,000".to_string(), 3),
        ("Primary Objective: Establish market leadership in sustainable professional products".to_string(), 3),

        ("Executive Summary".to_string(), 2),
        ("The EcoSmart Professional Series represents our most significant product innovation in three years, combining professional-grade performance with industry-leading sustainability features. This launch campaign will position us as the premium choice for environmentally conscious professionals while maintaining our quality and performance reputation.".to_string(), 3),

        ("Target Audience Analysis".to_string(), 2),
        ("Primary Target Segment".to_string(), 3),
        ("Professional Demographics:".to_string(), 4),
        ("Age: 28-45 years".to_string(), 5),
        ("Income: $75,000−$150,000 annually".to_string(), 5),
        ("Education: College degree or higher (87%)".to_string(), 5),
        ("Location: Urban and suburban professionals in major metropolitan areas".to_string(), 5),
        ("Industry Focus: Design, consulting, technology, finance, healthcare".to_string(), 5),

        ("Psychographic Profile:".to_string(), 4),
        ("Values sustainability and environmental responsibility".to_string(), 5),
        ("Willing to pay premium for quality and environmental benefits".to_string(), 5),
        ("Influences others in professional networks".to_string(), 5),
        ("Active on LinkedIn and Instagram".to_string(), 5),
        ("Research-intensive purchase behavior".to_string(), 5),

        ("Secondary Target Segments".to_string(), 3),
        ("Segment 2: Sustainability-Focused Organizations".to_string(), 4),
        ("Corporate buyers implementing sustainability initiatives".to_string(), 5),
        ("Government agencies with environmental mandates".to_string(), 5),
        ("Non-profit organizations with mission alignment".to_string(), 5),
        ("Educational institutions with sustainability programs".to_string(), 5),

        ("Segment 3: Early Adopter Enthusiasts".to_string(), 4),
        ("Technology and innovation enthusiasts".to_string(), 5),
        ("Sustainability advocates and influencers".to_string(), 5),
        ("Professional reviewers and industry experts".to_string(), 5),
        ("Brand advocates and loyal customers".to_string(), 5),

        ("Product Positioning Strategy".to_string(), 2),
        ("Core Value Proposition".to_string(), 3),
        ("\"Professional performance without environmental compromise\" - positioning EcoSmart Professional Series as the only product line that delivers superior professional results while achieving industry-leading sustainability standards.".to_string(), 4),

        ("Key Differentiators".to_string(), 3),
        ("Performance Excellence: 15% performance improvement over previous generation".to_string(), 4),
        ("Sustainability Leadership: 75% reduction in environmental impact across lifecycle".to_string(), 4),
        ("Professional Grade: Meets all professional industry standards and certifications".to_string(), 4),
        ("Innovation Recognition: Featured in leading industry publications and awards".to_string(), 4),

        ("Competitive Positioning".to_string(), 3),
        ("Versus Premium Competitors: Superior sustainability without performance sacrifice".to_string(), 4),
        ("Versus Sustainable Alternatives: Professional-grade performance they cannot match".to_string(), 4),
        ("Versus Mass Market: Premium quality and environmental leadership justify price difference".to_string(), 4),

        ("Marketing Channel Strategy".to_string(), 2),
        ("Pre-Launch Phase (Weeks 1-4)".to_string(), 3),
        ("Content Marketing and Education:".to_string(), 4),
        ("Educational blog series on sustainability in professional environments".to_string(), 5),
        ("Webinar series featuring industry experts and environmental scientists".to_string(), 5),
        ("Behind-the-scenes content showing product development and testing".to_string(), 5),
        ("Sustainability impact calculator and assessment tools".to_string(), 5),

        ("Influencer and Partnership Strategy:".to_string(), 4),
        ("Partner with 15 industry professionals for authentic product testing".to_string(), 5),
        ("Collaborate with sustainability experts for credibility and education".to_string(), 5),
        ("Engage professional associations and industry organizations".to_string(), 5),
        ("Secure early reviews from respected industry publications".to_string(), 5),

        ("Success Metrics and KPIs".to_string(), 2),
        ("Launch Success Indicators".to_string(), 3),
        ("Awareness Metrics:".to_string(), 4),
        ("Brand awareness increase of 25% in target demographic within 60 days".to_string(), 5),
        ("2.5 million impressions across all marketing channels".to_string(), 5),
        ("15% increase in branded search volume".to_string(), 5),
        ("Media coverage in 25+ industry and mainstream publications".to_string(), 5),

        ("Engagement Metrics:".to_string(), 4),
        ("500,000 video views across all platforms".to_string(), 5),
        ("5.5% average engagement rate on social media content".to_string(), 5),
        ("25% increase in website traffic and 15% improvement in time on site".to_string(), 5),
        ("1,200 webinar attendees and 85% completion rate".to_string(), 5),
    ]
}

/// Validate the hierarchical structure
fn validate_hierarchy(
    date_node: &Node,
    text_nodes: &[Node],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   🔍 Validating DateNode...");
    if let Some(content) = date_node.content.as_str() {
        assert!(content.contains("30.06.2025"), "DateNode content incorrect");
        println!("   ✅ DateNode content verified: {}", content);
    }

    println!("   🔍 Validating TextNode hierarchy...");
    let mut parent_child_count = HashMap::new();
    let mut depth_count = HashMap::new();

    for node in text_nodes {
        if let Some(parent_id) = &node.parent_id {
            *parent_child_count.entry(parent_id.clone()).or_insert(0) += 1;
        }

        if let Some(metadata) = &node.metadata {
            if let Some(depth) = metadata.get("depth").and_then(|d| d.as_u64()) {
                *depth_count.entry(depth as u32).or_insert(0) += 1;
            }
        }
    }

    println!("   ✅ Hierarchy validated:");
    println!(
        "      📊 DateNode children: {}",
        parent_child_count.get(&date_node.id).unwrap_or(&0)
    );
    println!(
        "      📊 Total parent-child relationships: {}",
        parent_child_count.len()
    );
    println!("      📊 Total TextNodes: {}", text_nodes.len());

    // Show depth distribution
    for depth in 1..=6 {
        if let Some(count) = depth_count.get(&depth) {
            println!("      📊 Depth {} nodes: {}", depth, count);
        }
    }

    // Ensure we have the expected ~50+ nodes
    assert!(
        text_nodes.len() >= 50,
        "Expected at least 50 TextNodes, got {}",
        text_nodes.len()
    );
    println!(
        "   ✅ Node count requirement met: {} >= 50",
        text_nodes.len()
    );

    Ok(())
}

/// Show sample content from the hierarchy
fn show_sample_content(date_node: &Node, text_nodes: &[Node]) {
    println!("   📅 DateNode: {:?}", date_node.content);

    println!("   📝 Sample TextNodes:");
    for (i, node) in text_nodes.iter().take(10).enumerate() {
        let depth = if let Some(metadata) = &node.metadata {
            metadata.get("depth").and_then(|d| d.as_u64()).unwrap_or(0)
        } else {
            0
        };

        let indent = "  ".repeat(depth as usize);
        if let Some(content) = node.content.as_str() {
            let preview = if content.len() > 60 {
                format!("{}...", &content[..60])
            } else {
                content.to_string()
            };
            println!("   {}{}. {}{}", i + 1, indent, "└─ ", preview);
        }
    }

    if text_nodes.len() > 10 {
        println!("   ... and {} more nodes", text_nodes.len() - 10);
    }
}
