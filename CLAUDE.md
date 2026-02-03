# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DevMan is an AI cognitive work management system - an external brain + project manager + quality assurance inspector for AI assistants. It structures AI's work outputs and decisions to be reusable and verifiable.

**Core value proposition:**
- Goal → Phase → Task → WorkRecord structured data model
- File-based JSON storage (`.devman/` directory)
- Extensible QualityEngine with generic and custom checks
- Knowledge service with retrieval and templates
- Progress tracking with blocker detection and time estimation
- MCP Server for AI interface (protocol layer complete, business logic in progress)

## Build & Test Commands

```bash
# Build all crates
cargo build --workspace

# Build specific crate
cargo build -p devman-cli    # CLI
cargo build -p devman-ai     # MCP Server

# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p devman-quality

# Run single test
cargo test -p devman-quality test_function_name

# Check formatting
cargo fmt --workspace

# Run linter
cargo clippy --workspace

# Install CLI/MCP Server locally
cargo install --path crates/cli --force
cargo install --path crates/ai --force
```

## Architecture (5-Layer Model)

```
Layer 5: Knowledge Service    (knowledge crate)
  - Knowledge storage, retrieval, templates

Layer 4: Quality Assurance    (quality crate)
  - Generic checks (compile, test, format, lint)
  - Custom checks with output parsing (Regex, JsonPath)
  - Human review integration (Slack, Email, Webhook)

Layer 3: Progress Tracking    (progress crate)
  - Goal/Phase/Task progress
  - Blocker detection (dependency cycles)
  - Time estimation (minutes + confidence)

Layer 2: Work Management      (work crate)
  - Task lifecycle (10 states)
  - Execution steps with tools
  - WorkRecord logging

Layer 1: Storage & State      (storage crate)
  - JsonStorage (file-based JSON)
  - Storage trait for extensions
```

## Crate Structure

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| `devman-core` | Data models | Goal, Project, Phase, Task, Knowledge, QualityCheck |
| `devman-storage` | Persistence | Storage trait, JsonStorage |
| `devman-quality` | Quality engine | QualityEngine, QualityGate, HumanReviewService |
| `devman-knowledge` | Knowledge service | KnowledgeService, TemplateRegistry |
| `devman-progress` | Progress tracking | ProgressTracker, BlockerDetector |
| `devman-work` | Work execution | WorkManager, WorkflowExecutor |
| `devman-tools` | Tool abstraction | Tool trait, BuiltinToolExecutor |
| `devman-ai` | AI interface | AIInterface, MCP Server |
| `devman-cli` | CLI entrypoint | Command handlers |

## Data Flow for Task Execution

1. AI creates task via `WorkManager.create_task()`
2. Knowledge retrieval via `KnowledgeService.search()`
3. Execute steps via `ToolExecutor`
4. Quality check via `QualityEngine.run_gate()`
5. Progress update via `ProgressTracker.update()`
6. Knowledge creation via `KnowledgeService.create()`

## Key Design Patterns

- **Async/await throughout**: All I/O operations use async/await (Tokio)
- **Trait-based abstractions**: Storage, Tool, QualityEngine, KnowledgeService are traits
- **State machines**: TaskState (10 states), StateTransition validation
- **ULID identifiers**: Sortable, globally unique IDs for all entities
- **Output parsing**: Regex, JsonPath, LineContains for validation

## Storage Location

Local data stored in `.devman/` directory (not committed). Each entity type has its own subdirectory with JSON files and optional `.meta.json` for version tracking.

## Commit Message Format

```
type(scope): description

types: feat, fix, docs, style, refactor, perf, test, chore
scopes: core, storage, quality, knowledge, progress, work, tools, ai, cli, docs
```

Example: `feat(quality): add security scan checker`

## Documentation

- Architecture: `docs/ARCHITECTURE.md`
- Design details: `docs/DESIGN.md`
- API reference: `docs/API.md`
- Quality extension: `docs/QUALITY_GUIDE.md`
- Knowledge management: `docs/KNOWLEDGE.md`
- Contributing: `docs/CONTRIBUTING.md`
- Current roadmap: `docs/TODO.md`
