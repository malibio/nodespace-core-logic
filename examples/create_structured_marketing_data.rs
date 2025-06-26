//! Create Structured Marketing Sample Data (Without Complex Relationships)
//!
//! This creates marketing data with logical structure in the content itself,
//! avoiding the cross-table relationship issues for now.

use nodespace_core_logic::{ServiceContainer, CoreLogic};
use std::error::Error;
use rand::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Creating structured marketing sample data...");
    println!("📊 Removing date redundancy and creating logical content structure\n");

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
        println!("📅 Creating content for {} ({}/{})", date_str, date_idx + 1, dates.len());
        
        // Create 3-5 structured content items per date
        let content_count = rng.gen_range(3..=5);
        
        for _ in 0..content_count {
            let content = generate_structured_content(&mut rng);
            
            match service_container.create_text_node(&content, date_str).await {
                Ok(node_id) => {
                    total_entries += 1;
                    let title = content.lines().next().unwrap_or(&content).trim_start_matches("## ");
                    println!("  ✅ {}: {}", total_entries, title);
                }
                Err(e) => {
                    println!("  ❌ Error creating content: {}", e);
                }
            }
        }
        
        if (date_idx + 1) % 10 == 0 {
            println!("📈 Progress: {} entries across {} days", total_entries, date_idx + 1);
        }
    }

    println!("\n🎉 Structured marketing data creation completed!");
    println!("✅ Created {} total entries", total_entries);
    println!("✅ Content structured without date redundancy");
    println!("✅ Ready for hierarchical relationship implementation later");
    
    Ok(())
}

fn generate_marketing_dates() -> Vec<String> {
    let mut dates = Vec::new();
    
    // Generate realistic business dates over 3 months
    for month in 4..=6 { // April, May, June 2025
        for day in [2, 5, 9, 12, 16, 19, 23, 26, 30] { // Business days
            if month == 6 && day > 25 { continue; } // Don't go past today
            if month == 4 && day == 30 { continue; } // April has 30 days
            let date_str = format!("2025-{:02}-{:02}", month, day);
            dates.push(date_str);
        }
    }
    
    dates
}

