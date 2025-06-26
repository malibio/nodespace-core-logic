use async_trait::async_trait;
use chrono::NaiveDate;
use nodespace_core_types::{Node, NodeId, NodeSpaceError, NodeSpaceResult};
use nodespace_data_store::DataStore; // Only for trait bounds, not direct usage
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
    pub max_retrieval_results: usize,     // Default: 5
    pub relevance_threshold: f32,         // Default: 0.7
    pub max_context_tokens: usize,        // Default: 2048
    pub conversation_context_limit: usize, // Default: 5 messages
    pub reserved_response_tokens: usize,  // Default: 512
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
    pub total_available: usize,        // Model's context window
    pub reserved_for_response: usize,  // 512 tokens for response
    pub conversation_history: usize,   // Recent chat context
    pub knowledge_context: usize,      // Retrieved information
    pub system_prompt: usize,          // Instructions
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
        self.total_available.saturating_sub(self.tokens_used() + self.reserved_for_response)
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
/// Uses internal storage and coordinates with external services via configuration only
pub struct ServiceContainer {
    // Internal storage - no direct external data-store dependencies
    internal_storage: Arc<RwLock<std::collections::HashMap<NodeId, Node>>>,
    date_index: Arc<RwLock<std::collections::HashMap<String, Vec<NodeId>>>>,
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
}

impl ServiceContainer {
    /// Create a new ServiceContainer with the shared database path
    /// This ensures both core-logic and data-store use the same SurrealDB instance
    pub async fn new() -> Result<Self, InitializationError> {
        Self::new_with_config(NodeSpaceConfig::default()).await
    }

    /// Create ServiceContainer with custom configuration
    pub async fn new_with_config(config: NodeSpaceConfig) -> Result<Self, InitializationError> {
        // Initialize NLP engine with shared model configuration
        let nlp_engine = LocalNLPEngine::new();
        nlp_engine.initialize().await?;

        // Create service container with internal storage
        let state = Arc::new(RwLock::new(ServiceState::Ready));

        Ok(ServiceContainer {
            internal_storage: Arc::new(RwLock::new(std::collections::HashMap::new())),
            date_index: Arc::new(RwLock::new(std::collections::HashMap::new())),
            nlp_engine,
            config,
            rag_config: RAGConfig::default(),
            state,
        })
    }

    /// Get internal storage size for debugging
    pub async fn storage_size(&self) -> usize {
        self.internal_storage.read().await.len()
    }

    /// Internal helper: Store a node in internal storage
    async fn store_node_internal(&self, node: Node) -> NodeSpaceResult<NodeId> {
        let node_id = node.id.clone();
        
        // Store the node
        self.internal_storage.write().await.insert(node_id.clone(), node.clone());
        
        // Update date index if this is a date-associated node  
        // Extract date from ISO timestamp (YYYY-MM-DDTHH:MM:SS format)
        let date_str = node.created_at.split('T').next().unwrap_or("").to_string();
        if !date_str.is_empty() {
            self.date_index.write().await
                .entry(date_str)
                .or_insert_with(Vec::new)
                .push(node_id.clone());
        }
        
        Ok(node_id)
    }

    /// Internal helper: Get a node from internal storage
    async fn get_node_internal(&self, node_id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        Ok(self.internal_storage.read().await.get(node_id).cloned())
    }

    /// Internal helper: Query nodes (placeholder - returns all for now)
    async fn query_nodes_internal(&self, _query: &str) -> NodeSpaceResult<Vec<Node>> {
        // Placeholder: return all nodes for now
        // In real implementation, this would do proper querying
        Ok(self.internal_storage.read().await.values().cloned().collect())
    }

    // search_similar_nodes not needed for MVP - removed

