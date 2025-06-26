// Create comprehensive marketing sample data for RAG testing
use nodespace_core_logic::{ServiceContainer, CoreLogic};
use std::error::Error;
use rand::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Creating comprehensive marketing sample data for RAG testing...");
    println!("📊 Generating ~1000 entries with diverse content types and lengths\n");

    // Use the same database path as the Tauri app
    let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
    let model_path = std::path::PathBuf::from(
        "/Users/malibio/nodespace/nodespace-nlp-engine/models/gemma-3-1b-it-onnx/model.onnx",
    );

    let service_container = ServiceContainer::new_with_database_and_model_paths(database_path, model_path).await?;
    println!("✅ ServiceContainer initialized with Tauri database path");

    let mut rng = thread_rng();
    let mut total_entries = 0;

    // Define date range for realistic historical data
    let dates = generate_marketing_dates();
    
    for (date_idx, date_str) in dates.iter().enumerate() {
        println!("\n📅 Creating entries for {} ({}/{})", date_str, date_idx + 1, dates.len());
        
        // Vary the number of entries per day (2-8 entries)
        let entries_today = rng.gen_range(2..=8);
        
        for entry_idx in 0..entries_today {
            let content = generate_marketing_content(&mut rng, date_str, entry_idx);
            
            match service_container.create_text_node(&content, date_str).await {
                Ok(node_id) => {
                    total_entries += 1;
                    let preview = content.lines().next().unwrap_or("").chars().take(60).collect::<String>();
                    println!("  ✅ Created entry {}: {} (ID: {})", 
                             total_entries, 
                             if preview.len() < content.len() { format!("{}...", preview) } else { preview },
                             node_id);
                }
                Err(e) => {
                    println!("  ❌ Failed to create entry: {}", e);
                }
            }
        }
        
        // Progress update every 10 days
        if (date_idx + 1) % 10 == 0 {
            println!("📈 Progress: {} entries created across {} days", total_entries, date_idx + 1);
        }
    }

    println!("\n🎉 Marketing sample data creation completed!");
    println!("✅ Created {} total entries across {} days", total_entries, dates.len());
    println!("✅ Content types included:");
    println!("   • Campaign strategies and performance analysis");
    println!("   • Meeting notes and action items");
    println!("   • Market research and competitor analysis");
    println!("   • Creative briefs and brand guidelines");
    println!("   • Customer insights and personas");
    println!("   • Budget planning and ROI analysis");
    println!("   • Social media strategies and content plans");
    println!("   • Email marketing campaigns and A/B tests");
    println!("   • Product launch plans and go-to-market strategies");
    println!("   • Partnership opportunities and vendor evaluations");
    println!("\n💡 This data is now ready for RAG testing with:");
    println!("   • Semantic search across marketing topics");
    println!("   • Context retrieval for campaign-related queries");
    println!("   • Historical analysis of marketing activities");
    println!("   • Cross-campaign insights and patterns");

    Ok(())
}

fn generate_marketing_dates() -> Vec<String> {
    // Generate realistic date range (last 6 months of marketing activities)
    let mut dates = Vec::new();
    
    // Start from 6 months ago and go to today
    for month in 1..=6 {
        for day in 1..=28 { // Use 28 to avoid month-end issues
            // Skip some days to make it more realistic (weekends, holidays)
            if day % 7 == 0 || day % 7 == 6 { continue; } // Skip weekends
            if day == 25 || day == 26 { continue; } // Holiday periods
            
            let date_str = format!("2025-{:02}-{:02}", 
                                 (7 - month), // Count backwards from June
                                 day);
            dates.push(date_str);
        }
    }
    
    // Add some recent dates including today
    dates.extend(vec![
        "2025-06-23".to_string(),
        "2025-06-24".to_string(),
        "2025-06-25".to_string(),
    ]);
    
    dates
}

fn generate_marketing_content(rng: &mut ThreadRng, date: &str, entry_idx: usize) -> String {
    let content_types = [
        generate_campaign_strategy,
        generate_meeting_notes,
        generate_market_research,
        generate_creative_brief,
        generate_customer_insights,
        generate_performance_analysis,
        generate_social_media_plan,
        generate_email_campaign,
        generate_budget_analysis,
        generate_competitor_analysis,
        generate_product_launch_plan,
        generate_partnership_notes,
        generate_brand_guidelines,
        generate_content_calendar,
        generate_roi_report,
        generate_customer_feedback,
        generate_ab_test_results,
        generate_influencer_campaign,
        generate_seo_strategy,
        generate_event_planning,
    ];
    
    let content_fn = content_types.choose(rng).unwrap();
    content_fn(rng, date, entry_idx)
}

