# Architecture Specification: Great Quality Unification & Intent Flattening

**Version**: 1.0.0
**Status**: PROPOSED
**Mission**: structural-integrity
**Created**: 2026-03-06

## 1. Executive Summary

The Clarity project has experienced significant structural drift and logic fragmentation. Specifically:
- **Quality Scoring** exists in three disconnected forms (Lattice, Intent, UI).
- **Module Hierarchy** in `src/intent` has exceeded manageable depth.
- **Causal Analysis** is duplicated across modules.

This specification mandates the unification of these systems into a single, mathematically rigorous, trait-based engine with a flattened module structure.

## 2. Structural Blueprint (Re-indexing)

The module hierarchy is restricted to a maximum depth of 3.

### 2.1 Core Domain Layer (`src/domain/`)
- `types.rs`: Canonical `Spec`, `Feature`, `Behavior`, `Answer` types.
- `quality.rs`: Unified `QualityReport`, `QualityDimension`, and `QualityAlgebra` traits.
- `error.rs`: Unified `ClarityError` (using `thiserror`).

### 2.2 Functional Analysis Engine (`src/lattice/`)
- Side-effect-free algorithm library.
- `ears.rs`: EARS pattern parsing.
- `inversion.rs`: Challenge generation.
- `effects.rs`: Dependency/Effect tracing.
- `quality.rs`: Algorithm implementation for the `QualityAlgebra` trait.

### 2.3 Stateful Workflow Context (`src/intent/`)
- Manages the lifecycle and persistence.
- `discovery.rs`: Express/Guided mode coordination.
- `planning.rs`: Bead generation and Resolver logic.
- `storage.rs`: Unified `redb` persistence handlers.

## 3. The Quality Algebra (KIRK Integration)

All quality evaluation MUST implement the `QualityEvaluator` trait.

```rust
pub trait QualityEvaluator<T> {
    fn evaluate(&self, input: &T) -> Result<QualityReport, ClarityError>;
}
```

### 3.1 Mathematical Dimensions
- **Completeness (20%)**: Required field coverage.
- **Consistency (20%)**: Contradiction detection (CAP/Auth).
- **Testability (20%)**: EARS with Acceptance Criteria.
- **Clarity (20%)**: Intent/Description ratio + Readability.
- **Security (20%)**: Auth/Encryption/Validation coverage.

## 4. Phase Gating (Typestate Pattern)

Transition from Discovery to Planning is guarded by a type-level transition.

```rust
pub struct DiscoveryState { ... }
pub struct ValidatedState { report: QualityReport, ... }

impl DiscoveryState {
    pub fn try_gate(self) -> Result<ValidatedState, QualityReport> {
        let report = self.evaluate()?;
        if report.overall_score >= 70 {
            Ok(ValidatedState { report, ... })
        } else {
            Err(report)
        }
    }
}
```

## 5. Persistence Mandate

- All state persists to `redb`.
- Schema is derived from CUE definitions in `schemas/`.
- Immediate-write pattern for all user interactions.

## 6. Implementation Strategy

1. **Horizontal Unification**: Move all Quality types to `src/domain`.
2. **Algorithmic Cleanup**: Delete duplicate `effects` and `quality` logic in `intent`.
3. **Hierarchy Collapse**: Move `intent/**/*` files to flattened modules.
4. **Type-Gate Enforcement**: Wrap UI flows in Typestate transitions.

## 7. Success Criteria

- 100% Score consistency across all UI components.
- Zero code duplication for "Effect Tracing."
- Maximum module depth <= 3.
- Zero panics (`clippy::unwrap_used` denied).
- 40-80 atomic beads created and tracked in `bd`.

---

**Authorized By**: Opencode Architect
**Reviewers**: Skeptical Implementer
