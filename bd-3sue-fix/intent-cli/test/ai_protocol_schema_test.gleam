//// Simple JSON schema checks for AI-facing command outputs

import gleam/json
import gleam/string
import gleeunit
import gleeunit/should
import intent/plan_mode

pub fn main() {
  gleeunit.main()
}

fn sample_plan() -> plan_mode.ExecutionPlan {
  let bead =
    plan_mode.PlanBead(
      id: "PLAN-001",
      title: "Schema test bead",
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
  plan_mode.ExecutionPlan(
    session_id: "plan-session",
    generated_at: "2026-01-01T00:00:00Z",
    phases: [phase],
    total_beads: 1,
    total_effort: "10min",
    risk: plan_mode.High,
    blockers: ["Missing dependency"],
  )
}

pub fn plan_work_directive_has_required_fields_test() {
  let plan = sample_plan()
  let output = plan_mode.format_plan_ai(plan)

  output |> string.contains("action") |> should.be_true()
  output |> string.contains("contract_version") |> should.be_true()
  output |> string.contains("session") |> should.be_true()
  output |> string.contains("handoff") |> should.be_true()
  case json.decode(output, fn(_) { Ok(Nil) }) {
    Ok(_) -> True |> should.be_true()
    Error(_) -> should.fail()
  }
}

pub fn plan_next_directive_has_required_fields_test() {
  let directive =
    json.object([
      #("action", json.string("execute_bead")),
      #("session_id", json.string("test-session")),
      #("bead", json.object([#("id", json.string("PLAN-001"))])),
    ])
  let output = json.to_string(directive)

  output |> string.contains("action") |> should.be_true()
  output |> string.contains("bead") |> should.be_true()
  case json.decode(output, fn(_) { Ok(Nil) }) {
    Ok(_) -> True |> should.be_true()
    Error(_) -> should.fail()
  }
}

pub fn plan_emit_beads_directive_has_required_fields_test() {
  let directive =
    json.object([
      #("session_id", json.string("test-session")),
      #("target", json.string("br")),
      #("dry_run", json.bool(True)),
    ])
  let output = json.to_string(directive)

  output |> string.contains("target") |> should.be_true()
  output |> string.contains("dry_run") |> should.be_true()
  case json.decode(output, fn(_) { Ok(Nil) }) {
    Ok(_) -> True |> should.be_true()
    Error(_) -> should.fail()
  }
}
