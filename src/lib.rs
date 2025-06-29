use async_trait::async_trait;
use chrono::NaiveDate;
use nodespace_core_types::{Node, NodeId, NodeSpaceError, NodeSpaceResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// Import traits from their respective repositories
pub use nodespace_data_store::DataStore;
pub use nodespace_nlp_engine::NLPEngine;

// Import additional types for embedding generation bridge
use nodespace_data_store::DataStoreError;
use nodespace_data_store::EmbeddingGenerator as DataStoreEmbeddingGenerator;

/// Adapter that bridges NLPEngine to DataStore's EmbeddingGenerator trait
/// This allows the data store to automatically generate embeddings using the NLP engine
pub struct NLPEmbeddingAdapter<N: NLPEngine> {
    nlp_engine: N,
}

impl<N: NLPEngine> NLPEmbeddingAdapter<N> {
    pub fn new(nlp_engine: N) -> Self {
        Self { nlp_engine }
    }
}

#[async_trait]
impl<N: NLPEngine + Send + Sync> DataStoreEmbeddingGenerator for NLPEmbeddingAdapter<N> {
    async fn generate_embedding(&self, content: &str) -> Result<Vec<f32>, DataStoreError> {
        // Bridge to the NLPEngine trait implementation
        match self.nlp_engine.generate_embedding(content).await {
            Ok(embedding) => Ok(embedding),
            Err(e) => Err(DataStoreError::EmbeddingError(format!(
                "NLP engine embedding failed: {}",
                e
            ))),
        }
    }
}

/// Simple performance monitoring (compatible with all dependencies)
pub mod monitoring {
    use std::time::Instant;

    /// Simple performance timer for operations
    pub struct OperationTimer {
        _operation_name: String,
        _start_time: Instant,
    }

    impl OperationTimer {
        pub fn new(operation_name: &str) -> Self {
            Self {
                _operation_name: operation_name.to_string(),
                _start_time: Instant::now(),
            }
        }

        /// Add metadata (no-op for compatibility)
        pub fn with_metadata(self, _key: String, _value: String) -> Self {
            self
        }

        /// Complete operation successfully (no-op for compatibility)
        pub fn complete_success(self) {
            // No-op for now to avoid dependency conflicts
        }

        /// Complete operation with error (no-op for compatibility)
        pub fn complete_error(self, _error: String) {
            // No-op for now to avoid dependency conflicts
        }
    }

    /// Simple performance monitor (stub for compatibility)
    #[derive(Debug, Default)]
    pub struct PerformanceMonitor;

    impl PerformanceMonitor {
        pub fn new() -> Self {
            Self
        }

        /// Start timing an operation
        pub fn start_operation(&self, operation_name: &str) -> OperationTimer {
            OperationTimer::new(operation_name)
        }
    }
}

/// Configuration constants for NodeSpace service
pub mod constants {
    /// Default embedding model
    pub const DEFAULT_EMBEDDING_MODEL: &str = "sentence-transformers/all-MiniLM-L6-v2";
    /// Default text generation model
    pub const DEFAULT_TEXT_MODEL: &str = "mistralai/Mistral-7B-Instruct-v0.1";
    /// Default download timeout in seconds
    pub const DEFAULT_DOWNLOAD_TIMEOUT: u64 = 300;
    /// Default maximum batch size for operations
    pub const DEFAULT_MAX_BATCH_SIZE: usize = 32;
    /// Default context window size
    pub const DEFAULT_CONTEXT_WINDOW: usize = 4096;
    /// Default temperature for text generation
    pub const DEFAULT_TEMPERATURE: f32 = 0.7;
    /// Default search limit for semantic search
    pub const DEFAULT_SEARCH_LIMIT: usize = 5;
    /// Default search limit for multi-strategy search
    pub const DEFAULT_MULTI_STRATEGY_LIMIT: usize = 20;
    /// Default maximum results per strategy
    pub const DEFAULT_MAX_RESULTS_PER_STRATEGY: usize = 10;
    /// Default final result limit
    pub const DEFAULT_FINAL_RESULT_LIMIT: usize = 10;
    /// Score decay factor for search results
    pub const SCORE_DECAY_FACTOR: f32 = 0.1;
    /// Minimum search score
    pub const MIN_SEARCH_SCORE: f32 = 0.1;
    /// Base confidence for queries with context
    pub const BASE_CONFIDENCE_WITH_CONTEXT: f32 = 0.8;
    /// Base confidence for queries without context
    pub const BASE_CONFIDENCE_NO_CONTEXT: f32 = 0.3;
    /// Confidence reduction factor for fallback responses
    pub const FALLBACK_CONFIDENCE_FACTOR: f32 = 0.5;
    /// Default embedding dimension (for fallback)
    pub const DEFAULT_EMBEDDING_DIMENSION: usize = 768;
    /// Reserved space for prompt structure in context window
    pub const PROMPT_STRUCTURE_RESERVE: usize = 200;
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
                embedding_model: Some(constants::DEFAULT_EMBEDDING_MODEL.to_string()),
                text_model: Some(constants::DEFAULT_TEXT_MODEL.to_string()),
                download_timeout: Some(constants::DEFAULT_DOWNLOAD_TIMEOUT),
                cache_dir: None, // Use system default
            },
            performance_config: PerformanceConfig {
                max_batch_size: Some(constants::DEFAULT_MAX_BATCH_SIZE),
                context_window: Some(constants::DEFAULT_CONTEXT_WINDOW),
                temperature: Some(constants::DEFAULT_TEMPERATURE),
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
    performance_monitor: monitoring::PerformanceMonitor,
    hierarchy_cache: Arc<RwLock<HierarchyCache>>,
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
            performance_monitor: monitoring::PerformanceMonitor::new(),
            hierarchy_cache: Arc::new(RwLock::new(HierarchyCache::new())),
        }
    }

    /// Get performance monitor for metrics access
    pub fn performance_monitor(&self) -> &monitoring::PerformanceMonitor {
        &self.performance_monitor
    }
}

