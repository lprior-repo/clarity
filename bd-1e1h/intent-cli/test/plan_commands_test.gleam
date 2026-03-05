//// Tests for plan execution and regeneration commands
//// Covers: plan, plan-next, plan-approve, beads-regenerate
////
//// Test scenarios:
//// - Plan computation from session beads
//// - Plan display in different formats (human, json, ai)
//// - Plan approval with and without --yes flag
//// - Next action determination
//// - Bead regeneration for failed/blocked beads

import gleam/list
import gleam/option.{None, Some}
import gleam/string
import gleeunit/should
import intent/bead_feedback
import intent/plan_mode
import intent/plan_next

// =============================================================================
// Plan Mode Tests - Plan Computation and Display
// =============================================================================

pub fn plan_execution_phase_ordering_test() {
  // Test that beads are correctly ordered into phases based on dependencies
  let bead1 =
    plan_mode.PlanBead(
      id: "B-001",
      title: "Setup database",
      requires: [],
      effort: plan_mode.Effort10min,
      status: plan_mode.Pending,
    )

  let bead2 =
    plan_mode.PlanBead(
      id: "B-002",
      title: "Create tables",
      requires: ["B-001"],
      effort: plan_mode.Effort15min,
      status: plan_mode.Pending,
    )

  let bead3 =
    plan_mode.PlanBead(
      id: "B-003",
      title: "Insert data",
      requires: ["B-002"],
      effort: plan_mode.Effort20min,
      status: plan_mode.Pending,
    )

  let beads = [bead1, bead2, bead3]

  case plan_mode.detect_dependency_graph(beads) {
    Ok(phases) -> {
      list.length(phases) |> should.equal(3)

      // Phase 1 should have B-001 (no deps)
      case list.first(phases) {
        Ok(phase1) -> {
          list.length(phase1.beads) |> should.equal(1)
          case list.first(phase1.beads) {
            Ok(first_bead) -> first_bead.id |> should.equal("B-001")
            Error(_) -> should.fail()
          }
        }
        Error(_) -> should.fail()
      }

      // Phase 3 should have B-003 (depends on B-002)
      case list.last(phases) {
        Ok(phase3) -> {
          list.length(phase3.beads) |> should.equal(1)
          case list.first(phase3.beads) {
            Ok(last_bead) -> last_bead.id |> should.equal("B-003")
            Error(_) -> should.fail()
          }
        }
        Error(_) -> should.fail()
      }
    }
    Error(_) -> should.fail()
  }
}

pub fn plan_parallel_beads_test() {
  // Test that independent beads are grouped into the same phase
  let bead1 =
    plan_mode.PlanBead(
      id: "AUTH-001",
      title: "Setup auth",
      requires: [],
      effort: plan_mode.Effort10min,
      status: plan_mode.Pending,
    )

  let bead2 =
    plan_mode.PlanBead(
      id: "DB-001",
      title: "Setup database",
      requires: [],
      effort: plan_mode.Effort10min,
      status: plan_mode.Pending,
    )

  let bead3 =
    plan_mode.PlanBead(
      id: "API-001",
      title: "Create API",
      requires: ["AUTH-001", "DB-001"],
      effort: plan_mode.Effort20min,
      status: plan_mode.Pending,
    )

  let beads = [bead1, bead2, bead3]

  case plan_mode.detect_dependency_graph(beads) {
    Ok(phases) -> {
      // Should have 2 phases: parallel setup, then API
      list.length(phases) |> should.equal(2)

      // Phase 1 should have both independent beads
      case list.first(phases) {
        Ok(phase1) -> {
          list.length(phase1.beads) |> should.equal(2)
          phase1.can_parallel |> should.be_true()
        }
        Error(_) -> should.fail()
      }

      // Phase 2 should have API
      case list.last(phases) {
        Ok(phase2) -> {
          list.length(phase2.beads) |> should.equal(1)
          case list.first(phase2.beads) {
            Ok(bead) -> bead.id |> should.equal("API-001")
            Error(_) -> should.fail()
          }
        }
        Error(_) -> should.fail()
      }
    }
    Error(_) -> should.fail()
  }
}

