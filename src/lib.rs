use async_trait::async_trait;
use chrono::NaiveDate;
use nodespace_core_types::{Node, NodeId, NodeSpaceError, NodeSpaceResult};
use nodespace_data_store::{DataStore, SurrealDataStore};
use nodespace_nlp_engine::{LocalNLPEngine, NLPEngine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Date navigation result with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResult {
    pub date: NaiveDate,
    pub nodes: Vec<Node>,
    pub has_previous: bool,
    pub has_next: bool,
}

/// Date node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateNode {
    pub id: NodeId,
    pub date: NaiveDate,
    pub description: Option<String>,
    pub child_count: usize,
}

/// Date navigation operations interface
#[async_trait]
pub trait DateNavigation: Send + Sync {
    /// Get all nodes for a specific date
    async fn get_nodes_for_date(&self, date: NaiveDate) -> NodeSpaceResult<Vec<Node>>;

    /// Navigate to a specific date and get context
    async fn navigate_to_date(&self, date: NaiveDate) -> NodeSpaceResult<NavigationResult>;

    /// Get the previous day with content
    async fn get_previous_day(&self, current_date: NaiveDate)
        -> NodeSpaceResult<Option<NaiveDate>>;

    /// Get the next day with content
    async fn get_next_day(&self, current_date: NaiveDate) -> NodeSpaceResult<Option<NaiveDate>>;

    /// Create or get a date node
    async fn create_or_get_date_node(&self, date: NaiveDate) -> NodeSpaceResult<DateNode>;

    /// Get children of a date node
    async fn get_date_node_children(&self, date_node_id: &NodeId) -> NodeSpaceResult<Vec<Node>>;
}

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
#[allow(dead_code)]
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
    #[allow(dead_code)]
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


// Implement DateNavigation trait for ServiceContainer
#[async_trait]
impl DateNavigation for ServiceContainer {
    async fn get_nodes_for_date(&self, date: NaiveDate) -> NodeSpaceResult<Vec<Node>> {
        let date_str = date.format("%Y-%m-%d").to_string();

        // Use the specialized DataStore method instead of raw SQL
        self.data_store.get_nodes_for_date(&date_str).await
    }

