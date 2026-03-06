# Intent CLI - AI Agent Instructions

## Project Overview

**Intent** is a contract-driven API testing framework with the tagline: "Human-writes, AI-verifies, AI-implements". It enables developers to write API specifications in CUE language that describe API behaviors and validate running APIs against these specifications.

**Tech Stack**: Gleam (compiles to Erlang/OTP), CUE for specifications

## Issue Tracking with br (beads_rust)

**Note:** `br` is non-invasive and never executes git commands. After `br sync --flush-only`, you must manually run `git add .beads/ && git commit`.

**IMPORTANT**: This project uses **br (beads_rust)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why br?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: JSONL for version control
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**
```bash
br ready --json
```

**Create new issues:**
```bash
br create "Issue title" -t bug|feature|task -p 0-4 --json
br create "Issue title" -p 1 --deps discovered-from:br-123 --json
br create "Subtask" --parent <epic-id> --json  # Hierarchical subtask (gets ID like epic-id.1)
```

**Claim and update:**
```bash
br update br-42 --status in_progress --json
br update br-42 --priority 1 --json
```

**Complete work:**
```bash
br close br-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `br ready` shows unblocked issues
2. **Claim your task**: `br update <id> --status in_progress`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `br create "Found bug" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `br close <id> --reason "Done"`
6. **Commit together**: Always commit the `.beads/issues.jsonl` file together with the code changes so issue state stays in sync with code state

### Viewing Issues with bv (beads viewer)

**CRITICAL**: Use ONLY `--robot-*` flags. Bare `bv` launches an interactive TUI that blocks your session.

**AI Agent Commands (non-blocking JSON output):**
```bash
bv --robot-triage          # Comprehensive analysis with recommendations
bv --robot-next            # Single top pick with claim command
bv --robot-plan            # Parallel execution tracks with unblock lists
bv --robot-insights        # Full metrics (PageRank, betweenness, critical path)
bv --robot-label-health    # Per-label health assessment
bv --robot-graph --graph-format=json  # Dependency graph export
```

**Human-only (interactive TUI):**
```bash
bv              # Launch interactive viewer - DO NOT USE as AI agent
```

The viewer provides:
- Split-pane dashboard
- Kanban board view
- Graph visualization
- Insights panels

Install: `curl -fsSL "https://raw.githubusercontent.com/Dicklesworthstone/beads_viewer/main/install.sh" | bash`

### CLI Help

Run `br <command> --help` to see all available flags for any command.
For example: `br create --help` shows `--parent`, `--deps`, `--assignee`, etc.

### Important Rules

- ✅ Use br for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `br ready` before asking "what should I work on?"
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems

## Project Structure

```
intent-cli/
├── src/
│   ├── intent.gleam         # CLI entry point
│   ├── intent_ffi.erl       # Erlang FFI for halt/timing
│   └── intent/
│       ├── types.gleam      # Core type definitions
│       ├── parser.gleam     # JSON to Gleam type parsing
│       ├── loader.gleam     # CUE file loading and validation
│       ├── runner.gleam     # Test execution orchestrator
│       ├── checker.gleam    # Response validation engine
│       ├── http_client.gleam # HTTP request execution
│       ├── resolver.gleam   # Behavior dependency resolution
│       ├── rules_engine.gleam # Global rule evaluation
│       ├── anti_patterns.gleam # Anti-pattern detection
│       ├── rule.gleam       # Rule expression parser
│       ├── interpolate.gleam # Variable interpolation
│       └── output.gleam     # Result formatting
├── test/                    # Test suite
├── examples/                # Example specifications
├── schema/                  # CUE schema definitions
└── gleam.toml              # Package configuration
```

## Build Commands

```bash
gleam build    # Compile the project
gleam test     # Run tests
gleam run      # Run the CLI
```

## CLI Commands

```bash
intent check <spec.cue> --target <url>  # Run spec against target
intent validate <spec.cue>               # Validate CUE syntax only
intent show <spec.cue>                   # Pretty print spec
intent export <spec.cue>                 # Export to JSON
```
