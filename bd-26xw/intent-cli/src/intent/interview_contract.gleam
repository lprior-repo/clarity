import gleam/dynamic
import gleam/json
import gleam/result
import gleam/string

pub fn validate_ai_directive_json(payload: String) -> Result(Nil, String) {
  case json.decode(payload, directive_decoder) {
    Ok(_) -> Ok(Nil)
    Error(err) ->
      Error("Invalid interview directive JSON: " <> string.inspect(err))
  }
}

fn directive_decoder(
  value: dynamic.Dynamic,
) -> Result(Nil, dynamic.DecodeErrors) {
  use action <- result.try(dynamic.field("action", dynamic.string)(value))
  use _session <- result.try(dynamic.field("session", session_decoder)(value))
  use _progress <- result.try(dynamic.field("progress", progress_decoder)(value))
  use _protocol <- result.try(dynamic.field(
    "agent_protocol",
    agent_protocol_decoder,
  )(value))
  use _guidance <- result.try(dynamic.field("guidance", guidance_decoder)(value))

  case action {
    "ask_question" -> {
      use _question <- result.try(dynamic.field("question", question_decoder)(
        value,
      ))
      Ok(Nil)
    }
    "generate_beads" -> Ok(Nil)
    _ ->
      Error([
        dynamic.DecodeError("action", "'ask_question' or 'generate_beads'", []),
      ])
  }
}

fn session_decoder(value: dynamic.Dynamic) -> Result(Nil, dynamic.DecodeErrors) {
  use _ <- result.try(dynamic.field("id", dynamic.string)(value))
  use _ <- result.try(dynamic.field("profile", dynamic.string)(value))
  use _ <- result.try(dynamic.field("created_at", dynamic.string)(value))
  use _ <- result.try(dynamic.field("updated_at", dynamic.string)(value))
  use _ <- result.try(dynamic.field("stage", dynamic.string)(value))
  Ok(Nil)
}

fn progress_decoder(value: dynamic.Dynamic) -> Result(Nil, dynamic.DecodeErrors) {
  use _ <- result.try(dynamic.field("current_round", dynamic.int)(value))
  use _ <- result.try(dynamic.field("total_rounds", dynamic.int)(value))
  use _ <- result.try(dynamic.field("questions_asked", dynamic.int)(value))
  use _ <- result.try(dynamic.field("questions_remaining", dynamic.int)(value))
  use _ <- result.try(dynamic.field("percent_complete", dynamic.int)(value))
  Ok(Nil)
}

fn agent_protocol_decoder(
  value: dynamic.Dynamic,
) -> Result(Nil, dynamic.DecodeErrors) {
  use target <- result.try(dynamic.field("target", dynamic.string)(value))
  use _ <- result.try(dynamic.field("contract_version", dynamic.string)(value))
  use _ <- result.try(dynamic.field("goal", dynamic.string)(value))

  case target {
    "claude_code" -> Ok(Nil)
    _ ->
      Error([dynamic.DecodeError("agent_protocol.target", "'claude_code'", [])])
  }
}

fn guidance_decoder(value: dynamic.Dynamic) -> Result(Nil, dynamic.DecodeErrors) {
  use _ <- result.try(dynamic.field("next_command", dynamic.string)(value))
  use _ <- result.try(dynamic.field("planning_focus", dynamic.string)(value))
  Ok(Nil)
}

fn question_decoder(value: dynamic.Dynamic) -> Result(Nil, dynamic.DecodeErrors) {
  use _ <- result.try(dynamic.field("id", dynamic.string)(value))
  use _ <- result.try(dynamic.field("round", dynamic.int)(value))
  use _ <- result.try(dynamic.field("text", dynamic.string)(value))
  use pattern <- result.try(dynamic.field("pattern", dynamic.string)(value))
  use _ <- result.try(dynamic.field("context", dynamic.string)(value))
  use _ <- result.try(dynamic.field("examples", dynamic.list(dynamic.string))(
    value,
  ))
  use _ <- result.try(dynamic.field("priority", dynamic.string)(value))
  use _ <- result.try(dynamic.field("perspective", dynamic.string)(value))
  use _ <- result.try(dynamic.field(
    "extract_into",
    dynamic.list(dynamic.string),
  )(value))

  case pattern {
    "ubiquitous"
    | "event_driven"
    | "state_driven"
    | "optional"
    | "unwanted"
    | "complex" -> Ok(Nil)
    _ ->
      Error([
        dynamic.DecodeError("question.pattern", "valid EARS pattern type", []),
      ])
  }
}