    /// Internal helper: Get child nodes using parent_id relationships
    async fn get_child_nodes_internal(&self, parent_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        let storage = self.internal_storage.read().await;
        let mut children = Vec::new();
        
        // Look through all nodes to find ones with this parent_id
        for node in storage.values() {
            // Check if this node has the parent_id in its metadata/content
            if let Some(metadata) = &node.metadata {
                if let Some(parent) = metadata.get("parent_id") {
                    if parent.as_str() == Some(parent_id.as_str()) {
                        children.push(node.clone());
                    }
                }
            }
            // Also check content for parent_id
            if let Some(parent) = node.content.get("parent_id") {
                if parent.as_str() == Some(parent_id.as_str()) {
                    children.push(node.clone());
                }
            }
        }
        
        Ok(children)
    }

    /// Internal helper: Update a node's content
    async fn update_node_internal(&self, node_id: &NodeId, content: &str) -> NodeSpaceResult<()> {
        let mut storage = self.internal_storage.write().await;
        
        if let Some(node) = storage.get_mut(node_id) {
            node.content = serde_json::json!({
                "text": content,
                "type": node.content.get("type").unwrap_or(&serde_json::Value::String("text".to_string()))
            });
            node.touch();
            Ok(())
        } else {
            Err(NodeSpaceError::NotFound(format!("Node {} not found", node_id)))
        }
    }

