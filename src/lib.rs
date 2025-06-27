use async_trait::async_trait;
use chrono::NaiveDate;
use nodespace_core_types::{Node, NodeId, NodeSpaceError, NodeSpaceResult};
use nodespace_data_store::DataStore;
use nodespace_nlp_engine::NLPEngine;
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
            next_sibling: None,
            previous_sibling: None,
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

        // For LanceDB: Simple content search using query_nodes
        let all_nodes = self.data_store.query_nodes(query).await?;
        let nodes: Vec<_> = all_nodes.into_iter().take(effective_limit).collect();

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

        // For LanceDB: relationships are stored in metadata, not separate tables
        // Get all nodes and filter by relationships in metadata
        let all_nodes = self.data_store.query_nodes("").await.unwrap_or_default();
        let result_nodes: Vec<_> = all_nodes
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
                }
                false
            })
            .collect();

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
        // For LanceDB: Get all nodes and filter by created_at date
        // The LanceDB implementation currently uses simple in-memory HashMap storage
        let all_nodes = self.data_store.query_nodes("").await?; // Empty query returns all nodes

        // Filter nodes by created_at date
        let filtered_nodes: Vec<Node> = all_nodes
            .into_iter()
            .filter(|node| {
                // Parse the created_at timestamp and compare dates
                if let Ok(node_datetime) = chrono::DateTime::parse_from_rfc3339(&node.created_at) {
                    node_datetime.date_naive() == date
                } else {
                    // Fallback: try parsing as date-only string
                    node
                        .created_at
                        .starts_with(&date.format("%Y-%m-%d").to_string())
                }
            })
            .collect();

        Ok(filtered_nodes)
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
        // For LanceDB: Get all nodes and find dates with content
        let all_nodes = self.data_store.query_nodes("").await?;

        let mut dates_with_content = Vec::new();
        for node in all_nodes {
            if let Ok(node_datetime) = chrono::DateTime::parse_from_rfc3339(&node.created_at) {
                let node_date = node_datetime.date_naive();
                if node_date < current_date && !dates_with_content.contains(&node_date) {
                    dates_with_content.push(node_date);
                }
            }
        }

        // Return the latest date before current_date
        dates_with_content.sort();
        Ok(dates_with_content.into_iter().next_back())
    }

    async fn get_next_day(&self, current_date: NaiveDate) -> NodeSpaceResult<Option<NaiveDate>> {
        // For LanceDB: Get all nodes and find dates with content
        let all_nodes = self.data_store.query_nodes("").await?;

        let mut dates_with_content = Vec::new();
        for node in all_nodes {
            if let Ok(node_datetime) = chrono::DateTime::parse_from_rfc3339(&node.created_at) {
                let node_date = node_datetime.date_naive();
                if node_date > current_date && !dates_with_content.contains(&node_date) {
                    dates_with_content.push(node_date);
                }
            }
        }

        // Return the earliest date after current_date
        dates_with_content.sort();
        Ok(dates_with_content.into_iter().next())
    }

    async fn create_or_get_date_node(&self, date: NaiveDate) -> NodeSpaceResult<DateNode> {
        let date_str = date.format("%Y-%m-%d").to_string();

        // For LanceDB: Find existing date node by metadata
        let all_nodes = self.data_store.query_nodes("").await?;

        // Look for a node with node_type="date" and matching date content
        for node in &all_nodes {
            if let Some(metadata) = &node.metadata {
                if let Some(node_type) = metadata.get("node_type") {
                    if node_type.as_str() == Some("date") {
                        if let Some(content_str) = node.content.as_str() {
                            if content_str == date_str {
                                // Count child nodes for this date
                                let child_count = all_nodes
                                    .iter()
                                    .filter(|n| {
                                        if let Some(n_meta) = &n.metadata {
                                            if let Some(parent) = n_meta.get("parent_id") {
                                                parent.as_str() == Some(node.id.as_str())
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    })
                                    .count();

                                return Ok(DateNode {
                                    id: node.id.clone(),
                                    date,
                                    description: metadata
                                        .get("description")
                                        .and_then(|d| d.as_str())
                                        .map(|s| s.to_string()),
                                    child_count,
                                });
                            }
                        }
                    }
                }
            }
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
            next_sibling: None,
            previous_sibling: None,
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
        // For LanceDB: Find child nodes by parent_id in metadata
        let all_nodes = self.data_store.query_nodes("").await?;

        let mut children: Vec<Node> = all_nodes
            .into_iter()
            .filter(|node| {
                if let Some(metadata) = &node.metadata {
                    if let Some(parent_id) = metadata.get("parent_id") {
                        parent_id.as_str() == Some(date_node_id.as_str())
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .collect();

        // Sort by created_at
        children.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(children)
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
            .semantic_search_with_embedding(query_embedding.clone(), 20)
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
        search_results.truncate(10);

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
        let nodes: Vec<_> = all_nodes.into_iter().take(10).collect();
        let results = nodes
            .into_iter()
            .enumerate()
            .map(|(index, node)| {
                SearchResult {
                    node_id: node.id.clone(),
                    node,
                    score: 0.8 - (index as f32 * 0.05), // Decreasing score by position
                }
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
                        .take(10)
                        .collect();
                    let results = nodes
                        .into_iter()
                        .enumerate()
                        .map(|(index, node)| SearchResult {
                            node_id: node.id.clone(),
                            node,
                            score: 0.9 - (index as f32 * 0.05),
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
}
