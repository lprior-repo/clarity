# Implementation Plan: Intent-CLI Full Port (Gleam → Rust)

**Branch**: `002-interview-engine-port` | **Date**: 2026-02-27 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/kitty-specs/002-interview-engine-port/spec.md`

## Summary

Port the complete intent-cli codebase from Gleam to Rust, enabling single-binary distribution by removing the Gleam/Erlang runtime dependency. This is a **~15,242 line** porting effort across **44 modules**, providing full feature parity with the original Gleam implementation.

**Scope Change**: Originally scoped as "Phase 0: Interview Engine Port" (~3,370 lines). User selected **Option C: Full Intent-CLI Port** for complete feature parity.

## Technical Context

**Language/Version**: Rust 1.75+ (matching clarity-web crate)
**Primary Dependencies**: serde, serde_json, chrono, regex, thiserror, anyhow
**Storage**: JSONL files (no database), CUE schema files (copied as-is)
**Testing**: cargo test, criterion for benchmarking
**Target Platform**: Linux/macOS/Windows (single binary)
**Project Type**: Library crate integrated into clarity-web
**Performance Goals**: JSONL serialization < 10ms/100 answers, gap detection < 5ms/50 answers
**Constraints**: Zero unwrap/expect in production code, >80% test coverage
**Scale/Scope**: ~15,242 lines of Gleam → Rust, 44 modules, 6 CUE schemas

## Constitution Check

*No constitution file found - skipping.*

## Project Structure

### Documentation (this feature)

```
kitty-specs/002-interview-engine-port/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Gleam→Rust mapping research
├── data-model.md        # Entity relationships
├── quickstart.md        # Usage examples
└── contracts/           # API contracts
```

### Source Code (repository root)

```
clarity-web/src/intent/           # New module for ported intent-cli
├── mod.rs                         # Public API exports
├── types.rs                       # Core types (Spec, Feature, Behavior, etc.)
├── parser.rs                      # JSON to Spec parsing
├── loader.rs                      # CUE file loading
├── formats.rs                     # RFC-compliant validators
├── security.rs                    # Input validation, path traversal prevention
├── errors.rs                      # Contextual error reporting
│
├── interview/                     # Interview subsystem
│   ├── mod.rs
│   ├── types.rs                   # InterviewSession, Answer, Gap, Conflict
│   ├── engine.rs                  # Session management, gap/conflict detection
│   ├── storage.rs                 # JSONL persistence, session diffing
│   ├── questions.rs               # Question definitions
│   ├── question_loader.rs         # YAML/JSON question loading
│   └── contract.rs                # Interview contracts
│
├── plan/                          # Planning subsystem
│   ├── mod.rs
│   ├── plan_mode.rs               # Execution plan, dependency graph
│   ├── plan_next.rs               # Next action determination
│   ├── plan_emit_beads.rs         # Bead emission with idempotency
│   └── resolver.rs                # Dependency resolution
│
├── beads/                         # Bead generation subsystem
│   ├── mod.rs
│   ├── templates.rs               # Bead templates, 16-section CUE
│   ├── feedback.rs                # Bead feedback collection
│   └── stats.rs                   # Bead statistics
│
├── quality/                       # Quality analysis subsystem
│   ├── mod.rs
│   ├── analyzer.rs                # Quality scoring
│   ├── effects.rs                 # Second-order effect detection
│   ├── improver.rs                # Spec improvement suggestions
│   └── linter.rs                  # Spec linting
│
├── validation/                    # Validation subsystem
│   ├── mod.rs
│   ├── spec_validator.rs          # Comprehensive spec validation
│   ├── semantic_validator.rs      # Semantic validation
│   ├── validator.rs               # Response validation
│   ├── rule.rs                    # Validation rules
│   └── interpolation.rs           # Variable interpolation
│
├── batch/                         # Batch processing
│   ├── mod.rs
│   └── processor.rs               # Multi-spec processing
│
├── documents/                     # Document generation
│   ├── mod.rs
│   ├── vision.rs                  # Vision document generation
│   ├── ready.rs                   # Readiness checking
│   ├── acceptance_synthesizer.rs  # Acceptance test synthesis
│   └── spec_builder.rs            # Spec construction
│
├── templates/                     # Template generation
│   ├── mod.rs
│   └── spec_templates.rs          # Spec templates
│
├── cli/                           # CLI support (optional, for standalone use)
│   ├── mod.rs
│   ├── ui.rs                      # Terminal output helpers
│   ├── config.rs                  # Configuration management
│   └── env.rs                     # Environment handling
│
└── util/                          # Utilities
    ├── mod.rs
    ├── array_indexing.rs          # JSON array navigation
    ├── case_insensitive.rs        # Case-insensitive matching
    └── stdin.rs                   # Standard input handling

