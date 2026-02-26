# Mental Lattice Architecture

## Overview

The Mental Lattice is Clarity's native Rust implementation of requirements analysis and quality assurance patterns. It provides a multi-layered approach to validating that user requirements are complete, consistent, testable, and secure.

### Philosophy: KIRK

The Mental Lattice follows **KIRK** (Keep Invariants Regular and Known) principles:

- **Completeness**: All required fields present and populated
- **Consistency**: No contradictory requirements
- **Testability**: Acceptance criteria defined for all requirements
- **Clarity**: Minimal jargon, maximum readability
- **Security**: Authentication, encryption, and validation considered

## Module Structure

```
clarity-web/src/lattice/
├── mod.rs              # Public exports and invariants
├── ears.rs             # EARS pattern parser
├── inversion.rs        # Assumption challenging
├── effects.rs          # Dependency tracing
├── quality.rs          # Quality scoring (5 dimensions)
├── premortem.rs        # Failure scenario analysis
├── coverage.rs         # Use case coverage analysis
└── compact.rs          # Artifact compaction
```

## Module Descriptions

### 1. EARS (Easy Approach to Requirements Syntax)

**File**: `lattice/ears.rs`

**Purpose**: Parse natural language requirements into structured patterns.

**Function Signatures**:

```rust
/// Parse a single requirement line
pub fn parse_requirement(input: &str) -> Result<EarsRequirement, EarsError>

/// Parse multiple requirements from multi-line input
pub fn parse_requirements(input: &str) -> EarsOutput
```

**Pattern Types**:

| Pattern | Syntax | Example |
|---------|--------|---------|
| Ubiquitous | "The system shall..." | "The system shall authenticate users" |
| State-driven | "When X, the system shall Y..." | "When logged in, the system shall show dashboard" |
| Event-driven | "During X, the system shall Y..." | "During startup, the system shall initialize services" |
| Unwanted | "If X, the system shall NOT..." | "If password invalid, the system shall not grant access" |
| Optional | "Where X, the system shall Y..." | "Where premium, the system shall enable advanced features" |

**Output Structure**:

```rust
pub struct EarsOutput {
    pub requirements: Vec<EarsRequirement>,
    pub errors: Vec<String>,
}

pub enum EarsRequirement {
    Ubiquitous { actor: String, action: String },
    StateDriven { actor: String, trigger: String, action: String },
    EventDriven { actor: String, trigger: String, action: String },
    Unwanted { actor: String, condition: String, action: String },
    Optional { actor: String, condition: String, action: String },
}
```

**Usage Example**:

```rust
let input = r#"
The system shall authenticate users.
When logged in, the system shall display the dashboard.
During startup, the system shall initialize all services.
"#;

let output = ears::parse_requirements(input);
assert_eq!(output.requirements.len(), 3);
assert!(matches!(output.requirements[0], EarsRequirement::Ubiquitous { .. }));
```

**Error Handling**:

- `EarsError::EmptyInput`: Empty or whitespace-only input
- `EarsError::UnrecognizedPattern`: Input doesn't match any EARS pattern
- `EarsError::MalformedRequirement`: Pattern detected but missing required components

---

### 2. Inversion (Assumption Challenging)

**File**: `lattice/inversion.rs`

**Purpose**: Generate challenges to assumptions by inverting logic, finding counterexamples, and identifying edge cases.

**Function Signatures**:

```rust
/// Main inversion function
pub fn invert(
    problem: &str,
    solution: &str
) -> Result<InversionOutput, InversionError>

/// Extract assumptions from text
pub fn extract_assumptions(problem: &str, solution: &str) -> Vec<String>

/// Generate challenges using all patterns
pub fn generate_challenges(assumption: &str) -> Vec<InversionChallenge>
```

**Challenge Patterns**:

| Pattern | Description | Example |
|---------|-------------|---------|
| Negation | Direct logical opposite | "will" → "will not" |
| Counterexample | Contextual exception | "except when under high load" |
| Edge Case | Extreme condition | "what about with zero items?" |
| Reversal | Invert core assertion | "improves" → "worsens" |

