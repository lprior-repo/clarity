# Clarity Architecture Documentation

This directory contains comprehensive architecture documentation for the Clarity Planner project.

## Documents

### [Discover Phase Architecture](./discover-phase.md)
**564 lines** | Last updated: 2025-02-25

Covers the redesigned Discover phase with dual-mode input (Express/Guided):

- **Mode Flows**: Detailed flows for Express (freeform + AI extraction) and Guided (sequential + AI suggestions)
- **Component Hierarchy**: Complete tree of UI components and their relationships
- **Data Flow Diagrams**: Visual representations of data movement through the system
- **Extraction Provider Architecture**: AI provider trait and implementations
- **Quality Scoring Integration**: How the 5-dimension quality score gates progression
- **Mental Lattice Triggers**: When each lattice module executes
- **State Persistence**: Write patterns and recovery strategies
- **Extension Points**: How to add new providers, fields, and quality dimensions

**Key Sections**:
- Express mode flow with field extraction pipeline
- Guided mode with sequential questions and follow-ups
- ExtractionProvider trait for AI abstraction
- Quality gate behavior (70% threshold)
- redb persistence patterns

---

### [Mental Lattice Architecture](./mental-lattice.md)
**932 lines** | Last updated: 2025-02-25

Comprehensive documentation of the Mental Lattice analysis framework:

- **Module Descriptions**: All 7 lattice modules with function signatures
  - EARS (Easy Approach to Requirements Syntax)
  - Inversion (assumption challenging)
  - Effects (dependency tracing)
  - Quality (multi-dimensional scoring)
  - Premortem (failure scenario analysis)
  - Coverage (use case coverage analysis)
  - Compact (artifact compaction)
- **Phase Triggers**: When each module executes during the workflow
- **Usage Examples**: Code samples for each module
- **Extension Points**: How to add new modules and patterns
- **Performance Considerations**: Time complexity and optimization strategies
- **Integration with UI**: Reactivity patterns and error display

**Key Sections**:
- EARS pattern types (Ubiquitous, State-driven, Event-driven, Unwanted, Optional)
- Inversion challenge patterns (Negation, Counterexample, Edge Case, Reversal)
- Quality dimensions (Completeness, Consistency, Testability, Clarity, Security)
- Premortem failure categories (Technical, User, Business, Security)
- Complete usage examples for each phase

---

### [Storage Layer Architecture](./storage-layer.md)
**719 lines** | Last updated: 2025-02-25

Complete guide to redb-based persistence:

- **Database Schema**: Table definitions and value types
  - `answers` - user responses
  - `extractions` - AI field extraction cache
  - `project_metadata` - project state
  - `lattice_cache` - Mental lattice results
- **Database Location**: Cross-platform path resolution
- **Write Patterns**: Immediate writes, transactions, upserts, cache invalidation
- **Read Patterns**: Single record lookup, bulk reads, projections
- **Backup and Restore**: Automatic daily backups, manual backup/restore
- **Error Handling**: StorageError types and propagation
- **Performance Considerations**: Latency metrics and optimization strategies
- **Testing**: In-memory and temporary database patterns
- **Configuration**: Environment variables and file-based config

**Key Sections**:
- redb table definitions with JSON value types
- Immediate write pattern with ACID guarantees
- Automatic backup system with daily rotation
- Schema versioning and data migration
- Integration with Dioxus reactive state

---