fn generate_campaign_strategy(_rng: &mut ThreadRng, date: &str, _entry_idx: usize) -> String {
    let campaigns = [
        "# Q3 Digital Marketing Campaign Strategy\n\n## Campaign Overview\nOur Q3 digital marketing campaign focuses on **customer retention** and **new market expansion**. The strategy leverages multi-channel approach combining social media, email marketing, and content marketing.\n\n## Key Objectives\n- Increase customer retention by 25%\n- Expand into new geographic markets\n- Boost brand awareness among millennials\n\n## Target Audience\n**Primary**: Existing customers aged 25-40\n**Secondary**: New prospects in urban areas\n\n## Channel Strategy\n1. **Social Media**: Instagram and LinkedIn focus\n2. **Email**: Personalized drip campaigns\n3. **Content**: Educational blog series\n\n## Budget Allocation\n- Social Media: 40% ($120k)\n- Email Marketing: 25% ($75k)\n- Content Creation: 35% ($105k)\n\n## Success Metrics\n- CTR improvement: Target 3.5%\n- Conversion rate: Target 8%\n- Customer lifetime value increase: 15%",

        "# Summer Product Launch Campaign\n\n## Executive Summary\nLaunching our new sustainable product line with an integrated marketing campaign emphasizing **environmental responsibility** and **premium quality**.\n\n## Market Positioning\nPositioning as the **premium eco-friendly alternative** in the market. Key messaging focuses on sustainability without compromising quality.\n\n## Launch Timeline\n- **Pre-launch** (4 weeks): Teaser content, influencer partnerships\n- **Launch week**: Press releases, social media blitz, email announcements\n- **Post-launch** (8 weeks): User-generated content, testimonials, optimization\n\n## Creative Direction\n- **Visual theme**: Clean, natural, premium\n- **Color palette**: Earth tones with premium accents\n- **Photography style**: Natural lighting, authentic moments\n\n## Influencer Strategy\nPartnering with **micro-influencers** (10k-100k followers) in:\n- Sustainability niche\n- Lifestyle and wellness\n- Premium product categories\n\n## Performance Tracking\nDaily monitoring of:\n- Social engagement rates\n- Website traffic and conversions\n- Email open and click rates\n- Influencer content performance",

        "# Brand Awareness Campaign Analysis\n\n## Campaign Performance\nOur brand awareness campaign exceeded expectations with **142% of target reach** achieved.\n\n## Key Metrics\n- **Reach**: 2.8M (target: 2M)\n- **Impressions**: 12.4M\n- **Engagement rate**: 4.2% (industry avg: 2.8%)\n- **Brand recall**: 34% improvement\n\n## Top Performing Content\n1. **Video series**: \"Behind the Brand\" - 580k views\n2. **Infographic**: Market comparison - 320k shares\n3. **User testimonials**: Average 4.8% engagement\n\n## Audience Insights\n- **Primary demographic**: Women 28-45, urban\n- **Peak engagement**: Tuesday-Thursday, 7-9 PM\n- **Preferred content**: Video content outperformed static by 340%\n\n## Recommendations\n- Increase video content production by 50%\n- Focus on Tuesday/Wednesday posting\n- Expand user-generated content initiatives\n- Test live streaming for product demos",
    ];
    
    campaigns.choose(_rng).unwrap().replace("date", date)
}

fn generate_meeting_notes(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    let meetings = [
        "## Weekly Marketing Team Meeting\n\n**Date**: Today's marketing sync\n**Attendees**: Sarah (CMO), Mike (Digital), Lisa (Content), Tom (Analytics)\n\n### Key Discussions\n\n#### Campaign Performance Review\n- Q2 email campaigns showing 23% improvement in open rates\n- Social media engagement up 34% month-over-month\n- Attribution model needs refinement for cross-channel tracking\n\n#### Upcoming Product Launch\n- Creative assets 80% complete\n- Influencer partnerships finalized (12 confirmed)\n- PR timeline adjusted due to industry event conflict\n\n#### Budget Allocation Q3\n- Reallocating $45k from print to digital channels\n- Increased investment in video content creation\n- Performance marketing budget increased by 30%\n\n### Action Items\n- [ ] Mike: Finalize Facebook ad creative by Friday\n- [ ] Lisa: Complete blog content calendar for Q3\n- [ ] Tom: Set up attribution tracking for new campaign\n- [ ] Sarah: Approve influencer contracts by Wednesday\n\n### Next Meeting\n**Focus**: Campaign launch readiness review\n**Date**: Next week same time",

        "## Creative Review Session\n\n**Campaign**: Summer Collection Launch\n**Attendees**: Creative team + Brand manager\n\n### Creative Assets Reviewed\n\n#### Video Content\n- **Hero video**: Approved with minor audio adjustments\n- **Social cutdowns**: 3 versions approved, 2 need revision\n- **Product demos**: Excellent, ready for production\n\n#### Static Assets\n- **Print ads**: Strong visual impact, copy needs strengthening\n- **Social graphics**: Approved for Instagram, Facebook versions pending\n- **Email headers**: Design approved, need mobile optimization\n\n#### Copy Review\n- **Tagline**: \"Summer Redefined\" - testing with focus groups\n- **Product descriptions**: Approved for all channels\n- **Email copy**: CTAs need optimization\n\n### Brand Compliance\n- All assets align with brand guidelines\n- Color usage consistent across channels\n- Typography follows brand standards\n\n### Feedback Summary\n- Increase emotional appeal in static ads\n- Ensure accessibility compliance for all digital assets\n- Test alternative CTAs for email campaigns\n\n### Next Steps\n- Implement feedback by Thursday\n- Final review session Friday morning\n- Asset delivery to media team by EOW",

        "## Customer Persona Workshop Results\n\n**Facilitator**: Research team\n**Participants**: Marketing, Sales, Customer Success\n\n### Primary Persona: \"Tech-Savvy Sarah\"\n\n#### Demographics\n- **Age**: 28-35\n- **Income**: $75k-$120k\n- **Location**: Urban areas, major cities\n- **Education**: College graduate, often advanced degrees\n\n#### Behavioral Patterns\n- Researches extensively before purchasing\n- Values sustainability and ethical brands\n- Active on Instagram and LinkedIn\n- Shops online but appreciates in-store experiences\n\n#### Pain Points\n- Limited time for extensive product research\n- Overwhelmed by too many options\n- Skeptical of marketing claims\n- Values authentic brand communication\n\n#### Preferred Channels\n- **Discovery**: Social media, word-of-mouth\n- **Research**: Brand website, reviews, comparison sites\n- **Purchase**: Online with easy return policy\n- **Support**: Chat, email, comprehensive FAQs\n\n### Secondary Persona: \"Busy Executive Ben\"\n\n#### Key Characteristics\n- Time-constrained decision maker\n- Values efficiency and quality\n- Delegates research to team members\n- Focused on ROI and business impact\n\n### Marketing Implications\n- Create concise, value-focused content\n- Emphasize social proof and testimonials\n- Develop mobile-optimized experiences\n- Invest in authentic brand storytelling",
    ];
    
    meetings.choose(_rng).unwrap().to_string()
}

