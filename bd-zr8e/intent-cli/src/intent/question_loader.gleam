/// Question Loader
/// Loads interview questions from CUE files at runtime
/// Supports custom questions from .intent/custom-questions.cue
import gleam/dynamic.{type Dynamic}
import gleam/json
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/string
import intent/question_types.{
  type Perspective, type Question, type QuestionCategory, type QuestionPriority,
  Business, Constraint, Critical, Dependency, Developer, EdgeCase, ErrorCase,
  HappyPath, Important, NiceTohave, NonFunctional, Ops, Question, Security, User,
}
import intent/security
import shellout

/// Error types for question loading
pub type QuestionLoadError {
  FileNotFound(path: String)
  CueExportError(message: String)
  JsonParseError(message: String)
  QuestionParseError(message: String)
  SecurityError(message: String)
}

/// Loaded questions database
pub type QuestionsDatabase {
  QuestionsDatabase(
    api: ProfileQuestions,
    cli: ProfileQuestions,
    event: ProfileQuestions,
    data: ProfileQuestions,
    workflow: ProfileQuestions,
    ui: ProfileQuestions,
    common: CommonQuestions,
  )
}

pub type ProfileQuestions {
  ProfileQuestions(round_1: List(Question), round_2: List(Question))
}

pub type CommonQuestions {
  CommonQuestions(
    round_3: List(Question),
    round_4: List(Question),
    round_5: List(Question),
  )
}

/// Custom questions - optional overrides/additions
pub type CustomQuestions {
  CustomQuestions(
    api: Option(CustomProfileQuestions),
    cli: Option(CustomProfileQuestions),
    event: Option(CustomProfileQuestions),
    data: Option(CustomProfileQuestions),
    workflow: Option(CustomProfileQuestions),
    ui: Option(CustomProfileQuestions),
    common: Option(CustomCommonQuestions),
  )
}

pub type CustomProfileQuestions {
  CustomProfileQuestions(
    round_1: Option(List(Question)),
    round_2: Option(List(Question)),
  )
}

pub type CustomCommonQuestions {
  CustomCommonQuestions(
    round_3: Option(List(Question)),
    round_4: Option(List(Question)),
    round_5: Option(List(Question)),
  )
}

/// Default path for custom questions
const custom_questions_path = ".intent/custom-questions.cue"

/// Load questions from a CUE file
pub fn load_questions(
  path: String,
) -> Result(QuestionsDatabase, QuestionLoadError) {
  case string.is_empty(path) {
    True -> Error(FileNotFound(path))
    False ->
      case security.validate_file_path(path) {
        Ok(validated_path) -> export_and_parse(validated_path)
        Error(security_error) -> Error(map_security_error(security_error))
      }
  }
}

/// Load questions from the default schema path, merging with custom questions
pub fn load_default_questions() -> Result(QuestionsDatabase, QuestionLoadError) {
  // Load built-in questions first
  case load_questions("schema/questions.cue") {
    Ok(db) -> {
      // Try to load custom questions and merge them
      case load_custom_questions(custom_questions_path) {
        Ok(custom) -> Ok(merge_custom_questions(db, custom))
        Error(_) -> Ok(db)
        // No custom questions or error loading - use defaults
      }
    }
    Error(e) -> Error(e)
  }
}

/// Load custom questions from a path
pub fn load_custom_questions(
  path: String,
) -> Result(CustomQuestions, QuestionLoadError) {
  case string.is_empty(path) {
    True -> Error(FileNotFound(path))
    False ->
      case security.validate_file_path(path) {
        Ok(validated_path) -> export_and_parse_custom(validated_path)
        Error(security_error) -> Error(map_security_error(security_error))
      }
  }
}