impl NodeSpaceService<nodespace_data_store::LanceDataStore, nodespace_nlp_engine::LocalNLPEngine> {
    /// Factory method for service container dependency injection
    /// Creates a fully configured NodeSpace service with database and model paths
    pub async fn create_with_paths(
        database_path: &str,
        model_directory: Option<&str>,
    ) -> NodeSpaceResult<Self> {
        use nodespace_data_store::LanceDataStore;
        use nodespace_nlp_engine::LocalNLPEngine;

        // Initialize NLP engine with optional model directory
        let nlp_engine1 = if let Some(model_dir) = model_directory {
            LocalNLPEngine::with_model_directory(model_dir)
        } else {
            LocalNLPEngine::new() // Uses smart path resolution
        };

        // Create a second NLP engine instance for the adapter
        let nlp_engine2 = if let Some(model_dir) = model_directory {
            LocalNLPEngine::with_model_directory(model_dir)
        } else {
            LocalNLPEngine::new() // Uses smart path resolution
        };

        // Initialize data store with injected database path
        let mut data_store = LanceDataStore::new(database_path).await.map_err(|e| {
            NodeSpaceError::InternalError(format!(
                "Failed to initialize data store at '{}': {}",
                database_path, e
            ))
        })?;

        // Create an adapter to bridge NLP engine to data store's embedding generator interface
        let embedding_adapter = NLPEmbeddingAdapter::new(nlp_engine2);

        // Set the adapter as the embedding generator for automatic embedding handling
        data_store.set_embedding_generator(Box::new(embedding_adapter));

        Ok(Self::new(data_store, nlp_engine1))
    }

    /// Factory method for development environment
    /// Uses default development paths
    pub async fn create_for_development() -> NodeSpaceResult<Self> {
        Self::create_with_paths("../data/lance_db/development.db", Some("../models")).await
    }

    /// Factory method for testing environment
    /// Uses in-memory database and test model paths
    pub async fn create_for_testing() -> NodeSpaceResult<Self> {
        Self::create_with_paths(
            "memory", None, // Use smart path resolution for tests
        )
        .await
    }

    /// Factory method for production environment
    /// Uses environment variables or explicit paths
    pub async fn create_for_production(
        database_path: &str,
        model_directory: &str,
    ) -> NodeSpaceResult<Self> {
        Self::create_with_paths(database_path, Some(model_directory)).await
    }
}

impl<D: DataStore, N: NLPEngine> NodeSpaceService<D, N> {
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

    /// Update node hierarchical relationships and sibling ordering
    async fn update_node_structure(
        &self,
        node_id: &NodeId,
        operation: &str,
        target_parent_id: Option<&NodeId>,
        previous_sibling_id: Option<&NodeId>,
    ) -> NodeSpaceResult<()>;

    /// Set node's parent (for indent/outdent operations)
    async fn set_node_parent(
        &self,
        node_id: &NodeId,
        parent_id: Option<&NodeId>,
    ) -> NodeSpaceResult<()>;

    /// Update sibling order (for move up/down operations)
    async fn update_sibling_order(
        &self,
        node_id: &NodeId,
        previous_sibling_id: Option<&NodeId>,
        next_sibling_id: Option<&NodeId>,
    ) -> NodeSpaceResult<()>;

    /// Batch lookup for multiple node relationships (optimization for N+1 queries)
    async fn get_batch_related_nodes(
        &self,
        node_ids: &[NodeId],
        relationship_types: Vec<String>,
    ) -> NodeSpaceResult<std::collections::HashMap<NodeId, Vec<NodeId>>>;
}

/// Cross-modal search orchestration interface for complex queries
#[async_trait]
pub trait CrossModalSearch: Send + Sync {
    /// Intelligent cross-modal search with entity and temporal extraction
    async fn intelligent_cross_modal_search(
        &self,
        query: &str,
    ) -> NodeSpaceResult<Vec<SearchResult>>;

    /// Extract entities from natural language queries
    async fn extract_entities(&self, query: &str) -> NodeSpaceResult<ExtractedEntities>;

    /// Extract temporal references from queries
    async fn extract_temporal_refs(&self, query: &str) -> NodeSpaceResult<Vec<TemporalReference>>;

    /// Extract visual attributes from queries
    async fn extract_visual_refs(&self, query: &str) -> NodeSpaceResult<VisualAttributes>;

    /// Multi-strategy search coordination
    async fn multi_strategy_search(
        &self,
        query_embedding: Vec<f32>,
        entities: &ExtractedEntities,
        temporal_refs: &[TemporalReference],
        visual_refs: &VisualAttributes,
    ) -> NodeSpaceResult<Vec<SearchResult>>;

    /// Intelligent result fusion and ranking
    async fn intelligent_result_fusion(
        &self,
        search_results: Vec<SearchResult>,
        original_query: &str,
    ) -> NodeSpaceResult<Vec<SearchResult>>;
}

/// Date navigation operations for hierarchical date-based content organization
#[async_trait]
pub trait DateNavigation: Send + Sync {
    /// Get all text nodes for a specific date
    async fn get_nodes_for_date(&self, date: NaiveDate) -> NodeSpaceResult<Vec<Node>>;

    /// Navigate to a specific date with navigation context
    async fn navigate_to_date(&self, date: NaiveDate) -> NodeSpaceResult<NavigationResult>;

    /// Get today's date for navigation
    fn get_today() -> NaiveDate {
        chrono::Utc::now().date_naive()
    }
}

/// Hierarchy computation operations for runtime node relationships
#[async_trait]
pub trait HierarchyComputation: Send + Sync {
    /// Compute node depth by traversing parent chain
    async fn get_node_depth(&self, node_id: &NodeId) -> NodeSpaceResult<u32>;

    /// Get direct children of a parent node
    async fn get_children(&self, parent_id: &NodeId) -> NodeSpaceResult<Vec<Node>>;

    /// Get all ancestors of a node (parent, grandparent, etc.)
    async fn get_ancestors(&self, node_id: &NodeId) -> NodeSpaceResult<Vec<Node>>;

    /// Get all siblings of a node (same parent)
    async fn get_siblings(&self, node_id: &NodeId) -> NodeSpaceResult<Vec<Node>>;

    /// Move a node to a new parent (validates hierarchy constraints)
    async fn move_node(&self, node_id: &NodeId, new_parent: &NodeId) -> NodeSpaceResult<()>;

    /// Move an entire subtree to a new parent
    async fn move_subtree(&self, root_id: &NodeId, new_parent: &NodeId) -> NodeSpaceResult<()>;

    /// Get a subtree with computed depths for each node
    async fn get_subtree_with_depths(&self, root_id: &NodeId) -> NodeSpaceResult<Vec<(Node, u32)>>;

    /// Validate that a hierarchy move operation is legal (no cycles, valid targets)
    async fn validate_hierarchy_move(
        &self,
        node_id: &NodeId,
        new_parent: &NodeId,
    ) -> NodeSpaceResult<()>;

