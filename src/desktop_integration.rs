//! Desktop app integration module
//!
//! This module provides enhanced APIs for desktop app integration including:
//! - Unified node management with upsert_node API
//! - Enhanced query responses with rich metadata
//! - AIChat node type support with vector embedding control

use crate::{CoreLogic, DataStore, HierarchyComputation, NLPEngine, NodeSpaceService};
use chrono::{DateTime, NaiveDate, Utc};
use nodespace_core_types::{NodeId, NodeSpaceError, NodeSpaceResult};
use nodespace_data_store::NodeType;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Enhanced query response for desktop app AIChatNode integration
/// Provides rich metadata and full source content for sophisticated UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedQueryResponse {
    pub answer: String,
    pub confidence: f64,

    // Performance metrics for metadata
    pub generation_time_ms: u64,
    pub overall_confidence: f64,

    // Rich source information with full content
    pub sources: Vec<NodeSource>,
}

/// Rich source information for AIChatNode metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSource {
    pub node_id: String,              // UUID for navigation
    pub content: String,              // FULL content of source node
    pub retrieval_score: f64,         // 0-1 confidence score
    pub context_tokens: usize,        // Token count for this source
    pub node_type: String,            // "text", "task", etc. for UI styling
    pub last_modified: DateTime<Utc>, // For freshness indication
}