fn export_and_parse_custom(
  path: String,
) -> Result(CustomQuestions, QuestionLoadError) {
  case
    shellout.command("cue", ["export", path, "-e", "custom_questions"], ".", [])
  {
    Ok(json_str) -> parse_custom_questions_json(json_str)
    Error(#(_, stderr)) -> Error(CueExportError(stderr))
  }
}

fn parse_custom_questions_json(
  json_str: String,
) -> Result(CustomQuestions, QuestionLoadError) {
  case json.decode(json_str, dynamic.dynamic) {
    Ok(data) -> parse_custom_database(data)
    Error(_) -> Error(JsonParseError("Failed to decode custom questions JSON"))
  }
}

fn map_security_error(error: security.SecurityError) -> QuestionLoadError {
  case error {
    security.FileNotAccessible(path) -> FileNotFound(path)
    security.InvalidPath(path, _) -> FileNotFound(path)
    security.ShellMetacharactersDetected(path) -> FileNotFound(path)
    _ -> SecurityError(security.format_security_error(error))
  }
}

fn parse_custom_database(
  data: Dynamic,
) -> Result(CustomQuestions, QuestionLoadError) {
  let decoder =
    dynamic.decode7(
      CustomQuestions,
      dynamic.optional_field("api", parse_custom_profile_questions),
      dynamic.optional_field("cli", parse_custom_profile_questions),
      dynamic.optional_field("event", parse_custom_profile_questions),
      dynamic.optional_field("data", parse_custom_profile_questions),
      dynamic.optional_field("workflow", parse_custom_profile_questions),
      dynamic.optional_field("ui", parse_custom_profile_questions),
      dynamic.optional_field("common", parse_custom_common_questions),
    )

  case decoder(data) {
    Ok(custom) -> Ok(custom)
    Error(errs) ->
      Error(QuestionParseError(
        "Failed to parse custom questions: " <> format_decode_errors(errs),
      ))
  }
}

fn parse_custom_profile_questions(
  data: Dynamic,
) -> Result(CustomProfileQuestions, List(dynamic.DecodeError)) {
  dynamic.decode2(
    CustomProfileQuestions,
    dynamic.optional_field("round_1", dynamic.list(parse_question)),
    dynamic.optional_field("round_2", dynamic.list(parse_question)),
  )(data)
}

fn parse_custom_common_questions(
  data: Dynamic,
) -> Result(CustomCommonQuestions, List(dynamic.DecodeError)) {
  dynamic.decode3(
    CustomCommonQuestions,
    dynamic.optional_field("round_3", dynamic.list(parse_question)),
    dynamic.optional_field("round_4", dynamic.list(parse_question)),
    dynamic.optional_field("round_5", dynamic.list(parse_question)),
  )(data)
}

/// Merge custom questions with built-in questions
/// Custom questions with same ID override built-ins; new IDs are added
fn merge_custom_questions(
  db: QuestionsDatabase,
  custom: CustomQuestions,
) -> QuestionsDatabase {
  QuestionsDatabase(
    api: merge_profile(db.api, custom.api),
    cli: merge_profile(db.cli, custom.cli),
    event: merge_profile(db.event, custom.event),
    data: merge_profile(db.data, custom.data),
    workflow: merge_profile(db.workflow, custom.workflow),
    ui: merge_profile(db.ui, custom.ui),
    common: merge_common(db.common, custom.common),
  )
}

fn merge_profile(
  base: ProfileQuestions,
  custom: Option(CustomProfileQuestions),
) -> ProfileQuestions {
  case custom {
    None -> base
    Some(c) ->
      ProfileQuestions(
        round_1: merge_question_list(base.round_1, c.round_1),
        round_2: merge_question_list(base.round_2, c.round_2),
      )
  }
}

fn merge_common(
  base: CommonQuestions,
  custom: Option(CustomCommonQuestions),
) -> CommonQuestions {
  case custom {
    None -> base
    Some(c) ->
      CommonQuestions(
        round_3: merge_question_list(base.round_3, c.round_3),
        round_4: merge_question_list(base.round_4, c.round_4),
        round_5: merge_question_list(base.round_5, c.round_5),
      )
  }
}

fn merge_question_list(
  base: List(Question),
  custom: Option(List(Question)),
) -> List(Question) {
  case custom {
    None -> base
    Some(custom_questions) -> {
      // Get IDs of custom questions for override detection
      let custom_ids = list.map(custom_questions, fn(q) { q.id })

      // Keep base questions that aren't overridden
      let filtered_base =
        list.filter(base, fn(q) { !list.contains(custom_ids, q.id) })

      // Append custom questions (overrides + new)
      list.append(filtered_base, custom_questions)
    }
  }
}

fn export_and_parse(
  path: String,
) -> Result(QuestionsDatabase, QuestionLoadError) {
  case shellout.command("cue", ["export", path, "-e", "questions"], ".", []) {
    Ok(json_str) -> parse_questions_json(json_str)
    Error(#(_, stderr)) -> Error(CueExportError(stderr))
  }
}

fn parse_questions_json(
  json_str: String,
) -> Result(QuestionsDatabase, QuestionLoadError) {
  case json.decode(json_str, dynamic.dynamic) {
    Ok(data) -> parse_database(data)
    Error(_) -> Error(JsonParseError("Failed to decode JSON"))
  }
}

fn parse_database(data: Dynamic) -> Result(QuestionsDatabase, QuestionLoadError) {
  let decoder =
    dynamic.decode7(
      QuestionsDatabase,
      dynamic.field("api", parse_profile_questions),
      dynamic.field("cli", parse_profile_questions),
      dynamic.field("event", parse_profile_questions),
      dynamic.field("data", parse_profile_questions),
      dynamic.field("workflow", parse_profile_questions),
      dynamic.field("ui", parse_profile_questions),
      dynamic.field("common", parse_common_questions),
    )

  case decoder(data) {
    Ok(db) -> Ok(db)
    Error(errs) ->
      Error(QuestionParseError(
        "Failed to parse questions: " <> format_decode_errors(errs),
      ))
  }
}

fn parse_profile_questions(
  data: Dynamic,
) -> Result(ProfileQuestions, List(dynamic.DecodeError)) {
  dynamic.decode2(
    ProfileQuestions,
    dynamic.field("round_1", dynamic.list(parse_question)),
    dynamic.field("round_2", dynamic.list(parse_question)),
  )(data)
}

fn parse_common_questions(
  data: Dynamic,
) -> Result(CommonQuestions, List(dynamic.DecodeError)) {
  dynamic.decode3(
    CommonQuestions,
    dynamic.field("round_3", dynamic.list(parse_question)),
    dynamic.field("round_4", dynamic.list(parse_question)),
    dynamic.field("round_5", dynamic.list(parse_question)),
  )(data)
}

fn parse_question(data: Dynamic) -> Result(Question, List(dynamic.DecodeError)) {
  // Use decode8 + additional fields
  let base_decoder =
    dynamic.decode8(
      fn(id, round, perspective, category, priority, question, context, example) {
        #(
          id,
          round,
          perspective,
          category,
          priority,
          question,
          context,
          example,
        )
      },
      dynamic.field("id", dynamic.string),
      dynamic.field("round", dynamic.int),
      dynamic.field("perspective", dynamic.string),
      dynamic.field("category", dynamic.string),
      dynamic.field("priority", dynamic.string),
      dynamic.field("question", dynamic.string),
      dynamic.field("context", dynamic.string),
      dynamic.field("example", dynamic.string),
    )

  let extra_decoder =
    dynamic.decode4(
      fn(expected_type, extract_into, depends_on, blocks) {
        #(expected_type, extract_into, depends_on, blocks)
      },
      dynamic.optional_field("expected_type", dynamic.string),
      dynamic.optional_field("extract_into", dynamic.list(dynamic.string)),
      dynamic.optional_field("depends_on", dynamic.list(dynamic.string)),
      dynamic.optional_field("blocks", dynamic.list(dynamic.string)),
    )

  case base_decoder(data), extra_decoder(data) {
    Ok(#(
      id,
      round,
      perspective_str,
      category_str,
      priority_str,
      question,
      context,
      example,
    )),
      Ok(#(expected_type_opt, extract_into_opt, depends_on_opt, blocks_opt))
    -> {
      let perspective = parse_perspective(perspective_str)
      let category = parse_category(category_str)
      let priority = parse_priority(priority_str)

      Ok(Question(
        id: id,
        round: round,
        perspective: perspective,
        category: category,
        priority: priority,
        question: question,
        context: context,
        example: example,
        expected_type: option.unwrap(expected_type_opt, "text"),
        extract_into: option.unwrap(extract_into_opt, []),
        depends_on: option.unwrap(depends_on_opt, []),
        blocks: option.unwrap(blocks_opt, []),
      ))
    }
    Error(errs), _ -> Error(errs)
    _, Error(errs) -> Error(errs)
  }
}

fn parse_perspective(s: String) -> Perspective {
  case string.lowercase(s) {
    "user" -> User
    "developer" -> Developer
    "ops" -> Ops
    "security" -> Security
    "business" -> Business
    _ -> User
  }
}

fn parse_category(s: String) -> QuestionCategory {
  case string.lowercase(s) {
    "happy_path" -> HappyPath
    "error_case" -> ErrorCase
    "edge_case" -> EdgeCase
    "constraint" -> Constraint
    "dependency" -> Dependency
    "nonfunctional" -> NonFunctional
    _ -> HappyPath
  }
}

fn parse_priority(s: String) -> QuestionPriority {
  case string.lowercase(s) {
    "critical" -> Critical
    "important" -> Important
    "nice_to_have" -> NiceTohave
    _ -> Important
  }
}

fn format_decode_errors(errors: List(dynamic.DecodeError)) -> String {
  errors
  |> list.map(fn(e) {
    "Expected " <> e.expected <> " at " <> string.join(e.path, ".")
  })
  |> string.join(", ")
}

/// Get questions for a specific profile and round from a loaded database
pub fn get_questions(
  db: QuestionsDatabase,
  profile: String,
  round: Int,
) -> List(Question) {
  case profile, round {
    "api", 1 -> db.api.round_1
    "api", 2 -> db.api.round_2
    "cli", 1 -> db.cli.round_1
    "cli", 2 -> db.cli.round_2
    "event", 1 -> db.event.round_1
    "event", 2 -> db.event.round_2
    "data", 1 -> db.data.round_1
    "data", 2 -> db.data.round_2
    "workflow", 1 -> db.workflow.round_1
    "workflow", 2 -> db.workflow.round_2
    "ui", 1 -> db.ui.round_1
    "ui", 2 -> db.ui.round_2
    _, 3 -> db.common.round_3
    _, 4 -> db.common.round_4
    _, 5 -> db.common.round_5
    _, _ -> []
  }
}

/// Format a QuestionLoadError as a human-readable string
pub fn format_error(error: QuestionLoadError) -> String {
  case error {
    FileNotFound(path) -> "Questions file not found: " <> path
    CueExportError(msg) -> "CUE export failed:\n" <> msg
    JsonParseError(msg) -> "JSON parse error: " <> msg
    QuestionParseError(msg) -> "Question parse error: " <> msg
    SecurityError(msg) -> msg
  }
}
