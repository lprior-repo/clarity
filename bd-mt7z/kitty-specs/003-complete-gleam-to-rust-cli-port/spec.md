# Feature Specification: Complete Gleam to Rust CLI Port

**Feature ID**: 003-complete-gleam-to-rust-cli-port
**Mission**: software-dev
**Status**: Draft
**Created**: 2025-02-28

---

## Problem Statement

The intent-cli tool was originally developed in Gleam and is being ported to Rust. The current port is incomplete at approximately 15-20% coverage, with critical functionality missing. Users cannot use the CLI because the main entry point has no CLI implementation (library only), and core business modules are missing their Rust equivalents.

The incomplete port blocks:
- End users from running the CLI tool
- Developers from testing the full application flow
- Integration with downstream systems that expect a working CLI

---

## User Personas

### Primary: CLI User
A developer or analyst who uses intent-cli to process interview data, validate specifications, and generate planning artifacts. They need a working command-line tool that accepts inputs, processes them correctly, and produces expected outputs.

### Secondary: Tool Maintainer
A developer working on the intent-cli codebase who needs the Rust port to be complete so they can maintain, extend, and test the tool without switching between Gleam and Rust contexts.

---

## User Scenarios & Testing

### Scenario 1: Basic CLI Invocation
**Given** the CLI is installed
**When** a user runs `intent --help`
**Then** they see a list of available commands with descriptions

### Scenario 2: Interview Processing
**Given** a valid interview input file
**When** a user runs `intent interview process <input-file>`
**Then** the interview is processed with phase gating applied
**And** answers are extracted correctly
**And** confidence scores are calculated

### Scenario 3: Interpolation
**Given** a template with variable references
**When** interpolation is performed
**Then** variables are resolved from Context storing proper JSON values
**And** all interpolation functions work correctly

### Scenario 4: Validation
**Given** input data to validate
**When** validation runs
**Then** path traversal attacks are blocked
**And** string length limits are enforced
**And** human-readable rule parsing works

### Scenario 5: Output Formatting
**Given** processed data
**When** a user requests output
**Then** plans can be formatted as human-readable, JSON, or AI-ready formats
**And** terminal output uses proper formatting (colors, indentation)

---

## Functional Requirements

### FR-001: CLI Binary Entry Point
The application MUST provide a CLI binary that accepts commands via standard command-line arguments.

**Acceptance Criteria**:
- Running the binary without arguments shows usage help
- The binary uses a standard argument parsing approach
- All command handlers are wired to their implementations

### FR-002: Command Handlers
All CLI commands MUST have working handlers that execute the intended operations.

**Acceptance Criteria**:
- Each command maps to a handler function
- Handlers return appropriate exit codes
- Error messages are user-friendly

### FR-003: Interpolation Context
The interpolation system MUST store variables as JSON values, not raw strings.

**Acceptance Criteria**:
- Context.variables field uses JSON-compatible types
- Nested data structures are preserved
- All interpolation functions handle JSON values correctly

### FR-004: Missing Interpolation Functions
All interpolation functions present in the Gleam source MUST be available in Rust.

**Acceptance Criteria**:
- Function parity between Gleam and Rust implementations
- Identical behavior for equivalent inputs
- Edge cases handled consistently

### FR-005: Interview Phase Gating
The interview engine MUST apply phase gating to control question flow.

**Acceptance Criteria**:
- Questions are gated by phase rules
- Invalid phase transitions are rejected
- Phase state is maintained throughout the interview

### FR-006: Answer Extraction
The interview engine MUST extract answers from interview responses.

**Acceptance Criteria**:
- All answer types are extracted correctly
- Missing answers are handled gracefully
- Extracted data matches expected schema

### FR-007: Confidence Calculation
The interview engine MUST calculate confidence scores for processed interviews.

**Acceptance Criteria**:
- Confidence is calculated using the same algorithm as Gleam
- Scores fall within expected ranges
- Edge cases (no answers, partial answers) produce sensible results

