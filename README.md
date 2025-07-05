# NodeSpace Core Logic

**Business logic and service orchestration for NodeSpace**

This repository implements the core business logic layer that orchestrates NodeSpace services, providing high-level operations and coordinating interactions between data storage, AI processing, and workflow automation.

## 🎯 Overview

NodeSpace Core Logic serves as the central orchestration layer in the NodeSpace ecosystem, implementing sophisticated business workflows that bridge multiple specialized services. It provides a unified API for complex operations while maintaining clean separation of concerns across the distributed architecture.

## 🚀 Key Features

- **🔄 RAG Pipeline Orchestration** - Complete text capture → embedding → search → response workflow
- **🏗️ Service Coordination** - Seamless integration between data store, NLP engine, and workflow services  
- **🧠 Enhanced Context Building** - Collective siblings approach for optimized multi-level embeddings
- **📊 Hierarchical Data Management** - Efficient node creation, querying, and relationship management
- **🖥️ Desktop Integration** - Enhanced APIs specifically designed for desktop application needs
- **⚡ Smart Embedding Cache** - Multi-level caching with relationship tracking and dependency invalidation

## 🆕 Recent Updates

### Collective Siblings Implementation ✅

Updated context building to use collective siblings approach for enhanced AI embeddings:

- **New `build_node_context()` method** - Builds proper `NodeContext` with collective siblings
- **Enhanced contextual embeddings** - Leverages NLP engine's optimized multi-level embedding generation
- **Improved performance** - Reduced context building overhead and better semantic understanding
- **Future-ready architecture** - Prepared for advanced context strategies (Phi4Enhanced, Adaptive)

```rust
// ✅ NEW: Collective sibling population
let context = NodeContext::default()
    .with_parent(parent_node)
    .with_siblings(vec![sibling1, sibling2, sibling3]);

// Enhanced contextual embedding generation
let embedding = service.get_enhanced_contextual_embedding(&node).await?;
```

## Architecture Context

Part of the NodeSpace system architecture:

1. [nodespace-core-types](https://github.com/malibio/nodespace-core-types) - Shared data structures and interfaces
2. [nodespace-data-store](https://github.com/malibio/nodespace-data-store) - LanceDB vector storage implementation
3. [nodespace-nlp-engine](https://github.com/malibio/nodespace-nlp-engine) - AI/ML processing and LLM integration  
4. [nodespace-workflow-engine](https://github.com/malibio/nodespace-workflow-engine) - Automation and event processing
5. **[nodespace-core-logic](https://github.com/malibio/nodespace-core-logic)** ← **You are here**
6. [nodespace-core-ui](https://github.com/malibio/nodespace-core-ui) - React components and UI
7. [nodespace-desktop-app](https://github.com/malibio/nodespace-desktop-app) - Tauri application shell

**Service Dependencies:**
- Imports `DataStore` trait from nodespace-data-store
- Imports `NLPEngine` trait from nodespace-nlp-engine  
- Imports `WorkflowEngine` trait from nodespace-workflow-engine
- Uses shared types from nodespace-core-types

## 📦 Installation & Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
nodespace-core-logic = { git = "https://github.com/malibio/nodespace-core-logic" }
```

### Basic Usage

```rust
use nodespace_core_logic::NodeSpaceService;

// Initialize service
let service = NodeSpaceService::create_with_paths(
    "/path/to/database",
    Some("/path/to/models")
).await?;

// Initialize AI services
service.initialize().await?;

// Create and store content with enhanced context
let node_id = service.create_knowledge_node(
    "Meeting notes from Q4 planning".to_string(),
    Some(metadata)
).await?;

// Enhanced contextual embedding generation
let embedding = service.get_enhanced_contextual_embedding(&node).await?;

// Semantic search and RAG queries
let results = service.semantic_search("Q4 planning", 10).await?;
let response = service.generate_ai_response(
    "What was discussed in Q4 planning?",
    &relevant_node_ids
).await?;
```

### Desktop Integration

```rust
use nodespace_core_logic::desktop_integration::EnhancedQueryResponse;

// Enhanced query processing for desktop apps
let response: EnhancedQueryResponse = service
    .process_query_enhanced("What are the key action items?".to_string())
    .await?;

// Rich metadata for UI rendering
println!("Sources: {}", response.sources.len());
println!("Confidence: {:.2}", response.overall_confidence);
```

## 🧪 Development

```bash
# Check compilation and run linting
cargo check && cargo clippy -- -D warnings

# Format code
cargo fmt

# Run tests including collective siblings tests
cargo test

# Run example
cargo run --example populate_from_json
```

The repository includes:
- **Comprehensive example** - JSON-based data population with hierarchical node creation
- **Unit tests** - Collective siblings implementation validation
- **Desktop integration** - Enhanced query responses with rich metadata

## 🧪 Testing

```bash
# Test collective siblings implementation
cargo test collective_sibling_tests

# Test NodeContext with siblings
cargo test test_node_context_collective_siblings
```

## 🛠️ Technology Stack

- **Language**: Rust (2021 edition)
- **Async Runtime**: Tokio
- **AI Integration**: Ollama with multi-level embeddings
- **Serialization**: Serde with JSON support
- **Error Handling**: Custom NodeSpaceError with comprehensive error types
- **Logging**: Enhanced logging with debug info for context building

## 🔧 Configuration

The service supports configurable context limits:

```rust
// Configure sibling and children context limits
let config = NLPConfig {
    performance_config: PerformanceConfig {
        max_siblings_context: Some(10),  // Limit siblings for context
        max_children_context: Some(10),  // Limit children for context
        ..Default::default()
    },
    ..Default::default()
};
```

## 📚 Related Documentation

For more details on the overall system architecture, see the complete NodeSpace ecosystem above in [Architecture Context](#architecture-context).

## 📋 API Reference

### Core Methods

- `build_node_context(node: &Node) -> NodeContext` - Build proper NodeContext with collective siblings
- `get_enhanced_contextual_embedding(node: &Node) -> Vec<f32>` - Enhanced embedding generation
- `process_query_enhanced(query: String) -> EnhancedQueryResponse` - Desktop-optimized query processing

### Desktop Integration

- `upsert_node()` - Universal node creation/updates with hierarchy support
- `EnhancedQueryResponse` - Rich response metadata for sophisticated UIs
- `NodeSource` - Detailed source information with confidence scores

---

*NodeSpace Core Logic provides the intelligent orchestration layer that makes NodeSpace's distributed AI-powered knowledge management system work seamlessly together, with enhanced collective siblings support for optimal AI understanding.*