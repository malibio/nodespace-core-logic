# NodeSpace Core Logic

**Business logic and service orchestration for NodeSpace**

This repository implements the core business logic layer that orchestrates NodeSpace services, providing high-level operations and coordinating interactions between data storage, AI processing, and workflow automation.

## 🎯 Overview

NodeSpace Core Logic serves as the central orchestration layer in the NodeSpace ecosystem, implementing sophisticated business workflows that bridge multiple specialized services. It provides a unified API for complex operations while maintaining clean separation of concerns across the distributed architecture.

## 🚀 Key Features

- **🔄 RAG Pipeline Orchestration** - Complete text capture → embedding → search → response workflow
- **🏗️ Service Coordination** - Seamless integration between data store, NLP engine, and workflow services  
- **🧠 Smart Embedding Cache** - Multi-level caching with contextual, hierarchical, and individual strategies
- **📊 Hierarchical Data Management** - Efficient node creation, querying, and relationship management
- **⚡ Performance Optimization** - Root-based fetching and optimized query patterns
- **🛡️ Robust Error Handling** - Comprehensive error propagation across service boundaries
- **🖥️ Desktop Integration** - Enhanced APIs specifically designed for desktop application needs

## 🏗️ Architecture

NodeSpace Core Logic operates within a distributed contract architecture:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│  Data Store     │    │   NLP Engine     │    │  Workflow Engine    │
│  (LanceDB)      │    │    (Ollama)      │    │  (Automation)       │
└─────┬───────────┘    └────────┬─────────┘    └──────────┬──────────┘
      │                         │                         │
      │ DataStore trait         │ NLPEngine trait         │ WorkflowEngine trait
      │                         │                         │
      └─────────────────────────┼─────────────────────────┘
                                │
                    ┌───────────▼──────────┐
                    │   NodeSpace Core     │
                    │      Logic           │
                    │  (Orchestration)     │
                    └───────────┬──────────┘
                                │
                    ┌───────────▼──────────┐
                    │   Desktop App        │
                    │   Integration        │
                    └──────────────────────┘
```

### Service Dependencies

- **`nodespace-core-types`** - Shared data structures and type definitions
- **`nodespace-data-store`** - Vector database storage and retrieval (imports `DataStore` trait)
- **`nodespace-nlp-engine`** - AI processing and embeddings (imports `NLPEngine` trait)  
- **`nodespace-workflow-engine`** - Event processing and automation (imports `WorkflowEngine` trait)

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
nodespace-core-logic = { path = "../nodespace-core-logic" }
```

## 🔧 Usage

### Basic Service Initialization

```rust
use nodespace_core_logic::NodeSpaceService;

// Initialize with default configuration
let service = NodeSpaceService::new().await?;

// Or with custom paths
let service = NodeSpaceService::create_with_paths(
    "/path/to/database",
    Some("/path/to/models")
).await?;

// Initialize AI services
service.initialize().await?;
```

### Core Operations

```rust
// Create and store content
let node_id = service.create_knowledge_node(
    "Meeting notes from Q4 planning session".to_string(),
    Some(metadata)
).await?;

// Semantic search
let results = service.semantic_search("Q4 planning", 10).await?;

// RAG query processing
let response = service.generate_ai_response(
    "What was discussed in Q4 planning?",
    &relevant_node_ids
).await?;
```

### Enhanced Desktop Integration

```rust
use nodespace_core_logic::desktop_integration::*;

// Rich query processing for desktop UI
let enhanced_response = service.process_query_enhanced(
    "Find all marketing campaign data".to_string()
).await?;

// Universal node management
service.upsert_node(
    node_id,
    date,
    content,
    parent_id,
    before_sibling_id,
    node_type,
    metadata
).await?;
```

## 📊 Core Workflows

### RAG Pipeline

1. **Content Ingestion** - Store text content with automatic embedding generation
2. **Semantic Search** - Vector similarity search across stored content  
3. **Context Assembly** - Intelligent context selection from search results
4. **AI Generation** - LLM-powered response generation with retrieved context
5. **Response Enhancement** - Rich metadata and source attribution

### Hierarchical Data Management

- **Date-based Organization** - Automatic date node creation and management
- **Parent-Child Relationships** - Flexible hierarchical content organization
- **Sibling Ordering** - Maintain content order within hierarchies
- **Efficient Querying** - Optimized root-based fetching for large hierarchies

## 🧪 Development

### Running Tests

```bash
# Check compilation
cargo check

# Run linting
cargo clippy -- -D warnings

# Format code
cargo fmt

# Run example
cargo run --example populate_from_json
```

### Example Usage

The repository includes a comprehensive example demonstrating JSON-based data population:

```bash
cargo run --example populate_from_json
```

This example shows:
- Service initialization and configuration
- JSON data parsing and validation  
- Hierarchical node creation with proper relationships
- Error handling and data verification

## 🔍 Key Components

### `NodeSpaceService`

The main service struct that coordinates all operations:

- **CoreLogic trait** - Essential business operations
- **HierarchyComputation trait** - Tree structure management
- **Smart caching** - Multi-level embedding and hierarchy caches
- **Service lifecycle** - Initialization, readiness checks, and cleanup

### Desktop Integration Module

Enhanced APIs for desktop application needs:

- **`EnhancedQueryResponse`** - Rich query results with metadata
- **`NodeSource`** - Detailed source information for UI display
- **Universal upsert operations** - Idempotent node creation/updates

### Performance Features

- **Root-based fetching** - Efficient O(1) hierarchy queries instead of O(N) scans
- **Smart embedding cache** - Context-aware caching with dependency invalidation
- **Batch operations** - Optimized bulk data processing
- **Connection pooling** - Efficient resource utilization

## 🛠️ Technology Stack

- **Language**: Rust (2021 edition)
- **Async Runtime**: Tokio
- **Serialization**: Serde with JSON support
- **Logging**: Standard Rust logging ecosystem
- **Error Handling**: Custom NodeSpaceError with comprehensive error types

## 📈 Performance Characteristics

- **Sub-second query responses** for semantic search operations
- **Efficient memory usage** through smart caching strategies  
- **Scalable hierarchy management** supporting thousands of nodes
- **Optimized embedding generation** with intelligent cache invalidation

## 🤝 Contributing

This repository follows Rust best practices:

- All code must pass `cargo clippy -- -D warnings`
- Consistent formatting with `cargo fmt`
- Comprehensive error handling with proper error types
- Clear documentation and examples

## 📚 Related Repositories

- **[nodespace-system-design](../nodespace-system-design)** - Architecture documentation and contracts
- **[nodespace-data-store](../nodespace-data-store)** - LanceDB vector storage implementation
- **[nodespace-nlp-engine](../nodespace-nlp-engine)** - Ollama AI processing engine
- **[nodespace-desktop-app](../nodespace-desktop-app)** - Tauri desktop application

---

*NodeSpace Core Logic provides the intelligent orchestration layer that makes NodeSpace's distributed AI-powered knowledge management system work seamlessly together.*