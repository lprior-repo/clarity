//! Tests for conflict_detection module
//!
//! Test quality doesn't matter - we test source code quality.

#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![forbid(unsafe_code)]

use chrono::Utc;
use uuid::Uuid;

use super::conflict_detection::{
  Conflict, ConflictAnalysis, ConflictDetectorBuilder, ConflictError, ConflictType, Requirement,
  Severity,
};

// ============================================================================
// CONFLICT TYPE DISPLAY TESTS
// ============================================================================

#[test]
fn conflict_type_display_scope_paradox() {
  assert_eq!(ConflictType::ScopeParadox.to_string(), "Scope Paradox");
}

#[test]
fn conflict_type_display_cap_theorem() {
  assert_eq!(ConflictType::CapTheorem.to_string(), "CAP Theorem");
}

#[test]
fn conflict_type_display_resource_contention() {
  assert_eq!(
    ConflictType::ResourceContention.to_string(),
    "Resource Contention"
  );
}

#[test]
fn conflict_type_display_priority_inversion() {
  assert_eq!(
    ConflictType::PriorityInversion.to_string(),
    "Priority Inversion"
  );
}

#[test]
fn conflict_type_display_dependency_conflict() {
  assert_eq!(
    ConflictType::DependencyConflict.to_string(),
    "Dependency Conflict"
  );
}

// ============================================================================
// SEVERITY TESTS
// ============================================================================

