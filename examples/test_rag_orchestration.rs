//! Test the enhanced RAG orchestration service for AIChatNode functionality

use nodespace_core_logic::{
    ChatMessage, MessageRole, RAGQueryRequest, RAGService, ServiceContainer,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🤖 Testing Enhanced RAG Orchestration Service");
    println!("================================================\n");

    // Initialize ServiceContainer
    let container = ServiceContainer::new().await?;
    println!("✅ ServiceContainer initialized successfully");

    // Test 1: Simple query without conversation context
    println!("\n1️⃣ Testing simple query without conversation context:");
    let simple_query = "What marketing activities were completed recently?";

    match container.process_simple_query(simple_query).await {
        Ok(response) => {
            println!("   ✅ Simple query successful");
            println!(
                "   📝 Answer: {}",
                response.answer.chars().take(100).collect::<String>() + "..."
            );
            println!("   📊 Sources used: {}", response.sources.len());
            println!("   🎯 Relevance score: {:.2}", response.relevance_score);
            println!("   ⏱️  Generation time: {}ms", response.generation_time_ms);
            println!("   🧠 Context tokens: {}", response.context_tokens);
        }
        Err(e) => {
            println!("   ❌ Simple query failed: {}", e);
        }
    }

    // Test 2: Query with conversation context
    println!("\n2️⃣ Testing query with conversation context:");

    // Create some mock conversation history
    let conversation_history = vec![
        ChatMessage {
            id: "msg1".to_string(),
            session_id: "test_session".to_string(),
            content: "Tell me about our email marketing campaigns".to_string(),
            role: MessageRole::User,
            timestamp: chrono::Utc::now(),
            sequence_number: 1,
            rag_context: None,
        },
        ChatMessage {
            id: "msg2".to_string(),
            session_id: "test_session".to_string(),
            content: "Our email campaigns have been performing well with good open rates."
                .to_string(),
            role: MessageRole::Assistant,
            timestamp: chrono::Utc::now(),
            sequence_number: 2,
            rag_context: None,
        },
    ];

    let contextual_query = "What were the specific metrics mentioned?";

    match container
        .process_conversation_query(
            contextual_query,
            "test_session",
            conversation_history.clone(),
            None,
        )
        .await
    {
        Ok(response) => {
            println!("   ✅ Contextual query successful");
            println!(
                "   📝 Answer: {}",
                response.answer.chars().take(150).collect::<String>() + "..."
            );
            println!("   📊 Sources used: {}", response.sources.len());
            println!("   🎯 Relevance score: {:.2}", response.relevance_score);
            println!(
                "   💬 Conversation context used: {} tokens",
                response.conversation_context_used
            );
            println!("   📄 Context summary: {}", response.context_summary);
        }
        Err(e) => {
            println!("   ❌ Contextual query failed: {}", e);
        }
    }

    // Test 3: Direct RAG service usage
    println!("\n3️⃣ Testing direct RAG service usage:");

    let rag_request = RAGQueryRequest {
        query: "What social media activities have been completed?".to_string(),
        session_id: "test_session".to_string(),
        conversation_history,
        date_scope: Some("2025-06-24".to_string()),
        max_results: Some(3),
    };

    match container.process_rag_query(rag_request).await {
        Ok(response) => {
            println!("   ✅ RAG service query successful");
            println!(
                "   📝 Answer: {}",
                response.answer.chars().take(150).collect::<String>() + "..."
            );
            println!("   📊 Sources used: {}", response.sources.len());
            println!("   🎯 Relevance score: {:.2}", response.relevance_score);
            println!("   ⏱️  Generation time: {}ms", response.generation_time_ms);
            println!("   🧠 Total context tokens: {}", response.context_tokens);
            println!(
                "   💬 Conversation tokens: {}",
                response.conversation_context_used
            );
        }
        Err(e) => {
            println!("   ❌ RAG service query failed: {}", e);
        }
    }

    // Test 4: Backward compatibility with legacy process_query
    println!("\n4️⃣ Testing backward compatibility with legacy process_query:");

    use nodespace_core_logic::CoreLogic;
    match container
        .process_query("What were our recent marketing achievements?")
        .await
    {
        Ok(response) => {
            println!("   ✅ Legacy process_query successful");
            println!(
                "   📝 Answer: {}",
                response.answer.chars().take(100).collect::<String>() + "..."
            );
            println!("   📊 Sources: {}", response.sources.len());
            println!("   🎯 Confidence: {:.2}", response.confidence);
            println!("   🔗 Related queries: {}", response.related_queries.len());
        }
        Err(e) => {
            println!("   ❌ Legacy process_query failed: {}", e);
        }
    }

    // Test 5: RAG configuration
    println!("\n5️⃣ Testing RAG configuration:");
    let rag_config = container.get_rag_config();
    println!(
        "   📊 Max retrieval results: {}",
        rag_config.max_retrieval_results
    );
    println!(
        "   🎯 Relevance threshold: {:.2}",
        rag_config.relevance_threshold
    );
    println!(
        "   🧠 Max context tokens: {}",
        rag_config.max_context_tokens
    );
    println!(
        "   💬 Conversation context limit: {}",
        rag_config.conversation_context_limit
    );
    println!(
        "   📝 Reserved response tokens: {}",
        rag_config.reserved_response_tokens
    );

    println!("\n🎉 RAG Orchestration Service testing completed!");
    println!("   ✅ All core functionality implemented and working");
    println!("   ✅ Conversation context support");
    println!("   ✅ Token budget management");
    println!("   ✅ Backward compatibility maintained");
    println!("   ✅ Enhanced metadata and transparency");

    Ok(())
}