**Severity Levels**:

```rust
pub enum Severity {
    Critical,  // Fundamental flaw (100 points)
    Moderate,  // Significant limitation (50 points)
    Low,       // Minor edge case (10 points)
}
```

**Output Structure**:

```rust
pub struct InversionOutput {
    pub challenges: Vec<InversionChallenge>,
    pub quality_score: u8,        // 0-100, lower = more challenges
    pub critical_count: usize,
    pub moderate_count: usize,
    pub low_count: usize,
}

pub struct InversionChallenge {
    pub assumption: String,
    pub challenge: String,
    pub pattern: ChallengePattern,
    pub severity: Severity,
}
```

**Usage Example**:

```rust
let problem = "The API will always respond within 100ms";
let solution = "We use caching to guarantee this performance";

let output = inversion::invert(problem, &solution)?;

// Should find critical issues with "always" and "guarantee"
assert!(output.critical_count >= 1);
assert!(output.quality_score < 100);
```

**Quality Impact Calculation**:

```
quality_score = max(0, 200 - total_impact)

Where:
- Critical challenge = 100 points
- Moderate challenge = 50 points
- Low challenge = 10 points
```

---

### 3. Effects (Dependency Tracing)

**File**: `lattice/effects.rs`

**Purpose**: Parse causal language and build dependency graphs to trace how outcomes relate to each other.

**Function Signatures**:

```rust
/// Parse causal relationships from solution text
pub fn trace_effects(solution: &str) -> EffectsOutput

/// Parse with custom causal patterns
pub fn trace_effects_with_patterns(
    solution: &str,
    patterns: &[CausalPattern]
) -> EffectsOutput

/// Detect circular dependencies in graph
pub fn detect_cycles(
    graph: &HashMap<String, Vec<String>>
) -> Result<(), EffectsError>
```

**Default Causal Patterns**:

| Keyword | Type | Confidence |
|---------|------|------------|
| causes | Positive | 0.8 |
| leads to | Positive | 0.7 |
| enables | Positive | 0.9 |
| results in | Positive | 0.75 |
| blocks | Negative | 0.8 |
| prevents | Negative | 0.9 |
| inhibits | Negative | 0.8 |

**Output Structure**:

```rust
pub struct EffectsOutput {
    pub effects: Vec<Effect>,
    pub dependency_graph: HashMap<String, Vec<String>>,
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub warnings: Vec<String>,
}

pub struct Effect {
    pub trigger: String,
    pub outcome: String,
    pub confidence: f64,
    pub indirect_effects: Vec<String>,
}
```

**Visualization Structures**:

```rust
pub struct DependencyNode {
    pub id: String,
    pub label: String,
    pub is_root: bool,      // No incoming edges
    pub is_leaf: bool,      // No outgoing edges
}

pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub confidence: f64,
    pub indirect: bool,
}
```

**Usage Example**:

```rust
let solution = "Caching causes faster responses. Faster results in better UX.
Better UX leads to higher retention.";

let output = effects::trace_effects(solution);

// Should build dependency chain
assert_eq!(output.effects.len(), 3);
assert!(output.dependency_graph.contains_key("Caching"));

// Visualization data ready for rendering
assert!(!output.nodes.is_empty());
assert!(!output.edges.is_empty());
```

**Cycle Detection**:

Uses depth-limited DFS (max depth 5) to prevent infinite loops:

```rust
// Cycle detected
assert!(matches!(
    detect_cycles(&cyclic_graph),
    Err(EffectsError::CircularDependency(_, _))
));

// No cycle
assert!(detect_cycles(&acyclic_graph).is_ok());
```

---

### 4. Quality (Multi-Dimensional Scoring)

**File**: `lattice/quality.rs`

**Purpose**: Calculate overall quality score across 5 dimensions to gate progression.

**Function Signatures**:

```rust
/// Calculate quality score from requirements data
pub fn calculate_quality(
    answers: &[Answer],
    ears: &[EarsRequirementRef],
    inversion: &InversionControl
) -> Result<QualityScore, QualityError>
```