fn generate_market_research(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    let research = [
        "# Market Research: Sustainable Products Trend Analysis\n\n## Executive Summary\nSustainable products market growing at **23% CAGR** with significant opportunities in our target demographic.\n\n## Market Size & Growth\n- **Current market size**: $2.8B in our category\n- **Projected 2025 size**: $4.2B\n- **Our addressable market**: $340M\n\n## Consumer Behavior Shifts\n\n### Key Findings\n1. **67%** of consumers willing to pay premium for sustainable products\n2. **84%** research sustainability claims before purchasing\n3. **Millennials** drive 73% of sustainable product purchases\n\n### Purchase Drivers\n- Environmental impact (89%)\n- Product quality (78%)\n- Brand transparency (65%)\n- Social responsibility (58%)\n\n## Competitive Landscape\n\n### Market Leaders\n1. **EcoLeader Corp**: 28% market share, strong brand recognition\n2. **GreenChoice**: 15% share, innovative packaging\n3. **SustainableSolutions**: 12% share, B2B focus\n\n### Competitive Gaps\n- Premium positioning with mass market accessibility\n- Technology integration with sustainability\n- Personalized sustainable solutions\n\n## Opportunity Assessment\n\n### High-Potential Segments\n- **Urban millennials**: $180M opportunity\n- **Eco-conscious families**: $95M opportunity\n- **Corporate buyers**: $65M opportunity\n\n### Barriers to Entry\n- Certification requirements\n- Supply chain complexity\n- Consumer education needs\n\n## Strategic Recommendations\n1. Focus on urban millennial segment initially\n2. Develop certification and transparency strategy\n3. Create educational content marketing approach\n4. Build partnerships with sustainability influencers",

        "# Competitor Analysis: Digital Marketing Strategies\n\n## Analysis Overview\nComprehensive review of top 5 competitors' digital marketing approaches across all channels.\n\n## Competitor Profiles\n\n### Competitor A: MarketLeader Inc.\n\n#### Digital Presence\n- **Website traffic**: 2.3M monthly visitors\n- **Social following**: 485k Instagram, 120k LinkedIn\n- **Email list**: Estimated 180k subscribers\n\n#### Content Strategy\n- **Blog**: 3-4 posts/week, high-quality educational content\n- **Video**: Weekly product demos, customer stories\n- **Social**: Daily posts, strong visual branding\n\n#### Advertising Approach\n- Heavy investment in Google Ads ($150k/month estimated)\n- Facebook/Instagram ads focus on video content\n- Retargeting campaigns with personalized messaging\n\n#### Strengths\n- Consistent brand voice across channels\n- High-quality video production\n- Strong customer testimonial integration\n\n#### Weaknesses\n- Limited influencer partnerships\n- Slow adaptation to new social platforms\n- Generic email marketing approach\n\n### Competitor B: InnovativeBrand Co.\n\n#### Unique Positioning\n- First-mover advantage in sustainability messaging\n- Strong B2B and B2C presence\n- Technology integration in marketing\n\n#### Digital Innovation\n- AR try-before-buy features\n- AI-powered product recommendations\n- Interactive content experiences\n\n## Market Opportunities\n\n### Underserved Areas\n1. **Micro-influencer partnerships**: Most competitors focus on macro-influencers\n2. **Community building**: Limited brand community initiatives\n3. **Personalization**: Basic segmentation strategies\n4. **Mobile experience**: Optimization gaps across competitors\n\n### Content Gaps\n- Behind-the-scenes brand storytelling\n- User-generated content campaigns\n- Educational series about industry trends\n- Interactive tools and calculators\n\n## Strategic Recommendations\n\n### Immediate Opportunities (0-3 months)\n- Launch micro-influencer partnership program\n- Develop mobile-first content strategy\n- Create brand community platform\n\n### Medium-term Initiatives (3-6 months)\n- Implement AI-driven personalization\n- Develop interactive content experiences\n- Launch educational content series\n\n### Long-term Strategy (6+ months)\n- Build proprietary marketing technology\n- Establish thought leadership position\n- Create industry partnership ecosystem",

        "Consumer behavior study reveals **64%** prefer brands that take social stands on issues they care about. **Authenticity** is the top factor influencing brand trust among our target demographic. Mobile shopping continues to grow with **78%** of our audience using mobile devices for product research.",
    ];
    
    research.choose(_rng).unwrap().to_string()
}

fn generate_creative_brief(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    let briefs = [
        "# Creative Brief: Q3 Brand Campaign\n\n## Campaign Overview\n**Campaign Name**: \"Authentically You\"\n**Duration**: 12 weeks\n**Budget**: $500k\n\n## Objective\nIncrease brand awareness among millennials while positioning our brand as authentic and relatable.\n\n## Target Audience\n**Primary**: Urban millennials, 25-35, household income $50k+\n**Secondary**: Gen Z early adopters, 18-25, socially conscious\n\n## Key Message\n\"Be authentically you with products that understand your real life.\"\n\n## Tone & Voice\n- **Authentic**: Real people, real stories\n- **Empowering**: Confidence-building messaging\n- **Inclusive**: Diverse representation\n- **Conversational**: Approachable, not corporate\n\n## Creative Direction\n\n### Visual Style\n- **Photography**: Natural lighting, candid moments\n- **Color palette**: Warm earth tones with bright accents\n- **Typography**: Modern, clean, readable fonts\n- **Layout**: Clean, breathing room, mobile-first\n\n### Content Themes\n1. **Real Life Moments**: Everyday situations\n2. **Personal Growth**: Self-improvement journey\n3. **Community**: Connection and belonging\n4. **Sustainability**: Environmental consciousness\n\n## Deliverables\n\n### Video Content\n- 1x Hero video (60 seconds)\n- 6x Social cutdowns (15-30 seconds)\n- 12x Product demonstration videos\n\n### Static Assets\n- 20x Social media graphics\n- 8x Email header designs\n- 4x Print advertisements\n- 1x Website banner suite\n\n### Copy Requirements\n- Campaign tagline and variations\n- Product descriptions (50 products)\n- Email campaign copy (8 emails)\n- Social media captions (60+ posts)\n\n## Success Metrics\n- Brand awareness lift: +25%\n- Social engagement rate: >4%\n- Website traffic increase: +35%\n- Conversion rate improvement: +15%\n\n## Timeline\n- **Week 1-2**: Concept development\n- **Week 3-4**: Asset creation\n- **Week 5**: Review and revisions\n- **Week 6**: Final approval and delivery",

        "# Email Campaign Creative Brief\n\n## Campaign: Welcome Series Redesign\n\n### Objective\nCreate a 7-email welcome series that educates new subscribers about our brand values and product benefits while driving first purchase.\n\n### Target Audience\nNew email subscribers who haven't made a purchase yet.\n\n### Email Sequence\n\n#### Email 1: Welcome & Brand Story\n- **Send**: Immediately after signup\n- **Subject**: \"Welcome to the [Brand] family!\"\n- **Content**: Brand story, values, what to expect\n- **CTA**: Explore our story\n\n#### Email 2: Product Education\n- **Send**: 2 days after signup\n- **Subject**: \"Here's what makes us different\"\n- **Content**: Key product benefits, differentiators\n- **CTA**: Shop bestsellers\n\n#### Email 3: Customer Stories\n- **Send**: 5 days after signup\n- **Subject**: \"See how [customers] use our products\"\n- **Content**: Customer testimonials, user photos\n- **CTA**: Read more stories\n\n#### Email 4: Exclusive Offer\n- **Send**: 7 days after signup\n- **Subject**: \"Your exclusive welcome gift inside\"\n- **Content**: First-time buyer discount\n- **CTA**: Shop now with discount\n\n### Design Requirements\n- Mobile-responsive templates\n- Consistent with brand guidelines\n- Clear hierarchy and scannable content\n- High-quality product imagery\n- Prominent, clear CTAs\n\n### Success Metrics\n- Open rate target: >35%\n- Click rate target: >8%\n- Conversion rate target: >12%\n- Welcome series completion: >60%",

        "## Social Media Campaign Brief\n\n**Campaign**: #MyBrandStory User-Generated Content\n\n### Concept\nEncourage customers to share their authentic experiences with our products using branded hashtag.\n\n### Execution\n- Launch with influencer partnerships\n- Feature customer stories on our channels\n- Create branded hashtag #MyBrandStory\n- Offer prizes for best submissions\n\n### Content Guidelines\n- Authentic, unpolished moments preferred\n- Include product in natural setting\n- Share personal story or transformation\n- Use branded hashtag and tag our account\n\n### Amplification Strategy\n- Repost best submissions to our channels\n- Create highlight reel videos\n- Feature stories in email newsletters\n- Use content for future advertising\n\n### Prize Structure\n- Weekly winner: $100 product credit\n- Monthly grand prize: $500 + feature in campaign\n- All participants: 15% discount on next purchase",
    ];
    
    briefs.choose(_rng).unwrap().to_string()
}