clarity-web/schemas/               # CUE schemas (copied as-is)
├── intent.cue                     # Core spec schema
├── interview.cue                  # Interview schema
├── kirk.cue                       # KIRK contract schema
├── questions.cue                  # Questions schema
├── enhanced-bead.cue              # Enhanced bead schema
├── ai_protocol.cue                # AI protocol schema
└── ai_interview.cue               # AI interview schema

clarity-web/tests/intent/          # Ported tests
├── types_test.rs
├── parser_test.rs
├── interview_engine_test.rs
├── interview_storage_test.rs
├── bead_templates_test.rs
├── quality_analyzer_test.rs
├── plan_mode_test.rs
├── spec_validator_test.rs
├── security_test.rs
├── formats_test.rs
├── interpolation_test.rs
└── integration/
    └── full_workflow_test.rs
```

**Structure Decision**: Library module under `clarity-web/src/intent/` with subsystem submodules. This allows gradual migration and clear ownership boundaries.

## Module Porting Inventory

Based on deep analysis of `/tmp/intent-cli/`:

### Tier 1: Core Types & Infrastructure (2,394 lines)

| File | Lines | Rust Target | Priority |
|------|-------|-------------|----------|
| `types.gleam` | 90 | `types.rs` | P0 |
| `question_types.gleam` | 42 | `interview/types.rs` | P0 |
| `errors.gleam` | 234 | `errors.rs` | P0 |
| `security.gleam` | 296 | `security.rs` | P0 |
| `formats.gleam` | 569 | `formats.rs` | P0 |
| `case_insensitive.gleam` | 85 | `util/case_insensitive.rs` | P1 |
| `array_indexing.gleam` | 287 | `util/array_indexing.rs` | P1 |
| `stdin.gleam` | 81 | `util/stdin.rs` | P2 |
| `cli_ui.gleam` | 72 | `cli/ui.rs` | P2 |
| `env.gleam` | 137 | `cli/env.rs` | P2 |
| `config.gleam` | 272 | `cli/config.rs` | P1 |

### Tier 2: Parsing & Loading (1,760 lines)

| File | Lines | Rust Target | Priority |
|------|-------|-------------|----------|
| `parser.gleam` | 320 | `parser.rs` | P0 |
| `loader.gleam` | 240 | `loader.rs` | P0 |
| `spec_validator.gleam` | 578 | `validation/spec_validator.rs` | P0 |
| `spec_linter.gleam` | 383 | `quality/linter.rs` | P1 |
| `spec_builder.gleam` | 221 | `documents/spec_builder.rs` | P1 |
| `answer_loader.gleam` | 262 | `interview/question_loader.rs` | P1 |

### Tier 3: Interview Engine (2,323 lines)

| File | Lines | Rust Target | Priority |
|------|-------|-------------|----------|
| `interview.gleam` | 851 | `interview/engine.rs` | P0 |
| `interview_storage.gleam` | 1162 | `interview/storage.rs` | P0 |
| `interview_contract.gleam` | 107 | `interview/contract.rs` | P1 |
| `interview_questions.gleam` | 88 | `interview/questions.rs` | P1 |
| `question_loader.gleam` | 476 | `interview/question_loader.rs` | P1 |

### Tier 4: Planning & Execution (1,560 lines)

| File | Lines | Rust Target | Priority |
|------|-------|-------------|----------|
| `plan_mode.gleam` | 923 | `plan/plan_mode.rs` | P0 |
| `plan_next.gleam` | 102 | `plan/plan_next.rs` | P1 |
| `plan_emit_beads.gleam` | 277 | `plan/plan_emit_beads.rs` | P1 |
| `resolver.gleam` | 258 | `plan/resolver.rs` | P1 |

### Tier 5: Bead Generation (1,188 lines)

| File | Lines | Rust Target | Priority |
|------|-------|-------------|----------|
| `bead_templates.gleam` | 778 | `beads/templates.rs` | P0 |
| `bead_feedback.gleam` | 410 | `beads/feedback.rs` | P1 |

### Tier 6: Quality Analysis (1,589 lines)

| File | Lines | Rust Target | Priority |
|------|-------|-------------|----------|
| `quality_analyzer.gleam` | 470 | `quality/analyzer.rs` | P0 |
| `effects_analyzer.gleam` | 286 | `quality/effects.rs` | P1 |
| `improver.gleam` | 403 | `quality/improver.rs` | P1 |

### Tier 7: Validation (1,122 lines)

| File | Lines | Rust Target | Priority |
|------|-------|-------------|----------|
| `validation.gleam` | 202 | `validation/mod.rs` | P1 |
| `validator.gleam` | 309 | `validation/validator.rs` | P1 |
| `rule.gleam` | 501 | `validation/rule.rs` | P1 |
| `semantic_validator.gleam` | 332 | `validation/semantic_validator.rs` | P1 |
| `interpolate.gleam` | 301 | `validation/interpolation.rs` | P0 |

### Tier 8: Documents & Templates (1,844 lines)

| File | Lines | Rust Target | Priority |
|------|-------|-------------|----------|
| `spec_templates.gleam` | 1206 | `templates/spec_templates.rs` | P1 |
| `vision_document.gleam` | 250 | `documents/vision.rs` | P2 |
| `ready_document.gleam` | 284 | `documents/ready.rs` | P2 |
| `acceptance_synthesizer.gleam` | 309 | `documents/acceptance_synthesizer.rs` | P2 |
| `init_prompt.gleam` | 115 | `cli/init_prompt.rs` | P2 |
| `flag_suggestions.gleam` | 188 | `cli/flag_suggestions.rs` | P2 |

### Tier 9: Batch Processing (487 lines)

| File | Lines | Rust Target | Priority |
|------|-------|-------------|----------|
| `batch.gleam` | 487 | `batch/processor.rs` | P2 |

### CUE Schemas (2,138 lines - copy as-is)

| File | Lines | Purpose |
|------|-------|---------|
| `intent.cue` | 113 | Core spec schema |
| `interview.cue` | 252 | Interview schema |
| `kirk.cue` | 237 | KIRK contract schema |
| `questions.cue` | 480 | Questions schema |
| `enhanced-bead.cue` | 570 | Enhanced bead schema |
| `ai_protocol.cue` | 143 | AI protocol schema |
| `ai_interview.cue` | 165 | AI interview schema |
| `custom-questions.cue` | 178 | Custom questions schema |

## Implementation Phases

### Phase 1: Foundation (Tier 1) - Days 1-3

1. **Types Module** (`types.rs`)
   - Port `Spec`, `Feature`, `Behavior`, `Verification`, `Invariant`, `AntiPattern`, `AIHints`
   - Derive `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`
   - Zero `unwrap()` - use `Result` throughout

2. **Error Handling** (`errors.rs`)
   - `ContextualError` with field suggestions (Levenshtein distance)
   - `ValidationError`, `FieldFailure` types
   - User-friendly error formatting

3. **Security** (`security.rs`)
   - Path traversal prevention
   - Shell metacharacter detection
   - ReDoS prevention
   - Session ID validation

4. **Format Validators** (`formats.rs`)
   - RFC 5322 email validation
   - RFC 4122 UUID validation
   - RFC 3986 URI validation
   - ISO 8601 datetime with calendar validation

### Phase 2: Parsing & Loading (Tier 2) - Days 4-6

1. **Parser** (`parser.rs`)
   - JSON to Spec parsing
   - Dynamic to JSON conversion
   - Field sanitization

2. **Loader** (`loader.rs`)
   - CUE file loading via `cue` CLI
   - Validation integration
   - Error handling

3. **Spec Validator** (`validation/spec_validator.rs`)
   - CUE syntax validation
   - Circular dependency detection (DFS)
   - Duplicate behavior detection
   - Reference validation

### Phase 3: Interview Engine (Tiers 3-4) - Days 7-12

1. **Interview Types** (`interview/types.rs`)
   - `Profile`, `InterviewStage`, `Perspective`, `QuestionPriority`, `QuestionCategory`
   - `Answer`, `Gap`, `Conflict`, `ConflictResolution`, `InterviewSession`

2. **Interview Engine** (`interview/engine.rs`)
   - Session creation and management
   - Gap detection (profile-specific required fields)
   - Conflict detection (CAP theorem, anonymous+audit)
   - Phase gating
   - Round management

3. **Interview Storage** (`interview/storage.rs`)
   - JSONL serialization/deserialization
   - Session diffing
   - History tracking
   - Snapshot creation

4. **Plan Mode** (`plan/plan_mode.rs`)
   - Execution plan computation
   - Dependency graph building
   - Risk assessment
   - Phase ordering

### Phase 4: Bead Generation (Tier 5) - Days 13-15

1. **Bead Templates** (`beads/templates.rs`)
   - Profile-specific bead generators
   - 16-section enhanced CUE template
   - JSONL output
   - Priority sorting

2. **Bead Feedback** (`beads/feedback.rs`)
   - Feedback collection
   - Status tracking

### Phase 5: Quality Analysis (Tier 6) - Days 16-18

1. **Quality Analyzer** (`quality/analyzer.rs`)
   - Coverage scoring
   - Clarity scoring
   - Testability scoring
   - AI readiness scoring

2. **Effects Analyzer** (`quality/effects.rs`)
   - State change detection
   - Cascade effect detection
   - Race condition detection
   - Rollback requirement detection

### Phase 6: Validation (Tier 7) - Days 19-21

1. **Interpolation** (`validation/interpolation.rs`)
   - Variable interpolation with `${var}` syntax
   - Array indexing support
   - JSON navigation

2. **Validation Rules** (`validation/rule.rs`)
   - Rule definitions
   - Field validation

3. **Semantic Validator** (`validation/semantic_validator.rs`)
   - Semantic analysis
   - Cross-reference validation

### Phase 7: Documents & Templates (Tier 8) - Days 22-25

1. **Spec Templates** (`templates/spec_templates.rs`)
   - Template generation for different profiles
   - Boilerplate generation

2. **Document Generation** (`documents/`)
   - Vision document
   - Ready document
   - Acceptance test synthesis

### Phase 8: Batch & CLI (Tier 9) - Days 26-27

1. **Batch Processing** (`batch/processor.rs`)
   - Multi-spec processing
   - Summary reports

2. **CLI Support** (`cli/`)
   - Configuration management
   - Environment handling
   - Flag suggestions

### Phase 9: Testing & Integration - Days 28-30

1. Port unit tests from Gleam
2. Integration tests with clarity-web
3. Performance benchmarks
4. Documentation

## Gleam → Rust Mapping Reference

| Gleam | Rust |
|-------|------|
| `Result(a, e)` | `Result<a, e>` |
| `option.Option(a)` | `Option<a>` |
| `dict.Dict(k, v)` | `HashMap<k, v>` |
| `list.List(a)` | `Vec<a>` |
| `String` | `String` |
| `Int` | `i64` |
| `Float` | `f64` |
| `Bool` | `bool` |
| `Nil` | `()` |
| Pattern matching | `match` expressions |
| Pipe operator `\|>` | Method chaining / iterators |
| `fn(a) -> b` | `fn(a) -> b` or `impl Fn(a) -> b` |
| `Result.then` | `and_then` / `?` operator |
| `list.map` | `.iter().map()` |
| `list.filter` | `.iter().filter()` |
| `list.fold` | `.iter().fold()` |
| `string.split` | `.split()` |
| `json.decode` | `serde_json::from_str` |
| `json.encode` | `serde_json::to_string` |

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| JSONL format incompatibility | High | Medium | Port decoder tests first, validate against existing files |
| Missing edge cases | Medium | High | Port unit tests alongside code, run both implementations in parallel |
| Performance regression | Medium | Low | Benchmark critical paths, use Rust idioms |
| API drift from original | Medium | Medium | Maintain API compatibility tests |
| Binary size bloat | Low | Low | Use `strip`, LTO, minimize dependencies |
| Type mapping errors | Medium | Medium | Property-based testing for round-trip serialization |

## Success Criteria

1. **API Parity**: All 44 modules have Rust equivalents with compatible APIs
2. **Test Coverage**: > 80% line coverage on all intent modules
3. **Compatibility**: Existing `.intent/` JSONL files load correctly
4. **No Panics**: Zero `unwrap()`, `expect()`, or panic paths in production code
5. **Documentation**: All public types and functions have rustdoc comments
6. **Performance**: JSONL ops < 10ms/100 answers, gap detection < 5ms/50 answers
7. **Binary Size**: Total addition < 2MB to release binary
8. **CUE Schemas**: All schemas copied and validated

## Dependencies to Add

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
regex = "1.10"
thiserror = "1.0"
anyhow = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"
tempfile = "3.10"
```

## Next Steps

1. Run `/spec-kitty.tasks` to generate work packages
2. Begin with Tier 1 (Foundation) modules
3. Port tests alongside code
4. Validate against existing `.intent/` directories