    async fn navigate_to_date(&self, date: NaiveDate) -> NodeSpaceResult<NavigationResult> {
        // Get nodes for the specified date using the DateNavigation method
        let nodes = DateNavigation::get_nodes_for_date(self, date).await?;

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
        // Use a simpler approach - query all text nodes and filter in Rust
        let query = "SELECT * FROM text WHERE parent_date IS NOT NULL";
        let result_nodes = self.data_store.query_nodes(query).await?;

        // Extract unique parent_date values and sort them
        let mut dates: Vec<String> = Vec::new();
        for node in &result_nodes {
            if let Some(metadata) = &node.metadata {
                if let Some(parent_date) = metadata.get("parent_date").and_then(|v| v.as_str()) {
                    if !dates.contains(&parent_date.to_string()) {
                        dates.push(parent_date.to_string());
                    }
                }
            }
        }
        dates.sort();
        dates.reverse(); // DESC order

        // Filter dates that are before current_date
        for date_str in &dates {
            if date_str < &current_str {
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
        // Use a simpler approach - query all dates with content and filter in Rust
        let query = "SELECT DISTINCT parent_date FROM text WHERE parent_date IS NOT NULL ORDER BY parent_date ASC";
        let result_nodes = self.data_store.query_nodes(query).await?;

        // Filter dates that are after current_date
        for node in result_nodes {
            if let Some(date_str) = node.content.as_str() {
                if date_str > current_str.as_str() {
                    if let Ok(parsed_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        return Ok(Some(parsed_date));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn create_or_get_date_node(&self, date: NaiveDate) -> NodeSpaceResult<DateNode> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let description = format!("{}", date.format("%A, %B %d, %Y"));

        // Use the specialized DataStore method to create or get the date node
        let node_id = self
            .data_store
            .create_or_get_date_node(&date_str, Some(&description))
            .await?;

        // Get child count using the specialized DataStore method
        let children = self.data_store.get_date_children(&date_str).await?;
        let child_count = children.len();

        Ok(DateNode {
            id: node_id,
            date,
            description: Some(description),
            child_count,
        })
    }

    async fn get_date_node_children(&self, date_node_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Use the date_node_id as the date string for the specialized DataStore method
        let date_str = date_node_id.to_string();

        // Get children using the specialized DataStore method
        let children_json = self.data_store.get_date_children(&date_str).await?;

        // Convert JSON values to Node structs
        let mut nodes = Vec::new();
        for child_json in children_json {
            // Parse each JSON value as a Node
            if let Ok(node) = serde_json::from_value::<Node>(child_json) {
                nodes.push(node);
            }
        }

        Ok(nodes)
    }
}

/// Core business logic operations interface - complete 8 method API
#[async_trait]
pub trait CoreLogic: Send + Sync {
    /// Get all nodes for a specific date
    async fn get_nodes_for_date(&self, date: &str) -> NodeSpaceResult<Vec<Node>>;

    /// Create a new text node with date association
    async fn create_text_node(&self, content: &str, date: &str) -> NodeSpaceResult<NodeId>;

    /// Search for nodes using semantic similarity
    async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
    ) -> NodeSpaceResult<Vec<SearchResult>>;

    /// Process natural language query with full RAG pipeline
    async fn process_query(&self, query: &str) -> NodeSpaceResult<QueryResponse>;

    /// Add a child relationship between nodes
    async fn add_child_node(&self, parent_id: &NodeId, child_id: &NodeId) -> NodeSpaceResult<()>;

    /// Get all child nodes of a parent
    async fn get_child_nodes(&self, parent_id: &NodeId) -> NodeSpaceResult<Vec<Node>>;

    /// Update node content
    async fn update_node(&self, node_id: &NodeId, content: &str) -> NodeSpaceResult<()>;

    /// Link two nodes as siblings while preserving existing chains
    async fn make_siblings(
        &self,
        left_node_id: &NodeId,
        right_node_id: &NodeId,
    ) -> NodeSpaceResult<()>;

    /// Retrieve a node by ID
    async fn get_node(&self, node_id: &NodeId) -> NodeSpaceResult<Option<Node>>;
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

/// Implementation of the clean 7-method CoreLogic interface for ServiceContainer
#[async_trait]
impl CoreLogic for ServiceContainer {
    async fn get_nodes_for_date(&self, date: &str) -> NodeSpaceResult<Vec<Node>> {
        // Use the specialized DataStore method
        self.data_store.get_nodes_for_date(date).await
    }

    async fn create_text_node(&self, content: &str, date: &str) -> NodeSpaceResult<NodeId> {
        // Generate embedding first
        let embedding = self.nlp_engine.generate_embedding(content).await?;

        // Create node with embedding and metadata
        let node = Node {
            id: NodeId::new(),
            content: serde_json::Value::String(content.to_string()),
            metadata: Some(serde_json::json!({"parent_date": date})),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let node_id = node.id.clone();

        // Store node with embedding using the data store's method
        self.data_store.store_node_with_embedding(node, embedding).await?;

        Ok(node_id)
    }

    async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        // Generate query embedding
        let query_embedding = self.nlp_engine.generate_embedding(query).await?;

        // Perform vector similarity search using the data store's semantic search method
        let results = self.data_store.semantic_search_with_embedding(query_embedding, limit).await?;

        // Convert to SearchResult format
        let search_results = results
            .into_iter()
            .map(|(node, score)| SearchResult {
                node_id: node.id.clone(),
                node,
                score,
            })
            .collect();

        Ok(search_results)
    }

    async fn process_query(&self, query: &str) -> NodeSpaceResult<QueryResponse> {
        // Full RAG pipeline: search + LLM generation
        let search_results = self.semantic_search(query, 5).await?;

        let context: Vec<String> = search_results
            .iter()
            .filter_map(|result| result.node.content.as_str().map(|s| s.to_string()))
            .collect();

        let sources: Vec<NodeId> = search_results
            .iter()
            .map(|result| result.node_id.clone())
            .collect();

        let context_text = context.join("\n\n");
        let prompt = if context_text.is_empty() {
            format!("Answer this question based on general knowledge: {}", query)
        } else {
            format!(
                "Based on the following context, answer the question: {}\n\nContext:\n{}",
                query, context_text
            )
        };

        let answer = self.nlp_engine.generate_text(&prompt).await?;
        let confidence = if context.is_empty() { 0.3 } else { 0.8 };

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

    async fn add_child_node(&self, parent_id: &NodeId, child_id: &NodeId) -> NodeSpaceResult<()> {
        // Use the DataStore method to create a "contains" relationship
        self.data_store
            .create_relationship(parent_id, child_id, "contains")
            .await
    }

    async fn get_child_nodes(&self, parent_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Use the DataStore method to get children via relationships
        // For date nodes, use get_date_children; for other nodes, use a query
        let parent_str = parent_id.to_string();

        // Check if this looks like a date (YYYY-MM-DD format)
        if parent_str.len() == 10
            && parent_str.chars().nth(4) == Some('-')
            && parent_str.chars().nth(7) == Some('-')
        {
            // Use date-specific method
            let children_json = self.data_store.get_date_children(&parent_str).await?;
            let mut nodes = Vec::new();
            for child_json in children_json {
                if let Ok(node) = serde_json::from_value::<Node>(child_json) {
                    nodes.push(node);
                }
            }
            Ok(nodes)
        } else {
            // Use generic relationship query for non-date nodes
            let query = format!("SELECT * FROM {} WHERE in = {}", "relationships", parent_id);
            self.data_store.query_nodes(&query).await
        }
    }

    async fn update_node(&self, node_id: &NodeId, content: &str) -> NodeSpaceResult<()> {
        // Get existing node, update content, store it back
        let mut node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::NotFound(format!("Node {} not found", node_id)))?;

        node.content = serde_json::Value::String(content.to_string());
        node.updated_at = chrono::Utc::now().to_rfc3339();

        self.data_store.store_node(node).await?;
        Ok(())
    }

    async fn make_siblings(
        &self,
        left_node_id: &NodeId,
        right_node_id: &NodeId,
    ) -> NodeSpaceResult<()> {
        // For now, implement a basic sibling relationship using the data store
        // Note: This requires the Node structure to have sibling pointer fields (NS-45)
        // Until then, we'll create relationships in the database

        // Validate both nodes exist
        let _left_node = self
            .data_store
            .get_node(left_node_id)
            .await?
            .ok_or_else(|| {
                NodeSpaceError::NotFound(format!("Left node {} not found", left_node_id))
            })?;
        let _right_node = self
            .data_store
            .get_node(right_node_id)
            .await?
            .ok_or_else(|| {
                NodeSpaceError::NotFound(format!("Right node {} not found", right_node_id))
            })?;

        // Create bidirectional sibling relationships
        self.data_store
            .create_relationship(left_node_id, right_node_id, "next_sibling")
            .await?;
        self.data_store
            .create_relationship(right_node_id, left_node_id, "previous_sibling")
            .await?;

        Ok(())
    }

    async fn get_node(&self, node_id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        self.data_store.get_node(node_id).await
    }
}
