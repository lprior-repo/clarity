# Gleam CLI to Rust Conversion

## Overview

This directory contains contract specifications and test plans for converting the Gleam CLI (`intent-cli`) to Rust.

## Beads Created (22 total)

| ID | Title | Priority | Status |
|----|-------|----------|--------|
| bd-2dud | cli: Implement all command handlers | P0 | epic |
| bd-1ylr | interpolation: Fix Context variables to store Json values | P0 | bug |
| bd-6nwq | cli: Create CLI binary with clap command structure | P0 | feature |
| bd-3nry | interview: Implement apply_phase_gating | P1 | feature |
| bd-1gox | validation: Add path traversal and string length checks | P1 | feature |
| bd-1or8 | interpolation: Add missing functions | P1 | feature |
| bd-p338 | types: Add missing Behavior fields | P1 | feature |
| bd-ta50 | types: Add missing Spec fields | P1 | feature |
| bd-24oa | interview: Add answer extraction logic | P1 | feature |
| bd-b464 | cli: Port cli_ui terminal formatting module | P1 | feature |
| bd-2tra | types: Add missing SecurityHints fields | P2 | feature |
| bd-3sue | types: Fix EntityHint structure to match Gleam | P2 | feature |
| bd-2a0q | types: Add ImplementationHints.suggested_stack | P2 | feature |
| bd-tnyx | types: Add AIHints.pitfalls field | P2 | feature |
| bd-1705 | types: Change Invariant constraint to criteria list | P2 | feature |
| bd-2nby | types: Change Behavior verification to plural | P2 | feature |
| bd-d4sc | types: Add AntiPattern example fields | P2 | feature |
| bd-1e4l | beads: Add missing template helper functions | P2 | feature |
| bd-13bo | cli: Add interactive init prompt functions | P2 | feature |
| bd-czpc | interview: Implement calculate_confidence function | P2 | feature |
| bd-a3ta | plan: Add format_plan_human/json/ai functions | P2 | feature |
| bd-adt5 | validation: Implement human-readable rule parser | P2 | feature |

## Contract Specifications

Critical P0/P1 beads have detailed contract specs:

| Bead | Contract | Tests |
|------|----------|-------|
| bd-1ylr | [contract-spec.md](./bd-1ylr-contract-spec.md) | [tests.md](./bd-1ylr-tests.md) |
| bd-b464 | [contract-spec.md](./bd-b464-contract-spec.md) | [tests.md](./bd-b464-tests.md) |
| bd-ta50 | [contract-spec.md](./bd-ta50-contract-spec.md) | [tests.md](./bd-ta50-tests.md) |
| bd-p338 | [contract-spec.md](./bd-p338-contract-spec.md) | [tests.md](./bd-p338-tests.md) |

## Dependencies

```
bd-2dud (CLI Epic)
├── bd-6nwq (CLI binary) → bd-b464 (cli_ui)
├── bd-24oa (extraction) → bd-ta50, bd-p338 (types)
├── bd-3nry (phase gating) → bd-ta50 (types)
└── bd-1or8 (interp functions) → bd-1ylr (Context fix)
```

## Recommended Order

1. **bd-ta50** / **bd-p338** - Type fixes (parallel, no deps)
2. **bd-1ylr** - Context fix (P0 bug)
3. **bd-b464** - cli_ui (blocks CLI binary)
4. Follow dependency chain to CLI epic

## Commands

```bash
# View ready work
br ready

# Show bead details
br show <bead-id>

# Start work on a bead
zjj work --bead-id <bead-id>
```
