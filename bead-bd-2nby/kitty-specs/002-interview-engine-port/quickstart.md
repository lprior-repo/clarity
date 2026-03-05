# Quickstart: Intent-CLI Rust Port

**Feature**: 002-interview-engine-port
**Date**: 2026-02-27

## Installation

The intent module is part of `clarity-web`. No separate installation required.

```rust
use clarity_web::intent::{
    Spec, Feature, Behavior,
    InterviewSession, Profile, InterviewStage,
    BeadRecord, QualityReport,
};
```

## Basic Usage

### 1. Creating a Spec

```rust
use clarity_web::intent::{Spec, Feature, Behavior, Verification};

let spec = Spec {
    name: "User Authentication API".to_string(),
    description: "REST API for user authentication".to_string(),
    audience: "Backend developers".to_string(),
    version: "1.0.0".to_string(),
    success_criteria: vec![
        "Users can register with email/password".to_string(),
        "Users can login and receive JWT tokens".to_string(),
    ],
    features: vec![
        Feature {
            name: "registration".to_string(),
            description: "User registration flow".to_string(),
            behaviors: vec![
                Behavior {
                    name: "register_with_valid_data".to_string(),
                    intent: "Create new user account".to_string(),
                    verifications: vec![
                        Verification {
                            description: "Returns 201 Created".to_string(),
                            criteria: vec!["status == 201".to_string()],
                            examples: vec![],
                        },
                    ],
                    ..Default::default()
                },
            ],
        },
    ],
    ..Default::default()
};
```

### 2. Running an Interview

```rust
use clarity_web::intent::interview::{
    InterviewSession, Profile, InterviewStage,
    Answer, Perspective, Question,
};

// Create a new session
let session = InterviewSession::new(
    "session-001".to_string(),
    Profile::Api,
    chrono::Utc::now().to_rfc3339(),
);

// Add an answer
let answer = Answer {
    question_id: "q-base-url".to_string(),
    question_text: "What is the base URL for the API?".to_string(),
    perspective: Perspective::Developer,
    round: 1,
    response: "https://api.example.com/v1".to_string(),
    extracted: HashMap::new(),
    confidence: 0.95,
    notes: String::new(),
    timestamp: chrono::Utc::now().to_rfc3339(),
};

let session = session.add_answer(answer);

// Check for gaps
let gaps = session.detect_gaps();
for gap in &gaps {
    println!("Gap: {} - {}", gap.field, gap.description);
}

// Check for conflicts
let conflicts = session.detect_conflicts();
for conflict in &conflicts {
    println!("Conflict: {}", conflict.description);
}
```

### 3. Persisting Sessions

```rust
use clarity_web::intent::interview::storage::{
    session_to_jsonl_line,
    append_session_to_jsonl,
    list_sessions_from_jsonl,
};
use std::path::Path;

// Save session
let path = Path::new(".intent/sessions.jsonl");
append_session_to_jsonl(&session, path)?;

// Load sessions
let sessions = list_sessions_from_jsonl(path)?;
for s in &sessions {
    println!("Session: {} ({})", s.id, s.profile);
}

// Diff sessions
let diff = diff_sessions(&old_session, &new_session);
println!("{}", format_diff(&diff));
```

### 4. Generating Beads

```rust
use clarity_web::intent::beads::{generate_beads_from_session, bead_to_jsonl_line};

// Generate beads from completed session
let beads = generate_beads_from_session(&session);

for bead in &beads {
    println!("Bead: {} [{}]", bead.title, bead.issue_type);
    println!("  Priority: {}", bead.priority);
    println!("  Acceptance Criteria:");
    for criteria in &bead.acceptance_criteria {
        println!("    - {}", criteria);
    }
}

// Output as JSONL
let jsonl = beads.iter()
    .map(bead_to_jsonl_line)
    .collect::<Vec<_>>()
    .join("\n");
```

### 5. Analyzing Quality

```rust
use clarity_web::intent::quality::analyze_spec;

let report = analyze_spec(&spec);

println!("Quality Report:");
println!("  Coverage: {}/100", report.coverage_score);
println!("  Clarity: {}/100", report.clarity_score);
println!("  Testability: {}/100", report.testability_score);
println!("  AI Readiness: {}/100", report.ai_readiness_score);
println!("  Overall: {}/100", report.overall_score);

if !report.issues.is_empty() {
    println!("\nIssues:");
    for issue in &report.issues {
        println!("  - {:?}", issue);
    }
}

if !report.suggestions.is_empty() {
    println!("\nSuggestions:");
    for suggestion in &report.suggestions {
        println!("  - {}", suggestion);
    }
}
```

### 6. Validating Specs

```rust
use clarity_web::intent::validation::validate_spec_file;

let result = validate_spec_file("spec.cue");

match result {
    ValidationResult::Valid => {
        println!("✓ Spec is valid");
    }
    ValidationResult::Invalid(errors) => {
        println!("✗ Spec validation failed:");
        for error in &errors {
            println!("  - {:?}", error);
        }
    }
}
```

