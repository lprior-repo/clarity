import gleam/json
import intent/interview
import intent/interview_storage
import intent/plan_mode

/// The `plan-next` command - determine next action for session execution
pub fn plan_next_command(session_id: String) -> Result(String, String) {
  let jsonl_path = ".intent/session-" <> session_id <> ".cue"
  case interview_storage.get_session_from_jsonl(jsonl_path, session_id) {
    Error(err) -> Error("Session not found: " <> err)
    Ok(session) -> {
      case plan_mode.compute_plan(session_id) {
        Error(err) -> Error(plan_mode.format_error(err))
        Ok(plan) -> {
          let gated_plan = interview.apply_phase_gating(session, plan)

          case gated_plan.blockers {
            [blocker, ..] -> {
              let directive =
                json.object([
                  #("action", json.string("blocked")),
                  #("session_id", json.string(session_id)),
                  #("rationale", json.string("Execution blocked: " <> blocker)),
                  #(
                    "next_command",
                    json.string("intent plan-next " <> session_id),
                  ),
                ])
              Ok(json.to_string(directive))
            }
            [] -> {
              case gated_plan.phases {
                [] -> {
                  let directive =
                    json.object([
                      #("action", json.string("done")),
                      #("session_id", json.string(session_id)),
                      #("rationale", json.string("All phases complete")),
                    ])
                  Ok(json.to_string(directive))
                }
                [first_phase, ..] -> {
                  case first_phase.beads {
                    [] -> {
                      let directive =
                        json.object([
                          #("action", json.string("done")),
                          #("session_id", json.string(session_id)),
                          #("rationale", json.string("Phase complete")),
                        ])
                      Ok(json.to_string(directive))
                    }
                    [first_bead, ..] -> {
                      let directive =
                        json.object([
                          #("action", json.string("execute_bead")),
                          #("session_id", json.string(session_id)),
                          #(
                            "rationale",
                            json.string("Execute first bead in phase"),
                          ),
                          #("phase", json.int(first_phase.phase_number)),
                          #(
                            "bead",
                            json.object([
                              #("id", json.string(first_bead.id)),
                              #("title", json.string(first_bead.title)),
                              #(
                                "requires",
                                json.array(first_bead.requires, json.string),
                              ),
                              #(
                                "effort",
                                json.string(plan_mode.effort_to_label(
                                  first_bead.effort,
                                )),
                              ),
                              #(
                                "status",
                                json.string(plan_mode.bead_status_to_string(
                                  first_bead.status,
                                )),
                              ),
                            ]),
                          ),
                          #(
                            "claim_command",
                            json.string("intent execute " <> first_bead.id),
                          ),
                        ])
                      Ok(json.to_string(directive))
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
