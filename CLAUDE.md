# CLAUDE.md

🚨 **STOP - READ WORKFLOW FIRST** 🚨
Before doing ANYTHING else, you MUST read the development workflow:
1. Read: `../nodespace-system-design/docs/development/workflow.md`
2. Check Linear for current tasks
3. Then return here for implementation guidance

❌ **FORBIDDEN:** Any code analysis, planning, or implementation before reading the workflow

## Development Workflow
**ALWAYS start with README.md** - This file contains the authoritative development workflow and setup instructions for this repository.

**Then return here** for repository-specific guidance and architecture details.

## Project Overview

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the **business logic orchestration layer** for NodeSpace - a Rust-based service that coordinates interactions between data storage, AI/NLP processing, and workflow automation. NodeSpace is an entity-centric, AI-powered knowledge management system using distributed contract architecture.

**Current Status**: Planning phase complete, awaiting implementation.

## 🎯 FINDING YOUR NEXT TASK

**See [development/workflow.md](../nodespace-system-design/docs/development/workflow.md)** for task management workflow.

## Essential Onboarding Process

**Follow the README.md onboarding steps exactly**:

1. **Read [NodeSpace System Design](../nodespace-system-design/README.md)** - Understand the full architecture
2. **Review [Development Workflow](../nodespace-system-design/docs/development/workflow.md)** - Process and procedures
3. **Study [Key Contracts](../nodespace-system-design/contracts/)** - Interface definitions to implement
4. **See [MVP User Flow](../nodespace-system-design/examples/mvp-user-flow.md)** - Implementation examples

## Development Commands

Once Rust project is initialized:

```bash
# Build and development
cargo build
cargo test
cargo fmt
cargo clippy

# MVP testing (planned)
cargo run --example mvp_workflow
cargo test --test integration
```

## System Architecture Context

**Technology Stack**: Rust + SurrealDB + Mistral.rs (Magistral-Small-2506, 23.6B, 128k context)

**Core Role**: Service orchestration implementing complete RAG pipeline (text capture → embedding → search → response)

**Distributed Contract Architecture**:
- `nodespace-core-types` - Shared data structures and interfaces  
- `nodespace-data-store` - SurrealDB implementation (owns DataStore trait)
- `nodespace-nlp-engine` - Mistral.rs integration (owns NLPEngine trait)
- `nodespace-workflow-engine` - Automation (owns WorkflowEngine trait)
- **`nodespace-core-logic`** - **This repository** (imports service traits)
- `nodespace-core-ui` - React components
- `nodespace-desktop-app` - Tauri application (imports service traits)

## Key Implementation Responsibilities

**MVP RAG Pipeline**:
1. **Create node** - Save text content and trigger embedding generation
2. **Generate embeddings** - Coordinate with NLP engine for vector generation  
3. **Search nodes** - Execute semantic and full-text search
4. **RAG queries** - Process questions with context retrieval and LLM generation
5. **Error handling** - Manage failures across all service layers

**Entity-Centric Operations**:
- Natural language entity creation with intent analysis
- Automatic relationship suggestions via LLM analysis
- Temporal reasoning for "recent", "last meeting" queries
- Cross-service error propagation and transaction management

## Development Workflow Integration

**Project Management**: All tasks tracked in [Linear workspace](https://linear.app/nodespace)
**Git Workflow**: Feature branches from main using Linear issue references (e.g., `feature/ns-11-description`)
**Contract Compliance**: Services own their interface traits, this repo imports them directly
**Cross-Repo Validation**: Use `../nodespace-system-design/validation/` framework

## Business Logic Patterns

**Service Coordination**: Orchestrate between data-store, nlp-engine, and workflow-engine services
**Error Handling**: Comprehensive error propagation across service boundaries with transaction management
**Performance**: Efficient service utilization and caching strategies for multiple external services
**Natural Language Processing**: Complete pipeline from intent analysis to structured entity creation

## Next Implementation Steps

1. Initialize Rust project structure (Cargo.toml, src/)
2. Import service traits from distributed repositories
3. Implement NodeSpaceService coordination layer
4. Build complete RAG pipeline with error handling
5. Create integration test suite validating all service interactions

## Architecture Dependencies

This repository coordinates but does not implement core services. Import traits from:
- DataStore trait from `nodespace-data-store`
- NLPEngine trait from `nodespace-nlp-engine`  
- WorkflowEngine trait from `nodespace-workflow-engine`

Focus on business logic orchestration, not individual service implementation.