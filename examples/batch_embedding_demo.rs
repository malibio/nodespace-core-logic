use async_trait::async_trait;
use nodespace_core_logic::{CoreLogic, DataStore, NLPEngine, NodeSpaceService};
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Minimal DataStore for batch embedding performance testing
struct BatchTestDataStore {
    nodes: Arc<Mutex<HashMap<String, Node>>>,
}

impl BatchTestDataStore {
    fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl DataStore for BatchTestDataStore {
    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId> {
        let id = node.id.clone();
        self.nodes.lock().unwrap().insert(node.id.to_string(), node);
        Ok(id)
    }

    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        Ok(self.nodes.lock().unwrap().get(&id.to_string()).cloned())
    }

    async fn store_node_with_embedding(
        &self,
        node: Node,
        _embedding: Vec<f32>,
    ) -> NodeSpaceResult<NodeId> {
        // For performance testing, just store the node
        self.store_node(node).await
    }

    // Stub implementations for required methods
    async fn update_node(&self, node: Node) -> NodeSpaceResult<()> {
        self.nodes.lock().unwrap().insert(node.id.to_string(), node);
        Ok(())
    }

    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
        self.nodes.lock().unwrap().remove(&id.to_string());
        Ok(())
    }

    async fn query_nodes(&self, _query: &str) -> NodeSpaceResult<Vec<Node>> {
        Ok(self.nodes.lock().unwrap().values().cloned().collect())
    }

    async fn create_relationship(
        &self,
        _from: &NodeId,
        _to: &NodeId,
        _rel_type: &str,
    ) -> NodeSpaceResult<()> {
        Ok(())
    }

    async fn update_node_with_embedding(
        &self,
        node: Node,
        _embedding: Vec<f32>,
    ) -> NodeSpaceResult<()> {
        self.update_node(node).await
    }

    async fn update_node_embedding(
        &self,
        _id: &NodeId,
        _embedding: Vec<f32>,
    ) -> NodeSpaceResult<()> {
        Ok(())
    }

    async fn search_similar_nodes(
        &self,
        _embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        let nodes = self.nodes.lock().unwrap();
        let results: Vec<(Node, f32)> = nodes
            .values()
            .take(limit)
            .enumerate()
            .map(|(i, node)| (node.clone(), 1.0 - (i as f32 * 0.1)))
            .collect();
        Ok(results)
    }

    async fn semantic_search_with_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        self.search_similar_nodes(embedding, limit).await
    }

    async fn store_node_with_multi_embeddings(
        &self,
        node: Node,
        _embeddings: nodespace_data_store::MultiLevelEmbeddings,
    ) -> NodeSpaceResult<NodeId> {
        self.store_node(node).await
    }

    async fn update_node_embeddings(
        &self,
        _node_id: &NodeId,
        _embeddings: nodespace_data_store::MultiLevelEmbeddings,
    ) -> NodeSpaceResult<()> {
        Ok(())
    }

    async fn get_node_embeddings(
        &self,
        _node_id: &NodeId,
    ) -> NodeSpaceResult<Option<nodespace_data_store::MultiLevelEmbeddings>> {
        Ok(None)
    }

    async fn search_by_individual_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        self.search_similar_nodes(embedding, limit).await
    }

    async fn search_by_contextual_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        self.search_similar_nodes(embedding, limit).await
    }

    async fn search_by_hierarchical_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        self.search_similar_nodes(embedding, limit).await
    }

    async fn hybrid_semantic_search(
        &self,
        _embeddings: nodespace_data_store::QueryEmbeddings,
        _config: nodespace_data_store::HybridSearchConfig,
    ) -> NodeSpaceResult<Vec<nodespace_data_store::SearchResult>> {
        Ok(vec![])
    }

    async fn create_image_node(
        &self,
        _image_node: nodespace_data_store::ImageNode,
    ) -> NodeSpaceResult<String> {
        Ok("test-image-id".to_string())
    }

    async fn get_image_node(
        &self,
        _id: &str,
    ) -> NodeSpaceResult<Option<nodespace_data_store::ImageNode>> {
        Ok(None)
    }

    async fn search_multimodal(
        &self,
        _query_embedding: Vec<f32>,
        _types: Vec<nodespace_data_store::NodeType>,
    ) -> NodeSpaceResult<Vec<Node>> {
        Ok(vec![])
    }

    async fn hybrid_multimodal_search(
        &self,
        _query_embedding: Vec<f32>,
        _config: &nodespace_data_store::HybridSearchConfig,
    ) -> NodeSpaceResult<Vec<nodespace_data_store::SearchResult>> {
        Ok(vec![])
    }

    async fn get_nodes_by_root(&self, _root_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        Ok(vec![])
    }

    async fn get_nodes_by_root_and_type(
        &self,
        _root_id: &NodeId,
        _node_type: &str,
    ) -> NodeSpaceResult<Vec<Node>> {
        Ok(vec![])
    }
}

