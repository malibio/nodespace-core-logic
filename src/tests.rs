#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        constants, CoreLogic, DateNavigation, HierarchyComputation, NodeSpaceConfig,
        NodeSpaceService, OfflineFallback, ServiceState,
    };
    use async_trait::async_trait;
    use nodespace_core_types::{Node, NodeId, NodeSpaceError, NodeSpaceResult, ProcessingError};
    use nodespace_data_store::{
        DataStore, HybridSearchConfig as DataStoreHybridSearchConfig,
        ImageNode as DataStoreImageNode, MultiLevelEmbeddings as DataStoreMultiLevelEmbeddings,
        NodeType as DataStoreNodeType, QueryEmbeddings, RelevanceFactors,
        SearchResult as DataStoreSearchResult,
    };
    use nodespace_nlp_engine::{ContextStrategy, MultiLevelEmbeddings, NodeContext};
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Mock types for testing
    #[derive(Debug, Clone)]
    pub struct MockImageNode {
        pub id: String,
        pub image_data: Vec<u8>,
        pub embedding: Vec<f32>,
    }

    #[derive(Debug, Clone)]
    pub enum MockNodeType {
        Text,
        Image,
        Date,
        Task,
    }

    #[derive(Debug, Clone)]
    pub struct MockHybridSearchConfig {
        pub max_results: usize,
        pub min_similarity_threshold: f64,
    }

    #[derive(Debug, Clone)]
    pub struct MockDataStoreSearchResult {
        pub node: Node,
        pub score: f32,
    }

    /// Mock DataStore implementation for testing
    #[derive(Default)]
    pub struct MockDataStore {
        pub nodes: Arc<Mutex<HashMap<String, Node>>>,
        pub query_responses: Arc<Mutex<HashMap<String, Vec<Node>>>>,
        pub failure_mode: Arc<Mutex<Option<String>>>,
    }

    impl MockDataStore {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn with_failure_mode(failure_mode: &str) -> Self {
            let store = Self::new();
            *store.failure_mode.lock().unwrap() = Some(failure_mode.to_string());
            store
        }

        pub fn add_node(&self, node: Node) {
            self.nodes.lock().unwrap().insert(node.id.to_string(), node);
        }

        pub fn set_query_response(&self, query: &str, nodes: Vec<Node>) {
            self.query_responses
                .lock()
                .unwrap()
                .insert(query.to_string(), nodes);
        }
    }

    #[async_trait]
    impl nodespace_data_store::DataStore for MockDataStore {
        async fn store_node(&self, node: Node) -> NodeSpaceResult<NodeId> {
            if let Some(ref failure) = *self.failure_mode.lock().unwrap() {
                if failure == "store_node" {
                    return Err(NodeSpaceError::database_error(
                        "Mock store failure",
                    ));
                }
            }
            let node_id = node.id.clone();
            self.add_node(node);
            Ok(node_id)
        }

        async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
            if let Some(ref failure) = *self.failure_mode.lock().unwrap() {
                if failure == "get_node" {
                    return Err(NodeSpaceError::database_error(
                        "Mock get failure",
                    ));
                }
            }
            Ok(self.nodes.lock().unwrap().get(&id.to_string()).cloned())
        }

        async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
            if let Some(ref failure) = *self.failure_mode.lock().unwrap() {
                if failure == "delete_node" {
                    return Err(NodeSpaceError::database_error(
                        "Mock delete failure",
                    ));
                }
            }
            self.nodes.lock().unwrap().remove(&id.to_string());
            Ok(())
        }

        async fn query_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>> {
            if let Some(ref failure) = *self.failure_mode.lock().unwrap() {
                if failure == "query_nodes" {
                    return Err(NodeSpaceError::database_error(
                        "Mock query failure",
                    ));
                }
            }

            if let Some(response) = self.query_responses.lock().unwrap().get(query) {
                return Ok(response.clone());
            }

            // Default behavior: search by content
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

        async fn create_relationship(
            &self,
            _from: &NodeId,
            _to: &NodeId,
            _rel_type: &str,
        ) -> NodeSpaceResult<()> {
            Ok(())
        }

        async fn search_similar_nodes(
            &self,
            _embedding: Vec<f32>,
            limit: usize,
        ) -> NodeSpaceResult<Vec<(Node, f32)>> {
            let nodes: Vec<Node> = self.nodes.lock().unwrap().values().cloned().collect();
            let results = nodes
                .into_iter()
                .take(limit)
                .map(|node| (node, 0.8))
                .collect();
            Ok(results)
        }

        async fn update_node_embedding(
            &self,
            _id: &NodeId,
            _embedding: Vec<f32>,
        ) -> NodeSpaceResult<()> {
            Ok(())
        }

        async fn semantic_search_with_embedding(
            &self,
            _embedding: Vec<f32>,
            limit: usize,
        ) -> NodeSpaceResult<Vec<(Node, f32)>> {
            let nodes: Vec<Node> = self.nodes.lock().unwrap().values().cloned().collect();
            let results = nodes
                .into_iter()
                .take(limit)
                .map(|node| (node, 0.8))
                .collect();
            Ok(results)
        }

        async fn create_image_node(
            &self,
            _image_node: DataStoreImageNode,
        ) -> NodeSpaceResult<String> {
            Ok("mock_image_id".to_string())
        }

        async fn get_image_node(&self, _id: &str) -> NodeSpaceResult<Option<DataStoreImageNode>> {
            Ok(None)
        }

        async fn search_multimodal(
            &self,
            _query_embedding: Vec<f32>,
            _types: Vec<DataStoreNodeType>,
        ) -> NodeSpaceResult<Vec<Node>> {
            Ok(vec![])
        }

        async fn hybrid_multimodal_search(
            &self,
            _query_embedding: Vec<f32>,
            _config: &DataStoreHybridSearchConfig,
        ) -> NodeSpaceResult<Vec<DataStoreSearchResult>> {
            Ok(vec![])
        }

        async fn update_node(&self, node: Node) -> NodeSpaceResult<()> {
            let node_id = node.id.clone();
            self.nodes.lock().unwrap().insert(node_id.to_string(), node);
            Ok(())
        }

        async fn update_node_with_embedding(
            &self,
            node: Node,
            _embedding: Vec<f32>,
        ) -> NodeSpaceResult<()> {
            let node_id = node.id.clone();
            self.nodes.lock().unwrap().insert(node_id.to_string(), node);
            Ok(())
        }

        async fn store_node_with_embedding(
            &self,
            node: Node,
            _embedding: Vec<f32>,
        ) -> NodeSpaceResult<NodeId> {
            let node_id = node.id.clone();
            self.add_node(node);
            Ok(node_id)
        }

        async fn store_node_with_multi_embeddings(
            &self,
            node: Node,
            _embeddings: DataStoreMultiLevelEmbeddings,
        ) -> NodeSpaceResult<NodeId> {
            let node_id = node.id.clone();
            self.add_node(node);
            Ok(node_id)
        }

        async fn update_node_embeddings(
            &self,
            _id: &NodeId,
            _embeddings: DataStoreMultiLevelEmbeddings,
        ) -> NodeSpaceResult<()> {
            Ok(())
        }

        async fn get_node_embeddings(
            &self,
            _id: &NodeId,
        ) -> NodeSpaceResult<Option<DataStoreMultiLevelEmbeddings>> {
            Ok(None)
        }

        async fn search_by_individual_embedding(
            &self,
            _embedding: Vec<f32>,
            limit: usize,
        ) -> NodeSpaceResult<Vec<(Node, f32)>> {
            let nodes: Vec<Node> = self.nodes.lock().unwrap().values().cloned().collect();
            let results = nodes
                .into_iter()
                .take(limit)
                .map(|node| (node, 0.8))
                .collect();
            Ok(results)
        }

        async fn search_by_contextual_embedding(
            &self,
            _embedding: Vec<f32>,
            limit: usize,
        ) -> NodeSpaceResult<Vec<(Node, f32)>> {
            let nodes: Vec<Node> = self.nodes.lock().unwrap().values().cloned().collect();
            let results = nodes
                .into_iter()
                .take(limit)
                .map(|node| (node, 0.8))
                .collect();
            Ok(results)
        }

        async fn search_by_hierarchical_embedding(
            &self,
            _embedding: Vec<f32>,
            limit: usize,
        ) -> NodeSpaceResult<Vec<(Node, f32)>> {
            let nodes: Vec<Node> = self.nodes.lock().unwrap().values().cloned().collect();
            let results = nodes
                .into_iter()
                .take(limit)
                .map(|node| (node, 0.8))
                .collect();
            Ok(results)
        }

        async fn hybrid_semantic_search(
            &self,
            _query_embeddings: QueryEmbeddings,
            _config: DataStoreHybridSearchConfig,
        ) -> NodeSpaceResult<Vec<DataStoreSearchResult>> {
            Ok(vec![])
        }
    }

    /// Mock NLP Engine for testing
    #[derive(Default)]
    pub struct MockNLPEngine {
        pub embedding_responses: Arc<Mutex<HashMap<String, Vec<f32>>>>,
        pub text_responses: Arc<Mutex<HashMap<String, String>>>,
        pub failure_mode: Arc<Mutex<Option<String>>>,
    }

    impl MockNLPEngine {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn with_failure_mode(failure_mode: &str) -> Self {
            let engine = Self::new();
            *engine.failure_mode.lock().unwrap() = Some(failure_mode.to_string());
            engine
        }

        pub fn set_embedding_response(&self, text: &str, embedding: Vec<f32>) {
            self.embedding_responses
                .lock()
                .unwrap()
                .insert(text.to_string(), embedding);
        }

        pub fn set_text_response(&self, prompt: &str, response: &str) {
            self.text_responses
                .lock()
                .unwrap()
                .insert(prompt.to_string(), response.to_string());
        }
    }

    #[async_trait]
    impl nodespace_nlp_engine::NLPEngine for MockNLPEngine {
        async fn generate_embedding(&self, text: &str) -> NodeSpaceResult<Vec<f32>> {
            if let Some(ref failure) = *self.failure_mode.lock().unwrap() {
                if failure == "generate_embedding" {
                    return Err(NodeSpaceError::Processing(ProcessingError::model_error(
                        "test-nlp-engine",
                        "embedding",
                        "Mock embedding failure"
                    )));
                }
            }

            if let Some(response) = self.embedding_responses.lock().unwrap().get(text) {
                return Ok(response.clone());
            }

            // Default: return a mock embedding
            Ok(vec![0.1; constants::DEFAULT_EMBEDDING_DIMENSION])
        }

        async fn generate_text(&self, prompt: &str) -> NodeSpaceResult<String> {
            if let Some(ref failure) = *self.failure_mode.lock().unwrap() {
                if failure == "generate_text" {
                    return Err(NodeSpaceError::Processing(ProcessingError::model_error(
                        "test-nlp-engine",
                        "text-generation",
                        "Mock text generation failure"
                    )));
                }
            }

            if let Some(response) = self.text_responses.lock().unwrap().get(prompt) {
                return Ok(response.clone());
            }

            // Default: return a mock response
            Ok("Mock response".to_string())
        }

        async fn batch_embeddings(&self, texts: &[String]) -> NodeSpaceResult<Vec<Vec<f32>>> {
            let mut results = Vec::new();
            for _text in texts {
                results.push(vec![0.1; constants::DEFAULT_EMBEDDING_DIMENSION]);
            }
            Ok(results)
        }

        async fn generate_text_enhanced(
            &self,
            _request: nodespace_nlp_engine::TextGenerationRequest,
        ) -> NodeSpaceResult<nodespace_nlp_engine::EnhancedTextGenerationResponse> {
            Ok(nodespace_nlp_engine::EnhancedTextGenerationResponse {
                text: "Mock enhanced response".to_string(),
                tokens_used: 50,
                generation_metrics: nodespace_nlp_engine::GenerationMetrics {
                    generation_time_ms: 100,
                    context_tokens: 20,
                    response_tokens: 50,
                    temperature_used: 0.7,
                },
                context_utilization: nodespace_nlp_engine::ContextUtilization {
                    context_referenced: true,
                    sources_mentioned: vec!["mock_source".to_string()],
                    relevance_score: 0.8,
                },
            })
        }

        async fn generate_surrealql(
            &self,
            _natural_query: &str,
            _context: &str,
        ) -> NodeSpaceResult<String> {
            Ok("SELECT * FROM mock;".to_string())
        }

        fn embedding_dimensions(&self) -> usize {
            constants::DEFAULT_EMBEDDING_DIMENSION
        }

        // Missing multi-level embedding methods
        async fn generate_contextual_embedding(
            &self,
            _node: &nodespace_core_types::Node,
            _context: &NodeContext,
        ) -> NodeSpaceResult<Vec<f32>> {
            if let Some(ref failure) = *self.failure_mode.lock().unwrap() {
                if failure == "generate_contextual_embedding" {
                    return Err(NodeSpaceError::Processing(ProcessingError::model_error(
                        "test-nlp-engine",
                        "contextual-embedding",
                        "Mock contextual embedding failure"
                    )));
                }
            }
            // Mock contextual embedding - slightly different from individual
            Ok(vec![0.2; constants::DEFAULT_EMBEDDING_DIMENSION])
        }

        async fn generate_hierarchical_embedding(
            &self,
            _node: &nodespace_core_types::Node,
            _path: &[nodespace_core_types::Node],
        ) -> NodeSpaceResult<Vec<f32>> {
            if let Some(ref failure) = *self.failure_mode.lock().unwrap() {
                if failure == "generate_hierarchical_embedding" {
                    return Err(NodeSpaceError::Processing(ProcessingError::model_error(
                        "test-nlp-engine",
                        "hierarchical-embedding",
                        "Mock hierarchical embedding failure"
                    )));
                }
            }
            // Mock hierarchical embedding - different from individual and contextual
            Ok(vec![0.3; constants::DEFAULT_EMBEDDING_DIMENSION])
        }

        async fn generate_all_embeddings(
            &self,
            node: &nodespace_core_types::Node,
            context: &NodeContext,
            path: &[nodespace_core_types::Node],
        ) -> NodeSpaceResult<nodespace_nlp_engine::MultiLevelEmbeddings> {
            if let Some(ref failure) = *self.failure_mode.lock().unwrap() {
                if failure == "generate_all_embeddings" {
                    return Err(NodeSpaceError::Processing(ProcessingError::model_error(
                        "test-nlp-engine",
                        "all-embeddings",
                        "Mock all embeddings failure"
                    )));
                }
            }

            // Generate each type of embedding using the respective methods
            let individual = self.generate_embedding(&node.content.to_string()).await?;
            let contextual = self.generate_contextual_embedding(node, context).await?;
            let hierarchical = self.generate_hierarchical_embedding(node, path).await?;

            Ok(nodespace_nlp_engine::MultiLevelEmbeddings {
                individual,
                contextual: Some(contextual),
                hierarchical: Some(hierarchical),
                context_strategy: ContextStrategy::RuleBased,
                generated_at: chrono::Utc::now(),
                generation_metrics: nodespace_nlp_engine::EmbeddingGenerationMetrics {
                    individual_time_ms: 10,
                    contextual_time_ms: Some(15),
                    hierarchical_time_ms: Some(20),
                    total_time_ms: 45,
                    context_length: Some(100),
                    path_depth: Some(path.len()),
                    cache_hits: 0,
                    cache_misses: 1,
                },
            })
        }
    }

    /// Test helpers
    fn create_test_node(id: &str, content: &str) -> Node {
        let now = chrono::Utc::now().to_rfc3339();
        Node {
            id: NodeId::from_string(id.to_string()),
            content: json!(content),
            metadata: Some(json!({"test": true})),
            created_at: now.clone(),
            updated_at: now,
            parent_id: None,
            next_sibling: None,
            previous_sibling: None,
        }
    }

    fn create_test_service() -> NodeSpaceService<MockDataStore, MockNLPEngine> {
        NodeSpaceService::new(MockDataStore::new(), MockNLPEngine::new())
    }

    #[tokio::test]
    async fn test_service_initialization() {
        let service = create_test_service();

        // Service should start uninitialized
        assert_eq!(service.get_state().await, ServiceState::Uninitialized);
        assert!(!service.is_ready().await);

        // Initialize service
        let result = service.initialize().await;
        assert!(result.is_ok());
        assert_eq!(service.get_state().await, ServiceState::Ready);
        assert!(service.is_ready().await);
    }

    #[tokio::test]
    async fn test_service_initialization_failure() {
        let data_store = MockDataStore::new();
        let nlp_engine = MockNLPEngine::with_failure_mode("generate_embedding");
        let service = NodeSpaceService::new(data_store, nlp_engine);

        let result = service.initialize().await;
        assert!(result.is_err());

        match service.get_state().await {
            ServiceState::Failed(_) => {}
            _ => panic!("Expected Failed state"),
        }
    }

    #[tokio::test]
    async fn test_create_knowledge_node_success() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        let content = "Test knowledge content";
        let metadata = json!({"category": "test"});

        let result = service.create_knowledge_node(content, metadata).await;
        assert!(result.is_ok());

        let node_id = result.unwrap();
        let stored_node = service.data_store.get_node(&node_id).await.unwrap();
        assert!(stored_node.is_some());

        let node = stored_node.unwrap();
        assert_eq!(node.content.as_str().unwrap(), content);
    }

    #[tokio::test]
    async fn test_create_knowledge_node_not_ready() {
        let service = create_test_service();
        // Don't initialize service

        let result = service.create_knowledge_node("test", json!({})).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            NodeSpaceError::InternalError { message: _, service: _ } => {}
            _ => panic!("Expected InternalError for service not ready"),
        }
    }

    #[tokio::test]
    async fn test_semantic_search() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        // Add test nodes
        let node1 = create_test_node("1", "rust programming language");
        let node2 = create_test_node("2", "python data science");
        let node3 = create_test_node("3", "rust systems programming");

        service.data_store.add_node(node1);
        service.data_store.add_node(node2);
        service.data_store.add_node(node3);

        let results = service.semantic_search("rust", 10).await.unwrap();
        assert_eq!(results.len(), 2); // Should find 2 rust-related nodes

        // Check scoring
        assert!(results[0].score >= results[1].score); // Results should be sorted by score
        assert!(results[0].score >= constants::MIN_SEARCH_SCORE);
    }

    #[tokio::test]
    async fn test_process_query_with_context() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        // Add test node
        let node = create_test_node("1", "Rust is a systems programming language");
        service.data_store.add_node(node);

        // Set up mock NLP response
        service.nlp_engine.set_text_response(
            &format!("Based on the following context, answer the question: What is Rust?\n\nContext:\nRust is a systems programming language"),
            "Rust is a systems programming language focused on safety and performance."
        );

        let response = service.process_query("What is Rust?").await.unwrap();

        assert!(!response.answer.is_empty());
        assert!(!response.sources.is_empty());
        assert!(response.confidence >= constants::BASE_CONFIDENCE_WITH_CONTEXT);
        assert!(!response.related_queries.is_empty());
    }

    #[tokio::test]
    async fn test_process_query_no_context() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        // No nodes added, so no context will be found
        let response = service
            .process_query("What is quantum computing?")
            .await
            .unwrap();

        assert!(!response.answer.is_empty());
        assert!(response.sources.is_empty());
        assert_eq!(response.confidence, constants::BASE_CONFIDENCE_NO_CONTEXT);
    }

    #[tokio::test]
    async fn test_update_node() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        // Create and store a node
        let original_content = "Original content";
        let node_id = service
            .create_knowledge_node(original_content, json!({}))
            .await
            .unwrap();

        // Update the node
        let new_content = "Updated content";
        let result = service.update_node(&node_id, new_content).await;
        assert!(result.is_ok());

        // Verify the update
        let updated_node = service
            .data_store
            .get_node(&node_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_node.content.as_str().unwrap(), new_content);
        assert_ne!(updated_node.created_at, updated_node.updated_at);
    }

    #[tokio::test]
    async fn test_update_nonexistent_node() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        let fake_id = NodeId::from_string("nonexistent".to_string());
        let result = service.update_node(&fake_id, "new content").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            NodeSpaceError::Database(_) => {} // NotFound is now a Database error
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_related_nodes() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        let target_id = NodeId::from_string("target".to_string());

        // Create nodes with relationships in metadata
        let mut related_node = create_test_node("related", "Related content");
        related_node.metadata = Some(json!({
            "mentions": [target_id.to_string()]
        }));

        let unrelated_node = create_test_node("unrelated", "Unrelated content");

        service.data_store.add_node(related_node);
        service.data_store.add_node(unrelated_node);

        let related_ids = service.get_related_nodes(&target_id, vec![]).await.unwrap();
        assert_eq!(related_ids.len(), 1);
        assert_eq!(related_ids[0].to_string(), "related");
    }

    #[tokio::test]
    async fn test_generate_insights() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        // Create test nodes
        let node1 = create_test_node("1", "AI is transforming healthcare");
        let node2 = create_test_node("2", "Machine learning improves diagnosis");
        let node3 = create_test_node("3", "Deep learning analyzes medical images");

        service.data_store.add_node(node1);
        service.data_store.add_node(node2);
        service.data_store.add_node(node3);

        // Set up mock response
        service.nlp_engine.set_text_response(
            &format!("Analyze the following content and provide key insights, patterns, and connections:\n\nAI is transforming healthcare\n\n---\n\nMachine learning improves diagnosis\n\n---\n\nDeep learning analyzes medical images\n\nProvide a concise summary with 3-5 key insights:"),
            "Key insights: 1) AI is revolutionizing healthcare, 2) Multiple AI technologies are being applied, 3) Focus on diagnostic improvements"
        );

        let node_ids = vec![
            NodeId::from_string("1".to_string()),
            NodeId::from_string("2".to_string()),
            NodeId::from_string("3".to_string()),
        ];

        let insights = service.generate_insights(node_ids).await.unwrap();
        assert!(!insights.is_empty());
        assert!(insights.contains("Key insights"));
    }

    #[tokio::test]
    async fn test_generate_insights_empty_input() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        let insights = service.generate_insights(vec![]).await.unwrap();
        assert_eq!(insights, "No nodes provided for insight generation.");
    }

    #[tokio::test]
    async fn test_node_structure_operations() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        // Create test nodes
        let parent_id = NodeId::from_string("parent".to_string());
        let child_id = NodeId::from_string("child".to_string());

        let parent_node = create_test_node("parent", "Parent content");
        let child_node = create_test_node("child", "Child content");

        service.data_store.add_node(parent_node);
        service.data_store.add_node(child_node);

        // Test indent operation (make child_node a child of parent_node)
        let result = service
            .update_node_structure(&child_id, "indent", Some(&parent_id), None)
            .await;
        assert!(result.is_ok());

        // Verify the relationship was set
        let updated_child = service
            .data_store
            .get_node(&child_id)
            .await
            .unwrap()
            .unwrap();
        assert!(updated_child.metadata.is_some());
        let metadata = updated_child.metadata.unwrap();
        assert_eq!(
            metadata["parent_id"].as_str().unwrap(),
            parent_id.to_string()
        );
    }

    #[tokio::test]
    async fn test_set_node_parent() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        let parent_id = NodeId::from_string("parent".to_string());
        let child_id = NodeId::from_string("child".to_string());

        let child_node = create_test_node("child", "Child content");
        service.data_store.add_node(child_node);

        // Set parent
        let result = service.set_node_parent(&child_id, Some(&parent_id)).await;
        assert!(result.is_ok());

        // Verify parent was set
        let updated_child = service
            .data_store
            .get_node(&child_id)
            .await
            .unwrap()
            .unwrap();
        let metadata = updated_child.metadata.unwrap();
        assert_eq!(
            metadata["parent_id"].as_str().unwrap(),
            parent_id.to_string()
        );

        // Remove parent
        let result = service.set_node_parent(&child_id, None).await;
        assert!(result.is_ok());

        // Verify parent was removed
        let updated_child = service
            .data_store
            .get_node(&child_id)
            .await
            .unwrap()
            .unwrap();
        let metadata = updated_child.metadata.unwrap();
        assert!(metadata.get("parent_id").is_none());
    }

    #[tokio::test]
    async fn test_update_sibling_order() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        let node1_id = NodeId::from_string("node1".to_string());
        let node2_id = NodeId::from_string("node2".to_string());
        let node3_id = NodeId::from_string("node3".to_string());

        let node1 = create_test_node("node1", "First node");
        let node2 = create_test_node("node2", "Second node");
        let node3 = create_test_node("node3", "Third node");

        service.data_store.add_node(node1);
        service.data_store.add_node(node2);
        service.data_store.add_node(node3);

        // Set node2 between node1 and node3
        let result = service
            .update_sibling_order(&node2_id, Some(&node1_id), Some(&node3_id))
            .await;
        assert!(result.is_ok());

        // Verify the sibling relationships
        let updated_node2 = service
            .data_store
            .get_node(&node2_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_node2.previous_sibling.unwrap(), node1_id);
        assert_eq!(updated_node2.next_sibling.unwrap(), node3_id);

        let updated_node1 = service
            .data_store
            .get_node(&node1_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_node1.next_sibling.unwrap(), node2_id);

        let updated_node3 = service
            .data_store
            .get_node(&node3_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_node3.previous_sibling.unwrap(), node2_id);
    }

    #[tokio::test]
    async fn test_date_navigation() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        let today = chrono::Utc::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        // Create nodes with today's date
        let mut node1 = create_test_node("1", "Today's content 1");
        let mut node2 = create_test_node("2", "Today's content 2");
        let mut node3 = create_test_node("3", "Child node");

        // Set created_at to today
        node1.created_at = format!("{}T10:00:00Z", today_str);
        node2.created_at = format!("{}T11:00:00Z", today_str);
        node3.created_at = format!("{}T12:00:00Z", today_str);

        // Make node3 a child (should be filtered out)
        node3.metadata = Some(json!({"parent_id": "1"}));

        service.data_store.add_node(node1);
        service.data_store.add_node(node2);
        service.data_store.add_node(node3);

        let today_nodes = service.get_nodes_for_date(today).await.unwrap();

        // Should return only top-level nodes (node1 and node2, not node3)
        assert_eq!(today_nodes.len(), 2);

        let node_ids: Vec<String> = today_nodes.iter().map(|n| n.id.to_string()).collect();
        assert!(node_ids.contains(&"1".to_string()));
        assert!(node_ids.contains(&"2".to_string()));
        assert!(!node_ids.contains(&"3".to_string())); // Child node filtered out
    }

    #[tokio::test]
    async fn test_offline_fallback_behavior() {
        let data_store = MockDataStore::new();
        let nlp_engine = MockNLPEngine::with_failure_mode("generate_text");

        // Create service with offline fallback enabled
        let mut config = NodeSpaceConfig::default();
        config.offline_config.offline_fallback = OfflineFallback::Cache;

        let service = NodeSpaceService::with_config(data_store, nlp_engine, config);
        service.initialize().await.unwrap(); // Should succeed despite NLP failure

        // Process query should work with fallback
        let response = service.process_query("test query").await.unwrap();
        assert!(!response.answer.is_empty());
        assert!(response.confidence < constants::BASE_CONFIDENCE_WITH_CONTEXT);
    }

    #[tokio::test]
    async fn test_performance_config_limits() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        // Add many test nodes
        for i in 0..100 {
            let node = create_test_node(&i.to_string(), &format!("Content {}", i));
            service.data_store.add_node(node);
        }

        // Test that max_batch_size is respected
        let results = service.semantic_search("Content", 50).await.unwrap();
        assert!(results.len() <= constants::DEFAULT_MAX_BATCH_SIZE);
    }

    #[tokio::test]
    async fn test_constants_usage() {
        // Test that our constants are properly defined and reasonable
        assert!(constants::DEFAULT_MAX_BATCH_SIZE > 0);
        assert!(constants::DEFAULT_CONTEXT_WINDOW > 0);
        assert!(constants::DEFAULT_TEMPERATURE >= 0.0 && constants::DEFAULT_TEMPERATURE <= 1.0);
        assert!(constants::MIN_SEARCH_SCORE >= 0.0 && constants::MIN_SEARCH_SCORE <= 1.0);
        assert!(constants::BASE_CONFIDENCE_WITH_CONTEXT > constants::BASE_CONFIDENCE_NO_CONTEXT);
        assert!(constants::FALLBACK_CONFIDENCE_FACTOR < 1.0);
    }

    // === HIERARCHY COMPUTATION TESTS ===

    fn create_test_node_with_parent(id: &str, content: &str, parent_id: Option<NodeId>) -> Node {
        let now = chrono::Utc::now().to_rfc3339();
        Node {
            id: NodeId::from_string(id.to_string()),
            content: json!(content),
            metadata: Some(json!({"test": true})),
            created_at: now.clone(),
            updated_at: now,
            parent_id,
            next_sibling: None,
            previous_sibling: None,
        }
    }

    async fn setup_hierarchy_test_data(service: &NodeSpaceService<MockDataStore, MockNLPEngine>) {
        // Create hierarchy:
        //   root
        //   ├── child1
        //   │   ├── grandchild1
        //   │   └── grandchild2
        //   ├── child2
        //   └── child3
        //       └── grandchild3

        let root = create_test_node_with_parent("root", "Root content", None);
        let child1 = create_test_node_with_parent(
            "child1",
            "Child 1 content",
            Some(NodeId::from_string("root".to_string())),
        );
        let child2 = create_test_node_with_parent(
            "child2",
            "Child 2 content",
            Some(NodeId::from_string("root".to_string())),
        );
        let child3 = create_test_node_with_parent(
            "child3",
            "Child 3 content",
            Some(NodeId::from_string("root".to_string())),
        );
        let grandchild1 = create_test_node_with_parent(
            "grandchild1",
            "Grandchild 1 content",
            Some(NodeId::from_string("child1".to_string())),
        );
        let grandchild2 = create_test_node_with_parent(
            "grandchild2",
            "Grandchild 2 content",
            Some(NodeId::from_string("child1".to_string())),
        );
        let grandchild3 = create_test_node_with_parent(
            "grandchild3",
            "Grandchild 3 content",
            Some(NodeId::from_string("child3".to_string())),
        );

        service.data_store.add_node(root);
        service.data_store.add_node(child1);
        service.data_store.add_node(child2);
        service.data_store.add_node(child3);
        service.data_store.add_node(grandchild1);
        service.data_store.add_node(grandchild2);
        service.data_store.add_node(grandchild3);
    }

    #[tokio::test]
    async fn test_get_node_depth() {
        let service = create_test_service();
        service.initialize().await.unwrap();
        setup_hierarchy_test_data(&service).await;

        // Test depth calculation
        let root_depth = service
            .get_node_depth(&NodeId::from_string("root".to_string()))
            .await
            .unwrap();
        assert_eq!(root_depth, 0, "Root node should have depth 0");

        let child_depth = service
            .get_node_depth(&NodeId::from_string("child1".to_string()))
            .await
            .unwrap();
        assert_eq!(child_depth, 1, "Child node should have depth 1");

        let grandchild_depth = service
            .get_node_depth(&NodeId::from_string("grandchild1".to_string()))
            .await
            .unwrap();
        assert_eq!(grandchild_depth, 2, "Grandchild node should have depth 2");
    }

    #[tokio::test]
    async fn test_get_node_depth_nonexistent() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        let result = service
            .get_node_depth(&NodeId::from_string("nonexistent".to_string()))
            .await;
        assert!(result.is_err(), "Should return error for nonexistent node");
    }

    #[tokio::test]
    async fn test_get_children() {
        let service = create_test_service();
        service.initialize().await.unwrap();
        setup_hierarchy_test_data(&service).await;

        // Test root children
        let root_children = service
            .get_children(&NodeId::from_string("root".to_string()))
            .await
            .unwrap();
        assert_eq!(root_children.len(), 3, "Root should have 3 children");

        let child_ids: Vec<String> = root_children.iter().map(|n| n.id.to_string()).collect();
        assert!(child_ids.contains(&"child1".to_string()));
        assert!(child_ids.contains(&"child2".to_string()));
        assert!(child_ids.contains(&"child3".to_string()));

        // Test child1 children
        let child1_children = service
            .get_children(&NodeId::from_string("child1".to_string()))
            .await
            .unwrap();
        assert_eq!(child1_children.len(), 2, "Child1 should have 2 children");

        let grandchild_ids: Vec<String> =
            child1_children.iter().map(|n| n.id.to_string()).collect();
        assert!(grandchild_ids.contains(&"grandchild1".to_string()));
        assert!(grandchild_ids.contains(&"grandchild2".to_string()));

        // Test leaf node (no children)
        let leaf_children = service
            .get_children(&NodeId::from_string("grandchild1".to_string()))
            .await
            .unwrap();
        assert_eq!(leaf_children.len(), 0, "Leaf node should have no children");
    }

    #[tokio::test]
    async fn test_get_ancestors() {
        let service = create_test_service();
        service.initialize().await.unwrap();
        setup_hierarchy_test_data(&service).await;

        // Test grandchild ancestors
        let ancestors = service
            .get_ancestors(&NodeId::from_string("grandchild1".to_string()))
            .await
            .unwrap();
        assert_eq!(ancestors.len(), 2, "Grandchild should have 2 ancestors");

        // Ancestors should be ordered from immediate parent to root
        assert_eq!(
            ancestors[0].id.to_string(),
            "child1",
            "First ancestor should be immediate parent"
        );
        assert_eq!(
            ancestors[1].id.to_string(),
            "root",
            "Second ancestor should be root"
        );

        // Test child ancestors
        let child_ancestors = service
            .get_ancestors(&NodeId::from_string("child1".to_string()))
            .await
            .unwrap();
        assert_eq!(child_ancestors.len(), 1, "Child should have 1 ancestor");
        assert_eq!(
            child_ancestors[0].id.to_string(),
            "root",
            "Child's ancestor should be root"
        );

        // Test root ancestors (should be empty)
        let root_ancestors = service
            .get_ancestors(&NodeId::from_string("root".to_string()))
            .await
            .unwrap();
        assert_eq!(root_ancestors.len(), 0, "Root should have no ancestors");
    }

    #[tokio::test]
    async fn test_get_siblings() {
        let service = create_test_service();
        service.initialize().await.unwrap();
        setup_hierarchy_test_data(&service).await;

        // Test siblings of child1
        let child1_siblings = service
            .get_siblings(&NodeId::from_string("child1".to_string()))
            .await
            .unwrap();
        assert_eq!(child1_siblings.len(), 2, "Child1 should have 2 siblings");

        let sibling_ids: Vec<String> = child1_siblings.iter().map(|n| n.id.to_string()).collect();
        assert!(sibling_ids.contains(&"child2".to_string()));
        assert!(sibling_ids.contains(&"child3".to_string()));
        assert!(
            !sibling_ids.contains(&"child1".to_string()),
            "Node should not be in its own siblings list"
        );

        // Test siblings of grandchild1
        let grandchild1_siblings = service
            .get_siblings(&NodeId::from_string("grandchild1".to_string()))
            .await
            .unwrap();
        assert_eq!(
            grandchild1_siblings.len(),
            1,
            "Grandchild1 should have 1 sibling"
        );
        assert_eq!(grandchild1_siblings[0].id.to_string(), "grandchild2");

        // Test root siblings (should be empty - root has no parent)
        let root_siblings = service
            .get_siblings(&NodeId::from_string("root".to_string()))
            .await
            .unwrap();
        assert_eq!(root_siblings.len(), 0, "Root should have no siblings");
    }

    #[tokio::test]
    async fn test_move_node_success() {
        let service = create_test_service();
        service.initialize().await.unwrap();
        setup_hierarchy_test_data(&service).await;

        // Move grandchild1 from child1 to child2
        let result = service
            .move_node(
                &NodeId::from_string("grandchild1".to_string()),
                &NodeId::from_string("child2".to_string()),
            )
            .await;
        assert!(result.is_ok(), "Move operation should succeed");

        // Verify the move
        let moved_node = service
            .data_store
            .get_node(&NodeId::from_string("grandchild1".to_string()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            moved_node.parent_id.unwrap().to_string(),
            "child2",
            "Node should have new parent"
        );

        // Verify old parent no longer has the child
        let old_parent_children = service
            .get_children(&NodeId::from_string("child1".to_string()))
            .await
            .unwrap();
        let old_child_ids: Vec<String> = old_parent_children
            .iter()
            .map(|n| n.id.to_string())
            .collect();
        assert!(
            !old_child_ids.contains(&"grandchild1".to_string()),
            "Old parent should not have moved child"
        );

        // Verify new parent has the child
        let new_parent_children = service
            .get_children(&NodeId::from_string("child2".to_string()))
            .await
            .unwrap();
        let new_child_ids: Vec<String> = new_parent_children
            .iter()
            .map(|n| n.id.to_string())
            .collect();
        assert!(
            new_child_ids.contains(&"grandchild1".to_string()),
            "New parent should have moved child"
        );
    }

    #[tokio::test]
    async fn test_move_node_cycle_detection() {
        let service = create_test_service();
        service.initialize().await.unwrap();
        setup_hierarchy_test_data(&service).await;

        // Try to move root under grandchild1 (would create cycle)
        let result = service
            .move_node(
                &NodeId::from_string("root".to_string()),
                &NodeId::from_string("grandchild1".to_string()),
            )
            .await;
        assert!(result.is_err(), "Move operation should fail due to cycle");

        match result.unwrap_err() {
            NodeSpaceError::Validation(_) => {
                // ValidationError is now a Validation error - cycle detection verified
            }
            _ => panic!("Expected ValidationError for cycle detection"),
        }
    }

    #[tokio::test]
    async fn test_move_node_to_descendant() {
        let service = create_test_service();
        service.initialize().await.unwrap();
        setup_hierarchy_test_data(&service).await;

        // Try to move child1 under its own descendant grandchild1
        let result = service
            .move_node(
                &NodeId::from_string("child1".to_string()),
                &NodeId::from_string("grandchild1".to_string()),
            )
            .await;
        assert!(
            result.is_err(),
            "Move operation should fail when moving to descendant"
        );
    }

    #[tokio::test]
    async fn test_move_subtree() {
        let service = create_test_service();
        service.initialize().await.unwrap();
        setup_hierarchy_test_data(&service).await;

        // Move entire child1 subtree under child2
        let result = service
            .move_subtree(
                &NodeId::from_string("child1".to_string()),
                &NodeId::from_string("child2".to_string()),
            )
            .await;
        assert!(result.is_ok(), "Move subtree operation should succeed");

        // Verify child1 moved under child2
        let moved_node = service
            .data_store
            .get_node(&NodeId::from_string("child1".to_string()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            moved_node.parent_id.unwrap().to_string(),
            "child2",
            "Subtree root should have new parent"
        );

        // Verify grandchildren still under child1
        let grandchild1 = service
            .data_store
            .get_node(&NodeId::from_string("grandchild1".to_string()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            grandchild1.parent_id.unwrap().to_string(),
            "child1",
            "Grandchild should still be under original parent"
        );

        // Verify hierarchy structure
        let child1_children = service
            .get_children(&NodeId::from_string("child1".to_string()))
            .await
            .unwrap();
        assert_eq!(
            child1_children.len(),
            2,
            "Child1 should still have its children after move"
        );
    }

    #[tokio::test]
    async fn test_get_tree_nodes() {
        let service = create_test_service();
        service.initialize().await.unwrap();
        setup_hierarchy_test_data(&service).await;

        // Get entire tree from root
        let tree_nodes = service
            .get_tree_nodes(&NodeId::from_string("root".to_string()))
            .await
            .unwrap();
        assert_eq!(tree_nodes.len(), 7, "Tree should contain all 7 nodes");

        // Get subtree from child1
        let subtree_nodes = service
            .get_tree_nodes(&NodeId::from_string("child1".to_string()))
            .await
            .unwrap();
        assert_eq!(
            subtree_nodes.len(),
            3,
            "Subtree should contain child1 and its 2 children"
        );

        // Get single node tree (leaf)
        let leaf_tree = service
            .get_tree_nodes(&NodeId::from_string("grandchild1".to_string()))
            .await
            .unwrap();
        assert_eq!(
            leaf_tree.len(),
            1,
            "Leaf tree should contain only the node itself"
        );
    }

    #[tokio::test]
    async fn test_is_ancestor_of() {
        let service = create_test_service();
        service.initialize().await.unwrap();
        setup_hierarchy_test_data(&service).await;

        // Root is ancestor of all nodes
        assert!(service
            .is_ancestor_of(
                &NodeId::from_string("root".to_string()),
                &NodeId::from_string("child1".to_string())
            )
            .await
            .unwrap());
        assert!(service
            .is_ancestor_of(
                &NodeId::from_string("root".to_string()),
                &NodeId::from_string("grandchild1".to_string())
            )
            .await
            .unwrap());

        // Child1 is ancestor of its grandchildren
        assert!(service
            .is_ancestor_of(
                &NodeId::from_string("child1".to_string()),
                &NodeId::from_string("grandchild1".to_string())
            )
            .await
            .unwrap());

        // Siblings are not ancestors of each other
        assert!(!service
            .is_ancestor_of(
                &NodeId::from_string("child1".to_string()),
                &NodeId::from_string("child2".to_string())
            )
            .await
            .unwrap());

        // Node is not ancestor of itself
        assert!(!service
            .is_ancestor_of(
                &NodeId::from_string("child1".to_string()),
                &NodeId::from_string("child1".to_string())
            )
            .await
            .unwrap());

        // Descendant is not ancestor of ancestor
        assert!(!service
            .is_ancestor_of(
                &NodeId::from_string("grandchild1".to_string()),
                &NodeId::from_string("child1".to_string())
            )
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_hierarchy_cache_functionality() {
        let service = create_test_service();
        service.initialize().await.unwrap();
        setup_hierarchy_test_data(&service).await;

        // First call should populate cache
        let depth1 = service
            .get_node_depth(&NodeId::from_string("grandchild1".to_string()))
            .await
            .unwrap();
        assert_eq!(depth1, 2);

        // Second call should use cache (same result)
        let depth2 = service
            .get_node_depth(&NodeId::from_string("grandchild1".to_string()))
            .await
            .unwrap();
        assert_eq!(depth2, 2);

        // Verify cache has entry
        let cache = service.hierarchy_cache.read().await;
        assert!(cache
            .depth_cache
            .contains_key(&NodeId::from_string("grandchild1".to_string())));
    }

    #[tokio::test]
    async fn test_hierarchy_cache_invalidation() {
        let service = create_test_service();
        service.initialize().await.unwrap();
        setup_hierarchy_test_data(&service).await;

        // Populate cache
        let _depth = service
            .get_node_depth(&NodeId::from_string("grandchild1".to_string()))
            .await
            .unwrap();

        // Verify cache has entry
        {
            let cache = service.hierarchy_cache.read().await;
            assert!(cache
                .depth_cache
                .contains_key(&NodeId::from_string("grandchild1".to_string())));
        }

        // Move node (should invalidate cache)
        service
            .move_node(
                &NodeId::from_string("grandchild1".to_string()),
                &NodeId::from_string("child2".to_string()),
            )
            .await
            .unwrap();

        // Cache should be cleared after move operation
        let cache = service.hierarchy_cache.read().await;
        assert!(
            cache.depth_cache.is_empty(),
            "Cache should be cleared after move operation"
        );
    }

    #[tokio::test]
    async fn test_hierarchy_error_conditions() {
        let service = create_test_service();
        service.initialize().await.unwrap();

        let nonexistent_id = NodeId::from_string("nonexistent".to_string());

        // Test operations on nonexistent nodes
        assert!(service.get_node_depth(&nonexistent_id).await.is_err());
        assert!(service.get_children(&nonexistent_id).await.is_err());
        assert!(service.get_ancestors(&nonexistent_id).await.is_err());
        assert!(service.get_siblings(&nonexistent_id).await.is_err());
        assert!(service.get_tree_nodes(&nonexistent_id).await.is_err());

        // Test move operations with nonexistent nodes
        let existing_id = NodeId::from_string("existing".to_string());
        let existing_node = create_test_node_with_parent("existing", "content", None);
        service.data_store.add_node(existing_node);

        assert!(service
            .move_node(&nonexistent_id, &existing_id)
            .await
            .is_err());
        assert!(service
            .move_node(&existing_id, &nonexistent_id)
            .await
            .is_err());
        assert!(service
            .move_subtree(&nonexistent_id, &existing_id)
            .await
            .is_err());
    }
}