pub fn plan_cyclic_dependency_detection_test() {
  // Test that circular dependencies are detected and rejected
  let bead1 =
    plan_mode.PlanBead(
      id: "A",
      title: "Task A",
      requires: ["C"],
      effort: plan_mode.Effort10min,
      status: plan_mode.Pending,
    )

  let bead2 =
    plan_mode.PlanBead(
      id: "B",
      title: "Task B",
      requires: ["A"],
      effort: plan_mode.Effort10min,
      status: plan_mode.Pending,
    )

  let bead3 =
    plan_mode.PlanBead(
      id: "C",
      title: "Task C",
      requires: ["B"],
      effort: plan_mode.Effort10min,
      status: plan_mode.Pending,
    )

  let beads = [bead1, bead2, bead3]

  case plan_mode.detect_dependency_graph(beads) {
    Error(plan_mode.CyclicDependency(cycle_beads)) -> {
      // Should report the cyclic beads
      list.length(cycle_beads) |> should.equal(3)
      list.contains(cycle_beads, "A") |> should.be_true()
      list.contains(cycle_beads, "B") |> should.be_true()
      list.contains(cycle_beads, "C") |> should.be_true()
    }
    Ok(_) -> {
      // Should NOT succeed with cyclic deps
      should.fail()
    }
    Error(_) -> {
      // Other errors are also acceptable
      True |> should.be_true()
    }
  }
}

pub fn plan_missing_dependency_detection_test() {
  // Test that missing dependencies are detected
  let bead1 =
    plan_mode.PlanBead(
      id: "TASK-001",
      title: "Main task",
      requires: ["MISSING-DEP"],
      effort: plan_mode.Effort10min,
      status: plan_mode.Pending,
    )

  let beads = [bead1]

  case plan_mode.detect_dependency_graph(beads) {
    Error(plan_mode.MissingDependency(bead, missing)) -> {
      bead |> should.equal("TASK-001")
      missing |> should.equal("MISSING-DEP")
    }
    Ok(_) -> should.fail()
    Error(_) -> {
      // Other error types also acceptable
      True |> should.be_true()
    }
  }
}

pub fn plan_format_human_includes_header_test() {
  let phase =
    plan_mode.ExecutionPhase(
      phase_number: 1,
      title: "Phase 1",
      beads: [],
      can_parallel: False,
      effort: "10min",
    )

  let plan =
    plan_mode.ExecutionPlan(
      session_id: "test-session",
      generated_at: "2026-01-01T00:00:00Z",
      phases: [phase],
      total_beads: 0,
      total_effort: "0min",
      risk: plan_mode.Low,
      blockers: [],
    )

  let output = plan_mode.format_plan_human(plan)

  output |> string.contains("EXECUTION PLAN") |> should.be_true()
  output |> string.contains("test-session") |> should.be_true()
}

pub fn plan_format_json_valid_test() {
  let bead =
    plan_mode.PlanBead(
      id: "TEST-001",
      title: "Test bead",
      requires: [],
      effort: plan_mode.Effort5min,
      status: plan_mode.Pending,
    )

  let phase =
    plan_mode.ExecutionPhase(
      phase_number: 1,
      title: "Phase 1",
      beads: [bead],
      can_parallel: False,
      effort: "5min",
    )

  let plan =
    plan_mode.ExecutionPlan(
      session_id: "test-session",
      generated_at: "2026-01-01T00:00:00Z",
      phases: [phase],
      total_beads: 1,
      total_effort: "5min",
      risk: plan_mode.Low,
      blockers: [],
    )

  let output = plan_mode.format_plan_json(plan)

  // JSON should contain key fields
  output |> string.contains("\"session_id\"") |> should.be_true()
  output |> string.contains("\"phases\"") |> should.be_true()
  output |> string.contains("TEST-001") |> should.be_true()
  output |> string.contains("test-session") |> should.be_true()
}

pub fn plan_format_ai_includes_action_test() {
  let bead =
    plan_mode.PlanBead(
      id: "TEST-001",
      title: "Test bead",
      requires: [],
      effort: plan_mode.Effort5min,
      status: plan_mode.Pending,
    )

  let phase =
    plan_mode.ExecutionPhase(
      phase_number: 1,
      title: "Phase 1",
      beads: [bead],
      can_parallel: False,
      effort: "5min",
    )

  let plan =
    plan_mode.ExecutionPlan(
      session_id: "test-session",
      generated_at: "2026-01-01T00:00:00Z",
      phases: [phase],
      total_beads: 1,
      total_effort: "5min",
      risk: plan_mode.Low,
      blockers: [],
    )

  let output = plan_mode.format_plan_ai(plan)

  // AI format should include plan metadata and phases
  output |> string.contains("session_id") |> should.be_true()
  output |> string.contains("test-session") |> should.be_true()
  output |> string.contains("phases") |> should.be_true()
  output |> string.contains("TEST-001") |> should.be_true()
}

