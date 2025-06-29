# ⚠️ BEFORE STARTING ANY WORK
👉 **STEP 1**: Read development workflow: `../nodespace-system-design/docs/development/workflow.md`
👉 **STEP 2**: Check Linear for assigned tasks
👉 **STEP 3**: Repository-specific patterns below

**This README.md only contains**: Repository-specific Rust business logic patterns

# NodeSpace Core Logic

**Business logic and service orchestration for NodeSpace**

This repository implements the business logic layer that orchestrates all NodeSpace services. It provides high-level operations and coordinates interactions between data storage, AI processing, and workflow automation.

## 🎯 Purpose

- **Service orchestration** - Coordinate interactions between all system components
- **Business logic** - Implement high-level operations and workflows
- **MVP RAG flow** - Complete text capture → embedding → search → response pipeline
- **API abstraction** - Simplified interface for desktop app integration

## 📦 Key Features

- **Complete RAG pipeline** - End-to-end implementation of the MVP workflow
- **Service coordination** - Manage dependencies between data, AI, and workflow services
- **Error propagation** - Comprehensive error handling across service boundaries
- **Transaction management** - Ensure data consistency across operations
- **Performance optimization** - Efficient service utilization and caching

## 🔗 Dependencies

- **`nodespace-core-types`** - Shared data structures (NodeId, Node, NodeSpaceResult)
- **`nodespace-data-store`** - Imports `DataStore` trait, node storage and search
- **`nodespace-nlp-engine`** - Imports `NLPEngine` trait, AI processing and embeddings
- **`nodespace-workflow-engine`** - Imports `WorkflowEngine` trait, event processing

## 🏗️ Distributed Architecture Role

This repository **orchestrates services** using NodeSpace's distributed contract architecture:
- **Imports service traits**: DataStore, NLPEngine, WorkflowEngine from their respective repositories
- **Provides coordination**: High-level business logic that coordinates multiple services
- **MVP implementation**: Complete RAG workflow (text storage → embedding → search → AI response)

## 🚀 Getting Started

### **New to NodeSpace? Start Here:**
1. **📖 System Context**: Read [NodeSpace System Design](../nodespace-system-design) for complete architecture
2. **📋 Current Work**: Check [Linear workspace](https://linear.app/nodespace) for tasks (filter: `nodespace-core-logic`)
3. **🤖 Development**: See [CLAUDE.md](./CLAUDE.md) for autonomous development workflow
4. **🎯 MVP Goal**: Orchestrate complete RAG workflow across all services

### **Development Setup:**
```bash
# Add to your Cargo.toml
[dependencies]
nodespace-core-logic = { git = "https://github.com/malibio/nodespace-core-logic" }

# Use in your code
use nodespace_core_logic::NodeSpaceService;

let service = NodeSpaceService::new().await?;
let response = service.process_rag_query(query).await?;
```

## 🏗️ Architecture Context

Part of the [NodeSpace system architecture](../nodespace-system-design/README.md):

1. `nodespace-core-types` - Shared data structures and interfaces
2. `nodespace-data-store` - Database and vector storage
3. `nodespace-nlp-engine` - AI/ML processing and LLM integration
4. `nodespace-workflow-engine` - Automation and event processing
5. **`nodespace-core-logic`** ← **You are here**
6. `nodespace-core-ui` - React components and UI
7. `nodespace-desktop-app` - Tauri application shell

## 🔄 MVP Implementation

The core business logic implements the complete RAG workflow:

1. **Create node** - Save text content and trigger embedding generation
2. **Generate embeddings** - Coordinate with NLP engine for vector generation
3. **Search nodes** - Execute semantic and full-text search
4. **RAG queries** - Process questions with context retrieval and LLM generation
5. **Error handling** - Manage failures across all service layers

## 🧪 Testing

```bash
# Run all tests including integration tests
cargo test

# Test complete MVP workflow
cargo run --example mvp_workflow

# Integration test with all services
cargo test --test integration
```

---

**Project Management:** All development tasks tracked in [Linear workspace](https://linear.app/nodespace)