fn generate_customer_insights(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    let insights = [
        "# Customer Journey Analysis: Key Insights\n\n## Journey Mapping Results\nAnalyzed 2,847 customer journeys over 6 months to identify optimization opportunities.\n\n## Discovery Phase Insights\n\n### Primary Discovery Channels\n1. **Social Media** (34%): Instagram leads, followed by TikTok\n2. **Word of Mouth** (28%): Strong recommendation rate\n3. **Search** (23%): Branded and category searches\n4. **Email Marketing** (15%): Existing customer referrals\n\n### Content Preferences by Channel\n- **Instagram**: Video content performs 3x better than static\n- **TikTok**: Authentic, behind-the-scenes content\n- **Search**: Educational, how-to content\n- **Email**: Product comparisons and reviews\n\n## Consideration Phase\n\n### Research Behavior\n- **Average research time**: 8.5 days\n- **Touchpoints visited**: 6.2 average\n- **Review importance**: 89% read reviews before purchase\n- **Comparison shopping**: 76% compare with 2+ competitors\n\n### Key Decision Factors\n1. Product quality and durability (92%)\n2. Price and value perception (87%)\n3. Brand reputation and reviews (81%)\n4. Sustainability credentials (67%)\n5. Return policy and guarantees (58%)\n\n## Purchase Conversion\n\n### Conversion Barriers\n- **Shipping costs** (42% of abandonments)\n- **Long checkout process** (31%)\n- **Security concerns** (18%)\n- **Product availability** (9%)\n\n### Conversion Accelerators\n- Free shipping offers (+23% conversion)\n- Customer reviews on product pages (+18%)\n- Live chat availability (+15%)\n- Limited-time offers (+12%)\n\n## Post-Purchase Experience\n\n### Satisfaction Drivers\n1. **Product meets expectations** (94% satisfaction)\n2. **Fast, reliable shipping** (89%)\n3. **Easy returns process** (76%)\n4. **Responsive customer service** (71%)\n\n### Loyalty Indicators\n- **Repeat purchase rate**: 34% within 6 months\n- **Referral rate**: 23% recommend to friends\n- **Review participation**: 18% leave reviews\n- **Social sharing**: 12% share purchase on social\n\n## Actionable Recommendations\n\n### Immediate Improvements\n1. Implement free shipping threshold strategy\n2. Streamline checkout to 3 steps maximum\n3. Add customer reviews to all product pages\n4. Create post-purchase email sequence\n\n### Medium-term Initiatives\n1. Develop loyalty program for repeat customers\n2. Create referral incentive program\n3. Implement live chat during business hours\n4. Build customer community platform\n\n### Long-term Strategy\n1. Personalized product recommendations\n2. Predictive analytics for customer lifetime value\n3. Omnichannel experience optimization\n4. AI-powered customer service integration",

        "Customer feedback analysis shows **92%** satisfaction with product quality but **concerns about packaging waste**. Top request: more sustainable packaging options. **73%** would pay extra for eco-friendly packaging. Opportunity to differentiate through environmental initiatives.",

        "# Voice of Customer: Product Feedback Summary\n\n## Feedback Collection Overview\n- **Survey responses**: 1,247\n- **Review analysis**: 3,891 reviews\n- **Social media mentions**: 2,156\n- **Customer service logs**: 892 interactions\n\n## Product Satisfaction Scores\n\n### Overall Satisfaction: 4.3/5\n- **Product A**: 4.6/5 (top performer)\n- **Product B**: 4.2/5 (solid performance)\n- **Product C**: 3.9/5 (needs improvement)\n\n## Common Praise Points\n1. **Quality**: \"Exceeds expectations\", \"Built to last\"\n2. **Design**: \"Beautiful\", \"Perfect for my space\"\n3. **Functionality**: \"Works exactly as described\"\n4. **Value**: \"Worth the investment\"\n\n## Areas for Improvement\n1. **Packaging**: 23% mention excessive packaging\n2. **Instructions**: 18% find setup instructions unclear\n3. **Size options**: 15% want more size variations\n4. **Color choices**: 12% want additional colors\n\n## Customer Suggestions\n- Refillable/recyclable packaging options\n- Video setup tutorials\n- Customization options\n- Subscription delivery model\n- Mobile app for product management\n\n## Action Items\n- Work with suppliers on sustainable packaging\n- Create video tutorial series\n- Research customization feasibility\n- Survey interest in subscription model",
    ];
    
    insights.choose(_rng).unwrap().to_string()
}