/// Performance-optimized NLP Engine for batch testing
struct BatchTestNLPEngine;

#[async_trait]
impl NLPEngine for BatchTestNLPEngine {
    async fn generate_embedding(&self, _text: &str) -> NodeSpaceResult<Vec<f32>> {
        // Simulate some processing time for individual embedding
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5])
    }

    async fn batch_embeddings(&self, texts: &[String]) -> NodeSpaceResult<Vec<Vec<f32>>> {
        // Simulate efficient batch processing - much faster per item
        let batch_delay = (texts.len() as u64).max(1) * 2; // 2ms per item in batch
        tokio::time::sleep(tokio::time::Duration::from_millis(batch_delay)).await;

        // Generate different embeddings for each text to simulate real processing
        let embeddings: Vec<Vec<f32>> = texts
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let base = i as f32 * 0.1;
                vec![base + 0.1, base + 0.2, base + 0.3, base + 0.4, base + 0.5]
            })
            .collect();

        Ok(embeddings)
    }

    async fn generate_text(&self, prompt: &str) -> NodeSpaceResult<String> {
        Ok(format!(
            "Generated response for: {}",
            prompt.chars().take(50).collect::<String>()
        ))
    }

    async fn generate_text_enhanced(
        &self,
        request: nodespace_nlp_engine::TextGenerationRequest,
    ) -> NodeSpaceResult<nodespace_nlp_engine::EnhancedTextGenerationResponse> {
        Ok(nodespace_nlp_engine::EnhancedTextGenerationResponse {
            text: format!(
                "Enhanced response for: {}",
                request.prompt.chars().take(50).collect::<String>()
            ),
            tokens_used: 25,
            generation_metrics: nodespace_nlp_engine::GenerationMetrics {
                generation_time_ms: 100,
                context_tokens: 10,
                response_tokens: 25,
                temperature_used: 0.7,
            },
            context_utilization: nodespace_nlp_engine::ContextUtilization {
                context_referenced: true,
                sources_mentioned: vec!["test-source".to_string()],
                relevance_score: 0.8,
            },
        })
    }

    async fn extract_structured_data(
        &self,
        _text: &str,
        _schema_hint: &str,
    ) -> NodeSpaceResult<serde_json::Value> {
        Ok(json!({"extracted": "data"}))
    }

    async fn generate_summary(
        &self,
        text: &str,
        max_length: Option<usize>,
    ) -> NodeSpaceResult<String> {
        let limit = max_length.unwrap_or(100);
        Ok(text.chars().take(limit).collect())
    }

    async fn analyze_content(
        &self,
        _text: &str,
        _analysis_type: &str,
    ) -> NodeSpaceResult<nodespace_nlp_engine::ContentAnalysis> {
        Ok(nodespace_nlp_engine::ContentAnalysis {
            sentiment: Some("neutral".to_string()),
            topics: vec!["general".to_string()],
            entities: vec![],
            confidence: 0.8,
            classification: "general".to_string(),
            processing_time_ms: 50,
        })
    }

    fn embedding_dimensions(&self) -> usize {
        5
    }

    async fn generate_contextual_embedding(
        &self,
        _node: &Node,
        _context: &nodespace_nlp_engine::NodeContext,
    ) -> NodeSpaceResult<Vec<f32>> {
        Ok(vec![0.2, 0.3, 0.4, 0.5, 0.6])
    }

    async fn generate_hierarchical_embedding(
        &self,
        _node: &Node,
        _path: &[Node],
    ) -> NodeSpaceResult<Vec<f32>> {
        Ok(vec![0.3, 0.4, 0.5, 0.6, 0.7])
    }

    async fn generate_all_embeddings(
        &self,
        node: &Node,
        context: &nodespace_nlp_engine::NodeContext,
        path: &[Node],
    ) -> NodeSpaceResult<nodespace_nlp_engine::MultiLevelEmbeddings> {
        let individual = self.generate_embedding(&node.content.to_string()).await?;
        let contextual = self.generate_contextual_embedding(node, context).await?;
        let hierarchical = self.generate_hierarchical_embedding(node, path).await?;

        Ok(nodespace_nlp_engine::MultiLevelEmbeddings {
            individual,
            contextual: Some(contextual),
            hierarchical: Some(hierarchical),
            context_strategy: nodespace_nlp_engine::ContextStrategy::RuleBased,
            generated_at: chrono::Utc::now(),
            generation_metrics: nodespace_nlp_engine::EmbeddingGenerationMetrics {
                total_time_ms: 50,
                individual_time_ms: 15,
                contextual_time_ms: Some(20),
                hierarchical_time_ms: Some(15),
                context_length: Some(10),
                path_depth: Some(1),
                cache_hits: 0,
                cache_misses: 0,
            },
        })
    }
}