### 7. Creating Execution Plans

```rust
use clarity_web::intent::plan::{compute_plan, apply_phase_gating};

let plan = compute_plan(&session)?;

println!("Execution Plan:");
println!("  Blockers: {:?}", plan.blockers);

for phase in &plan.phases {
    println!("\n  Phase {}:", phase.phase_number);
    for bead in &phase.beads {
        println!("    - {} [{}]", bead.title, bead.effort);
    }
}

// Apply phase gating based on session state
let gated_plan = apply_phase_gating(&session, plan);
```

## Common Patterns

### Pattern: Interview Workflow

```rust
fn run_interview(profile: Profile) -> Result<InterviewSession, Error> {
    let mut session = InterviewSession::new(
        uuid::Uuid::new_v4().to_string(),
        profile,
        chrono::Utc::now().to_rfc3339(),
    );

    // Round 1: Discovery
    for question in get_discovery_questions(profile) {
        let response = prompt_user(&question)?;
        let answer = Answer::from_question(&question, response, 1);
        session = session.add_answer(answer);

        // Check for immediate gaps
        let gaps = session.check_for_gaps(&question, &session.answers.last().unwrap());
        session.gaps.extend(gaps);
    }

    // Resolve gaps before proceeding
    while session.get_blocking_gaps().len() > 0 {
        for gap in session.get_blocking_gaps() {
            let resolution = prompt_gap_resolution(gap)?;
            session = session.resolve_gap(&gap.id, &resolution);
        }
    }

    // Round 2+: Refinement
    session = session.complete_round();
    // ... continue with refinement questions

    Ok(session)
}
```

### Pattern: Spec Loading

```rust
fn load_and_validate(path: &Path) -> Result<Spec, Error> {
    // Load from CUE file
    let spec = load_spec(path)?;

    // Validate structure
    let validation = validate_spec_structure(&spec);
    if let ValidationResult::Invalid(errors) = validation {
        return Err(Error::Validation(errors));
    }

    // Analyze quality
    let report = analyze_spec(&spec);
    if report.overall_score < 60 {
        eprintln!("Warning: Low quality score ({}/100)", report.overall_score);
    }

    Ok(spec)
}
```

### Pattern: Bead Emission

```rust
fn emit_beads_to_tracker(session_id: &str, dry_run: bool) -> Result<EmissionResult, Error> {
    let session = load_session(session_id)?;

    // Generate beads
    let beads = generate_beads_from_session(&session);

    // Check for existing beads (idempotency)
    let existing = list_existing_bead_titles()?;
    let new_beads: Vec<_> = beads.iter()
        .filter(|b| !existing.contains(&b.title.to_lowercase()))
        .collect();

    let result = EmissionResult {
        session_id: session_id.to_string(),
        dry_run,
        total_beads: beads.len(),
        already_exists: beads.len() - new_beads.len(),
        would_create: new_beads.len(),
        created: 0,
        failed: 0,
        commands: new_beads.iter()
            .map(|b| format!("br create \"{}\"", b.title))
            .collect(),
    };

    if !dry_run {
        // Execute br commands...
    }

    Ok(result)
}
```

## Error Handling

All fallible operations return `Result<T, IntentError>`:

```rust
use clarity_web::intent::IntentError;

match load_spec(path) {
    Ok(spec) => process(spec),
    Err(IntentError::FileNotFound(path)) => {
        eprintln!("File not found: {}", path);
    }
    Err(IntentError::CueValidationError(msg)) => {
        eprintln!("CUE validation failed:\n{}", msg);
    }
    Err(IntentError::JsonParseError(msg)) => {
        eprintln!("JSON parse error: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {:?}", e);
    }
}
```

## Performance Guidelines

1. **JSONL I/O**: Use buffered readers/writers for large files
2. **Gap Detection**: O(n) where n = number of answers
3. **Conflict Detection**: O(n²) pairwise comparison
4. **Bead Generation**: O(n) where n = number of answers

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_round_trip() {
        let session = InterviewSession::new_test();
        let json = session_to_jsonl_line(&session);
        let parsed: InterviewSession = serde_json::from_str(&json).unwrap();
        assert_eq!(session, parsed);
    }

    #[test]
    fn test_gap_detection() {
        let session = InterviewSession::new("id".into(), Profile::Api, "now".into());
        let gaps = session.detect_gaps();
        assert!(gaps.iter().any(|g| g.field == "base_url"));
    }

    #[test]
    fn test_conflict_detection() {
        let mut session = InterviewSession::new_test();
        session = session.add_answer(Answer {
            response: "fast response times".into(),
            ..Default::default()
        });
        session = session.add_answer(Answer {
            response: "strongly consistent data".into(),
            ..Default::default()
        });
        let conflicts = session.detect_conflicts();
        assert!(!conflicts.is_empty());
    }
}
```