/// Desktop app integration implementation for NodeSpaceService
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> NodeSpaceService<D, N> {
    /// Universal node upsert - handles creation, updates, hierarchy, and metadata
    ///
    /// This method is idempotent and atomic:
    /// - Creates new node when ID doesn't exist
    /// - Updates existing node when ID exists
    /// - Handles hierarchy changes atomically
    /// - Vector control: Only content gets embedded, never metadata for AIChat nodes
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_node(
        &self,
        node_id: NodeId,
        date: NaiveDate,
        content: String,
        parent_id: Option<NodeId>,
        before_sibling_id: Option<NodeId>,
        node_type: String,
        metadata: Option<serde_json::Value>,
    ) -> NodeSpaceResult<()> {
        log::info!(
            "🔄 Upserting node: {} (type: {})",
            node_id.as_str(),
            node_type
        );

        // Check if service is ready
        if !self.is_ready().await {
            return Err(NodeSpaceError::InternalError {
                message: "Service not ready for node upsert".to_string(),
                service: "core-logic".to_string(),
            });
        }

        // Convert string node type to NodeType enum
        let data_store_node_type = match node_type.as_str() {
            "text" => NodeType::Text,
            "ai-chat" => NodeType::Text, // AIChat nodes are stored as Text with special metadata
            "task" => NodeType::Task,
            "image" => NodeType::Image,
            "date" => NodeType::Date,
            _ => {
                return Err(NodeSpaceError::InternalError {
                    message: format!("Unsupported node type: {}", node_type),
                    service: "core-logic".to_string(),
                });
            }
        };

        // Vector embedding control: For AIChat nodes, only embed content (title), never metadata
        let embedding_content = if node_type.as_str() == "ai-chat" {
            log::info!("   📝 AIChat node: embedding title only, excluding metadata from vectors");
            &content // Only the chat title gets embedded
        } else {
            &content // Normal content embedding for other types
        };

        // Check if node already exists
        let existing_node = self.data_store.get_node(&node_id).await?;

        if let Some(mut node) = existing_node {
            log::info!("   📝 Updating existing node");

            // Update content and metadata atomically
            node.content = serde_json::Value::String(content.clone());
            if let Some(meta) = metadata {
                node.metadata = Some(meta);
            }

            // Update hierarchy if specified
            if let Some(ref parent) = parent_id {
                node.parent_id = Some(parent.clone());
            }

            // Update the node in the data store
            self.data_store.update_node(node).await?;

            // Handle sibling ordering if specified
            if let Some(ref before_sibling) = before_sibling_id {
                self.update_sibling_order(&node_id, Some(before_sibling), parent_id.as_ref())
                    .await?;
            }

            // Regenerate embedding with new content
            let embedding = self
                .nlp_engine
                .generate_embedding(embedding_content)
                .await?;
            self.data_store
                .update_node_embedding(&node_id, embedding)
                .await?;

            log::info!("   ✅ Node updated successfully");
        } else {
            log::info!("   🆕 Creating new node");

            // Create new node with proper hierarchy and metadata
            self.create_node_for_date_with_id(
                node_id.clone(),
                date,
                &content,
                data_store_node_type,
                metadata,
                parent_id,
                before_sibling_id,
            )
            .await?;

            log::info!("   ✅ Node created successfully");
        }

        log::info!("✅ Node upsert completed for {}", node_id.as_str());
        Ok(())
    }

    /// Enhanced query processing with rich metadata for AIChatNode
    /// Returns detailed source information and performance metrics
    pub async fn process_query_enhanced(
        &self,
        query: String,
    ) -> NodeSpaceResult<EnhancedQueryResponse> {
        log::info!("🔍 Processing enhanced query: '{}'", query);
        let start_time = Instant::now();

        // Check if service is ready
        if !self.is_ready().await {
            return Ok(EnhancedQueryResponse {
                answer: "I encountered an error processing your question.".to_string(),
                confidence: 0.0,
                generation_time_ms: 0,
                overall_confidence: 0.0,
                sources: vec![],
            });
        }

        // Perform semantic search to get relevant sources
        let search_results = match self.semantic_search(&query, 5).await {
            Ok(results) => results,
            Err(e) => {
                log::error!("   ❌ Semantic search failed: {}", e);
                return Ok(EnhancedQueryResponse {
                    answer: "I encountered an error processing your question.".to_string(),
                    confidence: 0.0,
                    generation_time_ms: start_time.elapsed().as_millis() as u64,
                    overall_confidence: 0.0,
                    sources: vec![],
                });
            }
        };

        // Build enhanced sources with full content and metadata
        let mut enhanced_sources = Vec::new();
        for result in &search_results {
            let content_str = result.node.content.as_str().unwrap_or("");
            let node_type_str = result.node.r#type.as_str();

            // Calculate token count (rough approximation)
            let token_count = content_str.split_whitespace().count();

            enhanced_sources.push(NodeSource {
                node_id: result.node.id.as_str().to_string(),
                content: content_str.to_string(),
                retrieval_score: result.score as f64,
                context_tokens: token_count,
                node_type: node_type_str.to_string(),
                last_modified: chrono::DateTime::parse_from_rfc3339(&result.node.updated_at)
                    .unwrap_or_else(|_| {
                        chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap()
                    })
                    .with_timezone(&chrono::Utc),
            });
        }

        // Generate AI response using enhanced text generation
        let ai_response = match self
            .generate_ai_response(
                &query,
                &search_results
                    .iter()
                    .map(|r| r.node.id.clone())
                    .collect::<Vec<_>>(),
            )
            .await
        {
            Ok(response) => response,
            Err(e) => {
                log::error!("   ❌ AI response generation failed: {}", e);
                "I encountered an error generating a response. Please try again.".to_string()
            }
        };

        let generation_time = start_time.elapsed().as_millis() as u64;

        // Calculate overall confidence based on source quality and generation success
        let overall_confidence = if enhanced_sources.is_empty() {
            0.3 // Low confidence with no sources
        } else {
            let avg_source_score = enhanced_sources
                .iter()
                .map(|s| s.retrieval_score)
                .sum::<f64>()
                / enhanced_sources.len() as f64;
            (avg_source_score * 0.8) + 0.2 // Weighted by source quality
        };

        log::info!("   ✅ Enhanced query processed in {}ms", generation_time);
        log::info!(
            "   📊 Sources: {}, Confidence: {:.2}",
            enhanced_sources.len(),
            overall_confidence
        );

        Ok(EnhancedQueryResponse {
            answer: ai_response,
            confidence: overall_confidence,
            generation_time_ms: generation_time,
            overall_confidence,
            sources: enhanced_sources,
        })
    }
}