    /// Internal helper: Delete a node
    async fn delete_node_internal(&self, node_id: &NodeId) -> NodeSpaceResult<()> {
        self.internal_storage.write().await.remove(node_id);
        
        // Also remove from date index
        let mut date_index = self.date_index.write().await;
        for (_, node_ids) in date_index.iter_mut() {
            node_ids.retain(|id| id != node_id);
        }
        
        Ok(())
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

        // Use internal storage with date index
        let date_index = self.date_index.read().await;
        let storage = self.internal_storage.read().await;
        
        if let Some(node_ids) = date_index.get(&date_str) {
            let mut nodes = Vec::new();
            for node_id in node_ids {
                if let Some(node) = storage.get(node_id) {
                    nodes.push(node.clone());
                }
            }
            Ok(nodes)
        } else {
            Ok(Vec::new())
        }
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
        let result_nodes = self.query_nodes_internal(query).await?;

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
        let result_nodes = self.query_nodes_internal(query).await?;

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
        
        // Date nodes use semantic IDs (YYYY-MM-DD format)
        let date_node_id = NodeId::from(date_str.as_str());
        
        // Check if date node already exists in internal storage
        if let Some(existing_node) = self.get_node_internal(&date_node_id).await? {
            // Date node exists, get child count
            let children = self.get_child_nodes(&date_node_id).await?;
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
            })
        );
        
        // Store the date node
        let stored_id = self.store_node_internal(date_node).await?;
        
        Ok(DateNode {
            id: stored_id,
            date,
            description: Some(description),
            child_count: 0, // New node has no children
        })
    }

    async fn get_date_node_children(&self, date_node_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Use the date_node_id as the date string for the specialized DataStore method
        let date_str = date_node_id.to_string();

        // Get children using the specialized DataStore method
        // Use proper universal schema pattern: get date node, then its children
        let date_node_id = NodeId::from(date_str.as_str()); // Date nodes use semantic YYYY-MM-DD IDs  
        let children_json = self.get_child_nodes(&date_node_id).await?;

        // children_json is already Vec<Node> from get_child_nodes
        let nodes = children_json;

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
        DateNavigation::get_nodes_for_date(self, date.parse().map_err(|_| NodeSpaceError::ProcessingError("Invalid date format".into()))?).await
    }

    async fn create_text_node(&self, content: &str, date: &str) -> NodeSpaceResult<NodeId> {
        // Generate embedding first
        let _embedding = self.nlp_engine.generate_embedding(content).await?;

        // Create node with embedding and metadata
        let node = Node::new(serde_json::Value::String(content.to_string()))
            .with_metadata(serde_json::json!({"parent_date": date}));

        let node_id = node.id.clone();

        // Store node with embedding using the data store's method
        // Store node with embedding in internal storage (embedding not used for now)
        self.store_node_internal(node).await?;

        Ok(node_id)
    }

    async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        // Generate query embedding
        let _query_embedding = self.nlp_engine.generate_embedding(query).await?;

        // For MVP: Simple text search through internal storage
        let storage = self.internal_storage.read().await;
        let search_results = storage
            .values()
            .filter(|node| {
                // Simple text matching in content
                if let Some(text) = node.content.as_str() {
                    text.to_lowercase().contains(&query.to_lowercase())
                } else if let Some(text) = node.content.get("text") {
                    text.as_str().unwrap_or("").to_lowercase().contains(&query.to_lowercase())
                } else {
                    false
                }
            })
            .take(limit)
            .map(|node| SearchResult {
                node_id: node.id.clone(),
                node: node.clone(),
                score: 0.8, // Placeholder score
            })
            .collect();

        Ok(search_results)
    }

    async fn process_query(&self, query: &str) -> NodeSpaceResult<QueryResponse> {
        // Use the enhanced RAG pipeline but convert to legacy QueryResponse format
        let rag_response = self.process_simple_query(query).await?;

        // Extract source IDs from the enhanced response
        let sources: Vec<NodeId> = rag_response.sources
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
        // Update child node to reference parent in universal schema
        let mut storage = self.internal_storage.write().await;
        
        if let Some(child_node) = storage.get_mut(child_id) {
            // Add parent_id to child's content or metadata
            if let Some(content_obj) = child_node.content.as_object_mut() {
                content_obj.insert("parent_id".to_string(), serde_json::Value::String(parent_id.to_string()));
            } else {
                // If content is not an object, wrap it
                let old_content = child_node.content.clone();
                child_node.content = serde_json::json!({
                    "content": old_content,
                    "parent_id": parent_id.to_string()
                });
            }
            child_node.touch();
        }
        
        Ok(())
    }

    async fn get_child_nodes(&self, parent_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Use the DataStore method to get children via relationships
        // For date nodes, use get_date_children; for other nodes, use a query
        let _parent_str = parent_id.to_string();

        // Use internal helper for all nodes (dates and others)
        self.get_child_nodes_internal(parent_id).await
    }

    async fn update_node(&self, node_id: &NodeId, content: &str) -> NodeSpaceResult<()> {
        // Use internal helper to update node
        self.update_node_internal(node_id, content).await
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
            .get_node_internal(left_node_id)
            .await?
            .ok_or_else(|| {
                NodeSpaceError::NotFound(format!("Left node {} not found", left_node_id))
            })?;
        let _right_node = self
            .get_node_internal(right_node_id)
            .await?
            .ok_or_else(|| {
                NodeSpaceError::NotFound(format!("Right node {} not found", right_node_id))
            })?;

        // Update sibling pointers in the nodes themselves (using Node struct fields)
        let mut storage = self.internal_storage.write().await;
        
        if let Some(left_node) = storage.get_mut(left_node_id) {
            left_node.next_sibling = Some(right_node_id.clone());
            left_node.touch();
        }
        
        if let Some(right_node) = storage.get_mut(right_node_id) {
            right_node.previous_sibling = Some(left_node_id.clone());
            right_node.touch();
        }

        Ok(())
    }

    async fn get_node(&self, node_id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        self.get_node_internal(node_id).await
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
                request.max_results.unwrap_or(self.rag_config.max_retrieval_results),
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
                format!("{}\n\nRecent conversation context: {}", query, recent_context.join("; "))
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
        let mut token_budget = self.calculate_token_budget(conversation_history, &retrieved_nodes, config);

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

        let conversation_tokens: usize = recent_messages.iter().map(|msg| msg.content.len() / 4).sum();
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
        let total_tokens = self.config.performance_config.context_window.unwrap_or(4096);
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

        budget.allocate_conversation_tokens(conversation_tokens.min(budget.available_for_context() / 2));
        budget.allocate_knowledge_tokens(knowledge_tokens.min(budget.available_for_context() / 2));

        budget
    }
}
