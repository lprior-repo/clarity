/// Parser for Intent specs from JSON (exported from CUE)
/// v3.0 - Declarative format
import gleam/bit_array
import gleam/dict.{type Dict}
import gleam/dynamic.{type DecodeError, type Dynamic}
import gleam/json.{type Json}
import gleam/list
import gleam/result
import gleam/string
import intent/types.{
  type AIHints, type AntiPattern, type Behavior, type EntityHint, type Feature,
  type ImplementationHints, type Invariant, type SecurityHints, type Spec,
  type Verification, AIHints, AntiPattern, Behavior, EntityHint, Feature,
  ImplementationHints, Invariant, SecurityHints, Spec, Verification,
}

/// Parse a spec from a JSON value
/// v3.0 format - All fields are required
pub fn parse_spec(data: Dynamic) -> Result(Spec, List(DecodeError)) {
  use name <- result.try(dynamic.field("name", parse_non_empty_string)(data))
  use description <- result.try(dynamic.field(
    "description",
    parse_sanitized_string,
  )(data))
  use audience <- result.try(dynamic.field("audience", parse_sanitized_string)(
    data,
  ))
  use version <- result.try(dynamic.field("version", parse_sanitized_string)(
    data,
  ))
  use success_criteria <- result.try(dynamic.field(
    "success_criteria",
    dynamic.list(dynamic.string),
  )(data))
  use features <- result.try(dynamic.field(
    "features",
    dynamic.list(parse_feature),
  )(data))
  use invariants <- result.try(dynamic.field(
    "invariants",
    dynamic.list(parse_invariant),
  )(data))
  use anti_patterns <- result.try(dynamic.field(
    "anti_patterns",
    dynamic.list(parse_anti_pattern),
  )(data))
  use ai_hints <- result.try(dynamic.field("ai_hints", parse_ai_hints)(data))

  Ok(Spec(
    name: name,
    description: description,
    audience: audience,
    version: version,
    success_criteria: success_criteria,
    features: features,
    invariants: invariants,
    anti_patterns: anti_patterns,
    ai_hints: ai_hints,
  ))
}

fn sanitize_string(s: String) -> String {
  let null_byte = <<0>> |> bit_array.to_string |> result.unwrap("")
  string.replace(s, null_byte, "")
}

fn parse_sanitized_string(data: Dynamic) -> Result(String, List(DecodeError)) {
  dynamic.string(data)
  |> result.map(sanitize_string)
}

fn parse_non_empty_string(data: Dynamic) -> Result(String, List(DecodeError)) {
  parse_sanitized_string(data)
  |> result.then(fn(s) {
    case string.is_empty(s) {
      True ->
        Error([
          dynamic.DecodeError(expected: "non-empty string", found: s, path: []),
        ])
      False -> Ok(s)
    }
  })
}

fn parse_feature(data: Dynamic) -> Result(Feature, List(DecodeError)) {
  use name <- result.try(dynamic.field("name", parse_non_empty_string)(data))
  use description <- result.try(dynamic.field(
    "description",
    parse_sanitized_string,
  )(data))
  use behaviors <- result.try(dynamic.field(
    "behaviors",
    dynamic.list(parse_behavior),
  )(data))
  Ok(Feature(name, description, behaviors))
}

fn parse_behavior(data: Dynamic) -> Result(Behavior, List(DecodeError)) {
  use name <- result.try(dynamic.field("name", parse_non_empty_string)(data))
  use intent <- result.try(dynamic.field("intent", parse_sanitized_string)(data))
  // Notes is optional - default to empty string
  let notes =
    dynamic.field("notes", parse_sanitized_string)(data)
    |> result.unwrap("")
  // Requires is optional - default to empty list
  let requires =
    dynamic.field("requires", dynamic.list(dynamic.string))(data)
    |> result.unwrap([])
  // Tags is optional - default to empty list
  let tags =
    dynamic.field("tags", dynamic.list(dynamic.string))(data)
    |> result.unwrap([])
  // Preconditions is optional - default to empty list
  let preconditions =
    dynamic.field("preconditions", dynamic.list(dynamic.string))(data)
    |> result.unwrap([])
  // Postconditions is optional - default to empty list
  let postconditions =
    dynamic.field("postconditions", dynamic.list(dynamic.string))(data)
    |> result.unwrap([])
  // Verifications is optional - default to empty list
  let verifications =
    dynamic.field("verifications", dynamic.list(parse_verification))(data)
    |> result.unwrap([])

  Ok(Behavior(
    name: name,
    intent: intent,
    notes: notes,
    requires: requires,
    tags: tags,
    preconditions: preconditions,
    postconditions: postconditions,
    verifications: verifications,
  ))
}

fn parse_invariant(data: Dynamic) -> Result(Invariant, List(DecodeError)) {
  use name <- result.try(dynamic.field("name", parse_non_empty_string)(data))
  use description <- result.try(dynamic.field(
    "description",
    parse_sanitized_string,
  )(data))
  use criteria <- result.try(dynamic.field(
    "criteria",
    dynamic.list(dynamic.string),
  )(data))
  Ok(Invariant(name: name, description: description, criteria: criteria))
}

