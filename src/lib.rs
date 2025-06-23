use async_trait::async_trait;
use chrono::NaiveDate;
use nodespace_core_types::{Node, NodeId, NodeSpaceError, NodeSpaceResult};
use nodespace_data_store::{DataStore, SurrealDataStore};
use nodespace_nlp_engine::{LocalNLPEngine, NLPEngine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for NodeSpace service initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSpaceConfig {
    /// Model download preferences
    pub model_config: ModelConfig,
    /// Performance tuning options
    pub performance_config: PerformanceConfig,
    /// Offline operation settings
    pub offline_config: OfflineConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Preferred embedding model (default: all-MiniLM-L6-v2)
    pub embedding_model: Option<String>,
    /// Preferred text generation model
    pub text_model: Option<String>,
    /// Model download timeout in seconds
    pub download_timeout: Option<u64>,
    /// Local model cache directory
    pub cache_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum batch size for embedding generation
    pub max_batch_size: Option<usize>,
    /// Context window size for text generation
    pub context_window: Option<usize>,
    /// Temperature for text generation (0.0-1.0)
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineConfig {
    /// Allow offline operation mode
    pub enable_offline: bool,
    /// Fallback behavior when models unavailable
    pub offline_fallback: OfflineFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OfflineFallback {
    /// Return error when models unavailable
    Error,
    /// Return stub responses for testing
    Stub,
    /// Use cached responses if available
    Cache,
}

impl Default for NodeSpaceConfig {
    fn default() -> Self {
        Self {
            model_config: ModelConfig {
                embedding_model: Some("sentence-transformers/all-MiniLM-L6-v2".to_string()),
                text_model: Some("mistralai/Mistral-7B-Instruct-v0.1".to_string()),
                download_timeout: Some(300), // 5 minutes
                cache_dir: None,             // Use system default
            },
            performance_config: PerformanceConfig {
                max_batch_size: Some(32),
                context_window: Some(4096),
                temperature: Some(0.7),
            },
            offline_config: OfflineConfig {
                enable_offline: true,
                offline_fallback: OfflineFallback::Cache,
            },
        }
    }
}

/// Service initialization state
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceState {
    Uninitialized,
    Initializing,
    Ready,
    Failed(String),
}

/// Core business logic service that orchestrates NodeSpace functionality
/// using distributed contract ownership
pub struct NodeSpaceService<D: DataStore, N: NLPEngine> {
    data_store: D,
    nlp_engine: N,
    config: NodeSpaceConfig,
    state: Arc<RwLock<ServiceState>>,
}

impl<D: DataStore, N: NLPEngine> NodeSpaceService<D, N> {
    /// Create a new NodeSpace service with injected dependencies
    pub fn new(data_store: D, nlp_engine: N) -> Self {
        Self::with_config(data_store, nlp_engine, NodeSpaceConfig::default())
    }

    /// Create a new NodeSpace service with custom configuration
    pub fn with_config(data_store: D, nlp_engine: N, config: NodeSpaceConfig) -> Self {
        Self {
            data_store,
            nlp_engine,
            config,
            state: Arc::new(RwLock::new(ServiceState::Uninitialized)),
        }
    }

    /// Initialize the service and load models
    pub async fn initialize(&self) -> NodeSpaceResult<()> {
        // Update state to initializing
        {
            let mut state = self.state.write().await;
            *state = ServiceState::Initializing;
        }

        // Initialize NLP engine with configuration
        match self.initialize_nlp_engine().await {
            Ok(_) => {
                let mut state = self.state.write().await;
                *state = ServiceState::Ready;
                Ok(())
            }
            Err(e) => {
                let mut state = self.state.write().await;
                *state = ServiceState::Failed(format!("NLP engine initialization failed: {}", e));
                Err(e)
            }
        }
    }

    /// Get current service state
    pub async fn get_state(&self) -> ServiceState {
        self.state.read().await.clone()
    }

    /// Check if service is ready for operations
    pub async fn is_ready(&self) -> bool {
        matches!(self.get_state().await, ServiceState::Ready)
    }

    /// Internal method to initialize NLP engine
    async fn initialize_nlp_engine(&self) -> NodeSpaceResult<()> {
        // For now, we'll validate the NLP engine by attempting a simple operation
        // This ensures the models are loaded and ready
        match self
            .nlp_engine
            .generate_embedding("initialization test")
            .await
        {
            Ok(_) => {
                // Test text generation as well
                match self.nlp_engine.generate_text("test initialization").await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        if self.config.offline_config.enable_offline {
                            // Log warning but continue in offline mode
                            eprintln!("Warning: Text generation initialization failed, continuing in offline mode: {}", e);
                            Ok(())
                        } else {
                            Err(e)
                        }
                    }
                }
            }
            Err(e) => {
                if self.config.offline_config.enable_offline {
                    // Log warning but continue in offline mode
                    eprintln!("Warning: Embedding generation initialization failed, continuing in offline mode: {}", e);
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Graceful shutdown of the service
    pub async fn shutdown(&self) -> NodeSpaceResult<()> {
        let mut state = self.state.write().await;
        *state = ServiceState::Uninitialized;
        Ok(())
    }
}

/// ServiceContainer for coordinating all NodeSpace services with specific database path
/// This addresses the critical MVP blocker by providing proper database coordination
pub struct ServiceContainer {
    data_store: SurrealDataStore,
    nlp_engine: LocalNLPEngine,
    config: NodeSpaceConfig,
    state: Arc<RwLock<ServiceState>>,
}

/// Initialization error for ServiceContainer
#[derive(Debug, thiserror::Error)]
pub enum InitializationError {
    #[error("Database initialization failed: {0}")]
    DatabaseError(#[from] nodespace_data_store::DataStoreError),
    #[error("NLP engine initialization failed: {0}")]
    NLPError(#[from] nodespace_nlp_engine::NLPError),
    #[error("Core service initialization failed: {0}")]
    CoreServiceError(#[from] NodeSpaceError),
}

impl ServiceContainer {
    /// Create a new ServiceContainer with the shared database path
    /// This ensures both core-logic and data-store use the same SurrealDB instance
    pub async fn new() -> Result<Self, InitializationError> {
        Self::new_with_config(NodeSpaceConfig::default()).await
    }

    /// Create ServiceContainer with custom configuration
    pub async fn new_with_config(config: NodeSpaceConfig) -> Result<Self, InitializationError> {
        // Initialize data store with the specific database path from the task requirements
        let database_path = "/Users/malibio/nodespace/nodespace-data-store/data/sample.db";
        let data_store = SurrealDataStore::new(database_path).await?;

        // Initialize NLP engine
        let nlp_engine = LocalNLPEngine::new();
        nlp_engine.initialize().await?;

        // Create service container with initialized state
        let state = Arc::new(RwLock::new(ServiceState::Ready));

        Ok(ServiceContainer {
            data_store,
            nlp_engine,
            config,
            state,
        })
    }

    /// Get a reference to the data store
    pub fn data_store(&self) -> &SurrealDataStore {
        &self.data_store
    }

    /// Get a reference to the NLP engine
    pub fn nlp_engine(&self) -> &LocalNLPEngine {
        &self.nlp_engine
    }

    /// Check if all services are ready
    pub async fn is_ready(&self) -> bool {
        matches!(self.get_state().await, ServiceState::Ready)
    }

    /// Get current service state
    pub async fn get_state(&self) -> ServiceState {
        self.state.read().await.clone()
    }

    /// Graceful shutdown of all services
    pub async fn shutdown(&self) -> Result<(), InitializationError> {
        let mut state = self.state.write().await;
        *state = ServiceState::Uninitialized;
        Ok(())
    }
}

// Implement CoreLogic trait for ServiceContainer to provide business logic operations
#[async_trait]
impl CoreLogic for ServiceContainer {
    async fn create_knowledge_node(
        &self,
        content: &str,
        metadata: serde_json::Value,
    ) -> NodeSpaceResult<NodeId> {
        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            return Err(NodeSpaceError::InternalError(format!(
                "Service not ready: {:?}",
                state
            )));
        }

        // Create the node with provided content and metadata
        let node_id = NodeId::new();
        let now = chrono::Utc::now().to_rfc3339();

        let node = Node {
            id: node_id.clone(),
            content: serde_json::Value::String(content.to_string()),
            metadata: Some(metadata),
            created_at: now.clone(),
            updated_at: now,
        };

        // Store the node
        self.data_store.store_node(node).await?;

        // Generate embeddings with improved error handling
        match self.nlp_engine.generate_embedding(content).await {
            Ok(_embedding) => {
                // TODO: Store embedding with node for semantic search
                // For MVP, embeddings are generated but not yet stored
            }
            Err(e) => {
                // Handle embedding failure based on configuration
                match self.config.offline_config.offline_fallback {
                    OfflineFallback::Error => {
                        // Delete the stored node since embedding failed
                        let _ = self.data_store.delete_node(&node_id).await;
                        return Err(NodeSpaceError::ProcessingError(format!(
                            "Embedding generation failed: {}",
                            e
                        )));
                    }
                    OfflineFallback::Stub | OfflineFallback::Cache => {
                        // Continue without embeddings, log warning
                        eprintln!("Warning: Embedding generation failed, continuing without semantic search: {}", e);
                    }
                }
            }
        }

        Ok(node_id)
    }

    async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            return Err(NodeSpaceError::InternalError(format!(
                "Service not ready: {:?}",
                state
            )));
        }

        // Apply performance configuration limits
        let effective_limit = if let Some(max_batch) = self.config.performance_config.max_batch_size
        {
            limit.min(max_batch)
        } else {
            limit
        };

        // For MVP, fall back to text-based search until vector search is implemented
        let search_query = format!(
            "SELECT * FROM nodes WHERE content CONTAINS '{}' LIMIT {}",
            query, effective_limit
        );
        let nodes = self.data_store.query_nodes(&search_query).await?;

        // Convert to SearchResult with basic scoring
        let mut results = Vec::new();
        for (index, node) in nodes.into_iter().enumerate() {
            // Simple scoring based on position (higher position = lower score)
            let score = 1.0 - (index as f32 * 0.1);

            results.push(SearchResult {
                node_id: node.id.clone(),
                node,
                score: score.max(0.1), // Minimum score of 0.1
            });
        }

        Ok(results)
    }

    async fn process_query(&self, query: &str) -> NodeSpaceResult<QueryResponse> {
        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            return Err(NodeSpaceError::InternalError(format!(
                "Service not ready: {:?}",
                state
            )));
        }

        // Step 1: Perform semantic search for context
        let search_results = self.semantic_search(query, 5).await?;

        // Step 2: Extract text content and source IDs
        let context: Vec<String> = search_results
            .iter()
            .filter_map(|result| result.node.content.as_str().map(|s| s.to_string()))
            .collect();

        let sources: Vec<NodeId> = search_results
            .iter()
            .map(|result| result.node_id.clone())
            .collect();

        // Step 3: Build prompt with context window management
        let context_text = context.join("\n\n");
        let max_context_len = self
            .config
            .performance_config
            .context_window
            .unwrap_or(4096)
            .saturating_sub(query.len() + 200); // Reserve space for prompt structure and query

        let truncated_context = if context_text.len() > max_context_len {
            format!("{}...", &context_text[..max_context_len])
        } else {
            context_text
        };

        let prompt = if truncated_context.is_empty() {
            format!("Answer this question based on general knowledge: {}", query)
        } else {
            format!(
                "Based on the following context, answer the question: {}\n\nContext:\n{}",
                query, truncated_context
            )
        };

        // Step 4: Generate response with error handling
        let answer = match self.nlp_engine.generate_text(&prompt).await {
            Ok(text) => text,
            Err(e) => {
                // Handle text generation failure based on configuration
                match self.config.offline_config.offline_fallback {
                    OfflineFallback::Error => {
                        return Err(NodeSpaceError::ProcessingError(format!(
                            "Text generation failed: {}",
                            e
                        )));
                    }
                    OfflineFallback::Stub => {
                        format!("I apologize, but I'm currently unable to generate a response due to AI system limitations. Please try again later. Query: {}", query)
                    }
                    OfflineFallback::Cache => {
                        // For MVP, provide a basic fallback response
                        if sources.is_empty() {
                            "I found no relevant information to answer your question.".to_string()
                        } else {
                            format!("I found {} related documents but cannot generate a detailed response at this time. Please review the source materials directly.", sources.len())
                        }
                    }
                }
            }
        };

        // Step 5: Calculate confidence based on context quality and AI availability
        let base_confidence = if context.is_empty() { 0.3 } else { 0.8 };
        let confidence =
            if answer.contains("currently unable") || answer.contains("cannot generate") {
                base_confidence * 0.5 // Reduce confidence for fallback responses
            } else {
                base_confidence
            };

        // Step 6: Generate related queries (simplified for MVP)
        let related_queries = vec![
            format!(
                "What else about {}?",
                query.split_whitespace().last().unwrap_or("this topic")
            ),
            format!(
                "How does {} work?",
                query
                    .split_whitespace()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        ];

        Ok(QueryResponse {
            answer,
            sources,
            confidence,
            related_queries,
        })
    }

    async fn update_node(&self, node_id: &NodeId, content: &str) -> NodeSpaceResult<()> {
        // Get existing node
        let mut node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::NotFound(format!("Node {} not found", node_id)))?;

        // Update content and timestamp
        node.content = serde_json::Value::String(content.to_string());
        node.updated_at = chrono::Utc::now().to_rfc3339();

        // Store updated node
        self.data_store.store_node(node).await?;

        // Regenerate embeddings for updated content
        let _embedding = self
            .nlp_engine
            .generate_embedding(content)
            .await
            .unwrap_or_else(|_| vec![0.0; 768]);

        // TODO: Update stored embeddings for semantic search

        Ok(())
    }

    async fn get_related_nodes(
        &self,
        node_id: &NodeId,
        relationship_types: Vec<String>,
    ) -> NodeSpaceResult<Vec<NodeId>> {
        // For MVP, use a simple query to find related nodes
        let relationship_filters = if relationship_types.is_empty() {
            "".to_string()
        } else {
            format!(
                " AND type IN [{}]",
                relationship_types
                    .iter()
                    .map(|t| format!("'{}'", t))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let query = format!(
            "SELECT out FROM {} WHERE in = nodes:{}{}",
            "relationships", // Assuming relationship table name
            node_id,
            relationship_filters
        );

        // Execute query and extract node IDs
        let result_nodes = self
            .data_store
            .query_nodes(&query)
            .await
            .unwrap_or_default(); // Gracefully handle relationship query failures

        let related_ids: Vec<NodeId> = result_nodes.into_iter().map(|node| node.id).collect();

        Ok(related_ids)
    }

    async fn generate_insights(&self, node_ids: Vec<NodeId>) -> NodeSpaceResult<String> {
        if node_ids.is_empty() {
            return Ok("No nodes provided for insight generation.".to_string());
        }

        // Collect content from all specified nodes
        let mut contents = Vec::new();
        for node_id in &node_ids {
            if let Ok(Some(node)) = self.data_store.get_node(node_id).await {
                if let Some(content_str) = node.content.as_str() {
                    contents.push(content_str.to_string());
                }
            }
        }

        if contents.is_empty() {
            return Ok("No readable content found in the specified nodes.".to_string());
        }

        // Generate insights using LLM
        let combined_content = contents.join("\n\n---\n\n");
        let prompt = format!(
            "Analyze the following content and provide key insights, patterns, and connections:\n\n{}\n\nProvide a concise summary with 3-5 key insights:",
            combined_content
        );

        let insights = self.nlp_engine.generate_text(&prompt).await?;

        Ok(insights)
    }
}

// Implement DateNavigation trait for ServiceContainer
#[async_trait]
impl DateNavigation for ServiceContainer {
    async fn get_nodes_for_date(&self, date: NaiveDate) -> NodeSpaceResult<Vec<Node>> {
        let date_str = date.format("%Y-%m-%d").to_string();

        // Query for all text nodes that have the specified date as parent
        let query = format!(
            "SELECT * FROM text WHERE parent_node = (SELECT id FROM date WHERE date_value = '{}' LIMIT 1)",
            date_str
        );

        let nodes = self.data_store.query_nodes(&query).await?;
        Ok(nodes)
    }

    async fn navigate_to_date(&self, date: NaiveDate) -> NodeSpaceResult<NavigationResult> {
        // Get nodes for the specified date
        let nodes = self.get_nodes_for_date(date).await?;

        // Check if there's a previous day with content
        let has_previous = (self.get_previous_day(date).await?).is_some();

        // Check if there's a next day with content
        let has_next = (self.get_next_day(date).await?).is_some();

        Ok(NavigationResult {
            date,
            nodes,
            has_previous,
            has_next,
        })
    }

    async fn get_previous_day(
        &self,
        current_date: NaiveDate,
    ) -> NodeSpaceResult<Option<NaiveDate>> {
        let current_str = current_date.format("%Y-%m-%d").to_string();

        // Find the latest date before the current date that has content
        let query = format!(
            "SELECT date_value FROM date WHERE date_value < '{}' AND id IN (SELECT DISTINCT parent_node FROM text) ORDER BY date_value DESC LIMIT 1",
            current_str
        );

        let result_nodes = self.data_store.query_nodes(&query).await?;

        if let Some(node) = result_nodes.first() {
            if let Some(date_str) = node.content.as_str() {
                if let Ok(parsed_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    return Ok(Some(parsed_date));
                }
            }
        }

        Ok(None)
    }

    async fn get_next_day(&self, current_date: NaiveDate) -> NodeSpaceResult<Option<NaiveDate>> {
        let current_str = current_date.format("%Y-%m-%d").to_string();

        // Find the earliest date after the current date that has content
        let query = format!(
            "SELECT date_value FROM date WHERE date_value > '{}' AND id IN (SELECT DISTINCT parent_node FROM text) ORDER BY date_value ASC LIMIT 1",
            current_str
        );

        let result_nodes = self.data_store.query_nodes(&query).await?;

        if let Some(node) = result_nodes.first() {
            if let Some(date_str) = node.content.as_str() {
                if let Ok(parsed_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    return Ok(Some(parsed_date));
                }
            }
        }

        Ok(None)
    }

    async fn create_or_get_date_node(&self, date: NaiveDate) -> NodeSpaceResult<DateNode> {
        let date_str = date.format("%Y-%m-%d").to_string();

        // First, try to find existing date node
        let query = format!("SELECT * FROM date WHERE date_value = '{}'", date_str);
        let existing_nodes = self.data_store.query_nodes(&query).await?;

        if let Some(existing_node) = existing_nodes.first() {
            // Get child count
            let count_query = format!(
                "SELECT COUNT() AS count FROM text WHERE parent_node = '{}'",
                existing_node.id
            );
            let count_result = self.data_store.query_nodes(&count_query).await?;
            let child_count = count_result
                .first()
                .and_then(|n| n.content.as_u64())
                .unwrap_or(0) as usize;

            return Ok(DateNode {
                id: existing_node.id.clone(),
                date,
                description: existing_node
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("description"))
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string()),
                child_count,
            });
        }

        // Create new date node if it doesn't exist
        let node_id = NodeId::new();
        let now = chrono::Utc::now().to_rfc3339();
        let description = format!("{}", date.format("%A, %B %d, %Y"));

        let date_node = Node {
            id: node_id.clone(),
            content: serde_json::Value::String(date_str),
            metadata: Some(serde_json::json!({
                "description": description,
                "node_type": "date"
            })),
            created_at: now.clone(),
            updated_at: now,
        };

        // Store the date node
        self.data_store.store_node(date_node).await?;

        Ok(DateNode {
            id: node_id,
            date,
            description: Some(description),
            child_count: 0,
        })
    }

    async fn get_date_node_children(&self, date_node_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Query for all nodes that have this date node as their parent
        let query = format!(
            "SELECT * FROM text WHERE parent_node = '{}' ORDER BY created_at ASC",
            date_node_id
        );

        let children = self.data_store.query_nodes(&query).await?;
        Ok(children)
    }
}

/// Core business logic operations interface following distributed contract pattern
#[async_trait]
pub trait CoreLogic: Send + Sync {
    /// Create a new knowledge node with AI processing
    async fn create_knowledge_node(
        &self,
        content: &str,
        metadata: serde_json::Value,
    ) -> NodeSpaceResult<NodeId>;

    /// Search for nodes using semantic similarity
    async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
    ) -> NodeSpaceResult<Vec<SearchResult>>;

    /// Process natural language query and return results
    async fn process_query(&self, query: &str) -> NodeSpaceResult<QueryResponse>;

    /// Update node content and reprocess embeddings
    async fn update_node(&self, node_id: &NodeId, content: &str) -> NodeSpaceResult<()>;

    /// Get related nodes using graph relationships
    async fn get_related_nodes(
        &self,
        node_id: &NodeId,
        relationship_types: Vec<String>,
    ) -> NodeSpaceResult<Vec<NodeId>>;

    /// Generate insights from a collection of nodes
    async fn generate_insights(&self, node_ids: Vec<NodeId>) -> NodeSpaceResult<String>;
}

/// Date navigation operations for hierarchical date-based content organization
#[async_trait]
pub trait DateNavigation: Send + Sync {
    /// Get all text nodes for a specific date
    async fn get_nodes_for_date(&self, date: NaiveDate) -> NodeSpaceResult<Vec<Node>>;

    /// Switch to viewing a specific date with navigation context
    async fn navigate_to_date(&self, date: NaiveDate) -> NodeSpaceResult<NavigationResult>;

    /// Navigate to previous day with content
    async fn get_previous_day(&self, current_date: NaiveDate)
        -> NodeSpaceResult<Option<NaiveDate>>;

    /// Navigate to next day with content
    async fn get_next_day(&self, current_date: NaiveDate) -> NodeSpaceResult<Option<NaiveDate>>;

    /// Ensure date node exists for organization
    async fn create_or_get_date_node(&self, date: NaiveDate) -> NodeSpaceResult<DateNode>;

    /// Get all child nodes of a date node (hierarchical retrieval)
    async fn get_date_node_children(&self, date_node_id: &NodeId) -> NodeSpaceResult<Vec<Node>>;

    /// Get today's date for navigation
    fn get_today() -> NaiveDate {
        chrono::Utc::now().date_naive()
    }
}

/// Search result with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub node_id: NodeId,
    pub node: Node,
    pub score: f32,
}

/// Query response with results and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub answer: String,
    pub sources: Vec<NodeId>,
    pub confidence: f32,
    pub related_queries: Vec<String>,
}

/// Navigation result for date operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResult {
    pub date: NaiveDate,
    pub nodes: Vec<Node>,
    pub has_previous: bool,
    pub has_next: bool,
}