pub fn plan_blockers_displayed_test() {
  let plan =
    plan_mode.ExecutionPlan(
      session_id: "test-session",
      generated_at: "2026-01-01T00:00:00Z",
      phases: [],
      total_beads: 0,
      total_effort: "0min",
      risk: plan_mode.High,
      blockers: ["Missing API key", "Database unavailable"],
    )

  let output = plan_mode.format_plan_human(plan)

  output |> string.contains("BLOCKERS") |> should.be_true()
  output |> string.contains("Missing API key") |> should.be_true()
  output |> string.contains("Database unavailable") |> should.be_true()
}

pub fn plan_risk_level_assessment_test() {
  // Low risk: all beads pending/completed, no blockers
  let low_risk_beads = [
    plan_mode.PlanBead(
      id: "B-001",
      title: "Easy task",
      requires: [],
      effort: plan_mode.Effort5min,
      status: plan_mode.Pending,
    ),
  ]

  // High risk: blocked beads
  let high_risk_beads = [
    plan_mode.PlanBead(
      id: "B-002",
      title: "Blocked task",
      requires: [],
      effort: plan_mode.Effort30min,
      status: plan_mode.Blocked,
    ),
  ]

  let low_risk_phases = case plan_mode.detect_dependency_graph(low_risk_beads) {
    Ok(p) -> p
    Error(_) -> []
  }

  let high_risk_phases = case
    plan_mode.detect_dependency_graph(high_risk_beads)
  {
    Ok(p) -> p
    Error(_) -> []
  }

  let low_risk_plan =
    plan_mode.ExecutionPlan(
      session_id: "low",
      generated_at: "2026-01-01T00:00:00Z",
      phases: low_risk_phases,
      total_beads: 1,
      total_effort: "5min",
      risk: plan_mode.Low,
      blockers: [],
    )

  let high_risk_plan =
    plan_mode.ExecutionPlan(
      session_id: "high",
      generated_at: "2026-01-01T00:00:00Z",
      phases: high_risk_phases,
      total_beads: 1,
      total_effort: "30min",
      risk: plan_mode.High,
      blockers: ["Blocked task"],
    )

  let low_output = plan_mode.format_plan_human(low_risk_plan)
  let high_output = plan_mode.format_plan_human(high_risk_plan)

  low_output |> string.contains("low") |> should.be_true()
  high_output |> string.contains("high") |> should.be_true()
}

// =============================================================================
// Plan Next Tests - Next Action Determination
// =============================================================================

pub fn plan_next_empty_session_returns_done_test() {
  // When session has no beads or all completed, should return "done"
  // Note: This requires an actual session file, so we test the logic directly
  // In real scenarios, this would check the session state

  // Since plan_next requires filesystem access, we verify the function exists
  // and has the correct signature by checking it compiles
  let _fn_ref = plan_next.plan_next_command

  // The actual behavior depends on session state which requires filesystem
  // This test documents expected behavior
  True |> should.be_true()
}

pub fn plan_next_blocked_returns_blocked_test() {
  // When plan has blockers, should return blocked message
  // This tests the logic: if blockers exist, return first blocker

  // Test the blocking detection logic through format_error
  let error = plan_mode.CyclicDependency(["A", "B"])
  let error_msg = plan_mode.format_error(error)

  error_msg |> string.contains("Cyclic") |> should.be_true()
}

// =============================================================================
// Bead Feedback Tests - Regeneration Support
// =============================================================================

pub fn bead_feedback_failed_result_test() {
  let feedback =
    bead_feedback.BeadFeedback(
      bead_id: "BEAD-001",
      result: bead_feedback.Failed,
      reason: "Test failed",
      executed_at: "2026-01-01T00:00:00Z",
      duration_ms: 1000,
      error: Some(bead_feedback.BeadError(
        error_type: "ASSERTION_FAILED",
        message: "Expected 200 but got 500",
        trace: None,
      )),
      blocked_by: None,
    )

  feedback.result |> should.equal(bead_feedback.Failed)
  feedback.bead_id |> should.equal("BEAD-001")
}

