use chrono::{DateTime, NaiveDate, Utc};
use nodespace_core_logic::{DataStore, NLPEngine, NodeSpaceService};
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Sample data population script for NodeSpace with meaningful RAG-queryable content
///
/// Structure:
/// - Date Node (2025-07-02)
///   - Product Launch Campaign Strategy (main document)
///     - Target Audience Analysis (detailed sections)
///     - Marketing Channels (comprehensive strategy)
///     - Budget Allocation (financial details)
///     - Timeline & Milestones (project phases)

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🚀 Populating NodeSpace database with meaningful sample data");
    println!("=======================================================");

    // Note: This example shows the data structure we want to create
    // In practice, this would use actual DataStore and NLP implementations

    let sample_data = create_comprehensive_sample_data();

    for (level, node_data) in sample_data.iter() {
        println!(
            "{} {}: {}",
            "  ".repeat(*level),
            node_data.node_type,
            node_data.title
        );
    }

    println!("\n✅ Comprehensive sample data structure created");
    println!("📊 Total nodes: {}", sample_data.len());
    println!("💬 Ready for RAG queries about:");
    println!("   • Product launch strategies and timelines");
    println!("   • Budget allocations and financial planning");
    println!("   • Target audience demographics and psychographics");
    println!("   • Marketing channels and campaign effectiveness");
    println!("   • Project phases and milestone deliverables");

    Ok(())
}

#[derive(Debug, Clone)]
struct SampleNodeData {
    title: String,
    content: String,
    node_type: String,
    parent_id: Option<String>,
    metadata: serde_json::Value,
}