    /// Invalidate hierarchy cache (call after structural changes)
    async fn invalidate_hierarchy_cache(&self);
}

/// Search result with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub node_id: NodeId,
    pub node: Node,
    pub score: f32,
}

/// Cross-modal search entities extracted from queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntities {
    pub people: Vec<String>,
    pub events: Vec<String>,
    pub objects: Vec<String>,
    pub locations: Vec<String>,
}

/// Temporal references extracted from queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalReference {
    pub raw_text: String,
    pub parsed_date: Option<NaiveDate>,
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    pub temporal_type: TemporalType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalType {
    Exact,    // "on June 15"
    Relative, // "yesterday", "last week"
    Event,    // "during Claire's birthday"
    Fuzzy,    // "around that time"
}

/// Visual attributes extracted from queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualAttributes {
    pub colors: Vec<String>,
    pub objects: Vec<String>,
    pub scene_types: Vec<String>,
    pub people_descriptions: Vec<String>,
}

/// Multi-strategy search configuration for intelligent fusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiStrategyConfig {
    pub semantic_weight: f32,
    pub entity_weight: f32,
    pub temporal_weight: f32,
    pub visual_weight: f32,
    pub max_results_per_strategy: usize,
    pub final_result_limit: usize,
}

/// Query response with results and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub answer: String,
    pub sources: Vec<NodeId>,
    pub confidence: f32,
    pub related_queries: Vec<String>,
}

/// Navigation result for date-based navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResult {
    pub date: NaiveDate,
    pub nodes: Vec<Node>,
    pub has_previous: bool,
    pub has_next: bool,
}

