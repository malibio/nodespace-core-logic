use nodespace_core_logic::{CoreLogic, NodeSpaceService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debugging Embedding Similarity Calculation");
    println!("==============================================");

    let service = NodeSpaceService::create_with_paths(
        "/Users/malibio/nodespace/data/lance_db",
        Some("./bundled_models"),
    )
    .await?;

    service.initialize().await?;

    // Test 1: Check what the actual database similarity search returns for different queries
    println!("\n🧪 Test 1: Compare similarity scores for different queries");

    let test_queries = vec![
        "marketing team resources Product Launch",
        "campaign management 40% capacity",
        "budget allocation planning",
        "cats and dogs completely unrelated",
    ];

    for (i, query) in test_queries.iter().enumerate() {
        println!("\n--- Query {}: '{}' ---", i + 1, query);
        let results = service.semantic_search(query, 5).await?;

        for (j, result) in results.iter().enumerate() {
            let content = result.node.content.as_str().unwrap_or("No content");
            println!(
                "{}. Score: {:.6} | Content: '{}'",
                j + 1,
                result.score,
                content.chars().take(60).collect::<String>()
            );
        }
    }

    // Test 2: Detailed analysis of the problematic query
    println!("\n\n🧪 Test 2: Detailed analysis of the exact problematic query");

    let query =
        "How much of the marketing team's resources would we need to support the Product Launch";
    let results = service.semantic_search(query, 15).await?;

    println!("Query: '{}'", query);
    println!("Total results: {}", results.len());

    // Analyze score distribution
    let scores: Vec<f32> = results.iter().map(|r| r.score).collect();
    let min_score = scores.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_score = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

    println!("Score range: {:.6} to {:.6}", min_score, max_score);

    // Count unique scores by converting to integers (multiply by 1M and round)
    use std::collections::HashMap;
    let mut score_counts = HashMap::new();
    for &score in &scores {
        let rounded_int = (score * 1_000_000.0).round() as i32;
        *score_counts.entry(rounded_int).or_insert(0) += 1;
    }

    println!("Unique scores: {}", score_counts.len());
    for (score_int, count) in score_counts.iter() {
        let score_float = *score_int as f32 / 1_000_000.0;
        println!("  Score {:.6}: {} occurrences", score_float, count);
    }

    // Show all results with their scores
    println!("\nAll results:");
    for (i, result) in results.iter().enumerate() {
        let content = result.node.content.as_str().unwrap_or("No content");
        println!(
            "{}. Score: {:.6} | Content: '{}'",
            i + 1,
            result.score,
            content.chars().take(80).collect::<String>()
        );

        if content.contains("Campaign Management") && content.contains("40%") {
            println!("   🎯 ← PERFECT ANSWER (should be ranked higher!)");
        }
    }

    // Test 3: Check if this is a LanceDB specific issue
    println!("\n\n🧪 Test 3: Testing score precision");

    // Try a query that should have very different similarity scores
    let diverse_query = "xyz123 random nonsense that should not match anything";
    let diverse_results = service.semantic_search(diverse_query, 5).await?;

    println!("Diverse query: '{}'", diverse_query);
    for (i, result) in diverse_results.iter().enumerate() {
        let content = result.node.content.as_str().unwrap_or("No content");
        println!(
            "{}. Score: {:.6} | Content: '{}'",
            i + 1,
            result.score,
            content.chars().take(60).collect::<String>()
        );
    }

    Ok(())
}