fn generate_performance_analysis(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    let analyses = [
        "# Monthly Performance Report: Digital Marketing\n\n## Executive Summary\nStrong performance across all digital channels with **23% increase** in qualified leads and **18% improvement** in conversion rates.\n\n## Channel Performance\n\n### Paid Search\n- **Spend**: $45,200 (within budget)\n- **Impressions**: 1.2M (+15% MoM)\n- **Clicks**: 24,500 (+22% MoM)\n- **CTR**: 2.04% (+0.3% MoM)\n- **CPC**: $1.85 (-$0.12 MoM)\n- **Conversions**: 892 (+28% MoM)\n- **Cost per conversion**: $50.67 (-15% MoM)\n\n### Social Media\n- **Organic reach**: 285k (+12% MoM)\n- **Engagement rate**: 4.2% (+0.8% MoM)\n- **Follower growth**: +2,150 (+34% MoM)\n- **Social conversions**: 156 (+45% MoM)\n- **Top performing post**: Product demo video (15k engagements)\n\n### Email Marketing\n- **Campaigns sent**: 12\n- **Total sends**: 156k\n- **Open rate**: 28.5% (+2.1% MoM)\n- **Click rate**: 6.8% (+1.2% MoM)\n- **Conversion rate**: 12.3% (+3.1% MoM)\n- **Revenue**: $89,400 (+31% MoM)\n\n### Content Marketing\n- **Blog traffic**: 45k visitors (+18% MoM)\n- **Time on page**: 3:24 (+0:45 MoM)\n- **Content downloads**: 1,247 (+52% MoM)\n- **Lead generation**: 234 (+67% MoM)\n\n## Top Performing Assets\n\n### Paid Search Ads\n1. \"Summer Collection Launch\" - 3.2% CTR\n2. \"Free Shipping Offer\" - 2.8% CTR\n3. \"Customer Testimonials\" - 2.6% CTR\n\n### Social Media Content\n1. Behind-the-scenes video: 15k engagements\n2. Customer spotlight post: 12k engagements\n3. Product comparison infographic: 9k engagements\n\n### Email Campaigns\n1. Welcome series finale: 34% open rate\n2. Abandoned cart recovery: 45% click rate\n3. Product recommendation: 18% conversion rate\n\n## Key Insights\n\n### Audience Behavior\n- **Mobile traffic**: 67% of all traffic (up from 61%)\n- **Peak engagement**: Tuesday-Thursday, 7-9 PM\n- **Preferred content**: Video content outperforming static by 240%\n- **Purchase journey**: Average 5.2 touchpoints before conversion\n\n### Performance Drivers\n- Video content integration increased engagement by 45%\n- Personalized email campaigns improved conversions by 31%\n- Mobile optimization reduced bounce rate by 22%\n- Customer testimonials improved ad performance by 28%\n\n## Recommendations\n\n### Immediate Actions\n1. Increase video content production budget by 40%\n2. Implement advanced email personalization\n3. Optimize remaining non-mobile pages\n4. Expand customer testimonial collection\n\n### Strategic Initiatives\n1. Develop omnichannel attribution model\n2. Create customer journey automation\n3. Implement predictive analytics\n4. Build brand community platform",

        "Q2 campaign performance exceeded targets by **34%**. Email marketing drove highest ROI at **$4.20** per dollar spent. Social media engagement up **67%** with video content leading performance. Mobile traffic now represents **72%** of all sessions.",

        "# A/B Test Results: Email Subject Lines\n\n## Test Overview\n- **Test duration**: 14 days\n- **Sample size**: 20,000 subscribers\n- **Confidence level**: 95%\n- **Statistical significance**: Achieved\n\n## Variants Tested\n\n### Version A (Control)\n**Subject**: \"New arrivals are here!\"\n- **Open rate**: 18.2%\n- **Click rate**: 3.4%\n- **Conversion rate**: 8.1%\n\n### Version B\n**Subject**: \"Sarah, your favorites are back in stock\"\n- **Open rate**: 24.7% (+35.7%)\n- **Click rate**: 4.9% (+44.1%)\n- **Conversion rate**: 11.2% (+38.3%)\n\n### Version C\n**Subject**: \"Only 24 hours left - don't miss out\"\n- **Open rate**: 21.3% (+17.0%)\n- **Click rate**: 4.1% (+20.6%)\n- **Conversion rate**: 9.8% (+21.0%)\n\n## Winner: Version B\nPersonalized subject line with product availability messaging significantly outperformed both control and urgency-based messaging.\n\n## Key Learnings\n1. **Personalization** drives highest engagement\n2. **Product availability** messaging resonates with audience\n3. **Urgency tactics** work but less effective than personalization\n4. **Name inclusion** in subject line increases opens by 35%+\n\n## Implementation\n- Roll out personalized subject lines to all email campaigns\n- Develop dynamic product availability messaging\n- Test additional personalization elements (location, purchase history)\n- Create subject line personalization guidelines for team",
    ];
    
    analyses.choose(_rng).unwrap().to_string()
}

