use async_trait::async_trait;
use chrono::NaiveDate;
use nodespace_core_types::{Node, NodeId, NodeSpaceError, NodeSpaceResult};
use nodespace_data_store::{DataStore, LanceDataStore}; // Import LanceDataStore implementation
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

/// Chat message for conversation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub content: String,
    pub role: MessageRole,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub sequence_number: u32,
    pub rag_context: Option<RAGMessageContext>,
}

/// Message roles in conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// RAG context metadata for message transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGMessageContext {
    pub sources_used: Vec<NodeId>,
    pub retrieval_score: f32,
    pub context_tokens: usize,
    pub generation_time_ms: u64,
    pub knowledge_summary: String,
}

/// Configuration for RAG operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGConfig {
    pub max_retrieval_results: usize,      // Default: 5
    pub relevance_threshold: f32,          // Default: 0.7
    pub max_context_tokens: usize,         // Default: 2048
    pub conversation_context_limit: usize, // Default: 5 messages
    pub reserved_response_tokens: usize,   // Default: 512
}

impl Default for RAGConfig {
    fn default() -> Self {
        Self {
            max_retrieval_results: 5,
            relevance_threshold: 0.7,
            max_context_tokens: 2048,
            conversation_context_limit: 5,
            reserved_response_tokens: 512,
        }
    }
}

/// Token budget management for conversation context
#[derive(Debug, Clone)]
pub struct TokenBudget {
    pub total_available: usize,       // Model's context window
    pub reserved_for_response: usize, // 512 tokens for response
    pub conversation_history: usize,  // Recent chat context
    pub knowledge_context: usize,     // Retrieved information
    pub system_prompt: usize,         // Instructions
}

impl TokenBudget {
    pub fn new(total_tokens: usize, reserved_response: usize) -> Self {
        Self {
            total_available: total_tokens,
            reserved_for_response: reserved_response,
            conversation_history: 0,
            knowledge_context: 0,
            system_prompt: 150, // Approximate system prompt size
        }
    }

    pub fn available_for_context(&self) -> usize {
        self.total_available
            .saturating_sub(self.reserved_for_response)
            .saturating_sub(self.system_prompt)
    }

    pub fn allocate_conversation_tokens(&mut self, tokens: usize) {
        self.conversation_history = tokens;
    }

    pub fn allocate_knowledge_tokens(&mut self, tokens: usize) {
        self.knowledge_context = tokens;
    }

    pub fn tokens_used(&self) -> usize {
        self.conversation_history + self.knowledge_context + self.system_prompt
    }

    pub fn tokens_remaining(&self) -> usize {
        self.total_available
            .saturating_sub(self.tokens_used() + self.reserved_for_response)
    }
}

/// Enhanced RAG response with conversation support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGResponse {
    pub answer: String,
    pub sources: Vec<Node>,
    pub context_summary: String,
    pub relevance_score: f32,
    pub context_tokens: usize,
    pub generation_time_ms: u64,
    pub conversation_context_used: usize,
}