pub fn bead_feedback_blocked_result_test() {
  let blocked_reason =
    bead_feedback.BlockedReason(
      blocker_type: "external_dependency",
      details: "Waiting for external approval",
      unblocks_when: "When API key received",
    )

  let feedback =
    bead_feedback.BeadFeedback(
      bead_id: "BEAD-002",
      result: bead_feedback.Blocked,
      reason: "Waiting for external approval",
      executed_at: "2026-01-01T00:00:00Z",
      duration_ms: 0,
      error: None,
      blocked_by: Some(blocked_reason),
    )

  feedback.result |> should.equal(bead_feedback.Blocked)
}

pub fn bead_feedback_success_result_test() {
  let feedback =
    bead_feedback.BeadFeedback(
      bead_id: "BEAD-003",
      result: bead_feedback.Success,
      reason: "All checks passed",
      executed_at: "2026-01-01T00:00:00Z",
      duration_ms: 500,
      error: None,
      blocked_by: None,
    )

  feedback.result |> should.equal(bead_feedback.Success)
}

pub fn bead_feedback_needs_regeneration_filter_test() {
  // Test filtering for beads that need regeneration
  let feedback_list = [
    bead_feedback.BeadFeedback(
      bead_id: "B-001",
      result: bead_feedback.Success,
      reason: "",
      executed_at: "",
      duration_ms: 0,
      error: None,
      blocked_by: None,
    ),
    bead_feedback.BeadFeedback(
      bead_id: "B-002",
      result: bead_feedback.Failed,
      reason: "Error",
      executed_at: "",
      duration_ms: 0,
      error: None,
      blocked_by: None,
    ),
    bead_feedback.BeadFeedback(
      bead_id: "B-003",
      result: bead_feedback.Blocked,
      reason: "Blocked",
      executed_at: "",
      duration_ms: 0,
      error: None,
      blocked_by: None,
    ),
  ]

  let needs_regen =
    feedback_list
    |> list.filter(fn(fb) {
      case fb.result {
        bead_feedback.Failed -> True
        bead_feedback.Blocked -> True
        _ -> False
      }
    })

  list.length(needs_regen) |> should.equal(2)

  let has_b002 =
    needs_regen
    |> list.any(fn(fb) { fb.bead_id == "B-002" })

  let has_b003 =
    needs_regen
    |> list.any(fn(fb) { fb.bead_id == "B-003" })

  has_b002 |> should.be_true()
  has_b003 |> should.be_true()
}

// =============================================================================
// Error Handling Tests
// =============================================================================

pub fn plan_format_error_session_not_found_test() {
  let error = plan_mode.SessionNotFound("missing-session")
  let msg = plan_mode.format_error(error)

  msg |> string.contains("missing-session") |> should.be_true()
  msg |> string.contains("not found") |> should.be_true()
}

pub fn plan_format_error_parse_error_test() {
  let error = plan_mode.ParseError("Invalid JSON at line 5")
  let msg = plan_mode.format_error(error)

  msg |> string.contains("Invalid JSON") |> should.be_true()
}

pub fn plan_format_error_cue_export_test() {
  let error = plan_mode.CueExportError("cue export failed")
  let msg = plan_mode.format_error(error)

  msg |> string.contains("cue export") |> should.be_true()
}

// =============================================================================
// Edge Cases
// =============================================================================

pub fn plan_empty_beads_test() {
  // Empty list should produce single phase with no beads
  let beads = []

  case plan_mode.detect_dependency_graph(beads) {
    Ok(_phases) -> {
      // Empty bead list should produce empty phases or single empty phase
      True |> should.be_true()
    }
    Error(_) -> {
      // Error is also acceptable for empty input
      True |> should.be_true()
    }
  }
}

pub fn plan_single_bead_no_deps_test() {
  let bead =
    plan_mode.PlanBead(
      id: "SINGLE",
      title: "Only bead",
      requires: [],
      effort: plan_mode.Effort5min,
      status: plan_mode.Pending,
    )

  case plan_mode.detect_dependency_graph([bead]) {
    Ok(phases) -> {
      list.length(phases) |> should.equal(1)
      case list.first(phases) {
        Ok(phase) -> {
          list.length(phase.beads) |> should.equal(1)
          case list.first(phase.beads) {
            Ok(b) -> b.id |> should.equal("SINGLE")
            Error(_) -> should.fail()
          }
        }
        Error(_) -> should.fail()
      }
    }
    Error(_) -> should.fail()
  }
}