#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> CoreLogic for NodeSpaceService<D, N> {
    async fn create_knowledge_node(
        &self,
        content: &str,
        metadata: serde_json::Value,
    ) -> NodeSpaceResult<NodeId> {
        let timer = self
            .performance_monitor
            .start_operation("create_knowledge_node")
            .with_metadata("content_length".to_string(), content.len().to_string());

        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            let error = NodeSpaceError::InternalError(format!("Service not ready: {:?}", state));
            timer.complete_error(error.to_string());
            return Err(error);
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
            parent_id: None,
            next_sibling: None,
            previous_sibling: None,
        };

        // Store the node - data store will automatically generate embeddings using the EmbeddingGenerator
        self.data_store.store_node(node).await?;

        timer.complete_success();
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

        // For LanceDB: Simple content search using query_nodes
        let all_nodes = self.data_store.query_nodes(query).await?;
        let nodes: Vec<_> = all_nodes.into_iter().take(effective_limit).collect();

        // Convert to SearchResult with basic scoring
        let mut results = Vec::new();
        for (index, node) in nodes.into_iter().enumerate() {
            // Simple scoring based on position (higher position = lower score)
            let score = 1.0 - (index as f32 * constants::SCORE_DECAY_FACTOR);

            results.push(SearchResult {
                node_id: node.id.clone(),
                node,
                score: score.max(constants::MIN_SEARCH_SCORE),
            });
        }

        Ok(results)
    }

    async fn process_query(&self, query: &str) -> NodeSpaceResult<QueryResponse> {
        let timer = self
            .performance_monitor
            .start_operation("process_query")
            .with_metadata("query_length".to_string(), query.len().to_string());

        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            let error = NodeSpaceError::InternalError(format!("Service not ready: {:?}", state));
            timer.complete_error(error.to_string());
            return Err(error);
        }

        // Step 1: Gather context from semantic search
        let (context, sources) = self.gather_query_context(query).await?;

        // Step 2: Build and execute prompt
        let prompt = self.build_contextual_prompt(query, &context);
        let answer = self.generate_contextual_answer(&prompt, &sources).await?;

        // Step 3: Calculate confidence and generate suggestions
        let confidence = self.calculate_response_confidence(&context, &answer);
        let related_queries = self.generate_related_queries(query);

        let response = QueryResponse {
            answer,
            sources,
            confidence,
            related_queries,
        };

        timer.complete_success();
        Ok(response)
    }

    async fn get_batch_related_nodes(
        &self,
        node_ids: &[NodeId],
        relationship_types: Vec<String>,
    ) -> NodeSpaceResult<std::collections::HashMap<NodeId, Vec<NodeId>>> {
        // Use the implementation from the impl block
        NodeSpaceService::get_batch_related_nodes(self, node_ids, relationship_types).await
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

        // Use the data store's update method which handles embedding regeneration automatically
        // The LanceDB data store now detects content changes and regenerates embeddings as needed
        self.data_store.update_node(node).await?;

        Ok(())
    }

    async fn get_related_nodes(
        &self,
        node_id: &NodeId,
        relationship_types: Vec<String>,
    ) -> NodeSpaceResult<Vec<NodeId>> {
        // For MVP, use a simple query to find related nodes
        let _relationship_filters = if relationship_types.is_empty() {
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

        // Optimized relationship lookup using batch operations
        // Instead of loading all nodes, we can use targeted queries
        let related_nodes = if relationship_types.is_empty() {
            // Search for any mentions of the target node ID
            self.data_store
                .query_nodes(&node_id.to_string())
                .await
                .unwrap_or_default()
        } else {
            // For specific relationship types, we'd need more sophisticated querying
            // For now, fall back to the general approach but cache results
            let all_nodes = self.data_store.query_nodes("").await.unwrap_or_default();
            all_nodes
                .into_iter()
                .filter(|node| {
                    // Check if this node has a relationship to our target node
                    if let Some(metadata) = &node.metadata {
                        if let Some(mentions) = metadata.get("mentions") {
                            if let Some(mentions_array) = mentions.as_array() {
                                return mentions_array
                                    .iter()
                                    .any(|mention| mention.as_str() == Some(node_id.as_str()));
                            }
                        }
                        // Also check for specific relationship types in metadata
                        for rel_type in &relationship_types {
                            if let Some(relationships) = metadata.get("relationships") {
                                if let Some(rel_obj) = relationships.as_object() {
                                    if let Some(targets) = rel_obj.get(rel_type) {
                                        if let Some(targets_array) = targets.as_array() {
                                            if targets_array
                                                .iter()
                                                .any(|t| t.as_str() == Some(node_id.as_str()))
                                            {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    false
                })
                .collect()
        };

        let related_ids: Vec<NodeId> = related_nodes.into_iter().map(|node| node.id).collect();

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

    /// Update node hierarchical relationships and sibling ordering
    async fn update_node_structure(
        &self,
        node_id: &NodeId,
        operation: &str,
        target_parent_id: Option<&NodeId>,
        previous_sibling_id: Option<&NodeId>,
    ) -> NodeSpaceResult<()> {
        let mut node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::NotFound(format!("Node {} not found", node_id)))?;

        match operation {
            "indent" => {
                // Make node a child of target parent
                if let Some(parent_id) = target_parent_id {
                    let mut metadata = node.metadata.unwrap_or_else(|| serde_json::json!({}));
                    metadata["parent_id"] = serde_json::Value::String(parent_id.to_string());
                    node.metadata = Some(metadata);

                    // Update sibling relationships if provided
                    node.previous_sibling = previous_sibling_id.cloned();
                    node.next_sibling = None; // Will be updated by subsequent operations if needed
                }
            }
            "outdent" => {
                // Remove parent relationship
                if let Some(metadata) = &mut node.metadata {
                    metadata
                        .as_object_mut()
                        .and_then(|obj| obj.remove("parent_id"));
                }

                // Reset sibling relationships
                node.previous_sibling = previous_sibling_id.cloned();
                node.next_sibling = None;
            }
            "move_up" | "move_down" => {
                // Update sibling order without changing parent
                node.previous_sibling = previous_sibling_id.cloned();
                // next_sibling will be handled by update_sibling_order if needed
            }
            "reorder" => {
                // Move to specific position in sibling list
                node.previous_sibling = previous_sibling_id.cloned();
            }
            _ => {
                return Err(NodeSpaceError::ValidationError(format!(
                    "Unknown operation: {}",
                    operation
                )))
            }
        }

        // Update the node using the data store's update method
        self.data_store.update_node(node).await?;
        Ok(())
    }

    /// Set node's parent (for indent/outdent operations)
    async fn set_node_parent(
        &self,
        node_id: &NodeId,
        parent_id: Option<&NodeId>,
    ) -> NodeSpaceResult<()> {
        let mut node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::NotFound(format!("Node {} not found", node_id)))?;

        let mut metadata = node.metadata.unwrap_or_else(|| serde_json::json!({}));

        if let Some(parent_id) = parent_id {
            // Set parent relationship
            metadata["parent_id"] = serde_json::Value::String(parent_id.to_string());
        } else {
            // Remove parent relationship
            metadata
                .as_object_mut()
                .and_then(|obj| obj.remove("parent_id"));
        }

        node.metadata = Some(metadata);
        self.data_store.update_node(node).await?;
        Ok(())
    }

    /// Update sibling order (for move up/down operations)
    async fn update_sibling_order(
        &self,
        node_id: &NodeId,
        previous_sibling_id: Option<&NodeId>,
        next_sibling_id: Option<&NodeId>,
    ) -> NodeSpaceResult<()> {
        let mut node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::NotFound(format!("Node {} not found", node_id)))?;

        // Update the node's sibling pointers
        node.previous_sibling = previous_sibling_id.cloned();
        node.next_sibling = next_sibling_id.cloned();

        // Update the updated node using the data store's update method
        self.data_store.update_node(node).await?;

        // Update affected siblings' pointers to maintain consistency
        if let Some(prev_id) = previous_sibling_id {
            if let Some(mut prev_node) = self.data_store.get_node(prev_id).await? {
                prev_node.next_sibling = Some(node_id.clone());
                self.data_store.update_node(prev_node).await?;
            }
        }

        if let Some(next_id) = next_sibling_id {
            if let Some(mut next_node) = self.data_store.get_node(next_id).await? {
                next_node.previous_sibling = Some(node_id.clone());
                self.data_store.update_node(next_node).await?;
            }
        }

        Ok(())
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
            parent_id: None,
            next_sibling: None,
            previous_sibling: None,
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
        // For LanceDB: Simple content search
        self.data_store.query_nodes(query).await
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
        // Step 1: Get all nodes
        let all_nodes = self.data_store.query_nodes("").await?; // Empty query returns all nodes

        // Step 2: Find the date node - the node that represents this date
        let date_str = date.format("%Y-%m-%d").to_string();
        let date_header = format!("# {}", date.format("%B %-d, %Y")); // e.g., "# June 27, 2025"

        let date_node_id = all_nodes
            .iter()
            .find(|node| {
                // Look for the date node by content
                if let Some(content) = node.content.as_str() {
                    // Remove surrounding quotes if present and trim
                    let clean_content = content.trim().trim_matches('"').trim();
                    clean_content == date_header
                        || clean_content.starts_with(&format!("# {}", date_str))
                        || clean_content == format!("# {}", date_str)
                } else {
                    false
                }
            })
            .map(|node| &node.id);

        if let Some(date_node_id) = date_node_id {
            // Step 3: Find all descendants of the date node (children, grandchildren, etc.)
            let descendants = get_all_descendants(&all_nodes, date_node_id);
            Ok(descendants)
        } else {
            // No date node found - return empty list
            Ok(vec![])
        }
    }

    /// Navigate to a specific date with navigation context
    async fn navigate_to_date(&self, date: NaiveDate) -> NodeSpaceResult<NavigationResult> {
        // Get nodes for the requested date
        let nodes = self.get_nodes_for_date(date).await?;

        // Check if there are nodes on previous day
        let previous_date = date - chrono::Duration::days(1);
        let previous_nodes = self
            .get_nodes_for_date(previous_date)
            .await
            .unwrap_or_default();
        let has_previous = !previous_nodes.is_empty();

        // Check if there are nodes on next day
        let next_date = date + chrono::Duration::days(1);
        let next_nodes = self.get_nodes_for_date(next_date).await.unwrap_or_default();
        let has_next = !next_nodes.is_empty();

        Ok(NavigationResult {
            date,
            nodes,
            has_previous,
            has_next,
        })
    }
}

/// Hierarchy computation implementation for NodeSpaceService
#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> HierarchyComputation
    for NodeSpaceService<D, N>
{
    async fn get_node_depth(&self, node_id: &NodeId) -> NodeSpaceResult<u32> {
        // Check cache first
        {
            let cache = self.hierarchy_cache.read().await;
            if let Some(depth) = cache.get_depth(node_id) {
                return Ok(depth);
            }
        }

        // Compute depth by traversing parent chain
        let mut depth = 0;
        let mut current_node_id = node_id.clone();

        loop {
            // Get the current node
            let node = self
                .data_store
                .get_node(&current_node_id)
                .await?
                .ok_or_else(|| {
                    NodeSpaceError::NotFound(format!("Node {} not found", current_node_id))
                })?;

            // Check if this node has a parent
            if let Some(parent_id) = &node.parent_id {
                depth += 1;
                current_node_id = parent_id.clone();

                // Safety check to prevent infinite loops
                if depth > 1000 {
                    return Err(NodeSpaceError::ValidationError(
                        "Hierarchy depth exceeds maximum limit (possible cycle)".to_string(),
                    ));
                }
            } else {
                // Reached root node
                break;
            }
        }

        // Cache the result
        {
            let mut cache = self.hierarchy_cache.write().await;
            cache.cache_depth(node_id.clone(), depth);
        }

        Ok(depth)
    }

    async fn get_children(&self, parent_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Check cache first
        {
            let cache = self.hierarchy_cache.read().await;
            if let Some(cached_child_ids) = cache.get_children(parent_id) {
                // Load nodes from cached IDs
                let mut children = Vec::new();
                for child_id in cached_child_ids {
                    if let Ok(Some(child)) = self.data_store.get_node(child_id).await {
                        children.push(child);
                    }
                }
                return Ok(children);
            }
        }

        // Get all nodes and filter for children
        let all_nodes = self.data_store.query_nodes("").await?;
        let mut children = Vec::new();
        let mut child_ids = Vec::new();

        for node in all_nodes {
            if let Some(node_parent_id) = &node.parent_id {
                if *node_parent_id == *parent_id {
                    child_ids.push(node.id.clone());
                    children.push(node);
                }
            }
        }

        // Cache the child IDs
        {
            let mut cache = self.hierarchy_cache.write().await;
            cache.cache_children(parent_id.clone(), child_ids);
        }

        Ok(children)
    }

    async fn get_ancestors(&self, node_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        let mut ancestors = Vec::new();
        let mut current_node_id = node_id.clone();

        loop {
            // Get the current node
            let node = self
                .data_store
                .get_node(&current_node_id)
                .await?
                .ok_or_else(|| {
                    NodeSpaceError::NotFound(format!("Node {} not found", current_node_id))
                })?;

            // Check if this node has a parent
            if let Some(parent_id) = &node.parent_id {
                // Get the parent node
                let parent_node = self.data_store.get_node(parent_id).await?.ok_or_else(|| {
                    NodeSpaceError::NotFound(format!("Parent node {} not found", parent_id))
                })?;

                ancestors.push(parent_node);
                current_node_id = parent_id.clone();

                // Safety check to prevent infinite loops
                if ancestors.len() > 1000 {
                    return Err(NodeSpaceError::ValidationError(
                        "Ancestry chain exceeds maximum limit (possible cycle)".to_string(),
                    ));
                }
            } else {
                // Reached root node
                break;
            }
        }

        Ok(ancestors)
    }

    async fn get_siblings(&self, node_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Get the node to find its parent
        let node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::NotFound(format!("Node {} not found", node_id)))?;

        // If node has no parent, it has no siblings
        let parent_id = match &node.parent_id {
            Some(parent_id) => parent_id.clone(),
            None => return Ok(Vec::new()),
        };

        // Get all children of the parent, excluding the original node
        let all_children = self.get_children(&parent_id).await?;
        let siblings: Vec<Node> = all_children
            .into_iter()
            .filter(|child| child.id != *node_id)
            .collect();

        Ok(siblings)
    }

    async fn move_node(&self, node_id: &NodeId, new_parent: &NodeId) -> NodeSpaceResult<()> {
        // Validate the move is legal
        self.validate_hierarchy_move(node_id, new_parent).await?;

        // Get the node to move
        let mut node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::NotFound(format!("Node {} not found", node_id)))?;

        // Update the node's parent_id directly
        node.parent_id = Some(new_parent.clone());
        node.updated_at = chrono::Utc::now().to_rfc3339();

        // Save the updated node
        self.data_store.update_node(node).await?;

        // Invalidate cache since hierarchy changed
        self.invalidate_hierarchy_cache().await;

        Ok(())
    }

    async fn move_subtree(&self, root_id: &NodeId, new_parent: &NodeId) -> NodeSpaceResult<()> {
        // Validate the move is legal for the root
        self.validate_hierarchy_move(root_id, new_parent).await?;

        // Get all descendants of the subtree
        let all_nodes = self.data_store.query_nodes("").await?;
        let descendants = get_all_descendants(&all_nodes, root_id);

        // Validate each descendant won't create cycles
        for descendant in &descendants {
            if descendant.id == *new_parent {
                return Err(NodeSpaceError::ValidationError(
                    "Cannot move subtree: would create a cycle".to_string(),
                ));
            }
        }

        // Move the root node
        self.move_node(root_id, new_parent).await?;

        Ok(())
    }

    async fn get_subtree_with_depths(&self, root_id: &NodeId) -> NodeSpaceResult<Vec<(Node, u32)>> {
        let mut result = Vec::new();

        // Get the root node and its depth
        let root_node =
            self.data_store.get_node(root_id).await?.ok_or_else(|| {
                NodeSpaceError::NotFound(format!("Root node {} not found", root_id))
            })?;
        let root_depth = self.get_node_depth(root_id).await?;
        result.push((root_node, root_depth));

        // Get all descendants
        let all_nodes = self.data_store.query_nodes("").await?;
        let descendants = get_all_descendants(&all_nodes, root_id);

        // Compute depth for each descendant
        for descendant in descendants {
            let depth = self.get_node_depth(&descendant.id).await?;
            result.push((descendant, depth));
        }

        // Sort by depth for consistent ordering
        result.sort_by(|a, b| a.1.cmp(&b.1));

        Ok(result)
    }

    async fn validate_hierarchy_move(
        &self,
        node_id: &NodeId,
        new_parent: &NodeId,
    ) -> NodeSpaceResult<()> {
        // Check if node exists
        self.data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::NotFound(format!("Node {} not found", node_id)))?;

        // Check if new parent exists
        self.data_store.get_node(new_parent).await?.ok_or_else(|| {
            NodeSpaceError::NotFound(format!("New parent {} not found", new_parent))
        })?;

        // Check if moving to self (invalid)
        if node_id == new_parent {
            return Err(NodeSpaceError::ValidationError(
                "Cannot move node to itself".to_string(),
            ));
        }

        // Check if new parent is a descendant of the node (would create cycle)
        let descendants = {
            let all_nodes = self.data_store.query_nodes("").await?;
            get_all_descendants(&all_nodes, node_id)
        };

        for descendant in descendants {
            if descendant.id == *new_parent {
                return Err(NodeSpaceError::ValidationError(
                    "Cannot move node: new parent is a descendant (would create cycle)".to_string(),
                ));
            }
        }

        Ok(())
    }

    async fn invalidate_hierarchy_cache(&self) {
        let mut cache = self.hierarchy_cache.write().await;
        cache.invalidate();
    }
}

/// Helper function to get all descendants (children, grandchildren, etc.) of a node
fn get_all_descendants(all_nodes: &[Node], parent_id: &NodeId) -> Vec<Node> {
    let mut descendants = Vec::new();
    let mut to_process = vec![parent_id.clone()];

    while let Some(current_parent_id) = to_process.pop() {
        // Find direct children of current parent
        for node in all_nodes {
            if let Some(node_parent_id) = &node.parent_id {
                if *node_parent_id == current_parent_id {
                    // This is a child - add it to results and queue for processing
                    descendants.push(node.clone());
                    to_process.push(node.id.clone());
                }
            }
        }
    }

    descendants
}

/// Smart caching for hierarchy operations
#[derive(Debug, Default)]
pub struct HierarchyCache {
    depth_cache: std::collections::HashMap<NodeId, u32>,
    children_cache: std::collections::HashMap<NodeId, Vec<NodeId>>,
    last_updated: Option<std::time::Instant>,
    cache_ttl: std::time::Duration,
}

impl HierarchyCache {
    pub fn new() -> Self {
        Self {
            depth_cache: std::collections::HashMap::new(),
            children_cache: std::collections::HashMap::new(),
            last_updated: None,
            cache_ttl: std::time::Duration::from_secs(300), // 5 minute TTL
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(last_updated) = self.last_updated {
            last_updated.elapsed() > self.cache_ttl
        } else {
            true
        }
    }

    pub fn invalidate(&mut self) {
        self.depth_cache.clear();
        self.children_cache.clear();
        self.last_updated = None;
    }

    pub fn get_depth(&self, node_id: &NodeId) -> Option<u32> {
        if self.is_expired() {
            None
        } else {
            self.depth_cache.get(node_id).copied()
        }
    }

    pub fn cache_depth(&mut self, node_id: NodeId, depth: u32) {
        self.depth_cache.insert(node_id, depth);
        self.last_updated = Some(std::time::Instant::now());
    }

    pub fn get_children(&self, parent_id: &NodeId) -> Option<&Vec<NodeId>> {
        if self.is_expired() {
            None
        } else {
            self.children_cache.get(parent_id)
        }
    }

    pub fn cache_children(&mut self, parent_id: NodeId, children: Vec<NodeId>) {
        self.children_cache.insert(parent_id, children);
        self.last_updated = Some(std::time::Instant::now());
    }
}

/// Cross-modal search orchestration implementation for NodeSpaceService
#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> CrossModalSearch
    for NodeSpaceService<D, N>
{
    async fn intelligent_cross_modal_search(
        &self,
        query: &str,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            return Err(NodeSpaceError::InternalError(format!(
                "Service not ready: {:?}",
                state
            )));
        }

        // Step 1: Extract entities, temporal refs, and visual attributes
        let entities = self.extract_entities(query).await?;
        let temporal_refs = self.extract_temporal_refs(query).await?;
        let visual_refs = self.extract_visual_refs(query).await?;

        // Step 2: Generate query embedding
        let query_embedding = self.nlp_engine.generate_embedding(query).await?;

        // Step 3: Coordinate multiple search strategies
        let search_results = self
            .multi_strategy_search(query_embedding, &entities, &temporal_refs, &visual_refs)
            .await?;

        // Step 4: Intelligent fusion and ranking
        self.intelligent_result_fusion(search_results, query).await
    }

    async fn extract_entities(&self, query: &str) -> NodeSpaceResult<ExtractedEntities> {
        // Use LLM for entity extraction with structured prompt
        let extraction_prompt = format!(
            "Extract entities from this query and respond in JSON format:
Query: '{}'

Extract:
- people: names of people mentioned
- events: events or occasions mentioned  
- objects: physical objects or items mentioned
- locations: places or locations mentioned

Respond with JSON: {{\"people\": [...], \"events\": [...], \"objects\": [...], \"locations\": [...]}}",
            query
        );

        match self.nlp_engine.generate_text(&extraction_prompt).await {
            Ok(response) => {
                // Parse JSON response
                match serde_json::from_str::<ExtractedEntities>(&response) {
                    Ok(entities) => Ok(entities),
                    Err(_) => {
                        // Fallback: simple pattern matching if JSON parsing fails
                        Ok(self.extract_entities_fallback(query))
                    }
                }
            }
            Err(_) => {
                // Fallback to pattern matching if LLM fails
                Ok(self.extract_entities_fallback(query))
            }
        }
    }

    async fn extract_temporal_refs(&self, query: &str) -> NodeSpaceResult<Vec<TemporalReference>> {
        let temporal_prompt = format!(
            "Extract temporal references from this query. Look for dates, times, or event-based temporal references:
Query: '{}'

Find references like:
- Specific dates (June 15, 2023-06-15)
- Relative times (yesterday, last week)  
- Event-based times (during birthday, at the meeting)
- Fuzzy times (around that time, recently)

For each reference, determine if it's exact, relative, event-based, or fuzzy.",
            query
        );

        match self.nlp_engine.generate_text(&temporal_prompt).await {
            Ok(_response) => {
                // For MVP, do simple pattern matching
                Ok(self.extract_temporal_refs_fallback(query))
            }
            Err(_) => {
                // Fallback to pattern matching
                Ok(self.extract_temporal_refs_fallback(query))
            }
        }
    }

    async fn extract_visual_refs(&self, query: &str) -> NodeSpaceResult<VisualAttributes> {
        // Look for visual attributes in the query
        let colors = self.extract_colors(query);
        let objects = self.extract_visual_objects(query);
        let scene_types = self.extract_scene_types(query);
        let people_descriptions = self.extract_people_descriptions(query);

        Ok(VisualAttributes {
            colors,
            objects,
            scene_types,
            people_descriptions,
        })
    }

    async fn multi_strategy_search(
        &self,
        query_embedding: Vec<f32>,
        entities: &ExtractedEntities,
        temporal_refs: &[TemporalReference],
        visual_refs: &VisualAttributes,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        let mut all_results = Vec::new();

        // Strategy 1: Semantic search
        match self
            .data_store
            .semantic_search_with_embedding(
                query_embedding.clone(),
                constants::DEFAULT_MULTI_STRATEGY_LIMIT,
            )
            .await
        {
            Ok(semantic_results) => {
                for (node, score) in semantic_results {
                    all_results.push(SearchResult {
                        node_id: node.id.clone(),
                        node,
                        score,
                    });
                }
            }
            Err(_) => {
                // Continue with other strategies if semantic search fails
            }
        }

        // Strategy 2: Entity-based search
        for person in &entities.people {
            if let Ok(entity_results) = self.search_by_entity(person).await {
                all_results.extend(entity_results);
            }
        }

        // Strategy 3: Temporal search
        for temporal_ref in temporal_refs {
            if let Ok(temporal_results) = self.search_by_temporal_ref(temporal_ref).await {
                all_results.extend(temporal_results);
            }
        }

        // Strategy 4: Visual search (if visual attributes found)
        if !visual_refs.colors.is_empty() || !visual_refs.objects.is_empty() {
            if let Ok(visual_results) = self.search_by_visual_attributes(visual_refs).await {
                all_results.extend(visual_results);
            }
        }

        Ok(all_results)
    }

    async fn intelligent_result_fusion(
        &self,
        mut search_results: Vec<SearchResult>,
        _original_query: &str,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        // Remove duplicates based on node ID (using string representation)
        search_results.sort_by(|a, b| a.node_id.to_string().cmp(&b.node_id.to_string()));
        search_results.dedup_by(|a, b| a.node_id == b.node_id);

        // Sort by score (highest first)
        search_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        search_results.truncate(constants::DEFAULT_FINAL_RESULT_LIMIT);

        Ok(search_results)
    }
}

impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> NodeSpaceService<D, N> {
    /// Fallback entity extraction using pattern matching
    fn extract_entities_fallback(&self, query: &str) -> ExtractedEntities {
        let query_lower = query.to_lowercase();
        let mut people = Vec::new();
        let mut events = Vec::new();
        let mut objects = Vec::new();
        let locations = Vec::new();

        // Common name patterns (basic detection)
        let name_patterns = ["claire", "john", "mary", "david", "sarah", "mike", "anna"];
        for pattern in &name_patterns {
            if query_lower.contains(pattern) {
                people.push(pattern.to_string());
            }
        }

        // Event patterns
        let event_patterns = [
            "birthday",
            "meeting",
            "party",
            "conference",
            "dinner",
            "lunch",
        ];
        for pattern in &event_patterns {
            if query_lower.contains(pattern) {
                events.push(pattern.to_string());
            }
        }

        // Object patterns
        let object_patterns = [
            "shirt",
            "document",
            "photo",
            "screenshot",
            "diagram",
            "chart",
        ];
        for pattern in &object_patterns {
            if query_lower.contains(pattern) {
                objects.push(pattern.to_string());
            }
        }

        ExtractedEntities {
            people,
            events,
            objects,
            locations,
        }
    }

    /// Fallback temporal reference extraction
    fn extract_temporal_refs_fallback(&self, query: &str) -> Vec<TemporalReference> {
        let query_lower = query.to_lowercase();
        let mut refs = Vec::new();

        // Look for relative time patterns
        if query_lower.contains("yesterday") {
            refs.push(TemporalReference {
                raw_text: "yesterday".to_string(),
                parsed_date: Some(chrono::Utc::now().date_naive() - chrono::Duration::days(1)),
                date_range: None,
                temporal_type: TemporalType::Relative,
            });
        }

        if query_lower.contains("birthday") {
            refs.push(TemporalReference {
                raw_text: "birthday".to_string(),
                parsed_date: None,
                date_range: None,
                temporal_type: TemporalType::Event,
            });
        }

        if query_lower.contains("last week") {
            let start_date = chrono::Utc::now().date_naive() - chrono::Duration::days(7);
            let end_date = chrono::Utc::now().date_naive();
            refs.push(TemporalReference {
                raw_text: "last week".to_string(),
                parsed_date: None,
                date_range: Some((start_date, end_date)),
                temporal_type: TemporalType::Relative,
            });
        }

        refs
    }

    /// Extract color references from query
    fn extract_colors(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let colors = [
            "red", "blue", "green", "yellow", "black", "white", "purple", "orange", "pink", "brown",
        ];

        colors
            .iter()
            .filter(|&color| query_lower.contains(color))
            .map(|&color| color.to_string())
            .collect()
    }

    /// Extract visual object references
    fn extract_visual_objects(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let objects = [
            "shirt", "dress", "hat", "car", "building", "tree", "person", "face",
        ];

        objects
            .iter()
            .filter(|&obj| query_lower.contains(obj))
            .map(|&obj| obj.to_string())
            .collect()
    }

    /// Extract scene type references
    fn extract_scene_types(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let scenes = [
            "indoor",
            "outdoor",
            "office",
            "restaurant",
            "home",
            "park",
            "street",
        ];

        scenes
            .iter()
            .filter(|&scene| query_lower.contains(scene))
            .map(|&scene| scene.to_string())
            .collect()
    }

    /// Extract people descriptions
    fn extract_people_descriptions(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let mut descriptions = Vec::new();

        if query_lower.contains("wearing") {
            descriptions.push("wearing clothing".to_string());
        }
        if query_lower.contains("smiling") {
            descriptions.push("smiling".to_string());
        }

        descriptions
    }

    /// Search nodes by entity mentions
    async fn search_by_entity(&self, entity: &str) -> NodeSpaceResult<Vec<SearchResult>> {
        // For LanceDB: Simple content search by entity
        let all_nodes = self.data_store.query_nodes(entity).await?;
        let nodes: Vec<_> = all_nodes
            .into_iter()
            .take(constants::DEFAULT_MAX_RESULTS_PER_STRATEGY)
            .collect();
        let results = nodes
            .into_iter()
            .enumerate()
            .map(|(index, node)| SearchResult {
                node_id: node.id.clone(),
                node,
                score: constants::BASE_CONFIDENCE_WITH_CONTEXT
                    - (index as f32 * constants::SCORE_DECAY_FACTOR * 0.5),
            })
            .collect();

        Ok(results)
    }

    /// Search nodes by temporal reference
    async fn search_by_temporal_ref(
        &self,
        temporal_ref: &TemporalReference,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        match &temporal_ref.temporal_type {
            TemporalType::Event => {
                // Search for event mentions
                self.search_by_entity(&temporal_ref.raw_text).await
            }
            TemporalType::Exact | TemporalType::Relative => {
                if let Some(date) = temporal_ref.parsed_date {
                    let date_str = date.format("%Y-%m-%d").to_string();
                    // For LanceDB: Get all nodes and filter by date
                    let all_nodes = self.data_store.query_nodes("").await?;
                    let nodes: Vec<_> = all_nodes
                        .into_iter()
                        .filter(|node| node.created_at.starts_with(&date_str))
                        .take(constants::DEFAULT_MAX_RESULTS_PER_STRATEGY)
                        .collect();
                    let results = nodes
                        .into_iter()
                        .enumerate()
                        .map(|(index, node)| SearchResult {
                            node_id: node.id.clone(),
                            node,
                            score: 0.9 - (index as f32 * constants::SCORE_DECAY_FACTOR * 0.5),
                        })
                        .collect();

                    Ok(results)
                } else {
                    Ok(Vec::new())
                }
            }
            TemporalType::Fuzzy => {
                // For fuzzy temporal references, return empty for now
                Ok(Vec::new())
            }
        }
    }

    /// Search nodes by visual attributes
    async fn search_by_visual_attributes(
        &self,
        visual_refs: &VisualAttributes,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Search for color mentions
        for color in &visual_refs.colors {
            if let Ok(color_results) = self.search_by_entity(color).await {
                results.extend(color_results);
            }
        }

        // Search for object mentions
        for object in &visual_refs.objects {
            if let Ok(object_results) = self.search_by_entity(object).await {
                results.extend(object_results);
            }
        }

        Ok(results)
    }

    /// Batch lookup for multiple node relationships (optimization for N+1 queries)
    async fn get_batch_related_nodes(
        &self,
        node_ids: &[NodeId],
        relationship_types: Vec<String>,
    ) -> NodeSpaceResult<std::collections::HashMap<NodeId, Vec<NodeId>>> {
        use std::collections::HashMap;

        if node_ids.is_empty() {
            return Ok(HashMap::new());
        }

        // Get all nodes once to avoid multiple database calls
        let all_nodes = self.data_store.query_nodes("").await.unwrap_or_default();
        let mut result_map = HashMap::new();

        // Initialize empty vectors for all requested node IDs
        for node_id in node_ids {
            result_map.insert(node_id.clone(), Vec::new());
        }

        // Process all nodes to find relationships
        for node in all_nodes {
            if let Some(metadata) = &node.metadata {
                // Check mentions
                if let Some(mentions) = metadata.get("mentions") {
                    if let Some(mentions_array) = mentions.as_array() {
                        for mention in mentions_array {
                            if let Some(mention_str) = mention.as_str() {
                                let mentioned_id = NodeId::from_string(mention_str.to_string());
                                if let Some(related_list) = result_map.get_mut(&mentioned_id) {
                                    related_list.push(node.id.clone());
                                }
                            }
                        }
                    }
                }

                // Check specific relationship types if provided
                if !relationship_types.is_empty() {
                    if let Some(relationships) = metadata.get("relationships") {
                        if let Some(rel_obj) = relationships.as_object() {
                            for rel_type in &relationship_types {
                                if let Some(targets) = rel_obj.get(rel_type) {
                                    if let Some(targets_array) = targets.as_array() {
                                        for target in targets_array {
                                            if let Some(target_str) = target.as_str() {
                                                let target_id =
                                                    NodeId::from_string(target_str.to_string());
                                                if let Some(related_list) =
                                                    result_map.get_mut(&target_id)
                                                {
                                                    related_list.push(node.id.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(result_map)
    }

    /// Helper method to gather context for query processing
    async fn gather_query_context(
        &self,
        query: &str,
    ) -> NodeSpaceResult<(Vec<String>, Vec<NodeId>)> {
        let search_results = self
            .semantic_search(query, constants::DEFAULT_SEARCH_LIMIT)
            .await?;

        let context: Vec<String> = search_results
            .iter()
            .filter_map(|result| result.node.content.as_str().map(|s| s.to_string()))
            .collect();

        let sources: Vec<NodeId> = search_results
            .iter()
            .map(|result| result.node_id.clone())
            .collect();

        Ok((context, sources))
    }

    /// Helper method to build contextual prompt with length management
    fn build_contextual_prompt(&self, query: &str, context: &[String]) -> String {
        let context_text = context.join("\n\n");
        let max_context_len = self
            .config
            .performance_config
            .context_window
            .unwrap_or(constants::DEFAULT_CONTEXT_WINDOW)
            .saturating_sub(query.len() + constants::PROMPT_STRUCTURE_RESERVE);

        let truncated_context = if context_text.len() > max_context_len {
            format!("{}...", &context_text[..max_context_len])
        } else {
            context_text
        };

        if truncated_context.is_empty() {
            format!("Answer this question based on general knowledge: {}", query)
        } else {
            format!(
                "Based on the following context, answer the question: {}\n\nContext:\n{}",
                query, truncated_context
            )
        }
    }

    /// Helper method to generate contextual answer with fallback handling
    async fn generate_contextual_answer(
        &self,
        prompt: &str,
        sources: &[NodeId],
    ) -> NodeSpaceResult<String> {
        match self.nlp_engine.generate_text(prompt).await {
            Ok(text) => Ok(text),
            Err(e) => {
                // Handle text generation failure based on configuration
                match self.config.offline_config.offline_fallback {
                    OfflineFallback::Error => {
                        Err(NodeSpaceError::ProcessingError(format!(
                            "Text generation failed: {}",
                            e
                        )))
                    }
                    OfflineFallback::Stub => {
                        Ok("I apologize, but I'm currently unable to generate a response due to AI system limitations. Please try again later.".to_string())
                    }
                    OfflineFallback::Cache => {
                        if sources.is_empty() {
                            Ok("I found no relevant information to answer your question.".to_string())
                        } else {
                            Ok(format!("I found {} related documents but cannot generate a detailed response at this time. Please review the source materials directly.", sources.len()))
                        }
                    }
                }
            }
        }
    }

    /// Helper method to calculate response confidence
    fn calculate_response_confidence(&self, context: &[String], answer: &str) -> f32 {
        let base_confidence = if context.is_empty() {
            constants::BASE_CONFIDENCE_NO_CONTEXT
        } else {
            constants::BASE_CONFIDENCE_WITH_CONTEXT
        };

        if answer.contains("currently unable") || answer.contains("cannot generate") {
            base_confidence * constants::FALLBACK_CONFIDENCE_FACTOR
        } else {
            base_confidence
        }
    }

    /// Helper method to generate related queries
    fn generate_related_queries(&self, query: &str) -> Vec<String> {
        vec![
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
        ]
    }
}

// Include tests module
#[cfg(test)]
mod tests;
