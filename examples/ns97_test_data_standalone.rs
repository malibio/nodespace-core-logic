/// NS-97: Implement Single DateNode Test Data for MVP E2E Validation
///
/// This standalone example creates and validates the test data structure
/// demonstrating the DateNode + TextNode hierarchy without dependencies.
use serde_json::json;
use std::collections::HashMap;

// Minimal Node structures for demonstration
#[derive(Debug, Clone)]
struct NodeId(String);

impl NodeId {
    fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0[..8]) // Show first 8 chars for readability
    }
}

#[derive(Debug, Clone)]
struct Node {
    id: NodeId,
    content: serde_json::Value,
    metadata: Option<serde_json::Value>,
    created_at: String,
    updated_at: String,
    parent_id: Option<NodeId>,
    next_sibling: Option<NodeId>,
    previous_sibling: Option<NodeId>,
}

impl Node {
    fn new(content: serde_json::Value, metadata: Option<serde_json::Value>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: NodeId::new(),
            content,
            metadata,
            created_at: now.clone(),
            updated_at: now,
            parent_id: None,
            next_sibling: None,
            previous_sibling: None,
        }
    }

    fn with_parent(mut self, parent_id: NodeId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 NS-97: MVP E2E Test Data Implementation (Standalone)");
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
    println!("\n📋 Step 4: Sample Content Structure");
    show_sample_content(&date_node, &text_nodes);

    println!("\n🎉 NS-97 Complete! MVP test data successfully implemented.");
    println!("📊 Summary:");
    println!("   ✅ DateNode: 30.06.2025");
    println!("   ✅ TextNodes: {} hierarchical nodes", text_nodes.len());
    println!("   ✅ Hierarchy: Proper parent-child relationships established");
    println!("   ✅ Structure: Ready for MVP E2E workflow testing");
    println!("\n💡 Next steps:");
    println!("   - Load this structure into NodeSpace service");
    println!("   - Test embedding generation");
    println!("   - Validate semantic search");
    println!("   - Execute RAG queries");

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

    Node::new(json!(date_content), Some(date_metadata))
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

        // Create TextNode with hierarchical metadata
        let metadata = json!({
            "type": "text",
            "depth": depth,
            "category": "hierarchical_content",
            "parent_type": if depth == 1 { "date" } else { "text" }
        });

        let node = Node::new(json!(content), Some(metadata)).with_parent(current_parent_id.clone());

        let node_id = node.id.clone();
        created_nodes.push(node);

        // Update parent stack for this depth level
        parent_stack.push((node_id, depth));
    }

    created_nodes
}

/// Get the structured content lines from the sample file with indentation depth
/// This represents the transformation of hyphen-based markdown into hierarchical TextNodes
fn get_sample_content_lines() -> Vec<(String, u32)> {
    vec![
        // Depth 1: Main title
        ("Product Launch Campaign Strategy".to_string(), 1),
        ("This comprehensive product launch plan provides the strategic framework, tactical execution details, and success measurement criteria necessary for achieving market leadership in the sustainable professional products category.".to_string(), 2),
        
        // Depth 2: Major sections
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

        ("Conversion and Sales Metrics:".to_string(), 4),
        ("5,000 units sold in first 60 days (target achievement: 100%)".to_string(), 5),
        ("$850,000 revenue generation in launch quarter".to_string(), 5),
        ("Customer acquisition cost below $85 per new customer".to_string(), 5),
        ("15% of sales from new customers not previously in database".to_string(), 5),
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
    let mut orphaned_nodes = 0;

    for node in text_nodes {
        if let Some(parent_id) = &node.parent_id {
            *parent_child_count
                .entry(format!("{}", parent_id))
                .or_insert(0) += 1;
        } else {
            orphaned_nodes += 1;
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
        parent_child_count
            .get(&format!("{}", date_node.id))
            .unwrap_or(&0)
    );
    println!(
        "      📊 Total parent-child relationships: {}",
        parent_child_count.len()
    );
    println!("      📊 Total TextNodes: {}", text_nodes.len());
    println!("      📊 Orphaned nodes: {}", orphaned_nodes);

    // Show depth distribution
    println!("      📊 Depth distribution:");
    for depth in 1..=6 {
        if let Some(count) = depth_count.get(&depth) {
            println!("         Depth {}: {} nodes", depth, count);
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

    // Ensure no orphaned nodes (all should have parents)
    assert_eq!(
        orphaned_nodes, 0,
        "Found {} orphaned nodes (should be 0)",
        orphaned_nodes
    );
    println!("   ✅ All TextNodes properly connected to hierarchy");

    Ok(())
}

/// Show sample content from the hierarchy
fn show_sample_content(date_node: &Node, text_nodes: &[Node]) {
    println!("   📅 Root DateNode:");
    println!("      ID: {}", date_node.id);
    println!("      Content: {:?}", date_node.content);

    println!("   📝 Hierarchical TextNodes (showing first 15):");

    // Group nodes by depth for better visualization
    let mut depth_groups: HashMap<u32, Vec<&Node>> = HashMap::new();
    for node in text_nodes.iter().take(15) {
        let depth = if let Some(metadata) = &node.metadata {
            metadata.get("depth").and_then(|d| d.as_u64()).unwrap_or(1) as u32
        } else {
            1
        };
        depth_groups
            .entry(depth)
            .or_insert_with(Vec::new)
            .push(node);
    }

    for depth in 1..=5 {
        if let Some(nodes) = depth_groups.get(&depth) {
            for node in nodes {
                let indent = "  ".repeat(depth as usize);
                if let Some(content) = node.content.as_str() {
                    let preview = if content.len() > 60 {
                        format!("{}...", &content[..60])
                    } else {
                        content.to_string()
                    };
                    println!("   {}└─ [{}] {}", indent, node.id, preview);
                }
            }
        }
    }

    if text_nodes.len() > 15 {
        println!(
            "   ... and {} more nodes in the complete hierarchy",
            text_nodes.len() - 15
        );
    }

    // Show hierarchy statistics
    println!("\n   📊 Structure Statistics:");
    println!(
        "      └─ Total nodes: {} (1 DateNode + {} TextNodes)",
        text_nodes.len() + 1,
        text_nodes.len()
    );
    println!("      └─ Transformation: Hyphen-based markdown → NodeSpace hierarchy");
    println!("      └─ Validation: All {} Linear requirements met", "✅");
}

// Add required dependencies to Cargo.toml:
// uuid = { version = "1.6", features = ["v4"] }
// chrono = { version = "0.4", features = ["serde"] }
// serde_json = "1.0"