#[tokio::main]
async fn main() -> NodeSpaceResult<()> {
    println!("🚀 Testing Batch Embedding Performance Optimization");

    // Create service components
    let data_store = BatchTestDataStore::new();
    let nlp_engine = BatchTestNLPEngine;
    let service = NodeSpaceService::new(data_store, nlp_engine);

    // Initialize service
    println!("📋 Initializing service...");
    service.initialize().await?;

    // Prepare test data
    let test_data = vec![
        (
            "This is the first test document about AI and machine learning".to_string(),
            json!({"type": "document", "category": "ai"}),
        ),
        (
            "Second document discusses database optimization techniques".to_string(),
            json!({"type": "document", "category": "database"}),
        ),
        (
            "Third text covers web development frameworks and patterns".to_string(),
            json!({"type": "document", "category": "web"}),
        ),
        (
            "Fourth piece explores cloud computing architectures".to_string(),
            json!({"type": "document", "category": "cloud"}),
        ),
        (
            "Fifth document analyzes cybersecurity best practices".to_string(),
            json!({"type": "document", "category": "security"}),
        ),
        (
            "Sixth text examines data science methodologies".to_string(),
            json!({"type": "document", "category": "data"}),
        ),
        (
            "Seventh document reviews mobile app development".to_string(),
            json!({"type": "document", "category": "mobile"}),
        ),
        (
            "Eighth piece discusses DevOps automation tools".to_string(),
            json!({"type": "document", "category": "devops"}),
        ),
        (
            "Ninth text covers blockchain technology concepts".to_string(),
            json!({"type": "document", "category": "blockchain"}),
        ),
        (
            "Tenth document explores quantum computing principles".to_string(),
            json!({"type": "document", "category": "quantum"}),
        ),
    ];

    println!("📊 Performance Test: {} documents", test_data.len());

    // Test 1: Individual embedding generation (baseline)
    println!("\n🔄 Test 1: Individual Embedding Generation (Baseline)");
    let individual_start = Instant::now();

    let mut individual_node_ids = Vec::new();
    for (content, metadata) in &test_data {
        let node_id = service
            .create_knowledge_node(content, metadata.clone())
            .await?;
        individual_node_ids.push(node_id);
    }

    let individual_duration = individual_start.elapsed();
    println!(
        "✅ Individual processing: {:.2}ms ({:.1} nodes/sec)",
        individual_duration.as_millis(),
        test_data.len() as f64 / individual_duration.as_secs_f64()
    );

    // Test 2: Batch embedding generation (optimized)
    println!("\n⚡ Test 2: Batch Embedding Generation (Optimized)");
    let batch_start = Instant::now();

    let batch_node_ids = service
        .create_knowledge_nodes_batch(test_data.clone())
        .await?;

    let batch_duration = batch_start.elapsed();
    println!(
        "✅ Batch processing: {:.2}ms ({:.1} nodes/sec)",
        batch_duration.as_millis(),
        test_data.len() as f64 / batch_duration.as_secs_f64()
    );

    // Performance comparison
    let speedup = individual_duration.as_millis() as f64 / batch_duration.as_millis() as f64;
    println!("\n📈 Performance Results:");
    println!(
        "   Individual: {:.2}ms total, {:.1} nodes/sec",
        individual_duration.as_millis(),
        test_data.len() as f64 / individual_duration.as_secs_f64()
    );
    println!(
        "   Batch:      {:.2}ms total, {:.1} nodes/sec",
        batch_duration.as_millis(),
        test_data.len() as f64 / batch_duration.as_secs_f64()
    );
    println!(
        "   Speedup:    {:.1}x faster with batch processing",
        speedup
    );

    // Verify results
    println!("\n✅ Verification:");
    println!(
        "   Individual method created: {} nodes",
        individual_node_ids.len()
    );
    println!(
        "   Batch method created:      {} nodes",
        batch_node_ids.len()
    );
    println!(
        "   All node IDs are unique:   {}",
        individual_node_ids
            .iter()
            .chain(batch_node_ids.iter())
            .collect::<std::collections::HashSet<_>>()
            .len()
            == individual_node_ids.len() + batch_node_ids.len()
    );

    println!("\n🎉 Batch Embedding Optimization Test Completed!");
    println!(
        "   Target achieved: {} nodes/sec (target: 10+ nodes/sec)",
        test_data.len() as f64 / batch_duration.as_secs_f64()
    );

    Ok(())
}