**Five Dimensions**:

| Dimension | Weight | Calculation |
|-----------|--------|-------------|
| Completeness | 20% | % of required fields filled |
| Consistency | 20% | Absence of contradictions |
| Testability | 20% | % of EARS with acceptance criteria |
| Clarity | 20% | Sentence complexity + jargon density |
| Security | 20% | Coverage of auth/encryption/validation |

**Required Field Patterns**:

```rust
const REQUIRED_PATTERNS: &[&str] = &[
    "user_goal",
    "actors",
    "precondition",
    "outcome",
    "acceptance_criteria",
];
```

**Output Structure**:

```rust
pub struct QualityScore {
    pub overall: u8,                        // Average of dimensions (0-100)
    pub dimensions: Vec<DimensionScore>,
    pub issues: Vec<QualityIssue>,
}

pub struct DimensionScore {
    pub dimension: QualityDimension,
    pub score: u8,                          // 0-100
}

pub struct QualityIssue {
    pub dimension: QualityDimension,
    pub severity: IssueSeverity,            // Warning | Error | Critical
    pub message: String,
}
```

**Usage Example**:

```rust
let answers = vec![
    Answer { step_id: "user_goal".into(), value: "Authenticate".into(), ... },
    Answer { step_id: "actors".into(), value: "Admin".into(), ... },
    Answer { step_id: "precondition".into(), value: "User exists".into(), ... },
    Answer { step_id: "outcome".into(), value: "Access granted".into(), ... },
    Answer { step_id: "acceptance_criteria".into(), value: "Within 2s".into(), ... },
];

let ears = vec![
    EarsRequirementRef {
        id: "1".into(),
        text: "User shall authenticate".into(),
        has_acceptance_criteria: true,
    },
];

let inversion = InversionControl {
    has_inversion_tests: true,
    inverted_count: 2,
};

let score = quality::calculate_quality(&answers, &ears, &inversion)?;

// Perfect score with all fields
assert_eq!(score.overall, 100);
assert!(score.issues.is_empty());
```

**Quality Gate Behavior**:

```rust
if score.overall >= 70 {
    // Enable "Continue to Define" CTA
} else {
    // Show improvement suggestions
    for issue in &score.issues {
        match issue.severity {
            IssueSeverity::Error => show_blocking_error(&issue.message),
            IssueSeverity::Warning => show_suggestion(&issue.message),
            IssueSeverity::Critical => halt_progression(&issue.message),
        }
    }
}
```

---

### 5. Premortem (Failure Scenario Analysis)

**File**: `lattice/premortem.rs`

**Purpose**: Generate potential failure scenarios across 4 categories to identify risks before implementation.

**Function Signatures**:

```rust
/// Generate premortem analysis
pub fn generate_premortem(
    solution: &str,
    constraints: &[&str]
) -> PremortemOutput
```

**Failure Categories**:

| Category | Description | Example Scenarios |
|----------|-------------|-------------------|
| Technical | Bugs, performance, scalability | Performance degradation under load, API failures |
| User | Adoption, usability, resistance | Low adoption due to complexity, usability barriers |
| Business | Cost, market fit, competition | Cost overruns, competitive pressure |
| Security | Data breaches, vulnerabilities | Data breach, authentication vulnerabilities |

**Likelihood Levels**:

```rust
pub enum Likelihood {
    VeryLikely,   // >70% probability
    Possible,     // 30-70% probability
    Unlikely,     // <30% probability
}
```

**Output Structure**:

```rust
pub struct PremortemOutput {
    pub solution: String,
    pub constraints: Vec<String>,
    pub scenarios: Vec<FailureScenario>,
    pub high_risk_scenarios: Vec<FailureScenario>,  // VeryLikely only
}

pub struct FailureScenario {
    pub category: FailureCategory,
    pub trigger: String,              // What could trigger this
    pub consequence: String,          // Result if it occurs
    pub likelihood: Likelihood,
    pub mitigation: Vec<String>,      // Suggested mitigations
}
```

**Usage Example**:

```rust
let solution = "Build a scalable API service";
let constraints = &["Must handle 10k concurrent users", "Low latency required"];

let output = premortem::generate_premortem(solution, constraints);

// Should generate scenarios across all categories
let categories: HashSet<_> = output.scenarios
    .iter()
    .map(|s| s.category)
    .collect();

assert!(categories.contains(&FailureCategory::Technical));
assert!(categories.contains(&FailureCategory::User));
assert!(categories.contains(&FailureCategory::Business));
assert!(categories.contains(&FailureCategory::Security));

// High-risk scenarios flagged
assert!(!output.high_risk_scenarios.is_empty());
```

**Context-Aware Generation**:

The analysis adapts based on keywords in the solution:

```rust
// Performance-related keywords → higher likelihood of technical issues
if contains_any(solution, &["scale", "performance", "concurrent"]) {
    likelihood = Likelihood::VeryLikely;
}

// Authentication keywords → higher security risk
if contains_any(solution, &["authentication", "login", "session"]) {
    likelihood = Likelihood::VeryLikely;
}
```

---

### 6. Coverage (Use Case Coverage Analysis)

**File**: `lattice/coverage.rs`

**Purpose**: Analyze which use cases are covered by implementation tasks and identify gaps.

**Function Signatures**:

```rust
/// Analyze coverage of use cases by tasks
pub fn analyze_coverage(
    use_cases: &[UseCase],
    tasks: &[Task]
) -> Result<CoverageOutput, CoverageError>
```

**Output Structure**:

```rust
pub struct CoverageOutput {
    pub covered_components: Vec<CoveredComponent>,
    pub coverage_gaps: Vec<CoverageGap>,
    pub overall_coverage_percent: u8,
    pub total_use_cases: usize,
    pub covered_use_cases_count: usize,
}

pub struct CoveredComponent {
    pub name: String,
    pub use_cases: Vec<String>,           // IDs of covered use cases
    pub coverage_percent: u8,             // % of total use cases covered
}

pub struct CoverageGap {
    pub use_case: String,
    pub missing_components: Vec<String>,
    pub suggestion: String,
}
```

**Component Extraction**:

Uses regex patterns to identify components from task descriptions:

```rust
// Pattern: Capitalized words with common suffixes
let pattern = r"\b[A-Z][a-z]+(?:Service|Controller|Repository|Manager|...)\b";

// Examples matched:
// - "AuthService"
// - "DatabaseRepository"
// - "PaymentGateway"
```

**Usage Example**:

```rust
let use_cases = vec![
    UseCase::new("uc1".into(), "Authentication".into(), "User login".into()),
    UseCase::new("uc2".into(), "DataStorage".into(), "Persist data".into()),
];

let tasks = vec![
    Task::new("t1".into(), "Implement AuthService".into(), "Handle login".into()),
];

let output = coverage::analyze_coverage(&use_cases, &tasks)?;

// 50% coverage (1 out of 2)
assert_eq!(output.overall_coverage_percent, 50);
assert_eq!(output.coverage_gaps.len(), 1);
assert_eq!(output.coverage_gaps[0].use_case, "uc2");
```

**Gap Suggestions**:

Generates actionable suggestions for uncovered use cases:

```rust
// Suggests component creation
assert!(output.coverage_gaps[0].suggestion.contains("Implement"));
assert!(output.coverage_gaps[0].missing_components[0].contains("Service"));
```

---

### 7. Compact (Artifact Compaction)

**File**: `lattice/compact.rs`

**Purpose**: Compress and optimize generated artifacts by removing redundancy and consolidating related items.

**Function Signatures**:

```rust
/// Compact artifacts across all phases
pub fn compact_artifacts(artifacts: &[Artifact]) -> CompactOutput

/// Clean and normalize text
pub fn clean_text(text: &str) -> String
```

**Output Structure**:

```rust
pub struct CompactOutput {
    pub phases: Vec<Phase>,
    pub total_artifacts: usize,
    pub compacted_artifacts: usize,
    pub reduction_percent: f64,
}

pub struct Phase {
    pub name: String,
    pub answers: Vec<CompactAnswer>,
    pub summary: CompactSummary,
}
```

