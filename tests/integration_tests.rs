use nodespace_core_logic::{CoreLogic, NodeSpaceService};
use nodespace_core_types::{Node, NodeId, NodeSpaceResult};
use nodespace_data_store::DataStore;
use nodespace_nlp_engine::NLPEngine;
use serde_json::json;

/// Mock DataStore for testing
#[derive(Clone)]
struct MockDataStore {
    nodes: std::sync::Arc<tokio::sync::RwLock<Vec<Node>>>,
}

impl MockDataStore {
    fn new() -> Self {
        Self {
            nodes: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl DataStore for MockDataStore {
    async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId> {
        let mut nodes = self.nodes.write().await;
        let id = node.id.clone();
        nodes.push(node);
        Ok(id)
    }

    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        let nodes = self.nodes.read().await;
        Ok(nodes.iter().find(|n| n.id == *id).cloned())
    }

    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
        let mut nodes = self.nodes.write().await;
        nodes.retain(|n| n.id != *id);
        Ok(())
    }

    async fn query_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>> {
        let nodes = self.nodes.read().await;

        // For relationship queries, return empty (mock relationships don't exist)
        if query.contains("relationships") || query.contains("SELECT out FROM") {
            Ok(Vec::new())
        } else {
            // For regular content queries, return all nodes
            Ok(nodes.clone())
        }
    }

    async fn create_relationship(
        &self,
        _from: &NodeId,
        _to: &NodeId,
        _rel_type: &str,
    ) -> NodeSpaceResult<()> {
        Ok(())
    }

    async fn store_node_with_embedding(
        &self,
        node: Node,
        _embedding: Vec<f32>,
    ) -> NodeSpaceResult<NodeId> {
        // For mock, just store the node normally
        self.store_node(node).await
    }

    async fn search_similar_nodes(
        &self,
        _embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        let nodes = self.nodes.read().await;
        let mut results = Vec::new();

        // Return mock results with decreasing similarity scores
        for (index, node) in nodes.iter().take(limit).enumerate() {
            let score = 1.0 - (index as f32 * 0.1);
            results.push((node.clone(), score.max(0.1)));
        }

        Ok(results)
    }

    async fn update_node_embedding(
        &self,
        _id: &NodeId,
        _embedding: Vec<f32>,
    ) -> NodeSpaceResult<()> {
        // For mock, this is a no-op since we don't actually store embeddings
        Ok(())
    }

    async fn semantic_search_with_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // For mock, delegate to search_similar_nodes
        self.search_similar_nodes(embedding, limit).await
    }

    // Cross-modal search methods (NS-81 support)
    async fn create_image_node(&self, _image_node: nodespace_data_store::ImageNode) -> NodeSpaceResult<String> {
        Ok("mock-image-node-id".to_string())
    }

    async fn get_image_node(&self, _id: &str) -> NodeSpaceResult<Option<nodespace_data_store::ImageNode>> {
        Ok(None) // Mock returns no image node
    }

    async fn search_multimodal(&self, _query_embedding: Vec<f32>, _types: Vec<nodespace_data_store::NodeType>) -> NodeSpaceResult<Vec<Node>> {
        let nodes = self.nodes.read().await;
        Ok(nodes.clone())
    }

    async fn hybrid_multimodal_search(&self, _query_embedding: Vec<f32>, _config: &nodespace_data_store::HybridSearchConfig) -> NodeSpaceResult<Vec<nodespace_data_store::SearchResult>> {
        let nodes = self.nodes.read().await;
        let results = nodes.iter().enumerate().map(|(index, node)| {
            nodespace_data_store::SearchResult {
                node: node.clone(),
                score: 0.9 - (index as f32 * 0.1),
                relevance_factors: nodespace_data_store::RelevanceFactors {
                    semantic_score: 0.8,
                    structural_score: 0.7,
                    temporal_score: 0.6,
                    cross_modal_score: Some(0.5),
                },
            }
        }).collect();
        Ok(results)
    }
}

/// Mock NLP Engine for testing
#[derive(Clone)]
struct MockNLPEngine;

#[async_trait::async_trait]
impl NLPEngine for MockNLPEngine {
    async fn generate_embedding(&self, _text: &str) -> NodeSpaceResult<Vec<f32>> {
        Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5]) // Mock embedding
    }

    async fn batch_embeddings(&self, texts: &[String]) -> NodeSpaceResult<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for _ in texts {
            embeddings.push(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
        }
        Ok(embeddings)
    }

    async fn generate_text(&self, prompt: &str) -> NodeSpaceResult<String> {
        Ok(format!(
            "Generated response for: {}",
            prompt.chars().take(50).collect::<String>()
        ))
    }

    async fn generate_surrealql(
        &self,
        _natural_query: &str,
        _schema_context: &str,
    ) -> NodeSpaceResult<String> {
        Ok("SELECT * FROM nodes".to_string())
    }

    fn embedding_dimensions(&self) -> usize {
        5
    }

    async fn generate_text_enhanced(
        &self,
        request: nodespace_nlp_engine::TextGenerationRequest,
    ) -> NodeSpaceResult<nodespace_nlp_engine::EnhancedTextGenerationResponse> {
        // For mock, create a simple enhanced response
        let answer = format!(
            "Generated response for: {}",
            request.prompt.chars().take(50).collect::<String>()
        );

        Ok(nodespace_nlp_engine::EnhancedTextGenerationResponse {
            text: answer,
            tokens_used: 50,
            generation_metrics: nodespace_nlp_engine::GenerationMetrics {
                generation_time_ms: 100,
                context_tokens: 25,
                response_tokens: 50,
                temperature_used: 0.7,
            },
            context_utilization: nodespace_nlp_engine::ContextUtilization {
                context_referenced: true,
                sources_mentioned: vec!["mock-source".to_string()],
                relevance_score: 0.8,
            },
        })
    }
}

