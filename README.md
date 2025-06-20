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

- **`nodespace-core-types`** - Data structures and service interfaces
- **`nodespace-data-store`** - Node storage and search capabilities
- **`nodespace-nlp-engine`** - AI processing and embedding generation
- **`nodespace-workflow-engine`** - Event processing and automation

## 🚀 Getting Started

### **New to NodeSpace? Start Here:**
1. **Read [NodeSpace System Design](../nodespace-system-design/README.md)** - Understand the full architecture
2. **Check [Linear workspace](https://linear.app/nodespace)** - Find your current tasks (filter by `nodespace-core-logic`)
3. **Review [Development Workflow](../nodespace-system-design/docs/development-workflow.md)** - Process and procedures
4. **Study [Key Contracts](../nodespace-system-design/contracts/)** - Interface definitions you'll implement
5. **See [MVP User Flow](../nodespace-system-design/examples/mvp-user-flow.md)** - What you're building

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