fn create_comprehensive_sample_data() -> Vec<(usize, SampleNodeData)> {
    let mut nodes = Vec::new();
    let today = chrono::Local::now().date_naive();

    // Level 0: Date Node (Root)
    let date_node_id = format!("date_{}", today.format("%Y_%m_%d"));
    nodes.push((0, SampleNodeData {
        title: format!("Daily Journal - {}", today.format("%B %d, %Y")),
        content: format!("Daily planning and activities for {}. Today marks an important milestone in our product development cycle as we finalize the launch campaign strategy. The team has been working intensively on market research, competitive analysis, and customer persona development over the past quarter.", today.format("%A, %B %d, %Y")),
        node_type: "date".to_string(),
        parent_id: None,
        metadata: json!({
            "date": today.format("%Y-%m-%d").to_string(),
            "day_of_week": today.format("%A").to_string(),
            "created_at": Utc::now().to_rfc3339(),
            "importance": "high",
            "tags": ["planning", "product-launch", "strategy"]
        }),
    }));

    // Level 1: Main Document - Detailed strategic overview
    let campaign_id = Uuid::new_v4().to_string();
    nodes.push((1, SampleNodeData {
        title: "Product Launch Campaign Strategy".to_string(),
        content: "# Product Launch Campaign Strategy\n\nComprehensive go-to-market strategy for launching \"FlowState Pro\" - our next-generation productivity platform designed for knowledge workers, creative professionals, and distributed teams.\n\n## Executive Summary\n\nFlowState Pro represents a paradigm shift in how professionals manage their knowledge, tasks, and collaborative workflows. Unlike traditional productivity tools that fragment attention across multiple applications, FlowState Pro provides a unified, AI-powered workspace that adapts to individual work patterns and team dynamics.\n\n## Market Opportunity\n\nThe global productivity software market is valued at $96.36 billion and growing at 13.4% CAGR. However, 73% of knowledge workers report feeling overwhelmed by tool fragmentation, and 68% spend over 2 hours daily switching between applications. FlowState Pro addresses this critical pain point through intelligent context switching and seamless workflow integration.\n\n## Competitive Advantage\n\nOur unique positioning centers on three core differentiators:\n1. **AI-Powered Context Awareness**: Machine learning algorithms that understand work patterns and proactively surface relevant information\n2. **Unified Knowledge Graph**: All work artifacts - documents, tasks, communications - connected through semantic relationships\n3. **Adaptive Interface**: UI that morphs based on current work context, reducing cognitive load by 40% in beta testing\n\n## Success Metrics\n\n- **Primary KPI**: 10,000 paid subscribers within 6 months of launch\n- **Engagement**: 80% monthly active user retention\n- **Revenue**: $2.5M ARR by end of Year 1\n- **Market Penetration**: 5% market share in productivity software for teams under 50 people".to_string(),
        node_type: "document".to_string(),
        parent_id: Some(date_node_id.clone()),
        metadata: json!({
            "document_type": "strategy",
            "priority": "high",
            "status": "in_planning",
            "tags": ["marketing", "product-launch", "strategy", "flowstate-pro"],
            "stakeholders": ["marketing", "product", "engineering", "sales"],
            "approval_required": true,
            "budget_impact": 150000
        }),
    }));

    // Level 2: Target Audience Analysis - Deep demographic insights
    let audience_id = Uuid::new_v4().to_string();
    nodes.push((2, SampleNodeData {
        title: "Target Audience Analysis".to_string(),
        content: "## Target Audience Analysis\n\nBased on 18 months of market research, user interviews with 847 professionals, and analysis of 2.3 million productivity app usage patterns, we've identified distinct audience segments with specific pain points and preferences.\n\n### Research Methodology\n\nOur audience analysis employed mixed-method research including:\n- **Quantitative Survey**: 2,500 knowledge workers across 15 countries\n- **Qualitative Interviews**: 125 in-depth interviews (45-90 minutes each)\n- **Usage Analytics**: Behavioral data from 50,000+ productivity app users\n- **Ethnographic Studies**: Workplace observation in 12 organizations\n\n### Key Findings\n\n**Pain Point Hierarchy** (ranked by frequency and intensity):\n1. **Context Switching Fatigue**: 89% report mental exhaustion from app switching\n2. **Information Fragmentation**: 76% struggle to find relevant information quickly\n3. **Collaboration Friction**: 68% cite tool inconsistency as collaboration barrier\n4. **Cognitive Overload**: 84% feel overwhelmed by notification volume\n\n**Behavioral Patterns**:\n- Average professional uses 9.4 different productivity tools daily\n- 67% prefer unified interfaces over specialized tools\n- 78% willing to pay premium for significant time savings\n- Early adopters influence 3.2 colleagues on average\n\n### Psychological Profiles\n\nOur research identified three distinct psychological archetypes:\n\n**The Optimizer** (42% of target market): Methodical professionals who systematically seek efficiency improvements. High conscientiousness, moderate openness to experience.\n\n**The Innovator** (31% of target market): Creative professionals who value flexibility and novel approaches. High openness, moderate conscientiousness.\n\n**The Collaborator** (27% of target market): Team-oriented professionals prioritizing seamless group workflows. High agreeableness, moderate extraversion.".to_string(),
        node_type: "section".to_string(),
        parent_id: Some(campaign_id.clone()),
        metadata: json!({
            "section_type": "analysis",
            "completion": 0.95,
            "research_cost": 125000,
            "confidence_level": 0.87,
            "data_sources": ["survey", "interviews", "analytics", "ethnography"],
            "sample_size": 52500
        }),
    }));

    // Level 2: Marketing Channels - Comprehensive strategy
    let channels_id = Uuid::new_v4().to_string();
    nodes.push((2, SampleNodeData {
        title: "Marketing Channels".to_string(),
        content: "## Marketing Channels Strategy\n\nMulti-channel approach leveraging both digital and traditional marketing to achieve 360-degree market penetration. Our channel selection is data-driven, based on audience research showing where our target demographics consume professional content and make software purchasing decisions.\n\n### Channel Performance Modeling\n\nUsing attribution modeling and customer journey mapping, we've projected the following channel effectiveness:\n\n**Digital Channels** (70% of budget, 85% of leads):\n- **Content Marketing**: Highest quality leads (47% conversion rate)\n- **LinkedIn Advertising**: Best B2B reach (2.3M relevant professionals)\n- **Search Marketing**: Highest intent signals (67% purchase consideration)\n- **Email Marketing**: Strongest retention tool (4.2x engagement vs. social)\n\n**Traditional Channels** (30% of budget, 15% of leads):\n- **Industry Publications**: Credibility building and thought leadership\n- **Conference Speaking**: Direct engagement with decision makers\n- **Partnership Marketing**: Leveraging existing vendor relationships\n\n### Customer Acquisition Cost (CAC) Projections\n\n- **Content Marketing**: $47 CAC, 18-month payback\n- **Paid Social**: $73 CAC, 12-month payback\n- **Search Advertising**: $89 CAC, 8-month payback\n- **Conference/Events**: $156 CAC, 24-month payback\n\n### Channel Synergy Strategy\n\nOur omnichannel approach creates reinforcing touchpoints:\n1. **Awareness**: Content marketing and thought leadership establish expertise\n2. **Consideration**: Targeted ads and case studies demonstrate value\n3. **Decision**: Free trials and sales demos close prospects\n4. **Retention**: Community building and success stories drive expansion".to_string(),
        node_type: "section".to_string(),
        parent_id: Some(campaign_id.clone()),
        metadata: json!({
            "section_type": "strategy",
            "budget_allocated": 105000,
            "projected_leads": 8500,
            "conversion_rate": 0.12,
            "attribution_model": "time_decay",
            "optimization_frequency": "weekly"
        }),
    }));

    // Level 2: Budget Allocation - Financial strategy
    let budget_id = Uuid::new_v4().to_string();
    nodes.push((2, SampleNodeData {
        title: "Budget Allocation & Financial Planning".to_string(),
        content: "## Budget Allocation & Financial Planning\n\nTotal campaign investment: $150,000 over 6 months (July-December 2025), with additional $50,000 contingency fund for high-performing channels.\n\n### Budget Distribution Philosophy\n\nOur allocation follows the 70-20-10 rule adapted for B2B software:\n- **70% Core Channels**: Proven channels with predictable ROI\n- **20% Growth Channels**: Emerging channels with high potential\n- **10% Experimental**: Novel approaches and creative testing\n\n### Monthly Budget Breakdown\n\n**Q3 2025 (July-September): $90,000**\n- Foundation building phase focusing on awareness and content creation\n- Heavy investment in content marketing and SEO foundation\n- Beta user acquisition and case study development\n\n**Q4 2025 (October-December): $60,000**\n- Launch execution and paid acquisition acceleration\n- Event marketing and PR campaign activation\n- Performance optimization and scaling successful channels\n\n### ROI Projections and Payback Analysis\n\n**Year 1 Financial Model**:\n- Total Marketing Investment: $200,000 (including contingency)\n- Projected Customer Acquisition: 1,667 customers\n- Average Customer Lifetime Value: $3,200\n- Projected Revenue: $5,334,400\n- Marketing ROI: 26.7x\n\n**Sensitivity Analysis**:\n- **Conservative Scenario** (50% of projections): 12.3x ROI\n- **Optimistic Scenario** (150% of projections): 38.2x ROI\n- **Break-even Threshold**: 63 customers (3.8% of projection)\n\n### Budget Control and Optimization\n\nWeekly budget reviews with real-time reallocation based on:\n- **Performance Metrics**: CPA, conversion rates, engagement quality\n- **Market Feedback**: Customer interviews, sales team insights\n- **Competitive Activity**: Competitor campaign analysis and response\n- **Seasonal Factors**: Industry events, economic conditions, holidays".to_string(),
        node_type: "section".to_string(),
        parent_id: Some(campaign_id.clone()),
        metadata: json!({
            "section_type": "financial",
            "total_budget": 150000,
            "contingency_fund": 50000,
            "currency": "USD",
            "projected_roi": 26.7,
            "payback_period_months": 8.5,
            "budget_model": "performance_based"
        }),
    }));

    // Level 2: Timeline & Milestones - Project management
    let timeline_id = Uuid::new_v4().to_string();
    nodes.push((2, SampleNodeData {
        title: "Timeline & Milestones".to_string(),
        content: "## Project Timeline & Key Milestones\n\nSix-month campaign execution divided into four distinct phases, each with specific objectives, deliverables, and success criteria. Timeline is designed for maximum market impact while maintaining operational excellence.\n\n### Critical Path Analysis\n\nOur timeline identifies dependencies and potential bottlenecks:\n\n**Critical Path Dependencies**:\n1. Product beta completion → Case study development → Launch materials\n2. Market research → Messaging framework → Content creation\n3. Brand assets → Website development → Paid advertising campaigns\n4. Sales training → Channel partner enablement → Launch execution\n\n### Phase Gate Reviews\n\nEach phase concludes with a formal review including:\n- **Deliverable Assessment**: Quality and completeness verification\n- **Performance Analysis**: KPI tracking against projections\n- **Risk Evaluation**: Identification and mitigation of emerging risks\n- **Budget Review**: Spend analysis and reallocation decisions\n- **Go/No-Go Decision**: Approval to proceed to next phase\n\n### Resource Allocation Timeline\n\n**Team Commitment** (Full-Time Equivalents):\n- **Marketing Team**: 2.5 FTE throughout campaign\n- **Product Marketing**: 1.0 FTE (Phases 1-2), 1.5 FTE (Phases 3-4)\n- **Design Team**: 1.0 FTE (Phase 2), 0.5 FTE (ongoing)\n- **Sales Team**: 0.5 FTE (Phase 1), 2.0 FTE (Phases 3-4)\n- **Executive Sponsorship**: 0.2 FTE throughout\n\n### Risk Mitigation Strategies\n\n**High-Impact Risks and Mitigation**:\n1. **Product Delays**: 2-week buffer built into timeline, MVP approach for launch\n2. **Competition**: Continuous competitive monitoring, agile messaging adaptation\n3. **Market Changes**: Quarterly strategy reviews, pivot protocols established\n4. **Team Capacity**: Cross-training, external contractor relationships, skill redundancy".to_string(),
        node_type: "section".to_string(),
        parent_id: Some(campaign_id.clone()),
        metadata: json!({
            "section_type": "planning",
            "duration_months": 6,
            "total_phases": 4,
            "team_fte": 7.2,
            "risk_level": "medium",
            "contingency_weeks": 2
        }),
    }));

    // Level 3: Primary Demographics - Detailed persona
    nodes.push((3, SampleNodeData {
        title: "Primary Demographics: Tech-Savvy Professionals".to_string(),
        content: "### Primary Demographics: Tech-Savvy Professionals (58% of target market)\n\n**Core Profile**: Knowledge workers in technology, consulting, and creative industries who are early adopters of productivity tools and willing to invest in efficiency improvements.\n\n**Detailed Demographics**:\n- **Age Range**: 28-42 years (median: 34 years)\n- **Education**: 87% bachelor's degree or higher, 34% advanced degrees\n- **Income**: $75,000-$140,000 annually (median: $95,000)\n- **Geographic Distribution**: 67% North America, 21% Europe, 12% Asia-Pacific\n- **Company Size**: 45% work at companies with 50-500 employees\n- **Role Types**: Product managers (23%), developers (19%), consultants (16%), designers (14%), analysts (12%), others (16%)\n\n**Psychographic Characteristics**:\n- **Technology Adoption**: Early majority to early adopters (Rogers curve)\n- **Work Style**: Prefer asynchronous communication, value deep work time\n- **Decision Making**: Research-driven, compare multiple options, seek peer recommendations\n- **Pain Tolerance**: Low tolerance for inefficient processes, high standards for tool quality\n- **Learning Preference**: Self-directed learning, prefer documentation over training calls\n\n**Professional Behaviors**:\n- **Tool Usage**: Use 8-12 professional software tools daily\n- **Information Consumption**: Read 2-3 industry publications weekly, follow 15-20 thought leaders\n- **Networking**: Active on LinkedIn, attend 1-2 industry conferences annually\n- **Purchase Authority**: 73% have budget influence, 45% are primary decision makers\n\n**Productivity Pain Points** (in order of severity):\n1. **Context Switching**: Lose 23 minutes per interruption, 67 switches daily average\n2. **Information Retrieval**: Spend 2.5 hours daily searching for information\n3. **Status Reporting**: 45 minutes daily on progress updates and check-ins\n4. **Tool Integration**: 38% of work involves copying data between applications\n\n**Willingness to Pay**: 78% would pay $50-100/month for 20% productivity improvement".to_string(),
        node_type: "persona".to_string(),
        parent_id: Some(audience_id.clone()),
        metadata: json!({
            "persona_name": "tech_professional",
            "market_share": 0.58,
            "confidence_level": 0.91,
            "data_source": "primary_research",
            "sample_size": 1450,
            "avg_income": 95000,
            "willingness_to_pay": 75
        }),
    }));

    // Level 3: Secondary Markets - Growth segments
    nodes.push((3, SampleNodeData {
        title: "Secondary Markets: Emerging Segments".to_string(),
        content: "### Secondary Markets: Emerging Segments (42% of target market)\n\n**Graduate Students & Early Career** (18% of target market):\nMaster's and PhD students in business, technology, and design programs, plus professionals in their first 3 years of career.\n\n- **Age**: 22-28 years\n- **Income**: $25,000-$55,000 (often supplemented by stipends/part-time work)\n- **Characteristics**: Highly motivated to optimize productivity, price-sensitive, influential within peer networks\n- **Use Cases**: Research management, thesis writing, job search organization, skill development tracking\n- **Acquisition Strategy**: Student discounts, university partnerships, campus ambassador programs\n\n**Small Business Owners & Entrepreneurs** (15% of target market):\nFounders and owners of businesses with 2-25 employees, particularly in consulting, creative services, and technology.\n\n- **Age**: 30-45 years\n- **Income**: Highly variable ($40,000-$200,000+)\n- **Characteristics**: Value ROI, need scalable solutions, wear multiple hats, budget conscious but willing to invest in growth\n- **Use Cases**: Client management, project tracking, team coordination, business development\n- **Acquisition Strategy**: Small business publications, entrepreneur meetups, partnership with business tools\n\n**Creative Professionals** (9% of target market):\nDesigners, writers, content creators, marketing professionals who manage multiple projects and clients.\n\n- **Age**: 25-40 years\n- **Income**: $45,000-$85,000\n- **Characteristics**: Visual learners, value aesthetics, project-based work patterns, collaborative\n- **Use Cases**: Portfolio management, client communication, project timelines, creative asset organization\n- **Acquisition Strategy**: Design conferences, creative platform partnerships, influencer collaborations".to_string(),
        node_type: "persona".to_string(),
        parent_id: Some(audience_id.clone()),
        metadata: json!({
            "persona_group": "secondary_markets",
            "market_share": 0.42,
            "segments": 3,
            "growth_potential": "high",
            "price_sensitivity": "medium_high"
        }),
    }));

    // Level 3: Digital Marketing - Comprehensive strategy
    let digital_id = Uuid::new_v4().to_string();
    nodes.push((3, SampleNodeData {
        title: "Digital Marketing Strategy".to_string(),
        content: "### Digital Marketing Strategy (70% of total budget: $105,000)\n\nComprehensive digital-first approach leveraging owned, earned, and paid media for maximum reach and engagement efficiency.\n\n**Content Marketing & SEO** ($35,000 - 33% of digital budget):\n\n*Strategy*: Establish thought leadership and organic discovery through high-value content addressing core productivity challenges.\n\n*Tactical Execution*:\n- **Blog Content**: 3 long-form articles weekly (2,000-4,000 words) covering productivity science, workflow optimization, and industry trends\n- **Resource Library**: Comprehensive guides, templates, and frameworks (15 major resources over 6 months)\n- **Video Content**: Weekly YouTube series \"Productivity Decoded\" featuring expert interviews and case studies\n- **Podcast Sponsorships**: Strategic sponsorships of 5 top productivity and business podcasts\n\n*Performance Targets*:\n- 500,000 monthly organic page views by month 6\n- 25,000 email subscribers with 35% monthly engagement\n- Top 3 rankings for 50 productivity-related keywords\n- 15,000 YouTube subscribers with 8% monthly growth\n\n**Paid Social & Display Advertising** ($45,000 - 43% of digital budget):\n\n*LinkedIn Advertising* ($28,000):\n- Sponsored content targeting decision makers at target companies\n- Lead generation campaigns with productivity assessment tools\n- Video ads showcasing product demonstrations\n- Retargeting campaigns for website visitors and content consumers\n\n*Google Ads* ($12,000):\n- Search campaigns for high-intent productivity software keywords\n- YouTube advertising on productivity and business channels\n- Display remarketing across Google network\n\n*Other Platforms* ($5,000):\n- Twitter promoted tweets for thought leadership content\n- Reddit community engagement and strategic promoted posts\n- Niche platform advertising (ProductHunt, Hacker News)\n\n**Email Marketing & Automation** ($15,000 - 14% of digital budget):\n\n*Lead Nurture Sequences*:\n- 12-email productivity masterclass for new subscribers\n- Segment-specific campaigns based on role and company size\n- Product education series for trial users\n\n*Behavioral Triggers*:\n- Abandoned trial recovery sequences\n- Engagement-based upgrade campaigns\n- Win-back campaigns for churned users\n\n**Marketing Technology Stack** ($10,000 - 10% of digital budget):\n- Marketing automation platform (HubSpot)\n- Social media management (Hootsuite)\n- Analytics and attribution (Google Analytics 4, Mixpanel)\n- A/B testing platform (Optimizely)\n- Customer feedback tools (Hotjar, Qualtrics)".to_string(),
        node_type: "strategy".to_string(),
        parent_id: Some(channels_id.clone()),
        metadata: json!({
            "budget_allocation": 105000,
            "budget_percentage": 0.7,
            "expected_leads": 5950,
            "channels": ["content", "paid_social", "email", "martech"],
            "primary_kpis": ["organic_traffic", "lead_generation", "email_growth", "social_engagement"]
        }),
    }));

    // Level 3: Traditional Marketing - Credibility building
    let traditional_id = Uuid::new_v4().to_string();
    nodes.push((3, SampleNodeData {
        title: "Traditional Marketing & Events".to_string(),
        content: "### Traditional Marketing & Events Strategy (30% of total budget: $45,000)\n\nStrategic traditional marketing focused on credibility building, relationship development, and high-value prospect engagement.\n\n**Industry Publications & Thought Leadership** ($18,000 - 40% of traditional budget):\n\n*Print & Digital Advertising*:\n- **Harvard Business Review**: Quarterly ads in productivity and management issues ($8,000)\n- **Fast Company**: Monthly digital sponsorships of productivity articles ($4,000)\n- **Inc. Magazine**: Sponsored content series on small business productivity ($3,000)\n- **TechCrunch**: Event guide advertisements during major conferences ($3,000)\n\n*Thought Leadership Content*:\n- Guest articles in major business publications (5-7 articles over 6 months)\n- Industry report collaboration with research firms\n- Executive interviews and podcast appearances\n- Speaking proposals for major conferences\n\n**Conference & Event Marketing** ($20,000 - 44% of traditional budget):\n\n*Major Conference Presence*:\n- **TechCrunch Disrupt** (October): Startup Alley booth + networking events ($8,000)\n- **SXSW Interactive** (March): Panel participation + sponsored meetup ($7,000)\n- **ProductCon** (September): Sponsorship + speaking opportunity ($5,000)\n\n*Industry Meetups & Local Events*:\n- Monthly productivity meetups in 5 major cities (SF, NYC, Boston, Austin, Seattle)\n- Co-working space partnerships for lunch-and-learn sessions\n- University guest lectures and career fair participation\n\n**Partnership & PR** ($7,000 - 16% of traditional budget):\n\n*Strategic Partnerships*:\n- Integration partnerships with complementary tools (Slack, Notion, Zapier)\n- Affiliate program development with productivity influencers\n- Co-marketing agreements with non-competing software vendors\n\n*Public Relations*:\n- PR agency retainer for product launch announcement\n- Press release distribution for major milestones\n- Media kit development and journalist relationship building\n- Crisis communication planning and reputation management\n\n**Performance Measurement**:\n- **Brand Awareness**: Quarterly surveys measuring aided/unaided recall\n- **Thought Leadership**: Media mentions, speaking invitations, social shares\n- **Lead Quality**: Conference leads typically have 2.3x higher lifetime value\n- **Sales Velocity**: Traditional marketing leads close 40% faster than digital-only".to_string(),
        node_type: "strategy".to_string(),
        parent_id: Some(channels_id.clone()),
        metadata: json!({
            "budget_allocation": 45000,
            "budget_percentage": 0.3,
            "expected_leads": 450,
            "lead_quality_multiplier": 2.3,
            "channels": ["publications", "events", "partnerships", "pr"]
        }),
    }));

    // Level 3: Q3 Budget Details
    nodes.push((3, SampleNodeData {
        title: "Q3 2025 Budget: Foundation Phase ($90,000)".to_string(),
        content: "### Q3 2025 Budget: Foundation Phase ($90,000)\n\n**Strategic Focus**: Market preparation, content creation, and beta user acquisition\n\n**Monthly Breakdown**:\n\n**July 2025 ($35,000)**:\n- Content creation and SEO foundation: $15,000\n- Market research completion: $8,000\n- Brand asset development: $7,000\n- Analytics setup and tool licensing: $5,000\n\n**August 2025 ($30,000)**:\n- Website development and optimization: $12,000\n- Initial content marketing campaigns: $10,000\n- Beta user recruitment: $5,000\n- Social media presence establishment: $3,000\n\n**September 2025 ($25,000)**:\n- Conference preparation and early events: $10,000\n- Paid advertising testing and optimization: $8,000\n- Case study development: $4,000\n- Email marketing automation setup: $3,000\n\n**Performance Expectations**:\n- 15,000 website visitors by end of Q3\n- 2,500 email subscribers with 40%+ engagement\n- 500 beta users providing feedback and testimonials\n- 50 pieces of content published across all channels\n- Foundation for 20 target keywords ranking in top 10".to_string(),
        node_type: "budget_detail".to_string(),
        parent_id: Some(budget_id.clone()),
        metadata: json!({
            "quarter": "Q3_2025",
            "amount": 90000,
            "phase": "foundation",
            "expected_visitors": 15000,
            "expected_subscribers": 2500,
            "expected_beta_users": 500
        }),
    }));

    // Level 3: Q4 Budget Details
    nodes.push((3, SampleNodeData {
        title: "Q4 2025 Budget: Launch Execution ($60,000)".to_string(),
        content: "### Q4 2025 Budget: Launch Execution ($60,000)\n\n**Strategic Focus**: Public launch, paid acquisition scaling, and momentum building\n\n**Monthly Breakdown**:\n\n**October 2025 ($25,000)**:\n- Launch event and PR campaign: $12,000\n- Paid advertising acceleration: $8,000\n- Influencer partnerships and sponsorships: $3,000\n- Launch week promotional activities: $2,000\n\n**November 2025 ($20,000)**:\n- Holiday season promotional campaigns: $10,000\n- Conference season participation: $6,000\n- Partnership marketing activation: $2,500\n- Performance optimization and testing: $1,500\n\n**December 2025 ($15,000)**:\n- Year-end campaigns and planning: $7,000\n- Customer success stories and case studies: $4,000\n- Market research for 2026 planning: $2,500\n- Team expansion and tool upgrades: $1,500\n\n**Launch Event Details** (October):\n- **Format**: Hybrid in-person/virtual product launch\n- **Venue**: San Francisco tech hub with livestream capability\n- **Audience**: 200 in-person attendees, 1,000+ virtual participants\n- **Agenda**: Product demos, customer panels, industry expert talks\n- **Follow-up**: 30-day trial offers, exclusive early-bird pricing\n\n**Performance Targets**:\n- 50,000 website visitors in October (launch month)\n- 1,000 trial signups within first week of launch\n- 10,000 total email subscribers by end of year\n- 25% conversion rate from trial to paid subscription".to_string(),
        node_type: "budget_detail".to_string(),
        parent_id: Some(budget_id.clone()),
        metadata: json!({
            "quarter": "Q4_2025",
            "amount": 60000,
            "phase": "launch",
            "launch_event_cost": 12000,
            "expected_trial_signups": 1000,
            "target_conversion_rate": 0.25
        }),
    }));

    // Level 3: Phase 1 Details
    nodes.push((3, SampleNodeData {
        title: "Phase 1: Market Research & Strategy (July-August 2025)".to_string(),
        content: "### Phase 1: Market Research & Strategy Finalization (July-August 2025)\n\n**Objective**: Complete market intelligence gathering and finalize go-to-market strategy based on comprehensive research findings.\n\n**Week-by-Week Breakdown**:\n\n**Weeks 1-2 (Early July)**:\n- Competitive landscape analysis: Deep dive into 15 direct and 25 indirect competitors\n- Pricing strategy analysis: Evaluation of freemium vs. paid models, price sensitivity testing\n- Feature gap analysis: Identification of unique value propositions and market whitespace\n- Initial customer interviews: 25 detailed interviews with target personas\n\n**Weeks 3-4 (Mid-July)**:\n- Survey deployment: Large-scale survey to 5,000 productivity software users\n- Focus group sessions: 6 sessions across different demographic segments\n- User journey mapping: Detailed analysis of current productivity workflows\n- Beta feature prioritization: Ranking features for MVP based on market feedback\n\n**Weeks 5-6 (Late July)**:\n- Data analysis and insight synthesis: Statistical analysis of all research data\n- Persona refinement: Final persona documents with detailed behavioral insights\n- Messaging framework development: Core value propositions and positioning statements\n- Competitive response planning: Anticipation of competitor reactions and counter-strategies\n\n**Weeks 7-8 (Early August)**:\n- Strategy validation: Testing messaging and positioning with focus groups\n- Go-to-market plan finalization: Channel strategy, timeline, and resource allocation\n- Team alignment workshops: Ensuring all stakeholders understand and commit to strategy\n- Phase 2 preparation: Detailed planning and resource allocation for content creation phase\n\n**Key Deliverables**:\n1. **Competitive Analysis Report**: 50-page comprehensive analysis with strategic recommendations\n2. **Customer Persona Guide**: Detailed profiles of 5 primary personas with behavioral insights\n3. **Messaging Framework**: Core positioning statements and value propositions for each segment\n4. **Market Sizing Model**: TAM/SAM/SOM analysis with growth projections\n5. **Pricing Strategy**: Recommended pricing tiers with supporting research\n6. **Go-to-Market Plan**: Detailed 6-month execution plan with metrics and milestones".to_string(),
        node_type: "phase_detail".to_string(),
        parent_id: Some(timeline_id.clone()),
        metadata: json!({
            "phase_number": 1,
            "start_date": "2025-07-01",
            "end_date": "2025-08-31",
            "duration_weeks": 8,
            "team_size": 6,
            "budget_allocation": 35000,
            "research_participants": 5025
        }),
    }));

    // Level 3: Phase 2 Details
    nodes.push((3, SampleNodeData {
        title: "Phase 2: Content Creation & Brand Building (September 2025)".to_string(),
        content: "### Phase 2: Content Creation & Brand Building (September 2025)\n\n**Objective**: Create comprehensive content library and establish brand presence across all marketing channels.\n\n**Content Creation Sprint**:\n\n**Week 1: Foundation Content**:\n- Website copy and landing pages: 15 pages of optimized content\n- Product positioning materials: Sales decks, one-pagers, FAQs\n- SEO content strategy: Keyword research and content calendar for 6 months\n- Brand voice and style guide: Comprehensive guidelines for all communications\n\n**Week 2: Educational Content**:\n- Blog article series: 12 in-depth articles on productivity science and best practices\n- Downloadable resources: 5 comprehensive guides, templates, and frameworks\n- Video content: 8 product demo videos and 4 thought leadership interviews\n- Webinar content: 3 educational webinars with registration landing pages\n\n**Week 3: Marketing Assets**:\n- Social media content: 100 posts across LinkedIn, Twitter, and YouTube\n- Email marketing templates: Welcome series, nurture campaigns, and promotional emails\n- Paid advertising creative: 25 ad variations for A/B testing across platforms\n- Case study development: 3 detailed customer success stories with metrics\n\n**Week 4: Launch Preparation**:\n- Press kit development: Media resources, executive bios, product fact sheets\n- Partnership materials: Co-marketing templates and collaboration frameworks\n- Sales enablement: Battle cards, objection handling guides, demo scripts\n- Customer onboarding: Tutorial content, help documentation, success frameworks\n\n**Quality Assurance Process**:\n- Content review by subject matter experts\n- Legal and compliance review for all claims and comparisons\n- Brand consistency check across all materials\n- Performance optimization for web content (speed, SEO, conversion)\n\n**Distribution Strategy**:\n- Content publication schedule across all owned channels\n- Guest posting outreach to 20 industry publications\n- Social media amplification with employee advocacy program\n- Email marketing automation setup with behavioral triggers\n\n**Success Metrics**:\n- 50+ pieces of content published across all formats\n- Website traffic increase of 300% from baseline\n- Email list growth to 2,000 subscribers with 35%+ engagement\n- Social media following growth of 150% across all platforms".to_string(),
        node_type: "phase_detail".to_string(),
        parent_id: Some(timeline_id.clone()),
        metadata: json!({
            "phase_number": 2,
            "start_date": "2025-09-01",
            "end_date": "2025-09-30",
            "content_pieces": 50,
            "team_size": 8,
            "expected_traffic_increase": 3.0,
            "target_subscribers": 2000
        }),
    }));

    nodes
}