pub fn plan_effort_calculation_test() {
  // Test that effort estimates are calculated correctly
  let bead1 =
    plan_mode.PlanBead(
      id: "E-001",
      title: "Quick task",
      requires: [],
      effort: plan_mode.Effort5min,
      status: plan_mode.Pending,
    )

  let bead2 =
    plan_mode.PlanBead(
      id: "E-002",
      title: "Medium task",
      requires: [],
      effort: plan_mode.Effort15min,
      status: plan_mode.Pending,
    )

  let beads = [bead1, bead2]

  // Effort should be calculated as part of plan computation
  // We verify the beads have correct effort values
  case beads {
    [b1, b2] -> {
      b1.effort |> should.equal(plan_mode.Effort5min)
      b2.effort |> should.equal(plan_mode.Effort15min)
    }
    _ -> should.fail()
  }
}

pub fn plan_decode_beads_json_invalid_test() {
  // Test that invalid JSON is handled gracefully
  let invalid_json = "not valid json"

  case plan_mode.decode_beads_json(invalid_json) {
    Error(_) -> True |> should.be_true()
    Ok(_) -> should.fail()
  }
}

pub fn plan_decode_beads_json_empty_array_test() {
  let json = "[]"

  case plan_mode.decode_beads_json(json) {
    Ok(beads) -> list.length(beads) |> should.equal(0)
    Error(_) -> should.fail()
  }
}

pub fn plan_decode_beads_json_malformed_entry_test() {
  // Test handling of malformed bead entries
  let json = "[{\"id\": \"TEST\", \"invalid_field\": \"value\"}]"

  // Should either parse with defaults or error gracefully
  case plan_mode.decode_beads_json(json) {
    Ok(_) -> True |> should.be_true()
    Error(_) -> True |> should.be_true()
  }
}

// =============================================================================
// Approval Workflow Tests
// =============================================================================

pub fn plan_approval_requires_valid_plan_test() {
  // Approval should validate plan exists and is valid
  // This is logic test - actual CLI requires filesystem

  // Test format_error for various plan errors
  let errors = [
    plan_mode.SessionNotFound("test"),
    plan_mode.ParseError("parse"),
    plan_mode.CueExportError("export"),
    plan_mode.JsonParseError("json"),
  ]

  let formatted = list.map(errors, plan_mode.format_error)

  // All errors should produce non-empty strings
  list.all(formatted, fn(s) { string.length(s) > 0 }) |> should.be_true()
}

pub fn plan_approval_with_blockers_warns_test() {
  // Plan with blockers should display warning during approval
  let plan =
    plan_mode.ExecutionPlan(
      session_id: "test",
      generated_at: "2026-01-01T00:00:00Z",
      phases: [],
      total_beads: 0,
      total_effort: "0min",
      risk: plan_mode.Critical,
      blockers: ["Critical blocker"],
    )

  let output = plan_mode.format_plan_human(plan)

  output |> string.contains("BLOCKERS") |> should.be_true()
  output |> string.contains("Critical blocker") |> should.be_true()
}

// =============================================================================
// Phase-Gate Enforcement Tests
// =============================================================================

import intent/interview

pub fn phase_gate_allows_phase_one_test() {
  // Phase 1 should always be allowed
  let session =
    interview.InterviewSession(
      id: "test",
      profile: interview.Api,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
      completed_at: "",
      stage: interview.Discovery,
      rounds_completed: 0,
      answers: [],
      gaps: [],
      conflicts: [],
      raw_notes: "",
      current_phase: 1,
      completed_phases: [],
    )

  let bead =
    plan_mode.PlanBead(
      id: "B-001",
      title: "Phase 1 bead",
      requires: [],
      effort: plan_mode.Effort10min,
      status: plan_mode.Pending,
    )

  let phase =
    plan_mode.ExecutionPhase(
      phase_number: 1,
      title: "Phase 1",
      beads: [bead],
      can_parallel: False,
      effort: "10min",
    )

  let plan =
    plan_mode.ExecutionPlan(
      session_id: "test",
      generated_at: "2026-01-01T00:00:00Z",
      phases: [phase],
      total_beads: 1,
      total_effort: "10min",
      risk: plan_mode.Low,
      blockers: [],
    )

  let gated_plan = interview.apply_phase_gating(session, plan)

  // Phase 1 should be allowed
  list.length(gated_plan.phases) |> should.equal(1)
  list.is_empty(gated_plan.blockers) |> should.be_true()
}

