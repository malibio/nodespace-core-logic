use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use nodespace_core_logic::{DataStore, NLPEngine, NodeSpaceService};
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Meaningful sample data population for NodeSpace database
/// Creates a comprehensive hierarchical structure with rich, RAG-queryable content

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🚀 Populating NodeSpace with meaningful sample data");
    println!("================================================");

    // Initialize mock services for demonstration
    let data_store = MockDataStore::new();
    let nlp_engine = MockNLPEngine::new();
    let service = NodeSpaceService::new(data_store, nlp_engine);

    // Create and populate the hierarchical structure
    let node_ids = populate_sample_data(&service).await?;

    println!("\n✅ Successfully populated database with meaningful sample data");
    println!(
        "📊 Created {} nodes with rich, queryable content",
        node_ids.len()
    );
    println!("\n💬 Ready for RAG queries such as:");
    println!("   • 'What is our target audience for the product launch?'");
    println!("   • 'How much budget is allocated for digital marketing?'");
    println!("   • 'What are the key phases in our launch timeline?'");
    println!("   • 'Who are the primary demographics we're targeting?'");
    println!("   • 'What marketing channels are we using and why?'");

    Ok(())
}

async fn populate_sample_data<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync>(
    service: &NodeSpaceService<D, N>,
) -> NodeSpaceResult<Vec<NodeId>> {
    let mut created_nodes = Vec::new();
    let today = chrono::Local::now().date_naive();

    println!("\n📅 Creating date node for {}", today.format("%B %d, %Y"));

    // Level 0: Create Date Node (Root)
    let date_node = Node::new(
        format!("Daily Journal - {}", today.format("%B %d, %Y")),
        json!(format!("Daily planning and activities for {}. Today marks an important milestone in our product development cycle as we finalize the launch campaign strategy. The team has been working intensively on market research, competitive analysis, and customer persona development over the past quarter.", today.format("%A, %B %d, %Y"))),
        Some(json!({
            "node_type": "date",
            "date": today.format("%Y-%m-%d").to_string(),
            "day_of_week": today.format("%A").to_string(),
            "created_at": Utc::now().to_rfc3339(),
            "importance": "high",
            "tags": ["planning", "product-launch", "strategy"]
        }))
    );

    let date_node_id = service.data_store.store_node(date_node).await?;
    created_nodes.push(date_node_id.clone());
    println!("  ✓ Date node created: {}", date_node_id);

    // Level 1: Main Strategy Document
    println!("\n📋 Creating main strategy document");
    let strategy_content = "# Product Launch Campaign Strategy\n\nComprehensive go-to-market strategy for launching \"FlowState Pro\" - our next-generation productivity platform designed for knowledge workers, creative professionals, and distributed teams.\n\n## Executive Summary\n\nFlowState Pro represents a paradigm shift in how professionals manage their knowledge, tasks, and collaborative workflows. Unlike traditional productivity tools that fragment attention across multiple applications, FlowState Pro provides a unified, AI-powered workspace that adapts to individual work patterns and team dynamics.\n\n## Market Opportunity\n\nThe global productivity software market is valued at $96.36 billion and growing at 13.4% CAGR. However, 73% of knowledge workers report feeling overwhelmed by tool fragmentation, and 68% spend over 2 hours daily switching between applications. FlowState Pro addresses this critical pain point through intelligent context switching and seamless workflow integration.\n\n## Competitive Advantage\n\nOur unique positioning centers on three core differentiators:\n1. **AI-Powered Context Awareness**: Machine learning algorithms that understand work patterns and proactively surface relevant information\n2. **Unified Knowledge Graph**: All work artifacts - documents, tasks, communications - connected through semantic relationships\n3. **Adaptive Interface**: UI that morphs based on current work context, reducing cognitive load by 40% in beta testing\n\n## Success Metrics\n\n- **Primary KPI**: 10,000 paid subscribers within 6 months of launch\n- **Engagement**: 80% monthly active user retention\n- **Revenue**: $2.5M ARR by end of Year 1\n- **Market Penetration**: 5% market share in productivity software for teams under 50 people";

    let strategy_node = Node::new(
        "Product Launch Campaign Strategy".to_string(),
        json!(strategy_content),
        Some(json!({
            "node_type": "document",
            "document_type": "strategy",
            "priority": "high",
            "status": "in_planning",
            "tags": ["marketing", "product-launch", "strategy", "flowstate-pro"],
            "stakeholders": ["marketing", "product", "engineering", "sales"],
            "parent_id": date_node_id.to_string(),
            "approval_required": true,
            "budget_impact": 150000
        })),
    );

    let strategy_node_id = service.data_store.store_node(strategy_node).await?;
    created_nodes.push(strategy_node_id.clone());
    println!("  ✓ Strategy document created: {}", strategy_node_id);

    // Level 2: Target Audience Analysis
    println!("\n👥 Creating target audience analysis");
    let audience_content = "## Target Audience Analysis\n\nBased on 18 months of market research, user interviews with 847 professionals, and analysis of 2.3 million productivity app usage patterns, we've identified distinct audience segments with specific pain points and preferences.\n\n### Research Methodology\n\nOur audience analysis employed mixed-method research including:\n- **Quantitative Survey**: 2,500 knowledge workers across 15 countries\n- **Qualitative Interviews**: 125 in-depth interviews (45-90 minutes each)\n- **Usage Analytics**: Behavioral data from 50,000+ productivity app users\n- **Ethnographic Studies**: Workplace observation in 12 organizations\n\n### Key Findings\n\n**Pain Point Hierarchy** (ranked by frequency and intensity):\n1. **Context Switching Fatigue**: 89% report mental exhaustion from app switching\n2. **Information Fragmentation**: 76% struggle to find relevant information quickly\n3. **Collaboration Friction**: 68% cite tool inconsistency as collaboration barrier\n4. **Cognitive Overload**: 84% feel overwhelmed by notification volume\n\n**Behavioral Patterns**:\n- Average professional uses 9.4 different productivity tools daily\n- 67% prefer unified interfaces over specialized tools\n- 78% willing to pay premium for significant time savings\n- Early adopters influence 3.2 colleagues on average\n\n### Psychological Profiles\n\nOur research identified three distinct psychological archetypes:\n\n**The Optimizer** (42% of target market): Methodical professionals who systematically seek efficiency improvements. High conscientiousness, moderate openness to experience.\n\n**The Innovator** (31% of target market): Creative professionals who value flexibility and novel approaches. High openness, moderate conscientiousness.\n\n**The Collaborator** (27% of target market): Team-oriented professionals prioritizing seamless group workflows. High agreeableness, moderate extraversion.";

    let audience_node = Node::new(
        "Target Audience Analysis".to_string(),
        json!(audience_content),
        Some(json!({
            "node_type": "section",
            "section_type": "analysis",
            "completion": 0.95,
            "research_cost": 125000,
            "confidence_level": 0.87,
            "parent_id": strategy_node_id.to_string(),
            "data_sources": ["survey", "interviews", "analytics", "ethnography"],
            "sample_size": 52500
        })),
    );

    let audience_node_id = service.data_store.store_node(audience_node).await?;
    created_nodes.push(audience_node_id.clone());
    println!("  ✓ Target audience analysis created: {}", audience_node_id);

    // Level 3: Primary Demographics
    println!("\n🎯 Creating primary demographics persona");
    let demographics_content = "### Primary Demographics: Tech-Savvy Professionals (58% of target market)\n\n**Core Profile**: Knowledge workers in technology, consulting, and creative industries who are early adopters of productivity tools and willing to invest in efficiency improvements.\n\n**Detailed Demographics**:\n- **Age Range**: 28-42 years (median: 34 years)\n- **Education**: 87% bachelor's degree or higher, 34% advanced degrees\n- **Income**: $75,000-$140,000 annually (median: $95,000)\n- **Geographic Distribution**: 67% North America, 21% Europe, 12% Asia-Pacific\n- **Company Size**: 45% work at companies with 50-500 employees\n- **Role Types**: Product managers (23%), developers (19%), consultants (16%), designers (14%), analysts (12%), others (16%)\n\n**Psychographic Characteristics**:\n- **Technology Adoption**: Early majority to early adopters (Rogers curve)\n- **Work Style**: Prefer asynchronous communication, value deep work time\n- **Decision Making**: Research-driven, compare multiple options, seek peer recommendations\n- **Pain Tolerance**: Low tolerance for inefficient processes, high standards for tool quality\n- **Learning Preference**: Self-directed learning, prefer documentation over training calls\n\n**Professional Behaviors**:\n- **Tool Usage**: Use 8-12 professional software tools daily\n- **Information Consumption**: Read 2-3 industry publications weekly, follow 15-20 thought leaders\n- **Networking**: Active on LinkedIn, attend 1-2 industry conferences annually\n- **Purchase Authority**: 73% have budget influence, 45% are primary decision makers\n\n**Productivity Pain Points** (in order of severity):\n1. **Context Switching**: Lose 23 minutes per interruption, 67 switches daily average\n2. **Information Retrieval**: Spend 2.5 hours daily searching for information\n3. **Status Reporting**: 45 minutes daily on progress updates and check-ins\n4. **Tool Integration**: 38% of work involves copying data between applications\n\n**Willingness to Pay**: 78% would pay $50-100/month for 20% productivity improvement";

    let demographics_node = Node::new(
        "Primary Demographics: Tech-Savvy Professionals".to_string(),
        json!(demographics_content),
        Some(json!({
            "node_type": "persona",
            "persona_name": "tech_professional",
            "market_share": 0.58,
            "confidence_level": 0.91,
            "parent_id": audience_node_id.to_string(),
            "data_source": "primary_research",
            "sample_size": 1450,
            "avg_income": 95000,
            "willingness_to_pay": 75
        })),
    );

    let demographics_node_id = service.data_store.store_node(demographics_node).await?;
    created_nodes.push(demographics_node_id.clone());
    println!("  ✓ Primary demographics created: {}", demographics_node_id);

    // Level 2: Marketing Channels
    println!("\n📢 Creating marketing channels strategy");
    let channels_content = "## Marketing Channels Strategy\n\nMulti-channel approach leveraging both digital and traditional marketing to achieve 360-degree market penetration. Our channel selection is data-driven, based on audience research showing where our target demographics consume professional content and make software purchasing decisions.\n\n### Channel Performance Modeling\n\nUsing attribution modeling and customer journey mapping, we've projected the following channel effectiveness:\n\n**Digital Channels** (70% of budget, 85% of leads):\n- **Content Marketing**: Highest quality leads (47% conversion rate)\n- **LinkedIn Advertising**: Best B2B reach (2.3M relevant professionals)\n- **Search Marketing**: Highest intent signals (67% purchase consideration)\n- **Email Marketing**: Strongest retention tool (4.2x engagement vs. social)\n\n**Traditional Channels** (30% of budget, 15% of leads):\n- **Industry Publications**: Credibility building and thought leadership\n- **Conference Speaking**: Direct engagement with decision makers\n- **Partnership Marketing**: Leveraging existing vendor relationships\n\n### Customer Acquisition Cost (CAC) Projections\n\n- **Content Marketing**: $47 CAC, 18-month payback\n- **Paid Social**: $73 CAC, 12-month payback\n- **Search Advertising**: $89 CAC, 8-month payback\n- **Conference/Events**: $156 CAC, 24-month payback\n\n### Channel Synergy Strategy\n\nOur omnichannel approach creates reinforcing touchpoints:\n1. **Awareness**: Content marketing and thought leadership establish expertise\n2. **Consideration**: Targeted ads and case studies demonstrate value\n3. **Decision**: Free trials and sales demos close prospects\n4. **Retention**: Community building and success stories drive expansion";

    let channels_node = Node::new(
        "Marketing Channels Strategy".to_string(),
        json!(channels_content),
        Some(json!({
            "node_type": "section",
            "section_type": "strategy",
            "budget_allocated": 105000,
            "projected_leads": 8500,
            "parent_id": strategy_node_id.to_string(),
            "conversion_rate": 0.12,
            "attribution_model": "time_decay",
            "optimization_frequency": "weekly"
        })),
    );

    let channels_node_id = service.data_store.store_node(channels_node).await?;
    created_nodes.push(channels_node_id.clone());
    println!(
        "  ✓ Marketing channels strategy created: {}",
        channels_node_id
    );

    // Level 2: Budget Allocation
    println!("\n💰 Creating budget allocation strategy");
    let budget_content = "## Budget Allocation & Financial Planning\n\nTotal campaign investment: $150,000 over 6 months (July-December 2025), with additional $50,000 contingency fund for high-performing channels.\n\n### Budget Distribution Philosophy\n\nOur allocation follows the 70-20-10 rule adapted for B2B software:\n- **70% Core Channels**: Proven channels with predictable ROI\n- **20% Growth Channels**: Emerging channels with high potential\n- **10% Experimental**: Novel approaches and creative testing\n\n### Monthly Budget Breakdown\n\n**Q3 2025 (July-September): $90,000**\n- Foundation building phase focusing on awareness and content creation\n- Heavy investment in content marketing and SEO foundation\n- Beta user acquisition and case study development\n\n**Q4 2025 (October-December): $60,000**\n- Launch execution and paid acquisition acceleration\n- Event marketing and PR campaign activation\n- Performance optimization and scaling successful channels\n\n### ROI Projections and Payback Analysis\n\n**Year 1 Financial Model**:\n- Total Marketing Investment: $200,000 (including contingency)\n- Projected Customer Acquisition: 1,667 customers\n- Average Customer Lifetime Value: $3,200\n- Projected Revenue: $5,334,400\n- Marketing ROI: 26.7x\n\n**Sensitivity Analysis**:\n- **Conservative Scenario** (50% of projections): 12.3x ROI\n- **Optimistic Scenario** (150% of projections): 38.2x ROI\n- **Break-even Threshold**: 63 customers (3.8% of projection)";

    let budget_node = Node::new(
        "Budget Allocation & Financial Planning".to_string(),
        json!(budget_content),
        Some(json!({
            "node_type": "section",
            "section_type": "financial",
            "total_budget": 150000,
            "contingency_fund": 50000,
            "parent_id": strategy_node_id.to_string(),
            "currency": "USD",
            "projected_roi": 26.7,
            "payback_period_months": 8.5,
            "budget_model": "performance_based"
        })),
    );

    let budget_node_id = service.data_store.store_node(budget_node).await?;
    created_nodes.push(budget_node_id.clone());
    println!("  ✓ Budget allocation created: {}", budget_node_id);

    // Level 3: Q3 Budget Detail
    println!("\n📊 Creating Q3 budget details");
    let q3_budget_content = "### Q3 2025 Budget: Foundation Phase ($90,000)\n\n**Strategic Focus**: Market preparation, content creation, and beta user acquisition\n\n**Monthly Breakdown**:\n\n**July 2025 ($35,000)**:\n- Content creation and SEO foundation: $15,000\n- Market research completion: $8,000\n- Brand asset development: $7,000\n- Analytics setup and tool licensing: $5,000\n\n**August 2025 ($30,000)**:\n- Website development and optimization: $12,000\n- Initial content marketing campaigns: $10,000\n- Beta user recruitment: $5,000\n- Social media presence establishment: $3,000\n\n**September 2025 ($25,000)**:\n- Conference preparation and early events: $10,000\n- Paid advertising testing and optimization: $8,000\n- Case study development: $4,000\n- Email marketing automation setup: $3,000\n\n**Performance Expectations**:\n- 15,000 website visitors by end of Q3\n- 2,500 email subscribers with 40%+ engagement\n- 500 beta users providing feedback and testimonials\n- 50 pieces of content published across all channels\n- Foundation for 20 target keywords ranking in top 10";

    let q3_budget_node = Node::new(
        "Q3 2025 Budget: Foundation Phase ($90,000)".to_string(),
        json!(q3_budget_content),
        Some(json!({
            "node_type": "budget_detail",
            "quarter": "Q3_2025",
            "amount": 90000,
            "phase": "foundation",
            "parent_id": budget_node_id.to_string(),
            "expected_visitors": 15000,
            "expected_subscribers": 2500,
            "expected_beta_users": 500
        })),
    );

    let q3_budget_node_id = service.data_store.store_node(q3_budget_node).await?;
    created_nodes.push(q3_budget_node_id.clone());
    println!("  ✓ Q3 budget details created: {}", q3_budget_node_id);

    println!("\n🎉 Sample data population complete!");
    println!(
        "📈 Created {} interconnected nodes with rich content for RAG queries",
        created_nodes.len()
    );

    Ok(created_nodes)
}

