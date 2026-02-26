---
lane: "done"
reviewed_by: "Lewis Prior"
review_status: "approved"
---
# WP01: Server Functions

---
work_package_id: "WP01"
title: "Server Functions"
lane: "planned"
dependencies: []
beads: ["bd-378l", "bd-28v1", "bd-2mcc", "bd-13yb", "bd-2uci", "bd-zf68", "bd-l1qq"]
---

## Objective

Implement all server-side validation and compilation functions for the Progressive Discover feature.

## Beads in This Package

| Bead ID | Title | File |
|---------|-------|------|
| bd-378l | validate_antithesis | server.rs |
| bd-28v1 | validate_straw_man_traps | server.rs |
| bd-2mcc | validate_vorp | server.rs |
| bd-13yb | validate_hole_punching | server.rs |
| bd-2uci | KirkContract types | kirk.rs |
| bd-zf68 | EARS extraction | kirk.rs |
| bd-l1qq | compile_to_kirk | server.rs |

## Implementation Guidance

### Functional Rust Patterns (CRITICAL)

- **No mut** unless absolutely necessary
- **No unnecessary clones** - use references where possible
- Use iterators and functional combinators
- Use `Result` for error handling, never panic

### Dioxus 0.7 Server Functions

```rust
#[server]
pub async fn validate_antithesis(points: [String; 3]) -> Result<AntithesisValidation, ServerFnError> {
    // Pure function - no side effects
    let score = calculate_quality_score(&points);
    Ok(AntithesisValidation { score, suggestions: vec![] })
}
```

### bd-378l: validate_antithesis

**Purpose**: Score the quality of 3 null hypothesis points (0.0-1.0).

**Implementation**:
```rust
#[server]
pub async fn validate_antithesis(points: [String; 3]) -> Result<AntithesisValidation, ServerFnError> {
    let score = points
        .iter()
        .map(|p| calculate_specificity(p))
        .sum::<f64>()
        / 3.0;

    let suggestions = points
        .iter()
        .enumerate()
        .filter(|(_, p)| p.len() < 20)
        .map(|(i, _)| format!("Point {} needs more specificity", i + 1))
        .collect();

    Ok(AntithesisValidation { score, suggestions })
}

fn calculate_specificity(text: &str) -> f64 {
    let word_count = text.split_whitespace().count();
    let has_numbers = text.chars().any(|c| c.is_numeric());
    let has_specific_terms = ["exactly", "specifically", "precisely", "only"]
        .iter()
        .any(|t| text.to_lowercase().contains(t));

    let base = (word_count as f64 / 20.0).min(1.0);
    let boost = if has_numbers { 0.1 } else { 0.0 } + if has_specific_terms { 0.1 } else { 0.0 };

    (base + boost).min(1.0)
}
```

### bd-28v1: validate_straw_man_traps

**Purpose**: Detect persona straw man argument traps.

**Trap Detection Rules**:
- **IrrationalActor**: User acts against self-interest, ignores obvious solutions
- **ManicPixieDreamUser**: Idealized user who loves everything, no friction
- **StoicMonk**: User with unrealistic self-discipline, never distracted
- **YourClone**: User thinks exactly like the designer, same background

**Implementation**:
```rust
#[server]
pub async fn validate_straw_man_traps(persona: String) -> Result<StrawManValidation, ServerFnError> {
    let traps = [
        (StrawManTrap::IrrationalActor, detect_irrational_actor(&persona)),
        (StrawManTrap::ManicPixieDreamUser, detect_manic_pixie(&persona)),
        (StrawManTrap::StoicMonk, detect_stoic_monk(&persona)),
        (StrawManTrap::YourClone, detect_your_clone(&persona)),
    ]
    .iter()
    .filter_map(|(trap, detected)| if *detected { Some(*trap) } else { None })
    .collect();

    Ok(StrawManValidation::new(traps))
}
```

### bd-2mcc: validate_vorp

**Purpose**: Validate VORP (Value, Obvious, Real, Possible) justification.

**Implementation**:
```rust
#[server]
pub async fn validate_vorp(
    value: String,
    obvious: String,
    real: String,
    possible: String,
) -> Result<VorpValidation, ServerFnError> {
    let value_score = validate_v_dimension(&value);
    let obvious_score = validate_o_dimension(&obvious);
    let real_score = validate_r_dimension(&real);
    let possible_score = validate_p_dimension(&possible);

    let overall = (value_score + obvious_score + real_score + possible_score) / 4.0;

    Ok(VorpValidation {
        overall_score: overall,
        dimensions: vec![
            ("Value".to_string(), value_score),
            ("Obvious".to_string(), obvious_score),
            ("Real".to_string(), real_score),
            ("Possible".to_string(), possible_score),
        ],
    })
}
```

### bd-13yb: validate_hole_punching

**Purpose**: Check if all 3 hole types have been addressed.

**Implementation**:
```rust
#[server]
pub async fn validate_hole_punching(
    discovery_hole: Option<String>,
    edge_case_hole: Option<String>,
    motivation_dropoff: Option<String>,
) -> Result<HolePunchingValidation, ServerFnError> {
    let results = HolePunchingResults {
        discovery_hole: discovery_hole.filter(|s| !s.trim().is_empty()),
        edge_case_hole: edge_case_hole.filter(|s| !s.trim().is_empty()),
        motivation_dropoff: motivation_dropoff.filter(|s| !s.trim().is_empty()),
    };

    Ok(HolePunchingValidation {
        is_complete: results.is_complete(),
        addressed_count: results.addressed_count(),
        results,
    })
}
```

### bd-2uci: KirkContract Types

**Purpose**: Define the 16-section KIRK contract structure.

**Location**: `clarity-web/src/kirk.rs`

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KirkContract {
    pub sections: [KirkSection; 16],
    pub compiled_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KirkSection {
    pub id: usize,
    pub title: String,
    pub content: String,
    pub source_field: String,
}
```

### bd-zf68: EARS Extraction

**Purpose**: Extract EARS (Easy to Approach, Readable, Standard) requirements.

### bd-l1qq: compile_to_kirk

**Purpose**: Compile transcript into 16-section KIRK contract.

## Workflow

```bash
# Claim beads before starting
br claim bd-378l bd-28v1 bd-2mcc bd-13yb bd-2uci bd-zf68 bd-l1qq

# Implement all functions
# ... code ...

# Close beads when complete
br close bd-378l bd-28v1 bd-2mcc bd-13yb bd-2uci bd-zf68 bd-l1qq
```

## Validation

- [ ] All 7 server functions compile
- [ ] Functions accessible via Dioxus server macros
- [ ] No panics or unwraps
- [ ] Pure functions where possible
- [ ] Tests pass

## Definition of Done

- [ ] All beads closed
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] No clippy warnings

## Activity Log

- 2026-02-26T17:54:23Z – unknown – lane=done – Already implemented in main branch