## Phase Triggers

### Discover Phase

Triggered when: All 5 questions answered OR all 5 fields confirmed

```rust
// Execute in sequence
let ears_output = ears::parse_requirements(&answers_text);
let inversion_output = inversion::invert(&problem, &solution)?;
let effects_output = effects::trace_effects(&solution_text);
let quality_score = quality::calculate_quality(&answers, &ears_reqs, &inversion)?;

// Quality gate
if quality_score.overall < 70 {
    return Err(QualityError::InsufficientScore);
}
```

### Define Phase

Triggered when: Use cases and constraints defined

```rust
// Additional analysis
let premortem_output = premortem::generate_premortem(&solution, &constraints);
let updated_quality = quality::calculate_quality(&all_answers, &ears, &inversion)?;

// Re-check gate
if updated_quality.overall < 70 {
    // Show improvement suggestions
}
```

### Develop Phase

Triggered when: Tasks defined

```rust
// Coverage analysis
let coverage_output = coverage::analyze_coverage(&use_cases, &tasks)?;

// Check for gaps
if !coverage_output.coverage_gaps.is_empty() {
    // Warn about missing implementation
}
```

### Deliver Phase

Triggered when: All tasks complete

```rust
// Final compaction
let compact_output = compact::compact_artifacts(&all_artifacts)?;
```

## Usage Examples

### Complete Discover Phase Analysis

```rust
use clarity::lattice::*;

// User input from Express or Guided mode
let answers = vec![
    Answer { step_id: "problem".into(), value: "Users forget passwords".into(), ... },
    Answer { step_id: "solution".into(), value: "Add password reset".into(), ... },
    // ... more answers
];

// Parse requirements
let ears_output = ears::parse_requirements(&combined_text);

// Challenge assumptions
let problem = answers.iter().find(|a| a.step_id == "problem").unwrap();
let solution = answers.iter().find(|a| a.step_id == "solution").unwrap();
let inversion_output = inversion::invert(&problem.value, &solution.value)?;

// Trace effects
let effects_output = effects::trace_effects(&solution.value);

// Calculate quality
let ears_refs: Vec<_> = ears_output.requirements
    .iter()
    .map(|r| EarsRequirementRef {
        id: uuid::Uuid::new_v4().to_string(),
        text: format!("{:?}", r),
        has_acceptance_criteria: false,
    })
    .collect();

let quality_score = quality::calculate_quality(
    &answers,
    &ears_refs,
    &InversionControl {
        has_inversion_tests: true,
        inverted_count: inversion_output.challenges.len(),
    }
)?;

// Check gate
if quality_score.overall >= 70 {
    // Enable progression to Define phase
} else {
    // Show issues
    for issue in &quality_score.issues {
        println!("{}: {}", issue.dimension.label(), issue.message);
    }
}
```

### Define Phase with Premortem

```rust
// After use cases defined
let use_cases = vec![
    UseCase::new("uc1".into(), "Password Reset".into(), "User resets password".into()),
    // ... more use cases
];

let constraints = vec
!["Must send email", "Token expires in 1 hour"];

// Run premortem
let solution_text = answers.iter()
    .find(|a| a.step_id == "solution")
    .map(|a| a.value.clone())
    .unwrap_or_default();

let premortem_output = premortem::generate_premortem(&solution_text, &constraints);

// Review high-risk scenarios
for scenario in &premortem_output.high_risk_scenarios {
    println!("HIGH RISK: {}", scenario.trigger);
    println!("Mitigations:");
    for mitigation in &scenario.mitigation {
        println!("  - {}", mitigation);
    }
}
```

### Develop Phase Coverage Check

```rust
// After tasks defined
let tasks = vec![
    Task::new("t1".into(), "ResetPasswordService".into(), "Handle reset flow".into()),
    // ... more tasks
];

let coverage_output = coverage::analyze_coverage(&use_cases, &tasks)?;

println!("Coverage: {}%", coverage_output.overall_coverage_percent);

// Address gaps
for gap in &coverage_output.coverage_gaps {
    println!("Uncovered: {}", gap.use_case);
    println!("Suggestion: {}", gap.suggestion);
}
```