fn generate_social_media_plan(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    let plans = [
        "# Social Media Content Calendar: Next 30 Days\n\n## Content Themes\n\n### Week 1: Behind the Scenes\n- **Monday**: Team spotlight featuring design process\n- **Wednesday**: Manufacturing facility tour video\n- **Friday**: Founder's story and company values\n\n### Week 2: Customer Stories\n- **Monday**: Customer transformation story\n- **Wednesday**: User-generated content compilation\n- **Friday**: Customer Q&A live session\n\n### Week 3: Product Education\n- **Monday**: How-to tutorial series launch\n- **Wednesday**: Product comparison infographic\n- **Friday**: Expert tips and tricks video\n\n### Week 4: Community Engagement\n- **Monday**: Poll: \"What's your biggest challenge?\"\n- **Wednesday**: Community challenge launch\n- **Friday**: Week in review and community highlights\n\n## Content Distribution\n\n### Instagram Strategy\n- **Feed posts**: 5 per week (M/W/F + 2 supplementary)\n- **Stories**: Daily updates, behind-the-scenes\n- **Reels**: 3 per week focusing on trending audio\n- **IGTV**: Weekly long-form educational content\n\n### LinkedIn Approach\n- **Company updates**: 3 per week\n- **Thought leadership**: Industry insights and trends\n- **Employee advocacy**: Team member spotlights\n- **Engagement**: Active commenting on industry posts\n\n### TikTok Content\n- **Trending challenges**: Adapt 2-3 relevant trends weekly\n- **Educational content**: Quick tips and tutorials\n- **Behind-the-scenes**: Authentic, unpolished moments\n- **User-generated**: Feature customer content\n\n## Engagement Strategy\n\n### Response Guidelines\n- Respond to comments within 2 hours during business hours\n- Engage with user-generated content within 24 hours\n- Share relevant industry content 2-3 times per week\n- Participate in relevant hashtag conversations\n\n### Community Building\n- Feature customer stories weekly\n- Create branded hashtag campaigns monthly\n- Host live Q&A sessions bi-weekly\n- Collaborate with micro-influencers monthly\n\n## Performance Targets\n\n### Growth Metrics\n- **Instagram followers**: +500 monthly\n- **Engagement rate**: Maintain >4%\n- **Reach**: Increase by 25% monthly\n- **Share rate**: Improve by 15% monthly\n\n### Content Performance\n- **Video content**: 60% of total posts\n- **User-generated content**: 20% of posts\n- **Educational content**: 30% of posts\n- **Behind-the-scenes**: 15% of posts\n\n## Hashtag Strategy\n\n### Branded Hashtags\n- #MyBrandStory (campaign-specific)\n- #BrandName (always include)\n- #BrandCommunity (community content)\n\n### Industry Hashtags\n- Mix of high-volume and niche hashtags\n- Research trending industry tags weekly\n- Use 15-20 hashtags per Instagram post\n- Rotate hashtag sets to avoid shadowbanning",

        "Launching **#SustainableSummer** campaign across all social platforms. Focus on user-generated content featuring products in outdoor settings. Partner with 15 eco-influencers for authentic content creation. Expected reach: **2.5M** impressions over 4 weeks.",

        "## Social Media Crisis Communication Plan\n\n### Escalation Levels\n\n#### Level 1: Minor Issues\n- Individual customer complaints\n- Product questions or concerns\n- General negative feedback\n\n**Response**: Customer service team handles within 2 hours\n\n#### Level 2: Moderate Issues\n- Multiple complaints about same issue\n- Viral negative content (>1k engagements)\n- Influencer criticism\n\n**Response**: Marketing manager involved, response within 1 hour\n\n#### Level 3: Major Crisis\n- Product safety concerns\n- Widespread negative coverage\n- Legal or regulatory issues\n\n**Response**: Executive team involved, immediate response\n\n### Response Framework\n\n#### Acknowledge\n- Respond quickly to show we're listening\n- Express concern for customer experience\n- Avoid defensive language\n\n#### Assess\n- Determine facts before full response\n- Consult with relevant teams (legal, product, etc.)\n- Identify root cause and solutions\n\n#### Act\n- Provide transparent update\n- Outline steps being taken\n- Offer appropriate remediation\n\n### Pre-approved Responses\n\n#### Product Quality Concerns\n\"We take product quality seriously and are investigating this issue. Please DM us your order details so we can make this right.\"\n\n#### Shipping Delays\n\"We apologize for the shipping delay. We're working with our logistics partner to resolve this and will provide an update within 24 hours.\"\n\n#### General Negative Feedback\n\"Thank you for your feedback. We're always looking to improve and would love to learn more about your experience. Please reach out to us directly.\"",
    ];
    
    plans.choose(_rng).unwrap().to_string()
}