## Architecture Diagrams

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Clarity Web Application                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────────────────────────────────────────────────────┐     │
│  │                    UI Layer (Dioxus)                    │     │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐│     │
│  │  │ ExpressFlow  │  │ GuidedFlow   │  │ QualityScore ││     │
│  │  └──────────────┘  └──────────────┘  └──────────────┘│     │
│  └───────────────────────────┬────────────────────────────┘     │
│                              │                                  │
│  ┌───────────────────────────▼────────────────────────────┐     │
│  │                  Extraction Provider Trait              │     │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐│     │
│  │  │ OpenCode     │  │ Claude (fut) │  │ Mock (test)  ││     │
│  │  └──────────────┘  └──────────────┘  └──────────────┘│     │
│  └───────────────────────────┬────────────────────────────┘     │
│                              │                                  │
│  ┌───────────────────────────▼────────────────────────────┐     │
│  │              Mental Lattice (Rust)                       │     │
│  │  ┌──────┐ ┌─────────┐ ┌─────────┐ ┌──────┐ ┌─────┐  │     │
│  │  │ EARS │ │Inversion│ │ Effects │ │Qual. │ │Prem. │  │     │
│  │  └──────┘ └─────────┘ └─────────┘ └──────┘ └─────┘  │     │
│  │  ┌──────────┐ ┌──────────┐                            │     │
│  │  │Coverage  │ │ Compact  │                            │     │
│  │  └──────────┘ └──────────┘                            │     │
│  └───────────────────────────┬────────────────────────────┘     │
│                              │                                  │
│  ┌───────────────────────────▼────────────────────────────┐     │
│  │                 Storage Layer (redb)                    │     │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────┐│     │
│  │  │ answers │ │extract. │ │metadata │ │lattice_cache││     │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────────┘│     │
│  └───────────────────────────────────────────────────────┘     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow: Express Mode

```
User Input → Textarea
                │
                ├─→ [typing pause 3s] → Auto-Extract
                └─→ [button click] → Manual-Extract
                                │
                                ▼
                    ExtractionProvider::extract_fields()
                                │
                                ▼
                        ┌───────────────────────┐
                        │ ExtractedFields        │
                        │ - 5 fields             │
                        │ - Confidence scores    │
                        └───────────────────────┘
                                │
                                ▼
                        ┌───────────────────────┐
                        │ Field Review Cards    │
                        │ - Editable            │
                        │ - Lockable            │
                        └───────────────────────┘
                                │
                                ▼
                        ┌───────────────────────┐
                        │ User Confirmation    │
                        └───────────────────────┘
                                │
                                ▼
                    ┌───────────────────────────────┐
                    │ Mental Lattice Analysis       │
                    │ - EARS parsing                │
                    │ - Inversion challenges        │
                    │ - Effects tracing            │
                    │ - Quality scoring            │
                    └───────────────────────────────┘
                                │
                                ▼
                        ┌───────────────────────┐
                        │ Quality Gate           │
                        │ Score ≥ 70%?           │
                        │ - Yes: Continue CTA    │
                        │ - No: Improvements     │
                        └───────────────────────┘
```

### Sequence Diagram: Extraction Flow

```
User          ExpressFlow    ExtractionProvider    OpenCodeAPI     RedbStore
 │                 │                  │                │             │
 │─Type text──────>│                  │                │             │
 │                 │                  │                │             │
 │<─Show───────────│                  │                │             │
 │  char count     │                  │                │             │
 │                 │                  │                │             │
 │─Click Extract──>│                  │                │             │
 │                 │                  │                │             │
 │                 │─extract_fields─>│                │             │
 │                 │  (input, schema)│                │             │
 │                 │                  │                │             │
 │                 │                  │─HTTP POST─────>│             │
 │                 │                  │  /extract      │             │
 │                 │                  │                │             │
 │                 │                  │<─JSON response─│             │
 │                 │<──ExtractedFields│                │             │
 │                 │                  │                │             │
 │<─Show Cards────│                  │                │             │
 │  with conf.     │                  │                │             │
 │                 │                  │                │             │
 │─Edit field─────>│                  │                │             │
 │                 │                  │                │             │
 │─Confirm All───>│                  │                │             │
 │                 │                  │                │             │
 │                 │─save_answer───────>──────────────>│             │
 │                 │                  │                │             │
 │                 │─save_extraction_cache─────────────>│             │
 │                 │                  │                │             │
 │                 │                  │                │             │
 │                 │─Trigger Mental Lattice────────────>│             │
 │                 │  (ears, inversion, effects)        │             │
 │                 │                  │                │             │
 │<─Quality Score─│                  │                │             │
 │  (or errors)    │                  │                │             │
```