/// Date node for organizing hierarchical content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateNode {
    pub id: NodeId,
    pub date: NaiveDate,
    pub description: Option<String>,
    pub child_count: usize,
}

#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> CoreLogic for NodeSpaceService<D, N> {
    async fn create_knowledge_node(
        &self,
        content: &str,
        metadata: serde_json::Value,
    ) -> NodeSpaceResult<NodeId> {
        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            return Err(NodeSpaceError::InternalError(format!(
                "Service not ready: {:?}",
                state
            )));
        }

        // Create the node with provided content and metadata
        let node_id = NodeId::new();
        let now = chrono::Utc::now().to_rfc3339();

        let node = Node {
            id: node_id.clone(),
            content: serde_json::Value::String(content.to_string()),
            metadata: Some(metadata),
            created_at: now.clone(),
            updated_at: now,
        };

        // Store the node
        self.data_store.store_node(node).await?;

        // Generate embeddings with improved error handling
        match self.nlp_engine.generate_embedding(content).await {
            Ok(_embedding) => {
                // TODO: Store embedding with node for semantic search
                // For MVP, embeddings are generated but not yet stored
            }
            Err(e) => {
                // Handle embedding failure based on configuration
                match self.config.offline_config.offline_fallback {
                    OfflineFallback::Error => {
                        // Delete the stored node since embedding failed
                        let _ = self.data_store.delete_node(&node_id).await;
                        return Err(NodeSpaceError::ProcessingError(format!(
                            "Embedding generation failed: {}",
                            e
                        )));
                    }
                    OfflineFallback::Stub | OfflineFallback::Cache => {
                        // Continue without embeddings, log warning
                        eprintln!("Warning: Embedding generation failed, continuing without semantic search: {}", e);
                    }
                }
            }
        }

        Ok(node_id)
    }

    async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            return Err(NodeSpaceError::InternalError(format!(
                "Service not ready: {:?}",
                state
            )));
        }

        // Apply performance configuration limits
        let effective_limit = if let Some(max_batch) = self.config.performance_config.max_batch_size
        {
            limit.min(max_batch)
        } else {
            limit
        };

        // For MVP, fall back to text-based search until vector search is implemented
        let search_query = format!(
            "SELECT * FROM nodes WHERE content CONTAINS '{}' LIMIT {}",
            query, effective_limit
        );
        let nodes = self.data_store.query_nodes(&search_query).await?;

        // Convert to SearchResult with basic scoring
        let mut results = Vec::new();
        for (index, node) in nodes.into_iter().enumerate() {
            // Simple scoring based on position (higher position = lower score)
            let score = 1.0 - (index as f32 * 0.1);

            results.push(SearchResult {
                node_id: node.id.clone(),
                node,
                score: score.max(0.1), // Minimum score of 0.1
            });
        }

        Ok(results)
    }

    async fn process_query(&self, query: &str) -> NodeSpaceResult<QueryResponse> {
        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            return Err(NodeSpaceError::InternalError(format!(
                "Service not ready: {:?}",
                state
            )));
        }

        // Step 1: Perform semantic search for context
        let search_results = self.semantic_search(query, 5).await?;

        // Step 2: Extract text content and source IDs
        let context: Vec<String> = search_results
            .iter()
            .filter_map(|result| result.node.content.as_str().map(|s| s.to_string()))
            .collect();

        let sources: Vec<NodeId> = search_results
            .iter()
            .map(|result| result.node_id.clone())
            .collect();

        // Step 3: Build prompt with context window management
        let context_text = context.join("\n\n");
        let max_context_len = self
            .config
            .performance_config
            .context_window
            .unwrap_or(4096)
            .saturating_sub(query.len() + 200); // Reserve space for prompt structure and query

        let truncated_context = if context_text.len() > max_context_len {
            format!("{}...", &context_text[..max_context_len])
        } else {
            context_text
        };

        let prompt = if truncated_context.is_empty() {
            format!("Answer this question based on general knowledge: {}", query)
        } else {
            format!(
                "Based on the following context, answer the question: {}\n\nContext:\n{}",
                query, truncated_context
            )
        };

        // Step 4: Generate response with error handling
        let answer = match self.nlp_engine.generate_text(&prompt).await {
            Ok(text) => text,
            Err(e) => {
                // Handle text generation failure based on configuration
                match self.config.offline_config.offline_fallback {
                    OfflineFallback::Error => {
                        return Err(NodeSpaceError::ProcessingError(format!(
                            "Text generation failed: {}",
                            e
                        )));
                    }
                    OfflineFallback::Stub => {
                        format!("I apologize, but I'm currently unable to generate a response due to AI system limitations. Please try again later. Query: {}", query)
                    }
                    OfflineFallback::Cache => {
                        // For MVP, provide a basic fallback response
                        if sources.is_empty() {
                            "I found no relevant information to answer your question.".to_string()
                        } else {
                            format!("I found {} related documents but cannot generate a detailed response at this time. Please review the source materials directly.", sources.len())
                        }
                    }
                }
            }
        };

        // Step 5: Calculate confidence based on context quality and AI availability
        let base_confidence = if context.is_empty() { 0.3 } else { 0.8 };
        let confidence =
            if answer.contains("currently unable") || answer.contains("cannot generate") {
                base_confidence * 0.5 // Reduce confidence for fallback responses
            } else {
                base_confidence
            };

        // Step 6: Generate related queries (simplified for MVP)
        let related_queries = vec![
            format!(
                "What else about {}?",
                query.split_whitespace().last().unwrap_or("this topic")
            ),
            format!(
                "How does {} work?",
                query
                    .split_whitespace()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        ];

        Ok(QueryResponse {
            answer,
            sources,
            confidence,
            related_queries,
        })
    }

    async fn update_node(&self, node_id: &NodeId, content: &str) -> NodeSpaceResult<()> {
        // Get existing node
        let mut node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::NotFound(format!("Node {} not found", node_id)))?;

        // Update content and timestamp
        node.content = serde_json::Value::String(content.to_string());
        node.updated_at = chrono::Utc::now().to_rfc3339();

        // Store updated node
        self.data_store.store_node(node).await?;

        // Regenerate embeddings for updated content
        let _embedding = self
            .nlp_engine
            .generate_embedding(content)
            .await
            .unwrap_or_else(|_| vec![0.0; 768]);

        // TODO: Update stored embeddings for semantic search

        Ok(())
    }

    async fn get_related_nodes(
        &self,
        node_id: &NodeId,
        relationship_types: Vec<String>,
    ) -> NodeSpaceResult<Vec<NodeId>> {
        // For MVP, use a simple query to find related nodes
        let relationship_filters = if relationship_types.is_empty() {
            "".to_string()
        } else {
            format!(
                " AND type IN [{}]",
                relationship_types
                    .iter()
                    .map(|t| format!("'{}'", t))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let query = format!(
            "SELECT out FROM {} WHERE in = nodes:{}{}",
            "relationships", // Assuming relationship table name
            node_id,
            relationship_filters
        );

        // Execute query and extract node IDs
        let result_nodes = self
            .data_store
            .query_nodes(&query)
            .await
            .unwrap_or_default(); // Gracefully handle relationship query failures

        let related_ids: Vec<NodeId> = result_nodes.into_iter().map(|node| node.id).collect();

        Ok(related_ids)
    }

    async fn generate_insights(&self, node_ids: Vec<NodeId>) -> NodeSpaceResult<String> {
        if node_ids.is_empty() {
            return Ok("No nodes provided for insight generation.".to_string());
        }

        // Collect content from all specified nodes
        let mut contents = Vec::new();
        for node_id in &node_ids {
            if let Ok(Some(node)) = self.data_store.get_node(node_id).await {
                if let Some(content_str) = node.content.as_str() {
                    contents.push(content_str.to_string());
                }
            }
        }

        if contents.is_empty() {
            return Ok("No readable content found in the specified nodes.".to_string());
        }

        // Generate insights using LLM
        let combined_content = contents.join("\n\n---\n\n");
        let prompt = format!(
            "Analyze the following content and provide key insights, patterns, and connections:\n\n{}\n\nProvide a concise summary with 3-5 key insights:",
            combined_content
        );

        let insights = self.nlp_engine.generate_text(&prompt).await?;

        Ok(insights)
    }
}

/// Legacy CoreLogic interface for backward compatibility
#[async_trait]
pub trait LegacyCoreLogic {
    /// Create a new node with automatic embedding generation
    async fn create_node(
        &self,
        content: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> NodeSpaceResult<NodeId>;

    /// Retrieve a node by ID
    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>>;

    /// Delete a node
    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()>;

    /// Search nodes using semantic and text search
    async fn search_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>>;

    /// Process a RAG query: search for context + generate response
    async fn process_rag_query(&self, query: &str) -> NodeSpaceResult<String>;

    /// Create a relationship between nodes
    async fn create_relationship(
        &self,
        from: &NodeId,
        to: &NodeId,
        rel_type: &str,
    ) -> NodeSpaceResult<()>;
}

/// Legacy implementation for backward compatibility
#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> LegacyCoreLogic
    for NodeSpaceService<D, N>
{
    async fn create_node(
        &self,
        content: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> NodeSpaceResult<NodeId> {
        let node_id = NodeId::new();
        let now = chrono::Utc::now().to_rfc3339();

        let node = Node {
            id: node_id.clone(),
            content,
            metadata,
            created_at: now.clone(),
            updated_at: now,
        };

        self.data_store.store_node(node).await?;
        Ok(node_id)
    }

    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        self.data_store.get_node(id).await
    }

    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
        self.data_store.delete_node(id).await
    }

    async fn search_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>> {
        let search_query = format!("SELECT * FROM nodes WHERE content CONTAINS '{}'", query);
        self.data_store.query_nodes(&search_query).await
    }

    async fn process_rag_query(&self, query: &str) -> NodeSpaceResult<String> {
        let response = self.process_query(query).await?;
        Ok(response.answer)
    }

    async fn create_relationship(
        &self,
        from: &NodeId,
        to: &NodeId,
        rel_type: &str,
    ) -> NodeSpaceResult<()> {
        self.data_store
            .create_relationship(from, to, rel_type)
            .await
    }
}

/// Date Navigation implementation for NodeSpaceService
#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> DateNavigation
    for NodeSpaceService<D, N>
{
    async fn get_nodes_for_date(&self, date: NaiveDate) -> NodeSpaceResult<Vec<Node>> {
        let date_str = date.format("%Y-%m-%d").to_string();

        // Query for all text nodes that have the specified date as parent
        let query = format!(
            "SELECT * FROM text WHERE parent_node = (SELECT id FROM date WHERE date_value = '{}' LIMIT 1)",
            date_str
        );

        let nodes = self.data_store.query_nodes(&query).await?;
        Ok(nodes)
    }

    async fn navigate_to_date(&self, date: NaiveDate) -> NodeSpaceResult<NavigationResult> {
        // Get nodes for the specified date
        let nodes = self.get_nodes_for_date(date).await?;

        // Check if there's a previous day with content
        let has_previous = (self.get_previous_day(date).await?).is_some();

        // Check if there's a next day with content
        let has_next = (self.get_next_day(date).await?).is_some();

        Ok(NavigationResult {
            date,
            nodes,
            has_previous,
            has_next,
        })
    }

    async fn get_previous_day(
        &self,
        current_date: NaiveDate,
    ) -> NodeSpaceResult<Option<NaiveDate>> {
        let current_str = current_date.format("%Y-%m-%d").to_string();

        // Find the latest date before the current date that has content
        let query = format!(
            "SELECT date_value FROM date WHERE date_value < '{}' AND id IN (SELECT DISTINCT parent_node FROM text) ORDER BY date_value DESC LIMIT 1",
            current_str
        );

        let result_nodes = self.data_store.query_nodes(&query).await?;

        if let Some(node) = result_nodes.first() {
            if let Some(date_str) = node.content.as_str() {
                if let Ok(parsed_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    return Ok(Some(parsed_date));
                }
            }
        }

        Ok(None)
    }

    async fn get_next_day(&self, current_date: NaiveDate) -> NodeSpaceResult<Option<NaiveDate>> {
        let current_str = current_date.format("%Y-%m-%d").to_string();

        // Find the earliest date after the current date that has content
        let query = format!(
            "SELECT date_value FROM date WHERE date_value > '{}' AND id IN (SELECT DISTINCT parent_node FROM text) ORDER BY date_value ASC LIMIT 1",
            current_str
        );

        let result_nodes = self.data_store.query_nodes(&query).await?;

        if let Some(node) = result_nodes.first() {
            if let Some(date_str) = node.content.as_str() {
                if let Ok(parsed_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    return Ok(Some(parsed_date));
                }
            }
        }

        Ok(None)
    }

    async fn create_or_get_date_node(&self, date: NaiveDate) -> NodeSpaceResult<DateNode> {
        let date_str = date.format("%Y-%m-%d").to_string();

        // First, try to find existing date node
        let query = format!("SELECT * FROM date WHERE date_value = '{}'", date_str);
        let existing_nodes = self.data_store.query_nodes(&query).await?;

        if let Some(existing_node) = existing_nodes.first() {
            // Get child count
            let count_query = format!(
                "SELECT COUNT() AS count FROM text WHERE parent_node = '{}'",
                existing_node.id
            );
            let count_result = self.data_store.query_nodes(&count_query).await?;
            let child_count = count_result
                .first()
                .and_then(|n| n.content.as_u64())
                .unwrap_or(0) as usize;

            return Ok(DateNode {
                id: existing_node.id.clone(),
                date,
                description: existing_node
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("description"))
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string()),
                child_count,
            });
        }

        // Create new date node if it doesn't exist
        let node_id = NodeId::new();
        let now = chrono::Utc::now().to_rfc3339();
        let description = format!("{}", date.format("%A, %B %d, %Y"));

        let date_node = Node {
            id: node_id.clone(),
            content: serde_json::Value::String(date_str),
            metadata: Some(serde_json::json!({
                "description": description,
                "node_type": "date"
            })),
            created_at: now.clone(),
            updated_at: now,
        };

        // Store the date node
        self.data_store.store_node(date_node).await?;

        Ok(DateNode {
            id: node_id,
            date,
            description: Some(description),
            child_count: 0,
        })
    }

    async fn get_date_node_children(&self, date_node_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Query for all nodes that have this date node as their parent
        let query = format!(
            "SELECT * FROM text WHERE parent_node = '{}' ORDER BY created_at ASC",
            date_node_id
        );

        let children = self.data_store.query_nodes(&query).await?;
        Ok(children)
    }
}