fn generate_email_campaign(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    let campaigns = [
        "# Email Campaign Strategy: Summer Sale\n\n## Campaign Overview\n**Campaign**: Summer Sale 2025\n**Duration**: 2 weeks\n**Target audience**: All subscribers + lookalike segments\n**Goal**: $250k revenue, 15% email revenue increase\n\n## Email Sequence\n\n### Email 1: Sale Announcement\n**Send date**: Campaign launch day\n**Subject line**: \"Summer Sale is here - 30% off everything!\"\n**Preview text**: \"Limited time only - shop your favorites now\"\n\n**Content blocks**:\n- Hero banner with sale messaging\n- Featured product grid (8 products)\n- Social proof (customer reviews)\n- Clear CTA buttons throughout\n- Urgency messaging (limited time)\n\n### Email 2: Category Focus\n**Send date**: Day 3\n**Subject line**: \"Last chance for summer essentials\"\n**Audience**: Non-openers from Email 1\n\n**Content strategy**:\n- Focus on summer-specific products\n- Lifestyle imagery showing products in use\n- Bundle offers and recommendations\n- Customer styling tips\n\n### Email 3: Cart Abandonment\n**Trigger**: 24 hours after abandonment\n**Subject line**: \"Your cart is waiting (plus 5% extra off)\"\n\n**Personalization**:\n- Show exact products in cart\n- Related product recommendations\n- Additional incentive (5% extra discount)\n- Easy one-click checkout\n\n### Email 4: Final Hours\n**Send date**: Last day of sale\n**Subject line**: \"⏰ 6 hours left - Summer Sale ends tonight\"\n\n**Urgency tactics**:\n- Countdown timer\n- Limited inventory messaging\n- Best-selling products\n- Social proof and reviews\n\n## Design Requirements\n\n### Visual Elements\n- Summer color palette (bright, vibrant)\n- High-quality product photography\n- Mobile-responsive design\n- Clear hierarchy and scannable layout\n\n### Technical Specifications\n- Maximum width: 600px\n- Alt text for all images\n- Web-safe fonts with fallbacks\n- Dark mode optimization\n\n## Segmentation Strategy\n\n### VIP Customers\n- Early access (24 hours before general announcement)\n- Exclusive additional discount (35% vs 30%)\n- Personal shopping assistance offer\n\n### Recent Buyers\n- Focus on complementary products\n- Cross-sell and upsell opportunities\n- Loyalty program benefits\n\n### Engaged Non-Buyers\n- Free shipping offer\n- Product education and benefits\n- Customer testimonials and reviews\n\n### Win-back Segment\n- Stronger incentives (40% off)\n- \"We miss you\" messaging\n- Product recommendations based on past purchases\n\n## Performance Tracking\n\n### Key Metrics\n- **Open rate target**: >25%\n- **Click rate target**: >6%\n- **Conversion rate target**: >12%\n- **Revenue per email**: >$2.50\n- **Unsubscribe rate**: <0.5%\n\n### A/B Testing\n- Subject line variations\n- Send time optimization\n- CTA button colors and text\n- Discount presentation (% vs $)\n\n## Post-Campaign Analysis\n- Revenue attribution and ROI calculation\n- Segment performance comparison\n- Creative element performance\n- Learnings for future campaigns",

        "Email campaign **\"Back to School Essentials\"** performed exceptionally well. **Open rate: 31.2%** (vs 24% average), **click rate: 8.7%** (vs 5.8% average). Personalized product recommendations increased click-through by **43%**. Segmented messaging strategy contributed to **$127k** in attributed revenue.",

        "## Newsletter Content Strategy\n\n### Monthly Newsletter Framework\n\n#### Section 1: From the Founder\n- Personal message from company leadership\n- Company updates and milestones\n- Behind-the-scenes insights\n- Future vision and direction\n\n#### Section 2: Customer Spotlight\n- Feature 2-3 customer stories monthly\n- Include photos and quotes\n- Show diverse use cases and demographics\n- Link to full case studies on website\n\n#### Section 3: Product Education\n- How-to guides and tips\n- Product care instructions\n- Styling and usage inspiration\n- Expert advice and recommendations\n\n#### Section 4: Community Corner\n- User-generated content highlights\n- Social media features\n- Community challenges and contests\n- Upcoming events and partnerships\n\n#### Section 5: Exclusive Offers\n- Subscriber-only discounts\n- Early access to new products\n- Limited edition items\n- Loyalty program updates\n\n### Content Calendar\n\n#### January: New Year, New You\n- Goal-setting content\n- Self-improvement products\n- Customer transformation stories\n\n#### February: Love Your Space\n- Home organization tips\n- Valentine's Day gift guides\n- Relationship with products theme\n\n#### March: Spring Refresh\n- Spring cleaning inspiration\n- New collection previews\n- Sustainability focus\n\n### Performance Goals\n- **Open rate**: Maintain >30%\n- **Click rate**: Target >8%\n- **Forward rate**: Target >2%\n- **Revenue attribution**: $15k+ monthly",
    ];
    
    campaigns.choose(_rng).unwrap().to_string()
}

fn generate_budget_analysis(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    let budgets = [
        "# Q3 Marketing Budget Analysis & Recommendations\n\n## Budget Overview\n**Total Q3 Budget**: $850,000\n**Spent to date**: $567,000 (67%)\n**Remaining**: $283,000\n**Burn rate**: On track with 68% of quarter elapsed\n\n## Channel Performance vs Budget\n\n### Digital Advertising: $340,000 allocated\n**Spent**: $228,000 (67%)\n**Performance**: 142% of target ROI\n**Recommendation**: Increase allocation by $50k\n\n#### Breakdown\n- **Google Ads**: $120k spent, 3.2x ROAS\n- **Facebook/Instagram**: $85k spent, 4.1x ROAS  \n- **LinkedIn**: $23k spent, 2.8x ROAS\n\n### Content Marketing: $180,000 allocated\n**Spent**: $121,000 (67%)\n**Performance**: Strong engagement, lower direct attribution\n**Recommendation**: Maintain current spend\n\n#### Investments\n- **Video production**: $45k (high-performing content)\n- **Photography**: $28k (product and lifestyle)\n- **Copywriting**: $35k (all channels)\n- **Design**: $13k (graphics and layouts)\n\n### Email Marketing: $95,000 allocated\n**Spent**: $63,000 (66%)\n**Performance**: Exceeding revenue targets by 23%\n**Recommendation**: Increase automation investment\n\n### Events & Partnerships: $125,000 allocated\n**Spent**: $67,000 (54%)\n**Performance**: Mixed results, some events cancelled\n**Recommendation**: Reallocate $30k to digital channels\n\n### Public Relations: $110,000 allocated\n**Spent**: $88,000 (80%)\n**Performance**: Strong media coverage, difficult to measure ROI\n**Recommendation**: Maintain current trajectory\n\n## ROI Analysis by Channel\n\n### Highest ROI Channels\n1. **Email marketing**: 6.2x return\n2. **Social media advertising**: 4.1x return\n3. **Google Ads**: 3.2x return\n4. **Content marketing**: 2.8x return (longer attribution window)\n\n### Underperforming Areas\n- **LinkedIn advertising**: 2.8x return (below 3.5x target)\n- **Trade shows**: 1.9x return (industry events)\n- **Print advertising**: 1.4x return (legacy channels)\n\n## Budget Reallocation Recommendations\n\n### Immediate Changes (Remaining Q3)\n- **Increase digital advertising**: +$50k\n- **Reduce event spending**: -$30k\n- **Increase email automation**: +$15k\n- **Reduce print advertising**: -$20k\n- **Increase video content**: +$10k\n\n### Q4 Budget Planning\n- **Digital advertising**: Increase to 45% of total budget\n- **Content marketing**: Maintain at 20%\n- **Email marketing**: Increase to 15%\n- **Events**: Reduce to 10%\n- **PR**: Maintain at 10%\n\n## Cost Optimization Opportunities\n\n### Immediate Savings\n- **Renegotiate agency fees**: Potential $15k quarterly savings\n- **Consolidate tool subscriptions**: $8k savings\n- **Optimize ad spend**: Better targeting could improve efficiency by 15%\n\n### Long-term Efficiencies\n- **In-house content creation**: 30% cost reduction\n- **Marketing automation**: Reduce manual work costs\n- **Performance-based partnerships**: Shift fixed costs to variable\n\n## Investment Priorities\n\n### Technology & Tools\n- **Attribution platform**: $25k setup, $5k monthly\n- **Marketing automation**: $15k implementation\n- **Analytics dashboard**: $10k custom development\n\n### Team & Resources\n- **Video content specialist**: $65k salary\n- **Performance marketing manager**: $75k salary\n- **Freelance designer pool**: $20k quarterly\n\n## Risk Assessment\n\n### Budget Risks\n- **Economic downturn**: Could impact performance marketing efficiency\n- **iOS updates**: May affect Facebook advertising attribution\n- **Competition increase**: Could drive up acquisition costs\n\n### Mitigation Strategies\n- **Diversify channels**: Reduce dependence on any single platform\n- **Focus on retention**: Lower customer acquisition pressure\n- **Build owned media**: Email list and content properties",

        "Marketing budget analysis shows **digital channels** delivering **4.2x ROI** vs traditional **1.8x ROI**. Recommendation: shift **$75k** from traditional to digital advertising for Q4. **Email marketing** continues to outperform with **6.1x** return on investment.",

        "## Cost-per-Acquisition Analysis\n\n### Channel CPA Comparison\n\n#### Paid Channels\n- **Google Ads**: $67 CPA (target: $65)\n- **Facebook Ads**: $52 CPA (target: $55) ✅\n- **LinkedIn Ads**: $124 CPA (target: $95) ⚠️\n- **Display Ads**: $89 CPA (target: $80)\n\n#### Organic Channels\n- **SEO**: $23 CPA (excellent performance)\n- **Email**: $15 CPA (best performing)\n- **Social Organic**: $31 CPA\n- **Referrals**: $19 CPA\n\n### Trending Analysis\n- **Google Ads CPA** decreased 12% this quarter\n- **Facebook CPA** stable with iOS changes\n- **LinkedIn CPA** increased 34% (needs optimization)\n- **Email CPA** improved 8% with automation\n\n### Optimization Recommendations\n1. **Pause underperforming LinkedIn campaigns**\n2. **Increase budget for Facebook and email**\n3. **Test new creative for Google Ads**\n4. **Implement lookalike audiences**",
    ];
    
    budgets.choose(_rng).unwrap().to_string()
}