#[tokio::test]
async fn test_create_knowledge_node() {
    let data_store = MockDataStore::new();
    let nlp_engine = MockNLPEngine;
    let service = NodeSpaceService::new(data_store, nlp_engine);

    // Initialize the service
    service.initialize().await.unwrap();

    let metadata = json!({"type": "document", "category": "test"});
    let node_id = service
        .create_knowledge_node("Test content", metadata)
        .await
        .unwrap();

    assert!(!node_id.to_string().is_empty());
}

#[tokio::test]
async fn test_semantic_search() {
    let data_store = MockDataStore::new();
    let nlp_engine = MockNLPEngine;
    let service = NodeSpaceService::new(data_store.clone(), nlp_engine);

    // Initialize the service
    service.initialize().await.unwrap();

    // Create some test nodes
    let metadata = json!({"type": "test"});
    let _id1 = service
        .create_knowledge_node("First test document", metadata.clone())
        .await
        .unwrap();
    let _id2 = service
        .create_knowledge_node("Second test document", metadata)
        .await
        .unwrap();

    // Perform semantic search
    let results = service.semantic_search("test document", 10).await.unwrap();

    assert_eq!(results.len(), 2);
    assert!(results[0].score > 0.0);
}

#[tokio::test]
async fn test_process_query() {
    let data_store = MockDataStore::new();
    let nlp_engine = MockNLPEngine;
    let service = NodeSpaceService::new(data_store.clone(), nlp_engine);

    // Initialize the service
    service.initialize().await.unwrap();

    // Create a test node with content
    let metadata = json!({"type": "knowledge"});
    let _node_id = service
        .create_knowledge_node("NodeSpace is a knowledge management system", metadata)
        .await
        .unwrap();

    // Process a query
    let response = service.process_query("What is NodeSpace?").await.unwrap();

    assert!(!response.answer.is_empty());
    assert!(response.confidence > 0.0);
    assert!(!response.related_queries.is_empty());
}

#[tokio::test]
async fn test_update_node() {
    let data_store = MockDataStore::new();
    let nlp_engine = MockNLPEngine;
    let service = NodeSpaceService::new(data_store.clone(), nlp_engine);

    // Initialize the service
    service.initialize().await.unwrap();

    // Create a test node
    let metadata = json!({"type": "test"});
    let node_id = service
        .create_knowledge_node("Original content", metadata)
        .await
        .unwrap();

    // Update the node
    let result = service.update_node(&node_id, "Updated content").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_related_nodes() {
    let data_store = MockDataStore::new();
    let nlp_engine = MockNLPEngine;
    let service = NodeSpaceService::new(data_store, nlp_engine);

    // Initialize the service
    service.initialize().await.unwrap();

    // Create a test node
    let metadata = json!({"type": "test"});
    let node_id = service
        .create_knowledge_node("Test node with relationships", metadata)
        .await
        .unwrap();

    // Get related nodes
    let related = service
        .get_related_nodes(&node_id, vec!["related".to_string()])
        .await
        .unwrap();

    // Should return empty for mock implementation
    assert!(related.is_empty());
}

#[tokio::test]
async fn test_generate_insights() {
    let data_store = MockDataStore::new();
    let nlp_engine = MockNLPEngine;
    let service = NodeSpaceService::new(data_store.clone(), nlp_engine);

    // Initialize the service
    service.initialize().await.unwrap();

    // Create test nodes
    let metadata = json!({"type": "insight_test"});
    let id1 = service
        .create_knowledge_node("First insight content", metadata.clone())
        .await
        .unwrap();
    let id2 = service
        .create_knowledge_node("Second insight content", metadata)
        .await
        .unwrap();

    // Generate insights
    let insights = service.generate_insights(vec![id1, id2]).await.unwrap();

    assert!(!insights.is_empty());
    assert!(insights.contains("Generated response"));
}

#[tokio::test]
async fn test_empty_insights() {
    let data_store = MockDataStore::new();
    let nlp_engine = MockNLPEngine;
    let service = NodeSpaceService::new(data_store, nlp_engine);

    // Test with empty node list
    let insights = service.generate_insights(vec![]).await.unwrap();

    assert_eq!(insights, "No nodes provided for insight generation.");
}
