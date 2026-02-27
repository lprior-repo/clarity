# Claude Instructions for Intent CLI

**Note**: This project uses [br (beads_rust)](https://github.com/steveyegge/beads) for issue tracking. Use `br` commands instead of markdown TODOs. See AGENTS.md for workflow details.
**Note:** `br` is non-invasive and never executes git commands. After `br sync --flush-only`, you must manually run `git add .beads/ && git commit`.

## Quick Reference

```bash
br ready --json           # Find ready work
br update <id> --status in_progress --json  # Claim work
br close <id> --reason "Done" --json        # Complete work
```

## Beads Viewer (bv) - Robot Commands Only

**CRITICAL**: Use ONLY `--robot-*` flags. Bare `bv` launches an interactive TUI that blocks your session.

```bash
bv --robot-triage          # Comprehensive analysis with recommendations
bv --robot-next            # Single top pick with claim command
bv --robot-plan            # Parallel execution tracks
bv --robot-insights        # Full metrics (PageRank, critical path)
bv --robot-graph --graph-format=json  # Dependency graph export
```

## Project Context

This is **Intent**, a planning and bead generation tool written in Gleam. It:
- Runs interactive interviews to capture requirements
- Generates structured CUE specifications from interviews
- Creates beads (tasks) from specifications for use with br (beads_rust)
- Analyzes specifications for quality, security, and second-order effects
- Supports multiple output formats (vision documents, ready documents, etc.)

## Key Files

- `src/intent.gleam` - CLI entry point with glint commands
- `src/intent/interview.gleam` - Interactive interview system for requirement capture
- `src/intent/interview_storage.gleam` - Session management for interviews
- `src/intent/bead_templates.gleam` - Bead generation templates and logic
- `src/intent/plan_mode.gleam` - Plan generation and analysis
- `src/intent/plan_emit_beads.gleam` - Emit beads to br (idempotent)
- `src/intent/effects_analyzer.gleam` - Second-order effects analysis
- `src/intent/quality_analyzer.gleam` - Specification quality analysis
- `src/intent/semantic_validator.gleam` - Semantic validation of specs
- `src/intent/parser.gleam` - JSON parsing with shared `dynamic_to_json` utility
- `src/intent_ffi.erl` - Erlang FFI for system operations

## Development Commands

```bash
gleam build    # Compile
gleam test     # Run tests
```

## Spec Format Requirements

All fields in Intent specifications are **required**. No backwards compatibility defaults are provided:

### Required Spec Fields
- `name` - Spec name
- `description` - Human-readable description
- `audience` - Target users of the system
- `version` - Semantic version
- `success_criteria` - List of acceptance criteria
- `features` - List of feature specifications
- `invariants` - Global invariants that apply to all behaviors
- `anti_patterns` - Anti-patterns to avoid
- `ai_hints` - Implementation guidance for AI

### Required Feature Fields
- `name` - Feature name
- `description` - Feature description
- `behaviors` - List of behavior specifications (cannot be empty)

### Required Behavior Fields
- `name` - Behavior identifier (must match `[a-z][a-z0-9_-]*`)
- `intent` - Plain English description of what this behavior demonstrates
- `preconditions` - What must be true before this behavior (optional, defaults to empty)
- `postconditions` - What must be true after this behavior (optional, defaults to empty)
- `verifications` - How to verify the behavior works (optional, defaults to empty)
- `notes` - Additional context (optional, defaults to empty string)
- `requires` - Behavior dependencies (optional, defaults to empty list)
- `tags` - Classification tags (optional, defaults to empty list)

### Optional Verification Fields (within verifications array)
- `description` - What is being verified
- `criteria` - List of verification criteria
- `examples` - JSON examples demonstrating the criteria (optional)

### Required Invariant Fields
- `name` - Invariant name
- `description` - What this invariant ensures
- `criteria` - What must always be true

All optional-looking fields (like empty strings, empty lists) must be explicitly provided in CUE specs.

**Note**: The schema no longer includes HTTP-specific fields like `request`, `response`, `config`, `captures`, or `checks`. Behaviors are now declarative and focus on preconditions, postconditions, and verifications.

## Code Style

- Use Result types for error handling
- Pattern match exhaustively
- Keep functions small and focused
- Prefer pipelines (`|>`) for data transformation
