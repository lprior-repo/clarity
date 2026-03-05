import gleam/string
import gleeunit/should
import intent/plan_mode

pub fn decode_beads_json_basic_test() {
  let json =
    "[{\"id\":\"USR-001\",\"title\":\"Create user\",\"requires\":[\"AUTH-001\"],\"effort\":\"20min\",\"status\":\"blocked\"}]"

  case plan_mode.decode_beads_json(json) {
    Ok([bead]) -> {
      bead.id |> should.equal("USR-001")
      bead.title |> should.equal("Create user")
      bead.requires |> should.equal(["AUTH-001"])
      bead.effort |> should.equal(plan_mode.Effort20min)
      bead.status |> should.equal(plan_mode.Blocked)
    }
    _ -> should.fail()
  }
}

pub fn decode_beads_json_alias_fields_test() {
  let json =
    "[{\"bead_id\":\"PAY-002\",\"summary\":\"Process payment\",\"dependencies\":[\"AUTH-001\",\"DB-001\"],\"effort_minutes\":7,\"state\":\"in_progress\"}]"

  case plan_mode.decode_beads_json(json) {
    Ok([bead]) -> {
      bead.id |> should.equal("PAY-002")
      bead.title |> should.equal("Process payment")
      bead.requires |> should.equal(["AUTH-001", "DB-001"])
      bead.effort |> should.equal(plan_mode.Effort5min)
      bead.status |> should.equal(plan_mode.InProgress)
    }
    _ -> should.fail()
  }
}

pub fn format_plan_ai_includes_plan_metadata_test() {
  let bead =
    plan_mode.PlanBead(
      id: "USR-001",
      title: "Create user",
      requires: [],
      effort: plan_mode.Effort20min,
      status: plan_mode.Pending,
    )

  let phase =
    plan_mode.ExecutionPhase(
      phase_number: 1,
      title: "Phase 1",
      beads: [bead],
      can_parallel: False,
      effort: "20min",
    )

  let plan =
    plan_mode.ExecutionPlan(
      session_id: "session-123",
      generated_at: "2026-01-01T00:00:00Z",
      phases: [phase],
      total_beads: 1,
      total_effort: "20min",
      risk: plan_mode.Low,
      blockers: [],
    )

  let output = plan_mode.format_plan_ai(plan)

  // AI format includes plan metadata and phases with beads
  string.contains(output, "session_id") |> should.be_true()
  string.contains(output, "session-123") |> should.be_true()
  string.contains(output, "phases") |> should.be_true()
  string.contains(output, "USR-001") |> should.be_true()
}