## Quick Reference

### Quality Dimensions

| Dimension | Weight | Calculation | Threshold |
|-----------|--------|-------------|-----------|
| Completeness | 20% | % of required fields | All 5 fields filled |
| Consistency | 20% | No contradictions | Zero conflicts |
| Testability | 20% | EARS with acceptance criteria | ≥ 80% |
| Clarity | 20% | Low complexity/jargon | Score ≥ 70 |
| Security | 20% | Auth/encryption/validation | All 3 areas |

**Overall Gate**: Score ≥ 70% to progress to Define phase

### EARS Patterns

| Pattern | Syntax | Example |
|---------|--------|---------|
| Ubiquitous | "The system shall..." | "The system shall authenticate users" |
| State-driven | "When X, the system shall Y..." | "When logged in, show dashboard" |
| Event-driven | "During X, the system shall Y..." | "During startup, initialize services" |
| Unwanted | "If X, the system shall NOT..." | "If invalid, shall not grant access" |
| Optional | "Where X, the system shall Y..." | "Where premium, enable features" |

### Storage Tables

| Table | Key | Value | Use |
|-------|-----|-------|-----|
| `answers` | step_id | AnswerRecord (JSON) | User responses |
| `extractions` | input_hash | ExtractionCache (JSON) | AI extraction results |
| `project_metadata` | "metadata" | ProjectMetadata (JSON) | Project state |
| `lattice_cache` | phase | LatticeCache (JSON) | Lattice outputs |

### Database Location

```
~/.local/share/clarity/projects/{project_id}/data.redb
```

### Configuration Files

```
~/.config/clarity/
├── ai.toml              # AI provider configuration
└── config.toml          # Storage and UI settings
```

## Development Workflow

### 1. Understanding the Codebase

Start here:
1. Read [Discover Phase Architecture](./discover-phase.md) for UI flow
2. Read [Mental Lattice Architecture](./mental-lattice.md) for analysis modules
3. Read [Storage Layer Architecture](./storage-layer.md) for persistence

### 2. Adding New Features

**New Extraction Provider**:
1. Implement `ExtractionProvider` trait
2. Add configuration to `ai.toml`
3. Register in provider factory
4. Update [Discover Phase](./discover-phase.md#extraction-provider-architecture) docs

**New Lattice Module**:
1. Create module in `clarity-web/src/lattice/`
2. Export from `lattice/mod.rs`
3. Wire into phase trigger
4. Document in [Mental Lattice](./mental-lattice.md#extension-points)

**New Storage Table**:
1. Add const to `tables` module
2. Implement read/write methods
3. Update schema version
4. Document in [Storage Layer](./storage-layer.md#database-schema)

### 3. Testing

Each module includes comprehensive tests:

```bash
# Test all architecture components
cargo test --package clarity-web --lib lattice
cargo test --package clarity-web --lib storage

# Test specific module
cargo test --package clarity-web --lib lattice::ears
cargo test --package clarity-web --lib lattice::inversion
cargo test --package clarity-web --lib lattice::quality
```

## Contributing

When making architectural changes:

1. **Update Documentation**: Keep these docs in sync with code
2. **Add Examples**: Include usage examples for new features
3. **Update Diagrams**: Modify sequence diagrams for new flows
4. **Document Trade-offs**: Explain why specific decisions were made

## Related Documentation

- [Discover Phase UX Redesign Plan](../plans/2026-02-25-discover-phase-ux-redesign.md) - Original design document
- [QUALITY_MODULE_SUMMARY.md](../QUALITY_MODULE_SUMMARY.md) - Quality scoring implementation notes
- [storage-path-util-implementation.md](../storage-path-util-implementation.md) - Path resolution implementation

---

**Last Updated**: 2025-02-25
**Maintained By**: Clarity Development Team
