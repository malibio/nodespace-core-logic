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
- **🖥️ Desktop Integration** - Enhanced APIs specifically designed for desktop application needs

## 🏗️ Architecture

NodeSpace Core Logic operates within a distributed contract architecture:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Data Store  │    │ NLP Engine  │    │ Workflow    │
│ (LanceDB)   │    │ (Ollama)    │    │ Engine      │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                  │
       └──────────────────┼──────────────────┘
                          │
              ┌───────────▼──────────┐
              │  NodeSpace Core      │
              │  Logic               │
              │  (Orchestration)     │
              └──────────────────────┘
```

**Service Dependencies:**
- **`nodespace-core-types`** - Shared data structures and type definitions
- **`nodespace-data-store`** - Vector database storage and retrieval (imports `DataStore` trait)
- **`nodespace-nlp-engine`** - AI processing and embeddings (imports `NLPEngine` trait)  
- **`nodespace-workflow-engine`** - Event processing and automation (imports `WorkflowEngine` trait)

## 📦 Installation & Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
nodespace-core-logic = { path = "../nodespace-core-logic" }
```

Basic usage:

```rust
use nodespace_core_logic::NodeSpaceService;

// Initialize service
let service = NodeSpaceService::create_with_paths(
    "/path/to/database",
    Some("/path/to/models")
).await?;

// Initialize AI services
service.initialize().await?;

// Create and store content
let node_id = service.create_knowledge_node(
    "Meeting notes from Q4 planning".to_string(),
    Some(metadata)
).await?;

// Semantic search and RAG queries
let results = service.semantic_search("Q4 planning", 10).await?;
let response = service.generate_ai_response(
    "What was discussed in Q4 planning?",
    &relevant_node_ids
).await?;
```

## 🧪 Development

```bash
# Check compilation and run linting
cargo check && cargo clippy -- -D warnings

# Format code
cargo fmt

# Run example
cargo run --example populate_from_json
```

The repository includes a comprehensive example demonstrating JSON-based data population that shows service initialization, data parsing, hierarchical node creation, and error handling.

## 🛠️ Technology Stack

- **Language**: Rust (2021 edition)
- **Async Runtime**: Tokio
- **Serialization**: Serde with JSON support
- **Error Handling**: Custom NodeSpaceError with comprehensive error types

## 📚 Related Repositories

- **[nodespace-system-design](../nodespace-system-design)** - Architecture documentation and contracts
- **[nodespace-data-store](../nodespace-data-store)** - LanceDB vector storage implementation
- **[nodespace-nlp-engine](../nodespace-nlp-engine)** - Ollama AI processing engine
- **[nodespace-desktop-app](../nodespace-desktop-app)** - Tauri desktop application

---

*NodeSpace Core Logic provides the intelligent orchestration layer that makes NodeSpace's distributed AI-powered knowledge management system work seamlessly together.*