// Additional content generation functions
fn generate_competitor_analysis(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    format!("Competitor **BrandX** launched new sustainability campaign. Strong visual identity but **limited social engagement**. Their pricing strategy shows **15% premium** over market average. Opportunity to position our brand as **accessible sustainability** alternative.")
}

fn generate_product_launch_plan(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    format!("## Product Launch Timeline\n\n**Product**: Eco-Friendly Collection\n**Launch Date**: 6 weeks from today\n\n### Pre-launch (Weeks 1-4)\n- Influencer seeding program\n- Content creation and asset development\n- Email list building campaign\n- PR outreach to sustainability publications\n\n### Launch Week (Week 5)\n- Press release distribution\n- Social media campaign activation\n- Email announcement sequence\n- Influencer content publication\n\n### Post-launch (Week 6+)\n- Performance monitoring and optimization\n- Customer feedback collection\n- User-generated content campaigns\n- Expansion to additional channels")
}

fn generate_partnership_notes(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    format!("Partnership opportunity with **SustainableLiving Magazine** for Q4 content collaboration. They offer **250k monthly readers** in our target demographic. Proposed partnership includes product features, expert quotes, and co-branded content series. Investment: **$25k** for 6-month partnership.")
}

fn generate_brand_guidelines(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    format!("Updated brand voice guidelines emphasize **authentic**, **empowering**, and **sustainable** messaging. Avoid corporate jargon, use inclusive language, and always connect back to real customer benefits. Tone should be conversational but knowledgeable, helpful but not pushy.")
}

fn generate_content_calendar(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    format!("## Weekly Content Themes\n\n**Monday**: Motivation and inspiration\n**Tuesday**: Educational content and tips\n**Wednesday**: Behind-the-scenes and company culture\n**Thursday**: Customer spotlights and testimonials\n**Friday**: Fun, engaging, and community-focused content\n\n**Monthly themes**: Rotate between sustainability, wellness, productivity, and community focus.")
}

fn generate_roi_report(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    format!("## Monthly ROI Summary\n\n**Overall marketing ROI**: 3.8x\n**Top performer**: Email marketing (6.2x)\n**Biggest opportunity**: LinkedIn optimization\n**Total revenue attributed**: $347k\n**Cost per acquisition average**: $62\n\n**Trending up**: Video content engagement (+45%), mobile conversions (+23%)\n**Trending down**: Display ad performance (-12%), newsletter open rates (-3%)")
}

fn generate_customer_feedback(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    format!("Customer feedback highlights: **98%** would recommend to friends, **92%** satisfied with product quality. Main requests: more color options (**34%** of feedback), faster shipping (**28%**), and better packaging (**23%**). Positive mentions: customer service (**89%**), product durability (**85%**).")
}

fn generate_ab_test_results(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    format!("A/B test results: **Personalized subject lines** increased email open rates by **31%**. **Video thumbnails** in social ads improved click-through by **28%**. **Green CTA buttons** outperformed blue by **15%**. **Customer photos** in ads performed **40%** better than stock imagery.")
}

fn generate_influencer_campaign(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    format!("## Influencer Campaign Results\n\n**Campaign**: #SustainableChoices\n**Participants**: 15 micro-influencers\n**Total reach**: 485k\n**Engagement rate**: 4.7%\n**Generated UGC**: 67 posts\n**Website traffic**: +2,340 visitors\n**Conversions**: 89 sales\n**ROI**: 3.2x\n\n**Top performer**: @EcoLifestyle_Sarah (12k reach, 6.8% engagement)")
}

fn generate_seo_strategy(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    format!("SEO strategy update: Focus on **long-tail keywords** around sustainable living. Target **\"eco-friendly [product category]\"** phrases. Content gaps identified in **how-to guides** and **comparison articles**. Opportunity to rank for **\"sustainable alternatives to [competitor products]\"**. Expected timeline: **3-6 months** for ranking improvements.")
}

fn generate_event_planning(_rng: &mut ThreadRng, _date: &str, _entry_idx: usize) -> String {
    format!("Planning participation in **Sustainable Living Expo**. Expected attendance: **15k visitors**. Booth design focuses on **interactive product demos** and **sustainability education**. Goals: collect **500 leads**, showcase new product line, and build brand awareness. Budget: **$35k** including booth, staff, and promotional materials.")
}