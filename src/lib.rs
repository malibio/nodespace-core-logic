use async_trait::async_trait;
use nodespace_core_types::{Node, NodeId, NodeSpaceResult, NodeSpaceError};
use nodespace_data_store::DataStore;
use nodespace_nlp_engine::NLPEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Core business logic operations interface following distributed contract pattern
#[async_trait]
pub trait CoreLogic: Send + Sync {
    /// Create a new knowledge node with AI processing
    async fn create_knowledge_node(&self, content: &str, metadata: serde_json::Value) -> NodeSpaceResult<NodeId>;
    
    /// Search for nodes using semantic similarity
    async fn semantic_search(&self, query: &str, limit: usize) -> NodeSpaceResult<Vec<SearchResult>>;
    
    /// Process natural language query and return results
    async fn process_query(&self, query: &str) -> NodeSpaceResult<QueryResponse>;
    
    /// Update node content and reprocess embeddings
    async fn update_node(&self, node_id: &NodeId, content: &str) -> NodeSpaceResult<()>;
    
    /// Get related nodes using graph relationships
    async fn get_related_nodes(&self, node_id: &NodeId, relationship_types: Vec<String>) -> NodeSpaceResult<Vec<NodeId>>;
    
    /// Generate insights from a collection of nodes
    async fn generate_insights(&self, node_ids: Vec<NodeId>) -> NodeSpaceResult<String>;
}

/// Legacy CoreLogic interface for backward compatibility
#[async_trait]
pub trait LegacyCoreLogic {
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

/// Natural language query context for enhanced processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalLanguageQuery {
    pub text: String,
    pub intent_type: String,
    pub response_type: EntityResponseType,
    pub context: NLQueryContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLQueryContext {
    pub include_relationships: Option<bool>,
    pub max_results: Option<usize>,
    pub similarity_threshold: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityResponseType {
    StatusUpdate,
    DetailedList,
    NaturalLanguage,
    Summary,
}

#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> CoreLogic for NodeSpaceService<D, N> {
    async fn create_knowledge_node(&self, content: &str, metadata: serde_json::Value) -> NodeSpaceResult<NodeId> {
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
        
        // Generate embeddings asynchronously for better search capabilities
        let _embedding = self.nlp_engine.generate_embedding(content).await
            .unwrap_or_else(|_| vec![0.0; 768]); // Default embedding size fallback
        
        // TODO: Store embedding with node for semantic search
        // For MVP, embeddings are generated but not yet stored
        
        Ok(node_id)
    }
    
    async fn semantic_search(&self, query: &str, limit: usize) -> NodeSpaceResult<Vec<SearchResult>> {
        // Generate embedding for the search query
        let query_embedding = self.nlp_engine.generate_embedding(query).await?;
        
        // For MVP, fall back to text-based search until vector search is implemented
        let search_query = format!("SELECT * FROM nodes WHERE content CONTAINS '{}' LIMIT {}", query, limit);
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
        // Step 1: Perform semantic search for context
        let search_results = self.semantic_search(query, 5).await?;
        
        // Step 2: Extract text content and source IDs
        let context: Vec<String> = search_results
            .iter()
            .filter_map(|result| {
                result.node.content.as_str().map(|s| s.to_string())
            })
            .collect();
            
        let sources: Vec<NodeId> = search_results
            .iter()
            .map(|result| result.node_id.clone())
            .collect();
        
        // Step 3: Generate response using NLP engine
        let context_text = context.join("\n\n");
        let prompt = if context_text.is_empty() {
            format!("Answer this question based on general knowledge: {}", query)
        } else {
            format!("Based on the following context, answer the question: {}\n\nContext:\n{}", query, context_text)
        };
        
        let answer = self.nlp_engine.generate_text(&prompt).await?;
        
        // Step 4: Calculate confidence based on context quality
        let confidence = if context.is_empty() { 0.3 } else { 0.8 };
        
        // Step 5: Generate related queries (simplified for MVP)
        let related_queries = vec![
            format!("What else about {}?", query.split_whitespace().last().unwrap_or("this topic")),
            format!("How does {} work?", query.split_whitespace().take(3).collect::<Vec<_>>().join(" ")),
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
        let mut node = self.data_store.get_node(node_id).await?
            .ok_or_else(|| NodeSpaceError::NotFound(format!("Node {} not found", node_id)))?;
        
        // Update content and timestamp
        node.content = serde_json::Value::String(content.to_string());
        node.updated_at = chrono::Utc::now().to_rfc3339();
        
        // Store updated node
        self.data_store.store_node(node).await?;
        
        // Regenerate embeddings for updated content
        let _embedding = self.nlp_engine.generate_embedding(content).await
            .unwrap_or_else(|_| vec![0.0; 768]);
        
        // TODO: Update stored embeddings for semantic search
        
        Ok(())
    }
    
    async fn get_related_nodes(&self, node_id: &NodeId, relationship_types: Vec<String>) -> NodeSpaceResult<Vec<NodeId>> {
        // For MVP, use a simple query to find related nodes
        let relationship_filters = if relationship_types.is_empty() {
            "".to_string()
        } else {
            format!(" AND type IN [{}]", relationship_types.iter().map(|t| format!("'{}'", t)).collect::<Vec<_>>().join(", "))
        };
        
        let query = format!(
            "SELECT out FROM {} WHERE in = nodes:{}{}", 
            "relationships", // Assuming relationship table name
            node_id,
            relationship_filters
        );
        
        // Execute query and extract node IDs
        let result_nodes = self.data_store.query_nodes(&query).await
            .unwrap_or_default(); // Gracefully handle relationship query failures
        
        let related_ids: Vec<NodeId> = result_nodes
            .into_iter()
            .map(|node| node.id)
            .collect();
        
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

/// Legacy implementation for backward compatibility
#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> LegacyCoreLogic for NodeSpaceService<D, N> {
    async fn create_node(&self, content: serde_json::Value, metadata: Option<serde_json::Value>) -> NodeSpaceResult<NodeId> {
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
    
    async fn create_relationship(&self, from: &NodeId, to: &NodeId, rel_type: &str) -> NodeSpaceResult<()> {
        self.data_store.create_relationship(from, to, rel_type).await
    }
}