// Mock implementations for demonstration
#[derive(Clone)]
struct MockDataStore {
    nodes: Arc<Mutex<HashMap<String, Node>>>,
}

impl MockDataStore {
    fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl DataStore for MockDataStore {
    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId> {
        let id = node.id.clone();
        self.nodes.lock().unwrap().insert(node.id.to_string(), node);
        Ok(id)
    }

    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        Ok(self.nodes.lock().unwrap().get(&id.to_string()).cloned())
    }

    async fn update_node(&self, node: Node) -> NodeSpaceResult<()> {
        self.nodes.lock().unwrap().insert(node.id.to_string(), node);
        Ok(())
    }

    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
        self.nodes.lock().unwrap().remove(&id.to_string());
        Ok(())
    }

    async fn query_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>> {
        let nodes = self.nodes.lock().unwrap();
        let results: Vec<Node> = nodes
            .values()
            .filter(|node| {
                if let Some(content) = node.content.as_str() {
                    content.to_lowercase().contains(&query.to_lowercase())
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        Ok(results)
    }

    async fn semantic_search(&self, _query: &str, _limit: usize) -> NodeSpaceResult<Vec<Node>> {
        // Mock implementation
        Ok(vec![])
    }
}

#[derive(Clone)]
struct MockNLPEngine;

impl MockNLPEngine {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl NLPEngine for MockNLPEngine {
    async fn generate_embedding(
        &self,
        _content: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        // Mock implementation - returns dummy embedding
        Ok(vec![0.1; 384])
    }

    async fn generate_text(
        &self,
        _prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("Mock generated text".to_string())
    }
}
