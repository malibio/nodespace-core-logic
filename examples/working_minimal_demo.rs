use async_trait::async_trait;
use nodespace_core_logic::{CoreLogic, DataStore, NLPEngine, NodeSpaceService};
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Minimal working DataStore implementation for testing
struct MinimalDataStore {
    nodes: Arc<Mutex<HashMap<String, Node>>>,
}

impl MinimalDataStore {
    fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl DataStore for MinimalDataStore {
    // Core CRUD Operations
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

    // Relationship Management
    async fn create_relationship(&self, _from: &NodeId, _to: &NodeId, _rel_type: &str) -> NodeSpaceResult<()> {
        // Minimal implementation - store relationships would go here
        Ok(())
    }

    // Vector Search Operations - minimal implementations
    async fn store_node_with_embedding(&self, node: Node, _embedding: Vec<f32>) -> NodeSpaceResult<NodeId> {
        self.store_node(node).await
    }

    async fn update_node_with_embedding(&self, node: Node, _embedding: Vec<f32>) -> NodeSpaceResult<()> {
        self.update_node(node).await
    }

    async fn update_node_embedding(&self, _id: &NodeId, _embedding: Vec<f32>) -> NodeSpaceResult<()> {
        Ok(())
    }

    async fn search_similar_nodes(&self, _embedding: Vec<f32>, limit: usize) -> NodeSpaceResult<Vec<(Node, f32)>> {
        let nodes = self.nodes.lock().unwrap();
        let results: Vec<(Node, f32)> = nodes
            .values()
            .take(limit)
            .enumerate()
            .map(|(i, node)| (node.clone(), 1.0 - (i as f32 * 0.1)))
            .collect();
        Ok(results)
    }

    async fn semantic_search_with_embedding(&self, embedding: Vec<f32>, limit: usize) -> NodeSpaceResult<Vec<(Node, f32)>> {
        self.search_similar_nodes(embedding, limit).await
    }

    // Multi-Level Embedding Support - stub implementations
    async fn store_node_with_multi_embeddings(&self, node: Node, _embeddings: nodespace_data_store::MultiLevelEmbeddings) -> NodeSpaceResult<NodeId> {
        self.store_node(node).await
    }

    async fn update_node_embeddings(&self, _node_id: &NodeId, _embeddings: nodespace_data_store::MultiLevelEmbeddings) -> NodeSpaceResult<()> {
        Ok(())
    }

    async fn get_node_embeddings(&self, _node_id: &NodeId) -> NodeSpaceResult<Option<nodespace_data_store::MultiLevelEmbeddings>> {
        Ok(None)
    }

    async fn search_by_individual_embedding(&self, embedding: Vec<f32>, limit: usize) -> NodeSpaceResult<Vec<(Node, f32)>> {
        self.search_similar_nodes(embedding, limit).await
    }

    async fn search_by_contextual_embedding(&self, embedding: Vec<f32>, limit: usize) -> NodeSpaceResult<Vec<(Node, f32)>> {
        self.search_similar_nodes(embedding, limit).await
    }

    async fn search_by_hierarchical_embedding(&self, embedding: Vec<f32>, limit: usize) -> NodeSpaceResult<Vec<(Node, f32)>> {
        self.search_similar_nodes(embedding, limit).await
    }

    async fn hybrid_semantic_search(&self, _embeddings: nodespace_data_store::QueryEmbeddings, _config: nodespace_data_store::HybridSearchConfig) -> NodeSpaceResult<Vec<nodespace_data_store::SearchResult>> {
        Ok(vec![])
    }

    // Cross-Modal Support - stub implementations  
    async fn create_image_node(&self, _image_node: nodespace_data_store::ImageNode) -> NodeSpaceResult<String> {
        Ok("stub-image-id".to_string())
    }

    async fn get_image_node(&self, _id: &str) -> NodeSpaceResult<Option<nodespace_data_store::ImageNode>> {
        Ok(None)
    }

    async fn search_multimodal(&self, _query_embedding: Vec<f32>, _types: Vec<nodespace_data_store::NodeType>) -> NodeSpaceResult<Vec<Node>> {
        Ok(vec![])
    }

    async fn hybrid_multimodal_search(&self, _query_embedding: Vec<f32>, _config: &nodespace_data_store::HybridSearchConfig) -> NodeSpaceResult<Vec<nodespace_data_store::SearchResult>> {
        Ok(vec![])
    }

    // Hierarchy Optimization
    async fn get_nodes_by_root(&self, _root_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        Ok(vec![])
    }

    async fn get_nodes_by_root_and_type(&self, _root_id: &NodeId, _node_type: &str) -> NodeSpaceResult<Vec<Node>> {
        Ok(vec![])
    }
}

/// Minimal working NLP Engine implementation
struct MinimalNLPEngine;

#[async_trait]
impl NLPEngine for MinimalNLPEngine {
    async fn generate_embedding(&self, _text: &str) -> NodeSpaceResult<Vec<f32>> {
        Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5])
    }

    async fn batch_embeddings(&self, texts: &[String]) -> NodeSpaceResult<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|_| vec![0.1, 0.2, 0.3, 0.4, 0.5]).collect())
    }

    async fn generate_text(&self, prompt: &str) -> NodeSpaceResult<String> {
        Ok(format!("Generated response for: {}", prompt.chars().take(50).collect::<String>()))
    }

    async fn generate_text_enhanced(&self, request: nodespace_nlp_engine::TextGenerationRequest) -> NodeSpaceResult<nodespace_nlp_engine::EnhancedTextGenerationResponse> {
        Ok(nodespace_nlp_engine::EnhancedTextGenerationResponse {
            text: format!("Enhanced response for: {}", request.prompt.chars().take(50).collect::<String>()),
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

    async fn extract_structured_data(&self, _text: &str, _schema_hint: &str) -> NodeSpaceResult<serde_json::Value> {
        Ok(json!({"extracted": "data"}))
    }

    async fn generate_summary(&self, text: &str, max_length: Option<usize>) -> NodeSpaceResult<String> {
        let limit = max_length.unwrap_or(100);
        Ok(text.chars().take(limit).collect())
    }

    async fn analyze_content(&self, _text: &str, _analysis_type: &str) -> NodeSpaceResult<nodespace_nlp_engine::ContentAnalysis> {
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

    async fn generate_contextual_embedding(&self, _node: &Node, _context: &nodespace_nlp_engine::NodeContext) -> NodeSpaceResult<Vec<f32>> {
        Ok(vec![0.2, 0.3, 0.4, 0.5, 0.6])
    }

    async fn generate_hierarchical_embedding(&self, _node: &Node, _path: &[Node]) -> NodeSpaceResult<Vec<f32>> {
        Ok(vec![0.3, 0.4, 0.5, 0.6, 0.7])
    }

    async fn generate_all_embeddings(&self, node: &Node, context: &nodespace_nlp_engine::NodeContext, path: &[Node]) -> NodeSpaceResult<nodespace_nlp_engine::MultiLevelEmbeddings> {
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
    println!("🚀 Testing Minimal NodeSpace Service Integration");

    // Create service components
    let data_store = MinimalDataStore::new();
    let nlp_engine = MinimalNLPEngine;

    // Create service
    let service = NodeSpaceService::new(data_store, nlp_engine);

    // Initialize service
    println!("📋 Initializing service...");
    service.initialize().await?;

    // Test basic node creation with proper fields
    println!("📝 Creating test node...");
    let _test_node = Node {
        id: NodeId::new(),
        content: json!("This is a test node for the minimal demo"),
        metadata: Some(json!({"type": "test", "demo": true})),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        parent_id: None,
        next_sibling: None,
        previous_sibling: None,
        root_id: None,  // Required field for NS-115
        root_type: None, // Required field for NS-115
    };

    let node_id = service.create_knowledge_node("Test content for minimal demo", json!({"type": "test"})).await?;
    println!("✅ Created node with ID: {}", node_id);

    // Test semantic search
    println!("🔍 Testing semantic search...");
    let search_results = service.semantic_search("test content", 5).await?;
    println!("✅ Search returned {} results", search_results.len());

    // Test query processing 
    println!("💬 Testing query processing...");
    let query_response = service.process_query("What is this test about?").await?;
    println!("✅ Query response: {}", query_response.answer.chars().take(100).collect::<String>());

    println!("🎉 Minimal demo completed successfully!");
    Ok(())
}