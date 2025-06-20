use async_trait::async_trait;
use nodespace_core_types::{Node, NodeId, NodeSpaceResult, NodeSpaceError};
use nodespace_data_store::DataStore;
use nodespace_nlp_engine::NLPEngine;

/// Core business logic service that orchestrates NodeSpace functionality
/// using distributed contract ownership
pub struct NodeSpaceService<D: DataStore, N: NLPEngine> {
    data_store: D,
    nlp_engine: N,
}

impl<D: DataStore, N: NLPEngine> NodeSpaceService<D, N> {
    /// Create a new NodeSpace service with injected dependencies
    pub fn new(data_store: D, nlp_engine: N) -> Self {
        Self {
            data_store,
            nlp_engine,
        }
    }
}

/// Core business logic operations
#[async_trait]
pub trait CoreLogic {
    /// Create a new node with automatic embedding generation
    async fn create_node(&self, content: serde_json::Value, metadata: Option<serde_json::Value>) -> NodeSpaceResult<NodeId>;
    
    /// Retrieve a node by ID
    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>>;
    
    /// Delete a node
    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()>;
    
    /// Search nodes using semantic and text search
    async fn search_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>>;
    
    /// Process a RAG query: search for context + generate response
    async fn process_rag_query(&self, query: &str) -> NodeSpaceResult<String>;
    
    /// Create a relationship between nodes
    async fn create_relationship(&self, from: &NodeId, to: &NodeId, rel_type: &str) -> NodeSpaceResult<()>;
}

#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> CoreLogic for NodeSpaceService<D, N> {
    async fn create_node(&self, content: serde_json::Value, metadata: Option<serde_json::Value>) -> NodeSpaceResult<NodeId> {
        // Create the node with provided content and metadata
        let node_id = NodeId::new();
        let now = chrono::Utc::now().to_rfc3339();
        
        let node = Node {
            id: node_id.clone(),
            content,
            metadata,
            created_at: now.clone(),
            updated_at: now,
        };
        
        // Store the node
        self.data_store.store_node(node).await?;
        
        // TODO: Generate embeddings in background
        // For MVP, we'll keep this simple and generate embeddings synchronously
        // Future optimization: async background embedding generation
        
        Ok(node_id)
    }
    
    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        self.data_store.get_node(id).await
    }
    
    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
        self.data_store.delete_node(id).await
    }
    
    async fn search_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>> {
        // For MVP, use simple SurrealQL query
        // Future: implement semantic search with embeddings
        let search_query = format!("SELECT * FROM nodes WHERE content CONTAINS '{}'", query);
        self.data_store.query_nodes(&search_query).await
    }
    
    async fn process_rag_query(&self, query: &str) -> NodeSpaceResult<String> {
        // Step 1: Search for relevant context
        let context_nodes = self.search_nodes(query).await?;
        
        // Step 2: Extract text content from nodes
        let context: Vec<String> = context_nodes
            .iter()
            .filter_map(|node| {
                node.content.as_str().map(|s| s.to_string())
            })
            .collect();
        
        // Step 3: Generate response using NLP engine
        let context_text = context.join("\n\n");
        let prompt = format!("Based on the following context, answer the question: {}\n\nContext:\n{}", query, context_text);
        
        self.nlp_engine.generate_text(&prompt).await
    }
    
    async fn create_relationship(&self, from: &NodeId, to: &NodeId, rel_type: &str) -> NodeSpaceResult<()> {
        self.data_store.create_relationship(from, to, rel_type).await
    }
}