/// RAG query request with conversation context
#[derive(Debug, Clone)]
pub struct RAGQueryRequest {
    pub query: String,
    pub session_id: String,
    pub conversation_history: Vec<ChatMessage>,
    pub date_scope: Option<String>,
    pub max_results: Option<usize>,
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

/// ServiceContainer for coordinating all NodeSpace services  
/// Uses LanceDataStore for simplified relationship model and vector-first operations
pub struct ServiceContainer {
    // LanceDB data store with simplified JSON-based relationships
    data_store: LanceDataStore,
    nlp_engine: LocalNLPEngine,
    config: NodeSpaceConfig,
    rag_config: RAGConfig,
    state: Arc<RwLock<ServiceState>>,
}

/// Initialization error for ServiceContainer
#[derive(Debug, thiserror::Error)]
pub enum InitializationError {
    #[error("NLP engine initialization failed: {0}")]
    NLPError(#[from] nodespace_nlp_engine::NLPError),
    #[error("Core service initialization failed: {0}")]
    CoreServiceError(#[from] NodeSpaceError),
    #[error("Data store initialization failed: {0}")]
    DataStoreError(#[from] nodespace_data_store::DataStoreError),
}

impl ServiceContainer {
    /// Create a new ServiceContainer with LanceDB data store
    /// Uses simplified relationship model and vector-first operations
    pub async fn new() -> Result<Self, InitializationError> {
        Self::new_with_config(NodeSpaceConfig::default()).await
    }

    /// Create ServiceContainer with custom configuration
    pub async fn new_with_config(config: NodeSpaceConfig) -> Result<Self, InitializationError> {
        // Initialize LanceDB data store with simplified relationship model
        let mut data_store = LanceDataStore::new("./data/lance.db").await?;
        data_store.initialize_table().await?;

        // Initialize NLP engine with shared model configuration
        let nlp_engine = LocalNLPEngine::new();
        nlp_engine.initialize().await?;

        // Create service container with LanceDB integration
        let state = Arc::new(RwLock::new(ServiceState::Ready));

        Ok(ServiceContainer {
            data_store,
            nlp_engine,
            config,
            rag_config: RAGConfig::default(),
            state,
        })
    }

    /// Get data store size for debugging (uses LanceDB)
    pub async fn storage_size(&self) -> usize {
        // Get all nodes from LanceDB data store
        match self.data_store.query_nodes("").await {
            Ok(nodes) => nodes.len(),
            Err(_) => 0,
        }
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

    /// Convenience method: Process a simple query without conversation context
    pub async fn process_simple_query(&self, query: &str) -> NodeSpaceResult<RAGResponse> {
        let request = RAGQueryRequest {
            query: query.to_string(),
            session_id: "simple".to_string(),
            conversation_history: Vec::new(),
            date_scope: None,
            max_results: None,
        };

        RAGService::process_rag_query(self, request).await
    }

    /// Convenience method: Process a query with conversation context
    pub async fn process_conversation_query(
        &self,
        query: &str,
        session_id: &str,
        conversation_history: Vec<ChatMessage>,
        date_scope: Option<String>,
    ) -> NodeSpaceResult<RAGResponse> {
        let request = RAGQueryRequest {
            query: query.to_string(),
            session_id: session_id.to_string(),
            conversation_history,
            date_scope,
            max_results: None,
        };

        RAGService::process_rag_query(self, request).await
    }

    /// Get RAG configuration
    pub fn get_rag_config(&self) -> &RAGConfig {
        &self.rag_config
    }

    /// Update RAG configuration
    pub fn update_rag_config(&mut self, config: RAGConfig) {
        self.rag_config = config;
    }
}

// Implement DateNavigation trait for ServiceContainer
#[async_trait]
impl DateNavigation for ServiceContainer {
    async fn get_nodes_for_date(&self, date: NaiveDate) -> NodeSpaceResult<Vec<Node>> {
        let date_str = date.format("%Y-%m-%d").to_string();

        // Use LanceDB data store with simplified querying
        // Query nodes that have parent_date metadata matching the date
        let all_nodes = self.data_store.query_nodes("").await?;

        let mut date_nodes = Vec::new();
        for node in all_nodes {
            if let Some(metadata) = &node.metadata {
                if let Some(parent_date) = metadata.get("parent_date") {
                    if parent_date.as_str() == Some(&date_str) {
                        date_nodes.push(node);
                    }
                }
            }
        }

        Ok(date_nodes)
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

        // Use LanceDB data store to get all nodes with parent_date metadata
        let all_nodes = self.data_store.query_nodes("").await?;

        // Extract unique parent_date values and sort them
        let mut dates: Vec<String> = Vec::new();
        for node in &all_nodes {
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

        // Use LanceDB data store to get all nodes with parent_date metadata
        let all_nodes = self.data_store.query_nodes("").await?;

        // Extract unique parent_date values and sort them ascending
        let mut dates: Vec<String> = Vec::new();
        for node in &all_nodes {
            if let Some(metadata) = &node.metadata {
                if let Some(parent_date) = metadata.get("parent_date").and_then(|v| v.as_str()) {
                    if !dates.contains(&parent_date.to_string()) {
                        dates.push(parent_date.to_string());
                    }
                }
            }
        }
        dates.sort(); // ASC order

        // Filter dates that are after current_date
        for date_str in &dates {
            if date_str > &current_str {
                if let Ok(parsed_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    return Ok(Some(parsed_date));
                }
            }
        }

        Ok(None)
    }

    async fn create_or_get_date_node(&self, date: NaiveDate) -> NodeSpaceResult<DateNode> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let description = format!("{}", date.format("%A, %B %d, %Y"));

        // Date nodes use semantic IDs (YYYY-MM-DD format)
        let date_node_id = NodeId::from(date_str.as_str());

        // Check if date node already exists in LanceDB data store
        if let Some(existing_node) = self.data_store.get_node(&date_node_id).await? {
            // Date node exists, get child count using LanceDB simplified relationship model
            let children = self.data_store.get_child_nodes(&date_node_id).await?;
            return Ok(DateNode {
                id: existing_node.id,
                date,
                description: Some(description),
                child_count: children.len(),
            });
        }

        // Create new date node using proper Node structure
        let date_node = Node::with_id(
            date_node_id.clone(),
            serde_json::json!({
                "type": "date",
                "description": description,
                "date": date_str
            }),
        );

        // Store the date node using LanceDB data store
        let stored_id = self.data_store.store_node(date_node).await?;

        Ok(DateNode {
            id: stored_id,
            date,
            description: Some(description),
            child_count: 0, // New node has no children
        })
    }

    async fn get_date_node_children(&self, date_node_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Use LanceDB data store's simplified relationship model
        // Get children using JSON-based parent_id filtering (no complex graph traversal!)
        self.data_store.get_child_nodes(date_node_id).await
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

    /// NEW: Hybrid search (semantic + structural constraints)
    async fn hybrid_search(
        &self,
        query: &str,
        node_type_filter: Option<String>,
        metadata_filter: Option<serde_json::Value>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<SearchResult>>;
}

/// Enhanced RAG orchestration service for AIChatNode functionality
#[async_trait]
pub trait RAGService: Send + Sync {
    /// Process RAG query with conversation context
    async fn process_rag_query(&self, request: RAGQueryRequest) -> NodeSpaceResult<RAGResponse>;

    /// Semantic search with conversation context awareness
    async fn semantic_search_with_context(
        &self,
        query: &str,
        conversation_context: &[ChatMessage],
        date_scope: Option<&str>,
        max_results: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>>;

    /// Assemble RAG context from retrieved nodes and conversation history
    async fn assemble_rag_context(
        &self,
        retrieved_nodes: Vec<(Node, f32)>,
        conversation_history: &[ChatMessage],
        current_query: &str,
        config: &RAGConfig,
    ) -> NodeSpaceResult<(String, TokenBudget, Vec<Node>)>;

    /// Manage token budget for conversation context
    fn calculate_token_budget(
        &self,
        conversation_history: &[ChatMessage],
        retrieved_nodes: &[(Node, f32)],
        config: &RAGConfig,
    ) -> TokenBudget;
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
        DateNavigation::get_nodes_for_date(
            self,
            date.parse()
                .map_err(|_| NodeSpaceError::ProcessingError("Invalid date format".into()))?,
        )
        .await
    }

    async fn create_text_node(&self, content: &str, date: &str) -> NodeSpaceResult<NodeId> {
        // Generate embedding for vector-first storage in LanceDB
        let embedding = self.nlp_engine.generate_embedding(content).await?;

        // Create node with metadata for simplified relationship model
        let node = Node::new(serde_json::Value::String(content.to_string()))
            .with_metadata(serde_json::json!({"parent_date": date}));

        let node_id = node.id.clone();

        // Store node with embedding using LanceDB's native vector storage
        self.data_store
            .store_node_with_embedding(node, embedding)
            .await?;

        Ok(node_id)
    }

    async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        // ENHANCED: Semantic search as primary capability (vector-first approach)
        // Generate query embedding for true semantic similarity
        let query_embedding = self.nlp_engine.generate_embedding(query).await?;

        // Direct vector search using LanceDB's native vector capabilities
        // (no complex workarounds needed!)
        let results = self
            .data_store
            .semantic_search_with_embedding(query_embedding, limit)
            .await?;

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
        // Use the enhanced RAG pipeline but convert to legacy QueryResponse format
        let rag_response = self.process_simple_query(query).await?;

        // Extract source IDs from the enhanced response
        let sources: Vec<NodeId> = rag_response
            .sources
            .iter()
            .map(|node| node.id.clone())
            .collect();

        // Generate related queries (keep original logic for compatibility)
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

        // Convert enhanced RAG response to legacy format
        Ok(QueryResponse {
            answer: rag_response.answer,
            sources,
            confidence: rag_response.relevance_score,
            related_queries,
        })
    }

    async fn add_child_node(&self, parent_id: &NodeId, child_id: &NodeId) -> NodeSpaceResult<()> {
        // Use LanceDB data store's simplified relationship creation
        // JSON-based relationship (replaces complex SurrealDB RELATE operations)
        self.data_store
            .create_relationship(parent_id, child_id, "contains")
            .await
    }

    async fn get_child_nodes(&self, parent_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Use LanceDB data store's simplified relationship model
        // Single query with JSON-based filtering (replaces complex SurrealDB graph traversal)
        self.data_store.get_child_nodes(parent_id).await
    }

    async fn update_node(&self, node_id: &NodeId, content: &str) -> NodeSpaceResult<()> {
        // Get the existing node, update its content, and store it back
        if let Some(mut node) = self.data_store.get_node(node_id).await? {
            node.content = serde_json::json!({
                "text": content,
                "type": node.content.get("type").unwrap_or(&serde_json::Value::String("text".to_string()))
            });
            node.touch();
            self.data_store.store_node(node).await?;
            Ok(())
        } else {
            Err(NodeSpaceError::NotFound(format!(
                "Node {} not found",
                node_id
            )))
        }
    }

    async fn make_siblings(
        &self,
        left_node_id: &NodeId,
        right_node_id: &NodeId,
    ) -> NodeSpaceResult<()> {
        // Validate both nodes exist using LanceDB data store
        let mut left_node = self
            .data_store
            .get_node(left_node_id)
            .await?
            .ok_or_else(|| {
                NodeSpaceError::NotFound(format!("Left node {} not found", left_node_id))
            })?;
        let mut right_node = self
            .data_store
            .get_node(right_node_id)
            .await?
            .ok_or_else(|| {
                NodeSpaceError::NotFound(format!("Right node {} not found", right_node_id))
            })?;

        // Update sibling pointers in the nodes themselves (using Node struct fields)
        left_node.next_sibling = Some(right_node_id.clone());
        left_node.touch();

        right_node.previous_sibling = Some(left_node_id.clone());
        right_node.touch();

        // Store updated nodes back to LanceDB
        self.data_store.store_node(left_node).await?;
        self.data_store.store_node(right_node).await?;

        Ok(())
    }

    async fn get_node(&self, node_id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        // Use LanceDB data store to get node directly
        self.data_store.get_node(node_id).await
    }

    async fn hybrid_search(
        &self,
        query: &str,
        node_type_filter: Option<String>,
        metadata_filter: Option<serde_json::Value>,
        limit: usize,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        // Generate query embedding for semantic component
        let query_embedding = self.nlp_engine.generate_embedding(query).await?;

        // Use LanceDB's hybrid search combining vector similarity with metadata filtering
        let results = self
            .data_store
            .hybrid_search(query_embedding, node_type_filter, metadata_filter, limit)
            .await?;

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
}

/// Enhanced RAG orchestration implementation for ServiceContainer
#[async_trait]
impl RAGService for ServiceContainer {
    async fn process_rag_query(&self, request: RAGQueryRequest) -> NodeSpaceResult<RAGResponse> {
        let start_time = std::time::Instant::now();

        // 1. Context-aware semantic search
        let retrieved_nodes = self
            .semantic_search_with_context(
                &request.query,
                &request.conversation_history,
                request.date_scope.as_deref(),
                request
                    .max_results
                    .unwrap_or(self.rag_config.max_retrieval_results),
            )
            .await?;

        // 2. Assemble RAG context with token management
        let (context_prompt, token_budget, sources) = self
            .assemble_rag_context(
                retrieved_nodes,
                &request.conversation_history,
                &request.query,
                &self.rag_config,
            )
            .await?;

        // 3. Generate AI response
        let answer = self.nlp_engine.generate_text(&context_prompt).await?;

        let generation_time_ms = start_time.elapsed().as_millis() as u64;

        // 4. Create comprehensive response
        let relevance_score = if sources.is_empty() { 0.0 } else { 0.8 };
        let context_summary = if sources.is_empty() {
            "No relevant knowledge sources found".to_string()
        } else {
            format!("Used {} knowledge sources from your notes", sources.len())
        };

        Ok(RAGResponse {
            answer,
            sources,
            context_summary,
            relevance_score,
            context_tokens: token_budget.tokens_used(),
            generation_time_ms,
            conversation_context_used: token_budget.conversation_history,
        })
    }

    async fn semantic_search_with_context(
        &self,
        query: &str,
        conversation_context: &[ChatMessage],
        date_scope: Option<&str>,
        max_results: usize,
    ) -> NodeSpaceResult<Vec<(Node, f32)>> {
        // Enhance query with conversation context
        let enhanced_query = if conversation_context.is_empty() {
            query.to_string()
        } else {
            // Get recent context from conversation
            let recent_context: Vec<String> = conversation_context
                .iter()
                .rev()
                .take(3)
                .filter_map(|msg| {
                    if msg.role == MessageRole::User {
                        Some(msg.content.clone())
                    } else {
                        None
                    }
                })
                .collect();

            if recent_context.is_empty() {
                query.to_string()
            } else {
                format!(
                    "{}\n\nRecent conversation context: {}",
                    query,
                    recent_context.join("; ")
                )
            }
        };

        // Perform semantic search
        let search_results = self.semantic_search(&enhanced_query, max_results).await?;

        // Filter by date scope if provided
        let filtered_results: Vec<(Node, f32)> = if let Some(date_filter) = date_scope {
            search_results
                .into_iter()
                .filter(|result| {
                    if let Some(metadata) = &result.node.metadata {
                        if let Some(parent_date) = metadata.get("parent_date") {
                            parent_date.as_str() == Some(date_filter)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                })
                .map(|result| (result.node, result.score))
                .collect()
        } else {
            search_results
                .into_iter()
                .map(|result| (result.node, result.score))
                .collect()
        };

        // Apply relevance threshold
        let relevant_results: Vec<(Node, f32)> = filtered_results
            .into_iter()
            .filter(|(_, score)| *score >= self.rag_config.relevance_threshold)
            .collect();

        Ok(relevant_results)
    }

    async fn assemble_rag_context(
        &self,
        retrieved_nodes: Vec<(Node, f32)>,
        conversation_history: &[ChatMessage],
        current_query: &str,
        config: &RAGConfig,
    ) -> NodeSpaceResult<(String, TokenBudget, Vec<Node>)> {
        // Calculate token budget
        let mut token_budget =
            self.calculate_token_budget(conversation_history, &retrieved_nodes, config);

        // Prioritize and truncate sources to fit budget
        let mut selected_sources = Vec::new();
        let mut knowledge_tokens = 0;

        for (node, _score) in retrieved_nodes.iter() {
            if let Some(content) = node.content.as_str() {
                let estimated_tokens = content.len() / 4; // Rough token estimation
                if knowledge_tokens + estimated_tokens <= token_budget.available_for_context() / 2 {
                    selected_sources.push(node.clone());
                    knowledge_tokens += estimated_tokens;
                } else {
                    break; // Stop when we hit budget limit
                }
            }
        }

        token_budget.allocate_knowledge_tokens(knowledge_tokens);

        // Build conversation context within remaining budget
        let remaining_tokens = token_budget.available_for_context() - knowledge_tokens;
        let recent_messages: Vec<&ChatMessage> = conversation_history
            .iter()
            .rev()
            .take(config.conversation_context_limit)
            .take_while(|msg| {
                let estimated_tokens = msg.content.len() / 4;
                estimated_tokens <= remaining_tokens
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        let conversation_tokens: usize = recent_messages
            .iter()
            .map(|msg| msg.content.len() / 4)
            .sum();
        token_budget.allocate_conversation_tokens(conversation_tokens);

        // Assemble the final prompt
        let mut prompt_parts = Vec::new();

        // System prompt
        prompt_parts.push("You are a helpful AI assistant that answers questions based on the user's knowledge base and conversation context.".to_string());

        // Add conversation context if available
        if !recent_messages.is_empty() {
            prompt_parts.push("\n\n## Recent Conversation:".to_string());
            for msg in recent_messages {
                let role = match msg.role {
                    MessageRole::User => "User",
                    MessageRole::Assistant => "Assistant",
                    MessageRole::System => "System",
                };
                prompt_parts.push(format!("{}: {}", role, msg.content));
            }
        }

        // Add knowledge context if available
        if !selected_sources.is_empty() {
            prompt_parts.push("\n\n## Relevant Knowledge:".to_string());
            for (i, source) in selected_sources.iter().enumerate() {
                if let Some(content) = source.content.as_str() {
                    prompt_parts.push(format!("Source {}: {}", i + 1, content));
                }
            }
        }

        // Add the current query
        prompt_parts.push(format!("\n\n## Current Question:\n{}", current_query));

        // Final instruction
        prompt_parts.push("\n\nPlease provide a helpful response based on the above context and conversation history.".to_string());

        let final_prompt = prompt_parts.join("\n");

        Ok((final_prompt, token_budget, selected_sources))
    }

    fn calculate_token_budget(
        &self,
        conversation_history: &[ChatMessage],
        retrieved_nodes: &[(Node, f32)],
        config: &RAGConfig,
    ) -> TokenBudget {
        // Use model's context window from performance config or default
        let total_tokens = self
            .config
            .performance_config
            .context_window
            .unwrap_or(4096);
        let reserved_tokens = config.reserved_response_tokens;

        let mut budget = TokenBudget::new(total_tokens, reserved_tokens);

        // Estimate conversation tokens
        let conversation_tokens: usize = conversation_history
            .iter()
            .map(|msg| msg.content.len() / 4) // Rough token estimation
            .sum();

        // Estimate knowledge tokens
        let knowledge_tokens: usize = retrieved_nodes
            .iter()
            .filter_map(|(node, _)| node.content.as_str())
            .map(|content| content.len() / 4)
            .sum();

        budget.allocate_conversation_tokens(
            conversation_tokens.min(budget.available_for_context() / 2),
        );
        budget.allocate_knowledge_tokens(knowledge_tokens.min(budget.available_for_context() / 2));

        budget
    }
}