### FR-008: Validation Security Checks
The validation module MUST enforce security constraints.

**Acceptance Criteria**:
- Path traversal attempts are detected and blocked
- String length limits prevent abuse
- Security violations produce clear error messages

### FR-009: Human-Readable Rule Parser
The validation module MUST parse human-readable validation rules.

**Acceptance Criteria**:
- Rules written in natural language syntax are parsed
- Parse errors are reported with helpful context
- Parsed rules execute correctly

### FR-010: Type Field Completeness
All data types MUST have complete field sets matching the Gleam definitions.

**Acceptance Criteria**:
- Behavior type has all required fields
- Spec type has all required fields
- SecurityHints, EntityHint, ImplementationHints, AIHints have complete fields
- Invariant.constraint uses criteria list format
- Behavior.verification is plural
- AntiPattern has example fields

### FR-011: CLI UI Terminal Formatting
The CLI MUST provide terminal formatting capabilities.

**Acceptance Criteria**:
- Output supports colors and styling
- Indentation and alignment work correctly
- Formatting degrades gracefully on non-TTY outputs

### FR-012: Interactive Init Prompts
The CLI MUST support interactive initialization prompts.

**Acceptance Criteria**:
- Users are prompted for required configuration
- Default values are offered where appropriate
- Prompts are clear and validation is immediate

### FR-013: Template Helper Functions
The beads module MUST include all template helper functions.

**Acceptance Criteria**:
- All helper functions from Gleam are present
- Helpers work with the template system
- Missing helpers are documented

### FR-014: Plan Formatters
The plan module MUST provide multiple output format options.

**Acceptance Criteria**:
- Human-readable format produces readable output
- JSON format produces valid, parseable JSON
- AI format produces structured output for AI consumption

---

## Success Criteria

1. **Functional Parity**: All 44 core Gleam modules have working Rust equivalents
2. **CLI Functional**: Users can invoke `intent --help` and see a working CLI interface
3. **Test Coverage**: All ported modules pass their corresponding tests
4. **No Regressions**: Output from Rust implementation matches Gleam output for equivalent inputs (where both exist)
5. **Type Completeness**: All data types have 100% field coverage compared to Gleam definitions

---

## Scope

### In Scope
- All 22 beads identified in the planning session
- Core CLI functionality (entry point, commands, UI)
- Interview engine (phase gating, extraction, confidence)
- Interpolation system (Context fix, missing functions)
- Validation (security, rule parser)
- Type definitions (all missing fields)
- Output formatting (plan formatters, cli_ui)

### Out of Scope
- Test file ports (18 Gleam test files)
- Performance optimization beyond functional parity
- New features not present in original Gleam code
- Documentation beyond inline comments

---

## Key Entities

| Entity | Description |
|--------|-------------|
| Context | Interpolation context storing variables as JSON |
| Behavior | Specification behavior definition with verification methods |
| Spec | Full specification document structure |
| Interview | Interview session with phases and answers |
| ValidationRule | Rule definition for input validation |
| Plan | Output plan document with multiple format options |

---

## Dependencies

- Existing Rust codebase structure and patterns
- Original Gleam source as reference implementation
- 22 pre-created beads defining work packages

---

## Assumptions

- The Gleam source represents the intended behavior
- Existing Rust code follows idiomatic patterns
- No breaking changes to public interfaces are required
- Test coverage in Gleam can guide Rust test development

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Behavioral differences between ports | Compare outputs for identical inputs |
| Missing edge case handling | Review Gleam tests for edge cases |
| Type mismatches causing runtime errors | Use strong typing with compile-time checks |

---

## Non-Functional Requirements

- **Compatibility**: CLI must work on Linux and macOS
- **Performance**: Response time comparable to or better than Gleam version
- **Maintainability**: Code follows Rust best practices and project conventions