fn parse_verification(data: Dynamic) -> Result(Verification, List(DecodeError)) {
  use description <- result.try(dynamic.field(
    "description",
    parse_sanitized_string,
  )(data))
  use criteria <- result.try(dynamic.field(
    "criteria",
    dynamic.list(dynamic.string),
  )(data))
  // Examples is optional - default to empty list
  let examples =
    dynamic.field("examples", dynamic.list(parse_json_value))(data)
    |> result.unwrap([])

  Ok(Verification(
    description: description,
    criteria: criteria,
    examples: examples,
  ))
}

fn parse_json_dict(
  data: Dynamic,
) -> Result(Dict(String, Json), List(DecodeError)) {
  data
  |> dynamic.dict(dynamic.string, dynamic.dynamic)
  |> result.map(fn(d) { dict.map_values(d, fn(_, v) { dynamic_to_json(v) }) })
}

fn parse_json_value(data: Dynamic) -> Result(Json, List(DecodeError)) {
  Ok(dynamic_to_json(data))
}

/// Convert a Dynamic value to Json
pub fn dynamic_to_json(data: Dynamic) -> Json {
  case dynamic.classify(data) {
    "Nil" -> json.null()
    "Bool" ->
      case dynamic.bool(data) {
        Ok(b) -> json.bool(b)
        Error(_) -> json.null()
      }
    "Int" ->
      case dynamic.int(data) {
        Ok(i) -> json.int(i)
        Error(_) -> json.null()
      }
    "Float" ->
      case dynamic.float(data) {
        Ok(f) -> json.float(f)
        Error(_) -> json.null()
      }
    "String" | "BitArray" ->
      case parse_sanitized_string(data) {
        Ok(s) -> json.string(s)
        Error(_) -> json.null()
      }
    "List" | "Tuple" ->
      case dynamic.list(dynamic.dynamic)(data) {
        Ok(items) -> json.array(items, dynamic_to_json)
        Error(_) -> json.null()
      }
    "Dict" | "Map" ->
      case dynamic.dict(dynamic.string, dynamic.dynamic)(data) {
        Ok(d) ->
          d
          |> dict.to_list
          |> list.map(fn(pair) { #(pair.0, dynamic_to_json(pair.1)) })
          |> json.object
        Error(_) -> json.null()
      }
    _ -> json.null()
  }
}

fn parse_anti_pattern(data: Dynamic) -> Result(AntiPattern, List(DecodeError)) {
  use name <- result.try(dynamic.field("name", parse_non_empty_string)(data))
  use description <- result.try(dynamic.field(
    "description",
    parse_sanitized_string,
  )(data))
  use bad_example <- result.try(dynamic.field("bad_example", parse_json_value)(
    data,
  ))
  use good_example <- result.try(dynamic.field("good_example", parse_json_value)(
    data,
  ))
  // why is optional - default to empty string
  let why =
    dynamic.field("why", parse_sanitized_string)(data)
    |> result.unwrap("")

  Ok(AntiPattern(
    name: name,
    description: description,
    bad_example: bad_example,
    good_example: good_example,
    why: why,
  ))
}

fn parse_ai_hints(data: Dynamic) -> Result(AIHints, List(DecodeError)) {
  use implementation <- result.try(dynamic.field(
    "implementation",
    parse_implementation_hints,
  )(data))
  use entities <- result.try(dynamic.field("entities", parse_entities)(data))
  let security =
    dynamic.field("security", parse_security_hints)(data)
    |> result.unwrap(SecurityHints("", "", "", ""))
  use pitfalls <- result.try(dynamic.field(
    "pitfalls",
    dynamic.list(parse_sanitized_string),
  )(data))
  Ok(AIHints(implementation, entities, security, pitfalls))
}

fn parse_implementation_hints(
  data: Dynamic,
) -> Result(ImplementationHints, List(DecodeError)) {
  use suggested_stack <- result.try(dynamic.field(
    "suggested_stack",
    dynamic.list(parse_sanitized_string),
  )(data))
  let architecture =
    dynamic.field("architecture", parse_sanitized_string)(data)
    |> result.unwrap("")
  use key_components <- result.try(dynamic.field(
    "key_components",
    dynamic.list(parse_sanitized_string),
  )(data))
  Ok(ImplementationHints(suggested_stack, architecture, key_components))
}

fn parse_entities(
  data: Dynamic,
) -> Result(Dict(String, EntityHint), List(DecodeError)) {
  dynamic.dict(dynamic.string, parse_entity_hint)(data)
}

fn parse_entity_hint(data: Dynamic) -> Result(EntityHint, List(DecodeError)) {
  use fields <- result.try(dynamic.field("fields", parse_json_dict)(data))
  Ok(EntityHint(fields))
}

fn parse_security_hints(
  data: Dynamic,
) -> Result(SecurityHints, List(DecodeError)) {
  use password_hashing <- result.try(dynamic.field(
    "password_hashing",
    parse_sanitized_string,
  )(data))
  use jwt_algorithm <- result.try(dynamic.field(
    "jwt_algorithm",
    parse_sanitized_string,
  )(data))
  use jwt_expiry <- result.try(dynamic.field(
    "jwt_expiry",
    parse_sanitized_string,
  )(data))
  use rate_limiting <- result.try(dynamic.field(
    "rate_limiting",
    parse_sanitized_string,
  )(data))
  Ok(SecurityHints(password_hashing, jwt_algorithm, jwt_expiry, rate_limiting))
}

/// Decode a spec from a dynamic JSON value (for use with shellout cue export)
pub fn decode_dynamic(
  data: dynamic.Dynamic,
) -> Result(Spec, List(dynamic.DecodeError)) {
  parse_spec(data)
}
