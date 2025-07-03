use chrono::{DateTime, Utc};
use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use serde_json::json;

/// Populate LanceDB with meaningful sample data for e2e semantic search testing
/// Database location: /Users/malibio/nodespace/data/lance_db
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Populating LanceDB with meaningful sample data for e2e testing");
    println!("================================================================");
    println!("📍 Database: /Users/malibio/nodespace/data/lance_db/sample_data.db");

    // Initialize service with real LanceDB
    println!("\n1️⃣ Initializing NodeSpace service with LanceDB");
    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db/sample_data.db",
        Some("./bundled_models"),
    )
    .await?;

    println!("   ✅ LanceDB service created");

    // Initialize AI services (may warn if models not available - that's OK)
    println!("\n2️⃣ Initializing AI services");
    match service.initialize().await {
        Ok(_) => println!("   ✅ AI services ready for embeddings"),
        Err(e) => {
            println!("   ⚠️  AI initialization warning: {}", e);
            println!("   ➡️  Continuing (embeddings will be mock for now)");
        }
    }

    // Get today's date for the root date node
    let today = chrono::Utc::now().date_naive();
    println!(
        "\n3️⃣ Creating date-based hierarchical sample data for {}",
        today
    );

    // Create comprehensive sample data structure
    let node_ids = populate_sample_data(&service, today).await?;

    println!("\n🎉 Sample data population complete!");
    println!("📊 Created {} nodes in LanceDB", node_ids.len());
    println!("🔍 Database ready for semantic search e2e testing");

    // Verify the data was created
    println!("\n4️⃣ Verifying sample data in database");
    let date_nodes = service.get_nodes_for_date(today).await?;
    println!("   ✅ {} nodes found for date {}", date_nodes.len(), today);

    // Test semantic search functionality
    println!("\n5️⃣ Testing semantic search with sample queries");
    test_semantic_search(&service).await?;

    println!("\n✅ LanceDB population and verification complete!");
    println!("🎯 Ready for comprehensive e2e semantic search testing");

    Ok(())
}