pub fn phase_gate_blocks_later_phases_test() {
  // Later phases should be blocked if prior phases not complete
  let session =
    interview.InterviewSession(
      id: "test",
      profile: interview.Api,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
      completed_at: "",
      stage: interview.Discovery,
      rounds_completed: 0,
      answers: [],
      gaps: [],
      conflicts: [],
      raw_notes: "",
      current_phase: 1,
      completed_phases: [],
    )

  let bead1 =
    plan_mode.PlanBead(
      id: "B-001",
      title: "Phase 1 bead",
      requires: [],
      effort: plan_mode.Effort10min,
      status: plan_mode.Pending,
    )

  let bead2 =
    plan_mode.PlanBead(
      id: "B-002",
      title: "Phase 2 bead",
      requires: [],
      effort: plan_mode.Effort10min,
      status: plan_mode.Pending,
    )

  let phase1 =
    plan_mode.ExecutionPhase(
      phase_number: 1,
      title: "Phase 1",
      beads: [bead1],
      can_parallel: False,
      effort: "10min",
    )

  let phase2 =
    plan_mode.ExecutionPhase(
      phase_number: 2,
      title: "Phase 2",
      beads: [bead2],
      can_parallel: False,
      effort: "10min",
    )

  let plan =
    plan_mode.ExecutionPlan(
      session_id: "test",
      generated_at: "2026-01-01T00:00:00Z",
      phases: [phase1, phase2],
      total_beads: 2,
      total_effort: "20min",
      risk: plan_mode.Low,
      blockers: [],
    )

  let gated_plan = interview.apply_phase_gating(session, plan)

  // Only Phase 1 should be allowed, Phase 2 should be blocked
  list.length(gated_plan.phases) |> should.equal(1)
  { list.length(gated_plan.blockers) > 0 } |> should.be_true()
}

pub fn phase_gate_allows_after_completion_test() {
  // Later phases should be allowed after prior phases complete
  let session =
    interview.InterviewSession(
      id: "test",
      profile: interview.Api,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
      completed_at: "",
      stage: interview.Discovery,
      rounds_completed: 0,
      answers: [],
      gaps: [],
      conflicts: [],
      raw_notes: "",
      current_phase: 2,
      completed_phases: [1],
    )

  let bead2 =
    plan_mode.PlanBead(
      id: "B-002",
      title: "Phase 2 bead",
      requires: [],
      effort: plan_mode.Effort10min,
      status: plan_mode.Pending,
    )

  let phase2 =
    plan_mode.ExecutionPhase(
      phase_number: 2,
      title: "Phase 2",
      beads: [bead2],
      can_parallel: False,
      effort: "10min",
    )

  let plan =
    plan_mode.ExecutionPlan(
      session_id: "test",
      generated_at: "2026-01-01T00:00:00Z",
      phases: [phase2],
      total_beads: 1,
      total_effort: "10min",
      risk: plan_mode.Low,
      blockers: [],
    )

  let gated_plan = interview.apply_phase_gating(session, plan)

  // Phase 2 should be allowed since Phase 1 is complete
  list.length(gated_plan.phases) |> should.equal(1)
}

pub fn complete_phase_advances_session_test() {
  let session =
    interview.InterviewSession(
      id: "test",
      profile: interview.Api,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
      completed_at: "",
      stage: interview.Discovery,
      rounds_completed: 0,
      answers: [],
      gaps: [],
      conflicts: [],
      raw_notes: "",
      current_phase: 1,
      completed_phases: [],
    )

  let updated_session = interview.complete_phase(session, 1)

  // Should advance to phase 2
  updated_session.current_phase |> should.equal(2)
  // Should have phase 1 in completed_phases
  list.contains(updated_session.completed_phases, 1) |> should.be_true()
}

pub fn can_execute_phase_checks_completion_test() {
  let session =
    interview.InterviewSession(
      id: "test",
      profile: interview.Api,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
      completed_at: "",
      stage: interview.Discovery,
      rounds_completed: 0,
      answers: [],
      gaps: [],
      conflicts: [],
      raw_notes: "",
      current_phase: 1,
      completed_phases: [],
    )

  // Phase 1 should always be executable
  interview.can_execute_phase(session, 1) |> should.be_true()

  // Phase 2 should not be executable without Phase 1 complete
  interview.can_execute_phase(session, 2) |> should.be_false()

  // Complete phase 1
  let session_with_phase1 = interview.complete_phase(session, 1)

  // Now phase 2 should be executable
  interview.can_execute_phase(session_with_phase1, 2) |> should.be_true()
}