fn generate_structured_content(rng: &mut ThreadRng) -> String {
    let content_templates = [
        // Customer Feedback Analysis
        "## Customer Feedback Analysis

**Satisfaction Score**: 94%

**Key Insights**:
Product quality exceeds expectations
Customer service response time excellent  
Mobile app usability needs improvement
Shipping speed consistently fast

**Recommended Actions**:
Prioritize mobile app UX improvements
Expand customer service team capacity
Maintain current quality standards
Consider premium shipping options

**Implementation Timeline**: Q3 2025",

        // Campaign Performance Report  
        "## Q2 Campaign Performance Report

**Email Marketing**:
Open rate: 28.5%
Click rate: 6.8% 
Conversion rate: 12.3%
Revenue generated: $245K

**Social Media**:
Engagement rate: 4.2%
Reach: 285K followers
Follower growth: +2,150
Top performing content: Behind-the-scenes videos

**Digital Advertising**:
CTR: 3.7%
CPC: $1.24
ROAS: 4.2x
Budget utilization: 96%

**Overall ROI**: 340%",

        // Market Research Summary
        "## Market Research Summary

**Market Trends**:
Sustainable products gaining 23% market share
Digital marketing ROI up 31% year-over-year  
Customer acquisition cost decreased 15%
Mobile commerce now 67% of total sales

**Competitive Analysis**:
Main competitors showing slower adaptation to market changes
Our brand recognition up 18% in target demographic
Price positioning optimal in premium segment
Customer loyalty scores above industry average

**Customer Behavior Insights**:
Primary research shows preference for authentic brand messaging
Video content engagement 340% higher than static
Purchase decisions heavily influenced by peer reviews
Seasonal buying patterns shifting due to remote work

**Opportunities Identified**:
Expand influencer partnership program
Develop video-first content strategy
Enhance customer review and testimonial features
Target remote worker demographic",

        // Strategy Meeting Notes
        "## Marketing Strategy Meeting

**Attendees**: CMO, Digital Marketing Lead, Content Manager, Analytics Lead

**Key Decisions Made**:
Increase video content production by 50%
Launch micro-influencer partnership program
Reallocate $75K from print to digital channels
Implement advanced attribution tracking

**Action Items Assigned**:
Finalize Q3 creative asset timeline by Friday (Sarah)
Research and outreach to 25 potential influencers (Mike)
Set up multi-touch attribution system (Analytics team)
Draft updated brand voice guidelines (Content team)

**Budget Reallocations Approved**:
Video production: +$45K
Influencer partnerships: +$30K  
Print advertising: -$75K
Performance marketing: +$25K

**Next Review Meeting**: July 2nd - Campaign launch readiness assessment",

        // Product Launch Plan
        "## Product Launch Campaign Plan

**Launch Timeline**: July 15, 2025

**Target Audience Segments**:
Primary: Urban professionals aged 25-40
Secondary: Eco-conscious millennials  
Growth opportunity: Gen Z early adopters
Geographic focus: Major metropolitan areas

**Marketing Channel Strategy**:
Pre-launch teasers on Instagram and TikTok
Influencer unboxing partnerships
Email campaign to existing customer base
PR outreach to sustainability-focused publications
Paid social advertising targeting lookalike audiences

**Success Metrics and Targets**:
Pre-order target: 5,000 units
Email open rate goal: 32%
Social media reach target: 500K
Press coverage goal: 15 major publications
First-month sales target: $850K

**Creative Direction Brief**:
Emphasize sustainability without compromising premium feel
Use earth tones with modern, clean design aesthetic
Feature authentic customer stories and use cases
Highlight product innovation and environmental impact",

        // Brand Guidelines Update
        "## Brand Guidelines Update

**Voice and Tone Evolution**:
Maintain authentic and empowering core messaging
Increase emphasis on sustainability and social impact
Professional yet approachable communication style
Confident without being arrogant

**Visual Identity Refinements**:
Introduce secondary color palette with earth tones
Update typography to improve digital readability
Standardize iconography across all touchpoints
Refresh photography style guide for authenticity

**Content Standards and Messaging**:
Lead with customer benefits, not just features
Include sustainability angle in 70% of content
Use inclusive language and imagery consistently
Maintain educational tone while being engaging

**Implementation Timeline and Rollout**:
Update website and digital assets by month-end
Retrain customer service team on voice guidelines
Audit and update all marketing materials
Partner with creative agencies to align campaigns",

        // Budget Analysis Report
        "## Marketing Budget Analysis

**Current Quarter Allocation and Performance**:
Digital advertising: 45% of budget, 380% ROI
Content creation: 25% of budget, 240% ROI  
Email marketing: 15% of budget, 520% ROI
Events and partnerships: 15% of budget, 180% ROI

**Channel Performance Insights**:
Email marketing delivering highest ROI but limited scale
Social media showing strong engagement but conversion challenges
Paid search performance declining due to increased competition
Video content production costs high but engagement excellent

**Recommended Budget Adjustments for Q3**:
Increase email marketing budget by 35%
Reduce traditional advertising spend by 60%
Invest additional $50K in video content creation
Allocate $25K for marketing automation tools

**Projected Q3 Impact**:
Overall marketing ROI increase of 25%
Cost per acquisition reduction of 18%
Customer lifetime value improvement of 12%
Marketing qualified leads increase of 40%",

        // Customer Journey Analysis
        "## Customer Journey Analysis

**Awareness Stage Performance**:
Social media driving 45% of initial brand awareness
Word-of-mouth referrals account for 32% of new prospects
Paid advertising contributing 23% of awareness
Content marketing and SEO generating sustained interest

**Consideration and Research Phase**:
Average research period: 3.2 weeks
Most visited pages: product comparisons, reviews, pricing
Key decision factors: quality, sustainability, peer recommendations
Common hesitation points: price concerns, feature complexity

**Purchase Decision Triggers**:
Limited-time promotions drive 28% of conversions
Peer recommendations influence 65% of decisions
Free trial or sample programs convert at 34% rate
Customer service interactions highly correlated with purchase

**Post-Purchase and Retention Insights**:
Customer satisfaction peaks at 2-month mark
Support ticket volume highest in first 30 days
Referral rate increases significantly after 6 months
Retention highest among customers acquired through referrals",
    ];
    
    content_templates.choose(rng).unwrap().to_string()
}