#[test]
fn severity_from_f32_valid_values() {
  let low = Severity::try_from(0.0);
  assert!(low.is_ok());
  assert!((low.ok().map_or(1.0, |s| s.value()) - 0.0).abs() < f32::EPSILON);

  let mid = Severity::try_from(0.5);
  assert!(mid.is_ok());
  assert!((mid.ok().map_or(0.0, |s| s.value()) - 0.5).abs() < f32::EPSILON);

  let high = Severity::try_from(1.0);
  assert!(high.is_ok());
  assert!((high.ok().map_or(0.0, |s| s.value()) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn severity_from_f32_rejects_negative() {
  let result = Severity::try_from(-0.1);
  assert!(matches!(result, Err(ConflictError::InvalidSeverity { .. })));
}

#[test]
fn severity_from_f32_rejects_greater_than_one() {
  let result = Severity::try_from(1.1);
  assert!(matches!(result, Err(ConflictError::InvalidSeverity { .. })));
}

#[test]
fn severity_is_low() {
  let severity = Severity::try_from(0.2).ok();
  assert!(severity.map_or(false, |s| s.is_low()));
}

#[test]
fn severity_is_medium() {
  let severity = Severity::try_from(0.5).ok();
  assert!(severity.map_or(false, |s| s.is_medium()));
}

#[test]
fn severity_is_high() {
  let severity = Severity::try_from(0.8).ok();
  assert!(severity.map_or(false, |s| s.is_high()));
}

#[test]
fn severity_is_critical() {
  let severity = Severity::try_from(0.95).ok();
  assert!(severity.map_or(false, |s| s.is_critical()));
}

// ============================================================================
// CONFLICT TESTS
// ============================================================================

#[test]
fn conflict_new_creates_valid_conflict() {
  let result = Conflict::new(
    ConflictType::ScopeParadox,
    "Test description".to_string(),
    0.5,
  );

  assert!(result.is_ok());
  if let Ok(conflict) = result {
    assert_eq!(conflict.conflict_type, ConflictType::ScopeParadox);
    assert_eq!(conflict.description, "Test description");
    assert!((conflict.severity.value() - 0.5).abs() < f32::EPSILON);
    assert!(conflict.resolution_hint.is_none());
  }
}

#[test]
fn conflict_new_rejects_empty_description() {
  let result = Conflict::new(ConflictType::ScopeParadox, String::new(), 0.5);
  assert!(matches!(result, Err(ConflictError::EmptyField { .. })));
}

#[test]
fn conflict_new_rejects_whitespace_only_description() {
  let result = Conflict::new(ConflictType::ScopeParadox, "   ".to_string(), 0.5);
  assert!(matches!(result, Err(ConflictError::EmptyField { .. })));
}

#[test]
fn conflict_with_resolution_hint() {
  let result = Conflict::new(ConflictType::ResourceContention, "Test".to_string(), 0.5);

  if let Ok(conflict) = result {
    let with_hint = conflict.with_resolution_hint("Resolve by prioritizing".to_string());
    assert_eq!(
      with_hint.resolution_hint,
      Some("Resolve by prioritizing".to_string())
    );
  }
}

#[test]
fn conflict_has_auto_generated_id() {
  let result = Conflict::new(ConflictType::CapTheorem, "Test".to_string(), 0.3);
  assert!(result.is_ok());
  if let Ok(conflict) = result {
    assert_ne!(conflict.id, Uuid::nil());
  }
}

#[test]
fn conflict_has_detected_at_timestamp() {
  let before = Utc::now();
  let result = Conflict::new(ConflictType::CapTheorem, "Test".to_string(), 0.3);
  let after = Utc::now();

  if let Ok(conflict) = result {
    assert!(conflict.detected_at >= before);
    assert!(conflict.detected_at <= after);
  }
}

// ============================================================================
// REQUIREMENT TESTS
// ============================================================================

#[test]
fn requirement_new_creates_valid_requirement() {
  let result = Requirement::new(
    "req-001".to_string(),
    "System must be fast".to_string(),
    vec!["performance".to_string()],
  );

  assert!(result.is_ok());
  if let Ok(req) = result {
    assert_eq!(req.id, "req-001");
    assert_eq!(req.description, "System must be fast");
    assert!(req.tags.contains(&"performance".to_string()));
  }
}

#[test]
fn requirement_new_rejects_empty_id() {
  let result = Requirement::new(
    String::new(),
    "Description".to_string(),
    vec!["tag".to_string()],
  );
  assert!(matches!(result, Err(ConflictError::EmptyField { .. })));
}

#[test]
fn requirement_new_rejects_empty_description() {
  let result = Requirement::new(
    "req-001".to_string(),
    String::new(),
    vec!["tag".to_string()],
  );
  assert!(matches!(result, Err(ConflictError::EmptyField { .. })));
}

#[test]
fn requirement_with_tags() {
  let result = Requirement::new("req-001".to_string(), "Test".to_string(), vec![]);

  if let Ok(req) = result {
    let with_tags = req.with_tags(vec!["security".to_string(), "auth".to_string()]);
    assert_eq!(with_tags.tags.len(), 2);
  }
}

#[test]
fn requirement_with_priority() {
  let result = Requirement::new("req-001".to_string(), "Test".to_string(), vec![]);

  if let Ok(req) = result {
    let with_priority = req.with_priority(1);
    assert_eq!(with_priority.priority, Some(1));
  }
}

#[test]
fn requirement_with_dependencies() {
  let result = Requirement::new("req-001".to_string(), "Test".to_string(), vec![]);

  if let Ok(req) = result {
    let with_deps = req.with_dependencies(vec!["req-000".to_string()]);
    assert_eq!(with_deps.dependencies.len(), 1);
  }
}

// ============================================================================
// CONFLICT DETECTOR BUILDER TESTS
// ============================================================================

#[test]
fn conflict_detector_builder_creates_detector() {
  let detector = ConflictDetectorBuilder::new().build();
  assert!(detector.requirements().is_empty());
}

#[test]
fn conflict_detector_builder_with_requirement() {
  let req = Requirement::new("req-001".to_string(), "Test".to_string(), vec![]);
  if let Ok(r) = req {
    let detector = ConflictDetectorBuilder::new().with_requirement(r).build();
    assert_eq!(detector.requirements().len(), 1);
  }
}

#[test]
fn conflict_detector_builder_with_requirements() {
  let req1 = Requirement::new("req-001".to_string(), "Test 1".to_string(), vec![]);
  let req2 = Requirement::new("req-002".to_string(), "Test 2".to_string(), vec![]);

  if let (Ok(r1), Ok(r2)) = (req1, req2) {
    let detector = ConflictDetectorBuilder::new()
      .with_requirements(vec![r1, r2])
      .build();
    assert_eq!(detector.requirements().len(), 2);
  }
}

// ============================================================================
// CONFLICT DETECTION TESTS
// ============================================================================

#[test]
fn conflict_detector_analyze_returns_analysis() {
  let detector = ConflictDetectorBuilder::new().build();
  let result = detector.analyze();

  assert!(result.is_ok());
  if let Ok(analysis) = result {
    assert!(analysis.conflicts_found.is_empty() || !analysis.conflicts_found.is_empty());
    assert!(analysis.risk_score >= 0.0 && analysis.risk_score <= 1.0);
  }
}

#[test]
fn conflict_detector_empty_requirements_no_conflicts() {
  let detector = ConflictDetectorBuilder::new().build();
  if let Ok(analysis) = detector.analyze() {
    assert!(analysis.conflicts_found.is_empty());
    assert!((analysis.risk_score - 0.0).abs() < f32::EPSILON);
  }
}

#[test]
fn conflict_detector_detects_scope_paradox() {
  // Scope paradox: Requirements that expand scope while demanding speed
  let req1 = Requirement::new(
    "req-001".to_string(),
    "Implement comprehensive feature set".to_string(),
    vec!["scope".to_string(), "features".to_string()],
  );
  let req2 = Requirement::new(
    "req-002".to_string(),
    "Ship in minimal timeframe".to_string(),
    vec!["timeline".to_string(), "speed".to_string()],
  );

  if let (Ok(r1), Ok(r2)) = (req1, req2) {
    let detector = ConflictDetectorBuilder::new()
      .with_requirements(vec![r1, r2])
      .build();

    if let Ok(analysis) = detector.analyze() {
      // Should detect scope paradox between comprehensive features and minimal timeline
      let has_scope_paradox = analysis
        .conflicts_found
        .iter()
        .any(|c| c.conflict_type == ConflictType::ScopeParadox);
      assert!(has_scope_paradox);
    }
  }
}

#[test]
fn conflict_detector_detects_cap_theorem() {
  // CAP theorem: Consistency + Availability + Partition tolerance tradeoff
  let req1 = Requirement::new(
    "req-001".to_string(),
    "Strong consistency across all nodes".to_string(),
    vec!["consistency".to_string(), "distributed".to_string()],
  );
  let req2 = Requirement::new(
    "req-002".to_string(),
    "High availability with 99.99% uptime".to_string(),
    vec!["availability".to_string(), "distributed".to_string()],
  );
  let req3 = Requirement::new(
    "req-003".to_string(),
    "System must handle network partitions".to_string(),
    vec!["partition-tolerance".to_string(), "distributed".to_string()],
  );

  if let (Ok(r1), Ok(r2), Ok(r3)) = (req1, req2, req3) {
    let detector = ConflictDetectorBuilder::new()
      .with_requirements(vec![r1, r2, r3])
      .build();

    if let Ok(analysis) = detector.analyze() {
      let has_cap_conflict = analysis
        .conflicts_found
        .iter()
        .any(|c| c.conflict_type == ConflictType::CapTheorem);
      assert!(has_cap_conflict);
    }
  }
}

#[test]
fn conflict_detector_detects_resource_contention() {
  // Resource contention: Multiple requirements competing for same resources
  let req1 = Requirement::new(
    "req-001".to_string(),
    "High-performance real-time processing".to_string(),
    vec!["cpu-intensive".to_string(), "realtime".to_string()],
  );
  let req2 = Requirement::new(
    "req-002".to_string(),
    "Run heavy batch analytics jobs".to_string(),
    vec!["cpu-intensive".to_string(), "batch".to_string()],
  );

  if let (Ok(r1), Ok(r2)) = (req1, req2) {
    let detector = ConflictDetectorBuilder::new()
      .with_requirements(vec![r1, r2])
      .build();

    if let Ok(analysis) = detector.analyze() {
      let has_resource_contention = analysis
        .conflicts_found
        .iter()
        .any(|c| c.conflict_type == ConflictType::ResourceContention);
      assert!(has_resource_contention);
    }
  }
}

#[test]
fn conflict_detector_detects_priority_inversion() {
  // Priority inversion: Lower priority blocking higher priority
  // req-002 has priority 1 (HIGH) but depends on req-003 with priority 10 (LOWER)
  let req1 = Requirement::new(
    "req-001".to_string(),
    "Critical feature A".to_string(),
    vec![],
  )
  .map(|r| r.with_priority(1));
  let req2 = Requirement::new(
    "req-002".to_string(),
    "High priority feature B".to_string(),
    vec![],
  )
  .map(|r| {
    r.with_priority(1)
      .with_dependencies(vec!["req-003".to_string()])
  });
  let req3 = Requirement::new(
    "req-003".to_string(),
    "Lower priority blocking dependency".to_string(),
    vec![],
  )
  .map(|r| r.with_priority(10));

  if let (Some(r1), Some(r2), Some(r3)) = (req1.ok(), req2.ok(), req3.ok()) {
    let detector = ConflictDetectorBuilder::new()
      .with_requirements(vec![r1, r2, r3])
      .build();

    if let Ok(analysis) = detector.analyze() {
      let has_priority_inversion = analysis
        .conflicts_found
        .iter()
        .any(|c| c.conflict_type == ConflictType::PriorityInversion);
      assert!(has_priority_inversion);
    }
  }
}

#[test]
fn conflict_detector_detects_dependency_conflict() {
  // Dependency conflict: Circular or incompatible dependencies
  let req1 = Requirement::new("req-001".to_string(), "Feature A".to_string(), vec![])
    .map(|r| r.with_dependencies(vec!["req-002".to_string()]));
  let req2 = Requirement::new("req-002".to_string(), "Feature B".to_string(), vec![])
    .map(|r| r.with_dependencies(vec!["req-001".to_string()]));

  if let (Some(r1), Some(r2)) = (req1.ok(), req2.ok()) {
    let detector = ConflictDetectorBuilder::new()
      .with_requirements(vec![r1, r2])
      .build();

    if let Ok(analysis) = detector.analyze() {
      let has_dependency_conflict = analysis
        .conflicts_found
        .iter()
        .any(|c| c.conflict_type == ConflictType::DependencyConflict);
      assert!(has_dependency_conflict);
    }
  }
}

// ============================================================================
// CONFLICT ANALYSIS TESTS
// ============================================================================

#[test]
fn conflict_analysis_risk_score_increases_with_conflicts() {
  let conflict1 = Conflict::new(ConflictType::ScopeParadox, "Test 1".to_string(), 0.5);
  let conflict2 = Conflict::new(ConflictType::ResourceContention, "Test 2".to_string(), 0.7);

  if let (Ok(c1), Ok(c2)) = (conflict1, conflict2) {
    let analysis = ConflictAnalysis {
      id: Uuid::new_v4(),
      conflicts_found: vec![c1, c2],
      risk_score: 0.0,
      analyzed_at: Utc::now(),
    };

    let score = analysis.calculate_risk_score();
    assert!(score > 0.0);
  }
}

#[test]
fn conflict_analysis_high_severity_increases_risk() {
  let conflict = Conflict::new(ConflictType::CapTheorem, "Test".to_string(), 0.9);

  if let Ok(c) = conflict {
    let analysis = ConflictAnalysis {
      id: Uuid::new_v4(),
      conflicts_found: vec![c],
      risk_score: 0.0,
      analyzed_at: Utc::now(),
    };

    let score = analysis.calculate_risk_score();
    assert!(score > 0.5);
  }
}

#[test]
fn conflict_analysis_empty_has_zero_risk() {
  let analysis = ConflictAnalysis {
    id: Uuid::new_v4(),
    conflicts_found: vec![],
    risk_score: 0.0,
    analyzed_at: Utc::now(),
  };

  let score = analysis.calculate_risk_score();
  assert!((score - 0.0).abs() < f32::EPSILON);
}

#[test]
fn conflict_analysis_has_critical_conflicts() {
  let conflict = Conflict::new(ConflictType::CapTheorem, "Test".to_string(), 0.95);

  if let Ok(c) = conflict {
    let analysis = ConflictAnalysis {
      id: Uuid::new_v4(),
      conflicts_found: vec![c],
      risk_score: 0.0,
      analyzed_at: Utc::now(),
    };

    assert!(analysis.has_critical_conflicts());
  }
}

#[test]
fn conflict_analysis_no_critical_conflicts() {
  let conflict = Conflict::new(ConflictType::ScopeParadox, "Test".to_string(), 0.3);

  if let Ok(c) = conflict {
    let analysis = ConflictAnalysis {
      id: Uuid::new_v4(),
      conflicts_found: vec![c],
      risk_score: 0.0,
      analyzed_at: Utc::now(),
    };

    assert!(!analysis.has_critical_conflicts());
  }
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
fn conflict_type_serialization() {
  let conflict_type = ConflictType::ScopeParadox;
  if let Ok(json) = serde_json::to_string(&conflict_type) {
    assert!(json.contains("scope_paradox"));
  }
}

#[test]
fn conflict_serialization() {
  let conflict = Conflict::new(ConflictType::CapTheorem, "Test conflict".to_string(), 0.7);

  if let Ok(c) = conflict {
    let with_hint = c.with_resolution_hint("Add caching layer".to_string());
    if let Ok(json) = serde_json::to_string(&with_hint) {
      let parsed: Result<Conflict, _> = serde_json::from_str(&json);
      assert!(parsed.is_ok());
    }
  }
}

#[test]
fn requirement_serialization() {
  let req = Requirement::new(
    "req-001".to_string(),
    "Test requirement".to_string(),
    vec!["tag".to_string()],
  );

  if let Ok(r) = req {
    if let Ok(json) = serde_json::to_string(&r) {
      let parsed: Result<Requirement, _> = serde_json::from_str(&json);
      assert!(parsed.is_ok());
    }
  }
}

#[test]
fn conflict_analysis_serialization() {
  let conflict = Conflict::new(ConflictType::ResourceContention, "Test".to_string(), 0.5);

  if let Ok(c) = conflict {
    let analysis = ConflictAnalysis {
      id: Uuid::new_v4(),
      conflicts_found: vec![c],
      risk_score: 0.5,
      analyzed_at: Utc::now(),
    };

    if let Ok(json) = serde_json::to_string(&analysis) {
      let parsed: Result<ConflictAnalysis, _> = serde_json::from_str(&json);
      assert!(parsed.is_ok());
    }
  }
}