async fn populate_sample_data(
    service: &NodeSpaceService<
        impl nodespace_core_logic::DataStore + Send + Sync,
        impl nodespace_core_logic::NLPEngine + Send + Sync,
    >,
    today: chrono::NaiveDate,
) -> NodeSpaceResult<Vec<NodeId>> {
    let mut created_nodes = Vec::new();

    println!("   📅 Creating main campaign strategy document");

    // Create the main strategy document as a knowledge node
    let strategy_content = r#"# Product Launch Campaign Strategy

Comprehensive go-to-market strategy for launching "FlowState Pro" - our next-generation productivity platform designed for knowledge workers, creative professionals, and distributed teams.

## Executive Summary

FlowState Pro represents a paradigm shift in how professionals manage their knowledge, tasks, and collaborative workflows. Unlike traditional productivity tools that fragment attention across multiple applications, FlowState Pro provides a unified, AI-powered workspace that adapts to individual work patterns and team dynamics.

## Market Opportunity

The global productivity software market is valued at $96.36 billion and growing at 13.4% CAGR. However, 73% of knowledge workers report feeling overwhelmed by tool fragmentation, and 68% spend over 2 hours daily switching between applications. FlowState Pro addresses this critical pain point through intelligent context switching and seamless workflow integration.

## Competitive Advantage

Our unique positioning centers on three core differentiators:
1. AI-Powered Context Awareness: Machine learning algorithms that understand work patterns and proactively surface relevant information
2. Unified Knowledge Graph: All work artifacts - documents, tasks, communications - connected through semantic relationships  
3. Adaptive Interface: UI that morphs based on current work context, reducing cognitive load by 40% in beta testing

## Success Metrics

- Primary KPI: 10,000 paid subscribers within 6 months of launch
- Engagement: 80% monthly active user retention
- Revenue: $2.5M ARR by end of Year 1
- Market Penetration: 5% market share in productivity software for teams under 50 people"#;

    let strategy_node_id = service
        .create_knowledge_node(
            strategy_content,
            json!({
                "document_type": "strategy",
                "priority": "high",
                "status": "in_planning",
                "tags": ["marketing", "product-launch", "strategy", "flowstate-pro"],
                "stakeholders": ["marketing", "product", "engineering", "sales"],
                "budget_impact": 150000
            }),
        )
        .await?;

    created_nodes.push(strategy_node_id.clone());
    println!("   ✅ Strategy document created: {}", strategy_node_id);

    // Create detailed target audience analysis
    println!("   👥 Creating target audience analysis");
    let audience_content = r#"## Target Audience Analysis

Based on 18 months of market research, user interviews with 847 professionals, and analysis of 2.3 million productivity app usage patterns, we've identified distinct audience segments with specific pain points and preferences.

### Research Methodology

Our audience analysis employed mixed-method research including:
- Quantitative Survey: 2,500 knowledge workers across 15 countries
- Qualitative Interviews: 125 in-depth interviews (45-90 minutes each)
- Usage Analytics: Behavioral data from 50,000+ productivity app users
- Ethnographic Studies: Workplace observation in 12 organizations

### Key Findings

Pain Point Hierarchy (ranked by frequency and intensity):
1. Context Switching Fatigue: 89% report mental exhaustion from app switching
2. Information Fragmentation: 76% struggle to find relevant information quickly
3. Collaboration Friction: 68% cite tool inconsistency as collaboration barrier
4. Cognitive Overload: 84% feel overwhelmed by notification volume

Behavioral Patterns:
- Average professional uses 9.4 different productivity tools daily
- 67% prefer unified interfaces over specialized tools
- 78% willing to pay premium for significant time savings
- Early adopters influence 3.2 colleagues on average

### Psychological Profiles

Our research identified three distinct psychological archetypes:

The Optimizer (42% of target market): Methodical professionals who systematically seek efficiency improvements. High conscientiousness, moderate openness to experience.

The Innovator (31% of target market): Creative professionals who value flexibility and novel approaches. High openness, moderate conscientiousness.

The Collaborator (27% of target market): Team-oriented professionals prioritizing seamless group workflows. High agreeableness, moderate extraversion."#;

    let audience_node_id = service
        .create_knowledge_node(
            audience_content,
            json!({
                "section_type": "analysis",
                "completion": 0.95,
                "research_cost": 125000,
                "confidence_level": 0.87,
                "data_sources": ["survey", "interviews", "analytics", "ethnography"],
                "sample_size": 52500
            }),
        )
        .await?;

    created_nodes.push(audience_node_id.clone());
    println!("   ✅ Audience analysis created: {}", audience_node_id);

    // Create primary demographics persona
    println!("   🎯 Creating primary demographics persona");
    let demographics_content = r#"### Primary Demographics: Tech-Savvy Professionals (58% of target market)

Core Profile: Knowledge workers in technology, consulting, and creative industries who are early adopters of productivity tools and willing to invest in efficiency improvements.

Detailed Demographics:
- Age Range: 28-42 years (median: 34 years)
- Education: 87% bachelor's degree or higher, 34% advanced degrees
- Income: $75,000-$140,000 annually (median: $95,000)
- Geographic Distribution: 67% North America, 21% Europe, 12% Asia-Pacific
- Company Size: 45% work at companies with 50-500 employees
- Role Types: Product managers (23%), developers (19%), consultants (16%), designers (14%), analysts (12%), others (16%)

Psychographic Characteristics:
- Technology Adoption: Early majority to early adopters (Rogers curve)
- Work Style: Prefer asynchronous communication, value deep work time
- Decision Making: Research-driven, compare multiple options, seek peer recommendations
- Pain Tolerance: Low tolerance for inefficient processes, high standards for tool quality
- Learning Preference: Self-directed learning, prefer documentation over training calls

Professional Behaviors:
- Tool Usage: Use 8-12 professional software tools daily
- Information Consumption: Read 2-3 industry publications weekly, follow 15-20 thought leaders
- Networking: Active on LinkedIn, attend 1-2 industry conferences annually
- Purchase Authority: 73% have budget influence, 45% are primary decision makers

Productivity Pain Points (in order of severity):
1. Context Switching: Lose 23 minutes per interruption, 67 switches daily average
2. Information Retrieval: Spend 2.5 hours daily searching for information
3. Status Reporting: 45 minutes daily on progress updates and check-ins
4. Tool Integration: 38% of work involves copying data between applications

Willingness to Pay: 78% would pay $50-100/month for 20% productivity improvement"#;

    let demographics_node_id = service
        .create_knowledge_node(
            demographics_content,
            json!({
                "persona_name": "tech_professional",
                "market_share": 0.58,
                "confidence_level": 0.91,
                "data_source": "primary_research",
                "sample_size": 1450,
                "avg_income": 95000,
                "willingness_to_pay": 75
            }),
        )
        .await?;

    created_nodes.push(demographics_node_id.clone());
    println!(
        "   ✅ Demographics persona created: {}",
        demographics_node_id
    );

    // Create marketing channels strategy
    println!("   📢 Creating marketing channels strategy");
    let channels_content = r#"## Marketing Channels Strategy

Multi-channel approach leveraging both digital and traditional marketing to achieve 360-degree market penetration. Our channel selection is data-driven, based on audience research showing where our target demographics consume professional content and make software purchasing decisions.

### Channel Performance Modeling

Using attribution modeling and customer journey mapping, we've projected the following channel effectiveness:

Digital Channels (70% of budget, 85% of leads):
- Content Marketing: Highest quality leads (47% conversion rate)
- LinkedIn Advertising: Best B2B reach (2.3M relevant professionals)
- Search Marketing: Highest intent signals (67% purchase consideration)
- Email Marketing: Strongest retention tool (4.2x engagement vs. social)

Traditional Channels (30% of budget, 15% of leads):
- Industry Publications: Credibility building and thought leadership
- Conference Speaking: Direct engagement with decision makers
- Partnership Marketing: Leveraging existing vendor relationships

### Customer Acquisition Cost (CAC) Projections

- Content Marketing: $47 CAC, 18-month payback
- Paid Social: $73 CAC, 12-month payback
- Search Advertising: $89 CAC, 8-month payback
- Conference/Events: $156 CAC, 24-month payback

### Channel Synergy Strategy

Our omnichannel approach creates reinforcing touchpoints:
1. Awareness: Content marketing and thought leadership establish expertise
2. Consideration: Targeted ads and case studies demonstrate value
3. Decision: Free trials and sales demos close prospects
4. Retention: Community building and success stories drive expansion

### Digital Marketing Deep Dive

Content Marketing & SEO ($35,000 - 33% of digital budget):
Strategy: Establish thought leadership and organic discovery through high-value content addressing core productivity challenges.

Tactical Execution:
- Blog Content: 3 long-form articles weekly (2,000-4,000 words) covering productivity science, workflow optimization, and industry trends
- Resource Library: Comprehensive guides, templates, and frameworks (15 major resources over 6 months)
- Video Content: Weekly YouTube series "Productivity Decoded" featuring expert interviews and case studies
- Podcast Sponsorships: Strategic sponsorships of 5 top productivity and business podcasts

Performance Targets:
- 500,000 monthly organic page views by month 6
- 25,000 email subscribers with 35% monthly engagement
- Top 3 rankings for 50 productivity-related keywords
- 15,000 YouTube subscribers with 8% monthly growth"#;

    let channels_node_id = service
        .create_knowledge_node(
            channels_content,
            json!({
                "section_type": "strategy",
                "budget_allocated": 105000,
                "projected_leads": 8500,
                "conversion_rate": 0.12,
                "attribution_model": "time_decay",
                "optimization_frequency": "weekly"
            }),
        )
        .await?;

    created_nodes.push(channels_node_id.clone());
    println!("   ✅ Marketing channels created: {}", channels_node_id);

    // Create budget allocation details
    println!("   💰 Creating budget allocation strategy");
    let budget_content = r#"## Budget Allocation & Financial Planning

Total campaign investment: $150,000 over 6 months (July-December 2025), with additional $50,000 contingency fund for high-performing channels.

### Budget Distribution Philosophy

Our allocation follows the 70-20-10 rule adapted for B2B software:
- 70% Core Channels: Proven channels with predictable ROI
- 20% Growth Channels: Emerging channels with high potential
- 10% Experimental: Novel approaches and creative testing

### Monthly Budget Breakdown

Q3 2025 (July-September): $90,000
- Foundation building phase focusing on awareness and content creation
- Heavy investment in content marketing and SEO foundation
- Beta user acquisition and case study development

Q4 2025 (October-December): $60,000
- Launch execution and paid acquisition acceleration
- Event marketing and PR campaign activation
- Performance optimization and scaling successful channels

### ROI Projections and Payback Analysis

Year 1 Financial Model:
- Total Marketing Investment: $200,000 (including contingency)
- Projected Customer Acquisition: 1,667 customers
- Average Customer Lifetime Value: $3,200
- Projected Revenue: $5,334,400
- Marketing ROI: 26.7x

Sensitivity Analysis:
- Conservative Scenario (50% of projections): 12.3x ROI
- Optimistic Scenario (150% of projections): 38.2x ROI
- Break-even Threshold: 63 customers (3.8% of projection)

### Budget Control and Optimization

Weekly budget reviews with real-time reallocation based on:
- Performance Metrics: CPA, conversion rates, engagement quality
- Market Feedback: Customer interviews, sales team insights
- Competitive Activity: Competitor campaign analysis and response
- Seasonal Factors: Industry events, economic conditions, holidays

### Q3 2025 Budget Details: Foundation Phase ($90,000)

Strategic Focus: Market preparation, content creation, and beta user acquisition

Monthly Breakdown:

July 2025 ($35,000):
- Content creation and SEO foundation: $15,000
- Market research completion: $8,000
- Brand asset development: $7,000
- Analytics setup and tool licensing: $5,000

August 2025 ($30,000):
- Website development and optimization: $12,000
- Initial content marketing campaigns: $10,000
- Beta user recruitment: $5,000
- Social media presence establishment: $3,000

September 2025 ($25,000):
- Conference preparation and early events: $10,000
- Paid advertising testing and optimization: $8,000
- Case study development: $4,000
- Email marketing automation setup: $3,000

Performance Expectations:
- 15,000 website visitors by end of Q3
- 2,500 email subscribers with 40%+ engagement
- 500 beta users providing feedback and testimonials
- 50 pieces of content published across all channels
- Foundation for 20 target keywords ranking in top 10"#;

    let budget_node_id = service
        .create_knowledge_node(
            budget_content,
            json!({
                "section_type": "financial",
                "total_budget": 150000,
                "contingency_fund": 50000,
                "currency": "USD",
                "projected_roi": 26.7,
                "payback_period_months": 8.5,
                "budget_model": "performance_based"
            }),
        )
        .await?;

    created_nodes.push(budget_node_id.clone());
    println!("   ✅ Budget allocation created: {}", budget_node_id);

    // Create timeline and milestones
    println!("   📅 Creating project timeline and milestones");
    let timeline_content = r#"## Project Timeline & Key Milestones

Six-month campaign execution divided into four distinct phases, each with specific objectives, deliverables, and success criteria. Timeline is designed for maximum market impact while maintaining operational excellence.

### Critical Path Analysis

Our timeline identifies dependencies and potential bottlenecks:

Critical Path Dependencies:
1. Product beta completion → Case study development → Launch materials
2. Market research → Messaging framework → Content creation
3. Brand assets → Website development → Paid advertising campaigns
4. Sales training → Channel partner enablement → Launch execution

### Phase 1: Market Research & Strategy Finalization (July-August 2025)

Objective: Complete market intelligence gathering and finalize go-to-market strategy based on comprehensive research findings.

Week-by-Week Breakdown:

Weeks 1-2 (Early July):
- Competitive landscape analysis: Deep dive into 15 direct and 25 indirect competitors
- Pricing strategy analysis: Evaluation of freemium vs. paid models, price sensitivity testing
- Feature gap analysis: Identification of unique value propositions and market whitespace
- Initial customer interviews: 25 detailed interviews with target personas

Weeks 3-4 (Mid-July):
- Survey deployment: Large-scale survey to 5,000 productivity software users
- Focus group sessions: 6 sessions across different demographic segments
- User journey mapping: Detailed analysis of current productivity workflows
- Beta feature prioritization: Ranking features for MVP based on market feedback

Weeks 5-6 (Late July):
- Data analysis and insight synthesis: Statistical analysis of all research data
- Persona refinement: Final persona documents with detailed behavioral insights
- Messaging framework development: Core value propositions and positioning statements
- Competitive response planning: Anticipation of competitor reactions and counter-strategies

Weeks 7-8 (Early August):
- Strategy validation: Testing messaging and positioning with focus groups
- Go-to-market plan finalization: Channel strategy, timeline, and resource allocation
- Team alignment workshops: Ensuring all stakeholders understand and commit to strategy
- Phase 2 preparation: Detailed planning and resource allocation for content creation phase

Key Deliverables:
1. Competitive Analysis Report: 50-page comprehensive analysis with strategic recommendations
2. Customer Persona Guide: Detailed profiles of 5 primary personas with behavioral insights
3. Messaging Framework: Core positioning statements and value propositions for each segment
4. Market Sizing Model: TAM/SAM/SOM analysis with growth projections
5. Pricing Strategy: Recommended pricing tiers with supporting research
6. Go-to-Market Plan: Detailed 6-month execution plan with metrics and milestones

### Phase 2: Content Creation & Brand Building (September 2025)

Objective: Create comprehensive content library and establish brand presence across all marketing channels.

Content Creation Sprint:

Week 1: Foundation Content:
- Website copy and landing pages: 15 pages of optimized content
- Product positioning materials: Sales decks, one-pagers, FAQs
- SEO content strategy: Keyword research and content calendar for 6 months
- Brand voice and style guide: Comprehensive guidelines for all communications

Week 2: Educational Content:
- Blog article series: 12 in-depth articles on productivity science and best practices
- Downloadable resources: 5 comprehensive guides, templates, and frameworks
- Video content: 8 product demo videos and 4 thought leadership interviews
- Webinar content: 3 educational webinars with registration landing pages

Week 3: Marketing Assets:
- Social media content: 100 posts across LinkedIn, Twitter, and YouTube
- Email marketing templates: Welcome series, nurture campaigns, and promotional emails
- Paid advertising creative: 25 ad variations for A/B testing across platforms
- Case study development: 3 detailed customer success stories with metrics

Week 4: Launch Preparation:
- Press kit development: Media resources, executive bios, product fact sheets
- Partnership materials: Co-marketing templates and collaboration frameworks
- Sales enablement: Battle cards, objection handling guides, demo scripts
- Customer onboarding: Tutorial content, help documentation, success frameworks

Success Metrics:
- 50+ pieces of content published across all formats
- Website traffic increase of 300% from baseline
- Email list growth to 2,000 subscribers with 35%+ engagement
- Social media following growth of 150% across all platforms"#;

    let timeline_node_id = service
        .create_knowledge_node(
            timeline_content,
            json!({
                "section_type": "planning",
                "duration_months": 6,
                "total_phases": 4,
                "team_fte": 7.2,
                "risk_level": "medium",
                "contingency_weeks": 2
            }),
        )
        .await?;

    created_nodes.push(timeline_node_id.clone());
    println!(
        "   ✅ Timeline and milestones created: {}",
        timeline_node_id
    );

    // Create competitive analysis
    println!("   🏆 Creating competitive analysis");
    let competitive_content = r#"## Competitive Analysis & Market Positioning

Comprehensive analysis of the productivity software competitive landscape, identifying opportunities for differentiation and strategic positioning.

### Direct Competitors

Notion:
- Strengths: Flexible database-driven approach, strong community, excellent customization
- Weaknesses: Steep learning curve, performance issues with large datasets, complex for casual users
- Market Position: All-in-one workspace for power users
- Pricing: Freemium, $8-16/user/month for teams
- User Base: 30M+ users, strong with technical teams and content creators

Obsidian:
- Strengths: Powerful linking and graph view, local-first storage, plugin ecosystem
- Weaknesses: Technical complexity, limited collaboration features, intimidating for non-technical users
- Market Position: Knowledge management for researchers and writers
- Pricing: Free for personal use, $50/user/year for commercial
- User Base: 1M+ users, primarily individual knowledge workers

Roam Research:
- Strengths: Pioneered bidirectional linking, strong community of researchers
- Weaknesses: Complex interface, limited formatting options, expensive pricing
- Market Position: Research and note-taking for academics and consultants
- Pricing: $15/month personal, $25/month professional
- User Base: 500K+ users, niche but passionate user base

### Indirect Competitors

Microsoft 365 Suite:
- Strengths: Enterprise integration, familiar interface, comprehensive feature set
- Weaknesses: Tool fragmentation (exactly what we're solving), complex licensing
- Market Position: Enterprise standard for productivity
- Opportunity: Users frustrated with context switching between Word, Excel, Teams, etc.

Google Workspace:
- Strengths: Real-time collaboration, cloud-native, simple sharing
- Weaknesses: Limited offline capabilities, privacy concerns, fragmented experience
- Market Position: Cloud-first collaboration for modern teams
- Opportunity: Privacy-conscious users wanting local-first alternative

Slack + Asana + Google Drive combinations:
- Strengths: Best-in-class for specific functions
- Weaknesses: Context switching fatigue, integration complexity, cost accumulation
- Opportunity: Users paying for 5-8 tools when they want unified experience

### Competitive Advantages

1. AI-Powered Context Awareness:
Unlike static tools, FlowState Pro learns user patterns and proactively surfaces relevant information. Competitors require manual organization and retrieval.

2. Unified Knowledge Graph:
While others connect documents through folders or tags, we create semantic relationships automatically, making information discovery natural and effortless.

3. Adaptive Interface:
Our UI changes based on context - writing mode, planning mode, analysis mode - reducing cognitive load. Competitors use one-size-fits-all interfaces.

4. Local-First Privacy:
Unlike cloud-dependent competitors, we offer the security of local storage with the power of AI, addressing growing privacy concerns.

### Market Positioning Strategy

Primary Message: "The productivity platform that thinks like you do"

Key Differentiators:
- Intelligent vs. Manual: AI discovers relationships you'd miss
- Unified vs. Fragmented: One tool instead of eight
- Adaptive vs. Static: Interface that changes with your needs
- Private vs. Surveilled: Your data stays yours

Target Positioning:
- Primary: "Notion for people who want it to think"
- Secondary: "The anti-fragmentation productivity solution"
- Tertiary: "AI-powered knowledge management that respects privacy"

### Pricing Strategy vs. Competitors

Market Analysis:
- Freemium models dominate (Notion, Obsidian)
- Professional tiers range $8-25/month
- Enterprise pricing varies widely ($15-50/user/month)

Our Strategy:
- Freemium: Basic AI features, limited storage
- Professional: $12/month (positioned between Notion and Roam)
- Teams: $20/user/month with advanced AI and collaboration
- Enterprise: Custom pricing with on-premise options

Value Justification:
At $12/month, users save money by replacing multiple tools:
- Notion ($8) + Roam ($15) + AI assistant ($20) = $43/month
- FlowState Pro Professional = $12/month
- Savings: $31/month per user ($372/year)

### Go-to-Market Implications

Channel Strategy:
- Content marketing emphasizing "unified workspace" messaging
- Direct comparison content showing tool consolidation savings
- Product-led growth with generous freemium tier
- Community building around "productivity without fragmentation"

Messaging Hierarchy:
1. Problem: Tool fragmentation is killing productivity
2. Solution: One intelligent tool that adapts to how you work
3. Proof: AI that learns your patterns and connects your ideas
4. Urgency: Stop paying for eight tools when one can do it all

Launch Sequence:
1. Beta with productivity influencers and early adopters
2. Public launch with "tool consolidation calculator"
3. Competitive comparison content and migration tools
4. Community building and user-generated success stories"#;

    let competitive_node_id = service
        .create_knowledge_node(
            competitive_content,
            json!({
                "analysis_type": "competitive",
                "competitors_analyzed": 12,
                "market_size": "96.36B",
                "positioning": "intelligent_unified_workspace",
                "pricing_strategy": "value_based",
                "competitive_advantage": "ai_context_awareness"
            }),
        )
        .await?;

    created_nodes.push(competitive_node_id.clone());
    println!(
        "   ✅ Competitive analysis created: {}",
        competitive_node_id
    );

    Ok(created_nodes)
}

async fn test_semantic_search(
    service: &NodeSpaceService<
        impl nodespace_core_logic::DataStore + Send + Sync,
        impl nodespace_core_logic::NLPEngine + Send + Sync,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   🔍 Testing semantic search queries");

    let test_queries = vec![
        "What is our target audience for the product launch?",
        "How much budget is allocated for digital marketing?",
        "What are the key phases in our launch timeline?",
        "Who are the primary demographics we're targeting?",
        "What marketing channels are we using and why?",
        "What are our competitive advantages?",
        "What is our pricing strategy compared to competitors?",
        "How do we measure campaign success?",
    ];

    for (i, query) in test_queries.iter().enumerate() {
        println!("      Query {}: \"{}\"", i + 1, query);

        match service.semantic_search(query, 3).await {
            Ok(results) => {
                println!("         ✅ Found {} relevant results", results.len());
                for (j, result) in results.iter().take(2).enumerate() {
                    let preview = result
                        .node
                        .content
                        .as_str()
                        .unwrap_or("No content")
                        .chars()
                        .take(100)
                        .collect::<String>();
                    println!(
                        "            {}. Score: {:.3} - {}",
                        j + 1,
                        result.score,
                        preview
                    );
                }
            }
            Err(e) => {
                println!("         ⚠️  Search error: {}", e);
            }
        }
        println!();
    }

    Ok(())
}
