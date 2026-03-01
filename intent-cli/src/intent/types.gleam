/// Core types for the Intent specification
/// These types mirror the CUE schema definitions (v3.0 declarative format)
import gleam/dict.{type Dict}
import gleam/json.{type Json}

/// The main specification type - all fields required
pub type Spec {
  Spec(
    name: String,
    description: String,
    audience: String,
    version: String,
    success_criteria: List(String),
    features: List(Feature),
    invariants: List(Invariant),
    anti_patterns: List(AntiPattern),
    ai_hints: AIHints,
  )
}

/// A feature groups related behaviors
pub type Feature {
  Feature(name: String, description: String, behaviors: List(Behavior))
}

/// A single behavior/test case
pub type Behavior {
  Behavior(
    name: String,
    intent: String,
    notes: String,
    requires: List(String),
    tags: List(String),
    preconditions: List(String),
    postconditions: List(String),
    verifications: List(Verification),
  )
}

/// Verification of behavior correctness
pub type Verification {
  Verification(
    description: String,
    criteria: List(String),
    examples: List(Json),
  )
}

/// Global invariants that apply to all behaviors
pub type Invariant {
  Invariant(name: String, description: String, criteria: List(String))
}

/// Anti-patterns with good/bad examples
pub type AntiPattern {
  AntiPattern(
    name: String,
    description: String,
    bad_example: Json,
    good_example: Json,
    why: String,
  )
}

/// AI implementation hints - all fields optional
pub type AIHints {
  AIHints(
    implementation: ImplementationHints,
    entities: Dict(String, EntityHint),
    security: SecurityHints,
    pitfalls: List(String),
  )
}

pub type ImplementationHints {
  ImplementationHints(
    suggested_stack: List(String),
    architecture: String,
    key_components: List(String),
  )
}

pub type EntityHint {
  EntityHint(fields: Dict(String, Json))
}

pub type SecurityHints {
  SecurityHints(
    password_hashing: String,
    jwt_algorithm: String,
    jwt_expiry: String,
    rate_limiting: String,
  )
}