## Extension Points

### Adding New Lattice Modules

1. Create new module file: `lattice/your_module.rs`
2. Implement analysis functions
3. Add to `lattice/mod.rs` exports
4. Wire into phase trigger

```rust
// lattice/your_module.rs
pub fn analyze_your_aspect(input: &str) -> YourOutput {
    // Analysis logic
}

// lattice/mod.rs
mod your_module;
pub use your_module::{analyze_your_aspect, YourOutput};
```

### Custom Causal Patterns (Effects)

```rust
use clarity::lattice::effects::CausalPattern;

let custom_patterns = vec![
    CausalPattern::new("triggers", true, 0.9),
    CausalPattern::new("yields", true, 0.85),
];

let output = effects::trace_effects_with_patterns(solution, &custom_patterns);
```

### Custom Quality Dimensions

Add new dimension to `quality.rs`:

```rust
pub enum QualityDimension {
    // ... existing dimensions
    CustomDimension,  // Add new variant
}

impl QualityDimension {
    pub fn description(&self) -> &'static str {
        match self {
            CustomDimension => "Custom description",
            // ... existing cases
        }
    }
}

// Add calculation
fn calculate_custom_dimension(
    answers: &[Answer],
    issues: &mut Vec<QualityIssue>
) -> DimensionScore {
    // Calculation logic
}

// Update calculate_quality()
let custom_score = calculate_custom_dimension(answers, &mut all_issues);
let dimensions = vec![completeness, consistency, testability, clarity, security, custom_score];
```

## Testing

Each module includes comprehensive tests:

```bash
# Test all lattice modules
cargo test --package clarity-web --lib lattice

# Test specific module
cargo test --package clarity-web --lib lattice::ears
cargo test --package clarity-web --lib lattice::inversion
cargo test --package clarity-web --lib lattice::effects
cargo test --package clarity-web --lib lattice::quality
cargo test --package clarity-web --lib lattice::premortem
cargo test --package clarity-web --lib lattice::coverage
```

## Performance Considerations

### EARS Parsing

- **Time Complexity**: O(n * m) where n = lines, m = patterns (5)
- **Optimization**: Early exit on keyword match
- **Best For**: < 100 requirements at once

### Inversion

- **Time Complexity**: O(a * p) where a = assumptions, p = patterns (4)
- **Optimization**: Deduplication by (assumption, pattern) pair
- **Best For**: < 50 assumptions

### Effects

- **Time Complexity**: O(n + e) where n = nodes, e = edges
- **Cycle Detection**: Depth-limited DFS (max depth 5)
- **Best For**: < 200 dependencies

### Quality Scoring

- **Time Complexity**: O(d * a) where d = dimensions (5), a = answers
- **Optimization**: Dimension calculations are independent
- **Best For**: < 1000 answers

## Integration with UI

### Reactivity

Lattice functions are pure and deterministic, making them ideal for Dioxus reactivity:

```rust
let quality_score = use_resource(move || {
    let answers = answers.read().clone();
    async move {
        quality::calculate_quality(&answers, &ears, &inversion)
    }
});

// Auto-update when answers change
if let Some(Ok(score)) = quality_score.read().as_ref() {
    rsx! {
        QualityScoreBar { score: score.overall }
    }
}
```

### Error Display

Convert lattice errors to user-friendly messages:

```rust
match lattice_result {
    Ok(output) => render_output(output),
    Err(EarsError::UnrecognizedPattern(line)) => {
        rsx! {
            ErrorMessage {
                "Couldn't parse: {line}"
                "Try using 'The system shall...' format"
            }
        }
    }
    Err(QualityError::InsufficientScore) => {
        rsx! {
            QualityGateBlocker {
                "Quality score too low"
                "Address the highlighted issues before continuing"
            }
        }
    }
    _ => rsx! { ErrorMessage { "An error occurred" } }
}
```
