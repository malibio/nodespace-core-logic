# Desktop App Integration APIs

This document describes the enhanced APIs implemented for NodeSpace desktop app integration, specifically for AIChatNode support with unified node management.

## Key Features

### 1. Unified Node Management API (`upsert_node`)

The `upsert_node` method provides idempotent, atomic node operations:

```rust
pub async fn upsert_node(
    &self,
    node_id: NodeId,
    date: NaiveDate,                        // For date context
    content: String,                        // For vector embedding (title only for AIChatNode)
    parent_id: Option<NodeId>,              // null = root node under date
    before_sibling_id: Option<NodeId>,      // null = first child or last sibling
    node_type: String,                      // "text", "ai-chat", "task", "image"
    metadata: Option<serde_json::Value>,    // null for TextNode, rich data for AIChatNode
) -> NodeSpaceResult<()>
```

**Features:**
- **Idempotent**: Creates new node if doesn't exist, updates if exists
- **Atomic**: All changes (content, hierarchy, metadata) succeed or fail together
- **Vector Control**: Only content gets embedded, never metadata for any node type
- **Hierarchy Aware**: Handles parent-child and sibling relationships properly

### 2. Enhanced Query Response (`process_query_enhanced`)

Returns rich metadata for AIChatNode integration:

```rust
pub async fn process_query_enhanced(
    &self, 
    query: String
) -> NodeSpaceResult<EnhancedQueryResponse>
```

**EnhancedQueryResponse Structure:**
```rust
pub struct EnhancedQueryResponse {
    pub answer: String,
    pub confidence: f64,
    
    // Performance metrics for metadata
    pub generation_time_ms: u64,
    pub overall_confidence: f64,
    
    // Rich source information with full content
    pub sources: Vec<NodeSource>,
}

pub struct NodeSource {
    pub node_id: String,                    // UUID for navigation
    pub content: String,                    // FULL content of source node
    pub retrieval_score: f64,               // 0-1 confidence score
    pub context_tokens: usize,              // Token count for this source
    pub node_type: String,                  // "text", "task", etc. for UI styling
    pub last_modified: DateTime<Utc>,       // For freshness indication
}
```

### 3. AIChat Node Type Support

**Vector Embedding Control:**
- **Text nodes**: Full content gets embedded
- **AI-Chat nodes**: Only title (content field) gets embedded, metadata excluded
- **Task/Image nodes**: Standard content embedding

**AIChatNode Metadata Schema:**
```json
{
  "question": "What is machine learning?",
  "question_timestamp": "2025-07-03T10:00:00Z", 
  "response": "Machine learning is a field of artificial intelligence...",
  "response_timestamp": "2025-07-03T10:00:01Z",
  "generation_time_ms": 1200,
  "overall_confidence": 0.87,
  "node_sources": [
    {
      "node_id": "50fab33e-d0f8-4afd-ae1d-e1679bbce093",
      "content": "AI and machine learning are transforming industries...",
      "retrieval_score": 0.89,
      "context_tokens": 150,
      "node_type": "text",
      "last_modified": "2025-07-02T15:30:00Z"
    }
  ],
  "error": null
}
```

## Usage Examples

### Creating/Updating Text Node
```rust
let text_node_id = NodeId::new();
service.upsert_node(
    text_node_id,
    chrono::Utc::now().date_naive(),
    "NodeSpace is a knowledge management system".to_string(),
    None, // Root node
    None, // First sibling
    "text".to_string(),
    None, // No metadata for text nodes
).await?;
```

### Creating AI Chat Node
```rust
let chat_node_id = NodeId::new();
let ai_metadata = serde_json::json!({
    "question": "What is NodeSpace?",
    "response": "NodeSpace is an AI-powered knowledge management system...",
    "generation_time_ms": 1200,
    "overall_confidence": 0.87,
    "node_sources": []
});

service.upsert_node(
    chat_node_id,
    chrono::Utc::now().date_naive(),
    "Chat: What is NodeSpace?".to_string(), // Only this gets embedded
    None,
    Some(previous_node_id), // Position after previous node
    "ai-chat".to_string(),
    Some(ai_metadata), // Rich conversation data
).await?;
```

### Enhanced Query Processing
```rust
let response: EnhancedQueryResponse = service.process_query_enhanced(
    "What technologies does NodeSpace use?".to_string()
).await?;

println!("Answer: {}", response.answer);
println!("Confidence: {:.2}", response.confidence);
println!("Generation time: {}ms", response.generation_time_ms);

for source in response.sources {
    println!("Source: {} (score: {:.3})", source.node_id, source.retrieval_score);
    println!("Content: {}", source.content);
}
```

## Error Handling

Enhanced query processing includes graceful error handling:

```rust
// On query success:
EnhancedQueryResponse {
    answer: "NodeSpace uses Rust, LanceDB, and Ollama...",
    confidence: 0.87,
    generation_time_ms: 1200,
    overall_confidence: 0.87,
    sources: vec![...],
}

// On query failure:
EnhancedQueryResponse {
    answer: "I encountered an error processing your question.",
    confidence: 0.0,
    generation_time_ms: 0,
    overall_confidence: 0.0,
    sources: vec![],
}
```

## Vector Embedding Strategy

The implementation ensures proper search behavior:

1. **Text Nodes**: Full content embedded → Shows in semantic search
2. **AI Chat Nodes**: Only title embedded → Chat title shows in search, not conversation metadata
3. **Task Nodes**: Content embedded → Task descriptions searchable
4. **Image Nodes**: Content embedded → Image descriptions/captions searchable

This prevents chat conversation metadata from polluting semantic search results while maintaining searchability of chat topics.

## Integration Testing

Run the desktop integration test:

```bash
cargo run --example test_desktop_integration
```

This validates:
- ✅ Unified node management
- ✅ Enhanced query responses  
- ✅ AIChat metadata support
- ✅ Vector embedding control
- ✅ Real Ollama AI integration

## Migration from Deprecated Methods

The desktop app should replace these deprecated methods:

```rust
// ❌ DEPRECATED - Replace with upsert_node()
create_node_for_date_with_id(...)
update_node(&NodeId, &str)
update_node_content(&NodeId, &str)
set_node_parent(&NodeId, Option<&NodeId>)
update_sibling_order(&NodeId, Option<&NodeId>, Option<&NodeId>)

// ✅ NEW - Single unified method
upsert_node(node_id, date, content, parent_id, before_sibling_id, node_type, metadata)
```

## Performance Considerations

- **Atomic Operations**: All node changes are atomic
- **Vector Control**: Only necessary content gets embedded
- **Rich Metadata**: Full source content returned for UI display
- **Caching**: Leverages existing smart embedding cache
- **Real AI**: Uses actual Ollama integration for production responses

This implementation provides the foundation for sophisticated desktop app features while maintaining performance and data integrity.