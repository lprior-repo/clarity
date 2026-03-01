/// Interview Engine
/// Structured interrogation system for discovering and refining specifications
/// Supports 5 rounds × multiple perspectives = comprehensive requirement capture
import gleam/dict.{type Dict}
import gleam/int
import gleam/list
import gleam/option
import gleam/string
import intent/interview_questions
import intent/plan_mode
import intent/question_types.{
  type Perspective, type Question, Critical, Developer, Important, NiceTohave,
  Ops,
}

/// Profile types - determines which questions to ask
pub type Profile {
  Api
  Cli
  Event
  Data
  Workflow
  UI
}

/// Interview stages - persistent state machine
pub type InterviewStage {
  Discovery
  Refinement
  Validation
  Complete
  Paused
}

/// A single answer with metadata
pub type Answer {
  Answer(
    question_id: String,
    question_text: String,
    perspective: Perspective,
    round: Int,
    response: String,
    extracted: Dict(String, String),
    confidence: Float,
    notes: String,
    timestamp: String,
  )
}

/// Gap - missing information blocking spec completion
pub type Gap {
  Gap(
    id: String,
    field: String,
    description: String,
    blocking: Bool,
    suggested_default: String,
    why_needed: String,
    round: Int,
    resolved: Bool,
    resolution: String,
  )
}

/// Conflict - contradictions in requirements
pub type Conflict {
  Conflict(
    id: String,
    between: #(String, String),
    description: String,
    impact: String,
    options: List(ConflictResolution),
    chosen: Int,
  )
}

pub type ConflictResolution {
  ConflictResolution(
    option: String,
    description: String,
    tradeoffs: String,
    recommendation: String,
  )
}

/// Interview session - persistent state machine
pub type InterviewSession {
  InterviewSession(
    id: String,
    profile: Profile,
    created_at: String,
    updated_at: String,
    completed_at: String,
    stage: InterviewStage,
    rounds_completed: Int,
    answers: List(Answer),
    gaps: List(Gap),
    conflicts: List(Conflict),
    raw_notes: String,
    current_phase: Int,
    completed_phases: List(Int),
  )
}

/// Extract field from answer text (AI-driven)
/// This is where the "AI adapts" - the extraction logic learns from patterns
pub fn extract_from_answer(
  _question_id: String,
  response: String,
  extract_fields: List(String),
) -> Dict(String, String) {
  list.fold(extract_fields, dict.new(), fn(acc, field) {
    let extracted_value = simple_extract(field, response)
    case extracted_value {
      Ok(value) -> dict.insert(acc, field, value)
      Error(_) -> acc
    }
  })
}

/// Simple extraction patterns - can be extended to use NLP/LLM
fn simple_extract(field: String, text: String) -> Result(String, String) {
  case field {
    // Auth-related extractions
    "auth_method" -> extract_auth_method(text)

    // Entity/data model extractions
    "entities" -> extract_entities(text)

    // Audience extraction
    "audience" -> extract_audience(text)

    // Generic field - just return the text if it's substantial
    _ -> {
      let trimmed = string.trim(text)
      case string.length(trimmed) > 0 {
        True -> Ok(trimmed)
        False -> Error("Empty response")
      }
    }
  }
}

fn extract_auth_method(text: String) -> Result(String, String) {
  let lower = string.lowercase(text)
  case string.contains(lower, "jwt") {
    True -> Ok("jwt")
    False ->
      case string.contains(lower, "oauth") {
        True -> Ok("oauth")
        False ->
          case string.contains(lower, "session") {
            True -> Ok("session")
            False ->
              case string.contains(lower, "api key") {
                True -> Ok("api_key")
                False ->
                  case string.contains(lower, "none") {
                    True -> Ok("none")
                    False -> Error("Could not identify auth method")
                  }
              }
          }
      }
  }
}

pub fn extract_entities(text: String) -> Result(String, String) {
  // Look for capitalized words (likely entity names)
  let words = string.split(text, " ")
  let entities =
    list.filter_map(words, fn(word) {
      // Remove trailing punctuation
      let clean_word = case string.ends_with(word, ",") {
        True -> string.slice(word, 0, string.length(word) - 1)
        False -> word
      }
      let first_char = string.slice(clean_word, 0, 1)
      case
        string.uppercase(first_char) == first_char
        && string.length(clean_word) > 2
      {
        True -> Ok(clean_word)
        False -> Error(Nil)
      }
    })
  case entities {
    [] -> Error("No entities found")
    _ -> Ok(string.join(entities, ", "))
  }
}

pub fn extract_audience(text: String) -> Result(String, String) {
  let lower = string.lowercase(text)
  case string.contains(lower, "mobile") {
    True -> Ok("mobile")
    False ->
      case string.contains(lower, "web") {
        True -> Ok("web")
        False ->
          case string.contains(lower, "api") {
            True -> Ok("api")
            False ->
              case string.contains(lower, "cli") {
                True -> Ok("cli")
                False ->
                  case string.contains(lower, "internal") {
                    True -> Ok("internal")
                    False -> Error("Could not identify audience")
                  }
              }
          }
      }
  }
}

/// Detect gaps from collected answers
pub fn detect_gaps(profile: Profile, answers: List(Answer)) -> List(Gap) {
  // Check critical fields for the profile
  let required_fields = case profile {
    Api -> [
      "base_url", "auth_method", "happy_path", "error_cases", "response_format",
    ]
    Cli -> ["command_name", "happy_path", "help_text", "exit_codes"]
    Event -> ["event_type", "payload_schema", "trigger"]
    Data -> ["data_model", "access_patterns", "retention"]
    Workflow -> ["steps", "happy_path", "error_recovery"]
    UI -> ["user_flows", "happy_path", "states"]
  }

  let answered_fields =
    list.fold(answers, dict.new(), fn(acc, answer) {
      list.fold(dict.to_list(answer.extracted), acc, fn(inner_acc, pair) {
        dict.insert(inner_acc, pair.0, pair.1)
      })
    })

  list.filter_map(required_fields, fn(field) {
    case dict.get(answered_fields, field) {
      Ok(_) -> Error(Nil)
      Error(_) ->
        Ok(Gap(
          id: "gap-" <> field,
          field: field,
          description: "Missing: " <> field,
          blocking: True,
          suggested_default: "",
          why_needed: "Required for " <> field,
          round: 0,
          resolved: False,
          resolution: "",
        ))
    }
  })
}

/// Detect conflicts between answers
pub fn detect_conflicts(answers: List(Answer)) -> List(Conflict) {
  // Example conflict patterns
  // "fast" + "strongly_consistent" = CAP theorem
  // "anonymous" + "audit_trail" = traceability
  // "simple" + 20_requirements = scope creep

  let lower_responses =
    list.map(answers, fn(ans) {
      #(ans.question_id, string.lowercase(ans.response))
    })

  let conflicts = []

  // Check for CAP theorem conflict
  let has_fast =
    list.any(lower_responses, fn(pair) {
      string.contains(pair.1, "fast") || string.contains(pair.1, "latency")
    })
  let has_consistent =
    list.any(lower_responses, fn(pair) {
      string.contains(pair.1, "consistent")
      || string.contains(pair.1, "accurate")
    })

  let conflicts = case has_fast && has_consistent {
    True ->
      list.append(conflicts, [
        Conflict(
          id: "conflict-cap",
          between: #("latency", "consistency"),
          description: "You want both speed AND strong consistency",
          impact: "CAP theorem: impossible to have both at scale",
          options: [
            ConflictResolution(
              option: "Prioritize latency",
              description: "Accept eventual consistency, cache aggressively",
              tradeoffs: "Data may be stale for a few seconds",
              recommendation: "Use when user experience matters more than instant accuracy",
            ),
            ConflictResolution(
              option: "Prioritize consistency",
              description: "Use strong consistency, accept higher latency",
              tradeoffs: "P99 latencies may be 100-500ms, not 10ms",
              recommendation: "Use for financial/banking systems",
            ),
          ],
          chosen: -1,
        ),
      ])
    False -> conflicts
  }

  // Check for anonymous + audit
  let has_anonymous =
    list.any(lower_responses, fn(pair) { string.contains(pair.1, "anonymous") })
  let has_audit =
    list.any(lower_responses, fn(pair) {
      string.contains(pair.1, "audit") || string.contains(pair.1, "log")
    })

  case has_anonymous && has_audit {
    True ->
      list.append(conflicts, [
        Conflict(
          id: "conflict-anon-audit",
          between: #("anonymous", "audit"),
          description: "You want anonymous users AND an audit trail",
          impact: "Can't audit completely anonymous users without some tracking",
          options: [
            ConflictResolution(
              option: "Use pseudonymous IDs",
              description: "Generate per-session IDs, don't link to user identity",
              tradeoffs: "Still traceable within a session",
              recommendation: "Most balanced approach",
            ),
            ConflictResolution(
              option: "Aggregate logging only",
              description: "Log statistics (actions per hour) not individual actions",
              tradeoffs: "Can't trace individual user behavior",
              recommendation: "For privacy-critical systems",
            ),
          ],
          chosen: -1,
        ),
      ])
    False -> conflicts
  }
}

/// Calculate confidence in answer extraction (0-1)
pub fn calculate_confidence(
  _question_id: String,
  response: String,
  extracted: Dict(String, String),
) -> Float {
  let response_length = string.length(string.trim(response))
  let field_count = dict.size(extracted)

  // Longer responses with more fields = higher confidence
  case response_length > 50 && field_count > 0 {
    True -> 0.85
    False -> 0.6
  }
}

/// Format a question for display
pub fn format_question(question: Question) -> String {
  let priority_str = case question.priority {
    Critical -> "[CRITICAL]"
    Important -> "[IMPORTANT]"
    NiceTohave -> ""
  }

  let context_str = case string.length(question.context) > 0 {
    True -> "\n" <> question.context
    False -> ""
  }

  let example_str = case string.length(question.example) > 0 {
    True -> "\nExample: " <> question.example
    False -> ""
  }

  priority_str <> " " <> question.question <> context_str <> example_str
}

/// Create a new session
pub fn create_session(
  id: String,
  profile: Profile,
  timestamp: String,
) -> InterviewSession {
  InterviewSession(
    id: id,
    profile: profile,
    created_at: timestamp,
    updated_at: timestamp,
    completed_at: "",
    stage: Discovery,
    rounds_completed: 0,
    answers: [],
    gaps: [],
    conflicts: [],
    raw_notes: "",
    current_phase: 1,
    completed_phases: [],
  )
}

/// Add answer to session
pub fn add_answer(session: InterviewSession, answer: Answer) -> InterviewSession {
  let new_answers = list.append(session.answers, [answer])
  InterviewSession(
    ..session,
    answers: new_answers,
    updated_at: answer.timestamp,
    current_phase: session.current_phase,
  )
}

// ============================================================================
// SESSION ORCHESTRATION
// ============================================================================

/// Get first question for a given round
pub fn get_first_question_for_round(
  session: InterviewSession,
  round: Int,
) -> Result(Question, String) {
  let profile_str = profile_to_string(session.profile)
  let questions =
    interview_questions.get_questions_for_round(profile_str, round)

  case questions {
    [] -> Error("No questions found for round " <> string.inspect(round))
    [first, ..] -> Ok(first)
  }
}

/// Get next unanswered question in current round
pub fn get_next_question_in_round(
  session: InterviewSession,
  round: Int,
) -> Result(Question, String) {
  let profile_str = profile_to_string(session.profile)
  let answered_ids = get_answered_question_ids(session)

  case interview_questions.get_next_question(profile_str, round, answered_ids) {
    option.Some(question) -> Ok(question)
    option.None ->
      Error("No more unanswered questions in round " <> string.inspect(round))
  }
}

/// Get all answered question IDs
fn get_answered_question_ids(session: InterviewSession) -> List(String) {
  list.map(session.answers, fn(answer) { answer.question_id })
}

/// Check for gaps after answering a question
pub fn check_for_gaps(
  session: InterviewSession,
  question: Question,
  answer: Answer,
) -> #(InterviewSession, List(Gap)) {
  let blocking_gaps = detect_blocking_gaps(question, answer)
  let updated_session =
    InterviewSession(
      ..session,
      gaps: list.append(session.gaps, blocking_gaps),
      current_phase: session.current_phase,
    )
  #(updated_session, blocking_gaps)
}

/// Detect blocking gaps in the answer
fn detect_blocking_gaps(question: Question, answer: Answer) -> List(Gap) {
  // Check if answer is too vague/short for critical questions
  let response_length = string.length(string.trim(answer.response))

  case question.priority {
    Critical if response_length < 10 -> [
      Gap(
        id: "gap_" <> question.id,
        field: question.question,
        description: "Critical question answered too briefly",
        blocking: True,
        suggested_default: "Please provide a more detailed answer",
        why_needed: "This is a critical requirement for spec generation",
        round: answer.round,
        resolved: False,
        resolution: "",
      ),
    ]
    _ -> []
  }
}

/// Check for conflicts between answers
pub fn check_for_conflicts(
  session: InterviewSession,
  new_answer: Answer,
) -> #(InterviewSession, List(Conflict)) {
  let conflicts = detect_conflicts_in_session(session, new_answer)
  let updated_session =
    InterviewSession(
      ..session,
      conflicts: list.append(session.conflicts, conflicts),
    )
  #(updated_session, conflicts)
}

/// Detect conflicting requirements
fn detect_conflicts_in_session(
  session: InterviewSession,
  new_answer: Answer,
) -> List(Conflict) {
  let conflicts_found =
    list.fold(session.answers, [], fn(acc, existing) {
      let new_conflicts = detect_answer_pair_conflicts(existing, new_answer)
      list.append(acc, new_conflicts)
    })
  conflicts_found
}

/// Detect conflicts between two answers
fn detect_answer_pair_conflicts(
  answer1: Answer,
  answer2: Answer,
) -> List(Conflict) {
  // Check for CAP theorem conflicts
  case answer1.perspective, answer2.perspective {
    Developer, Ops -> {
      let ans1_lower = string.lowercase(answer1.response)
      let ans2_lower = string.lowercase(answer2.response)
      case
        string.contains(ans1_lower, "consistency")
        && string.contains(ans2_lower, "high latency")
      {
        True -> [
          Conflict(
            id: "cap_conflict_"
              <> answer1.question_id
              <> "_"
              <> answer2.question_id,
            between: #(answer1.question_id, answer2.question_id),
            description: "Consistency vs. Availability tension (CAP theorem)",
            impact: "Requires architectural decision on data replication strategy",
            options: [
              ConflictResolution(
                option: "Prioritize Consistency",
                description: "Strong consistency with potential latency",
                tradeoffs: "Higher latency, complex distributed coordination",
                recommendation: "Use when data correctness is critical (financial, medical)",
              ),
              ConflictResolution(
                option: "Prioritize Availability",
                description: "High availability with eventual consistency",
                tradeoffs: "Temporary data inconsistency, conflict resolution needed",
                recommendation: "Use for content, social feeds, non-critical data",
              ),
            ],
            chosen: -1,
          ),
        ]
        False -> []
      }
    }
    _, _ -> []
  }
}

/// Mark a round as complete
pub fn complete_round(session: InterviewSession) -> InterviewSession {
  let new_stage = case session.rounds_completed + 1 {
    1 -> Discovery
    2 -> Discovery
    3 -> Refinement
    4 -> Validation
    5 -> Complete
    _ -> Complete
  }

  InterviewSession(
    ..session,
    rounds_completed: session.rounds_completed + 1,
    stage: new_stage,
  )
}

/// Get current round number based on answers
pub fn get_current_round(session: InterviewSession) -> Int {
  case session.answers {
    [] -> 1
    answers -> {
      let max_round =
        list.fold(answers, 0, fn(acc, answer) {
          case answer.round > acc {
            True -> answer.round
            False -> acc
          }
        })
      // If we've answered all questions in current round, move to next
      let current_round_count =
        list.length(list.filter(answers, fn(a) { a.round == max_round }))
      let questions_in_round =
        list.length(interview_questions.get_questions_for_round(
          profile_to_string(session.profile),
          max_round,
        ))

      case current_round_count >= questions_in_round {
        True -> max_round + 1
        False -> max_round
      }
    }
  }
}

/// Resolve a conflict by choosing an option
pub fn resolve_conflict(
  session: InterviewSession,
  conflict_id: String,
  chosen_option: Int,
) -> Result(InterviewSession, String) {
  let updated_conflicts =
    list.map(session.conflicts, fn(conflict) {
      case conflict.id == conflict_id {
        True -> Conflict(..conflict, chosen: chosen_option)
        False -> conflict
      }
    })

  Ok(InterviewSession(..session, conflicts: updated_conflicts))
}

/// Mark a gap as resolved
pub fn resolve_gap(
  session: InterviewSession,
  gap_id: String,
  resolution: String,
) -> InterviewSession {
  let updated_gaps =
    list.map(session.gaps, fn(gap) {
      case gap.id == gap_id {
        True -> Gap(..gap, resolved: True, resolution: resolution)
        False -> gap
      }
    })

  InterviewSession(..session, gaps: updated_gaps)
}

/// Get all unresolved blocking gaps
pub fn get_blocking_gaps(session: InterviewSession) -> List(Gap) {
  list.filter(session.gaps, fn(gap) { gap.blocking && !gap.resolved })
}

/// Get all unresolved conflicts
pub fn get_unresolved_conflicts(session: InterviewSession) -> List(Conflict) {
  list.filter(session.conflicts, fn(conflict) { conflict.chosen == -1 })
}

/// Check if interview can proceed (no blocking gaps)
pub fn can_proceed(session: InterviewSession) -> Result(Nil, String) {
  let blocking = get_blocking_gaps(session)
  case blocking {
    [] -> Ok(Nil)
    gaps -> {
      let gap_descriptions = list.map(gaps, fn(gap) { gap.description })
      Error("Blocking gaps: " <> string.join(gap_descriptions, "; "))
    }
  }
}

/// Apply phase-gate enforcement to an execution plan
/// Ensures that later phases can only execute after prior phases complete
pub fn apply_phase_gating(
  session: InterviewSession,
  plan: plan_mode.ExecutionPlan,
) -> plan_mode.ExecutionPlan {
  let current_phase = session.current_phase
  let completed_phases = session.completed_phases
  let phases = plan.phases

  // Find the highest phase number in the plan
  let max_plan_phase =
    phases
    |> list.map(fn(phase) { phase.phase_number })
    |> list.fold(0, int.max)

  // Check if we're trying to execute phases beyond current phase without completing prior phases
  let can_execute_current =
    list.contains(completed_phases, current_phase - 1) || current_phase == 1
  let can_execute_next = list.contains(completed_phases, current_phase)

  // Build list of phase gate blockers
  let phase_blockers =
    phases
    |> list.filter(fn(phase) {
      // Block phases that are ahead of current phase without completion
      phase.phase_number > current_phase && !can_execute_next
    })
    |> list.map(fn(phase) {
      "Phase-gate: Phase "
      <> int.to_string(phase.phase_number)
      <> " is locked. Complete Phase "
      <> int.to_string(current_phase)
      <> " first."
    })

  // Add general blocker if trying to skip ahead
  let general_blocker = case
    max_plan_phase > current_phase && !can_execute_next
  {
    True -> [
      "Phase-gate: Cannot execute Phase "
      <> int.to_string(max_plan_phase)
      <> " while Phase "
      <> int.to_string(current_phase)
      <> " is incomplete",
    ]
    False -> []
  }

  let all_blockers =
    list.append(plan.blockers, list.append(phase_blockers, general_blocker))

  // Filter phases to only include those that can be executed
  let allowed_phases = case can_execute_current {
    True ->
      phases
      |> list.filter(fn(phase) {
        // Allow current phase and any completed phases
        phase.phase_number <= current_phase
        || list.contains(completed_phases, phase.phase_number)
      })
    False ->
      // Can't even execute current phase - return empty phases with blocker
      []
  }

  plan_mode.ExecutionPlan(
    ..plan,
    phases: allowed_phases,
    blockers: all_blockers,
  )
}

/// Mark a phase as completed and advance to the next phase
pub fn complete_phase(
  session: InterviewSession,
  phase_number: Int,
) -> InterviewSession {
  let updated_completed = case
    list.contains(session.completed_phases, phase_number)
  {
    True -> session.completed_phases
    False -> [phase_number, ..session.completed_phases]
  }

  let next_phase = case session.current_phase == phase_number {
    True -> session.current_phase + 1
    False -> session.current_phase
  }

  InterviewSession(
    ..session,
    completed_phases: updated_completed,
    current_phase: next_phase,
  )
}

/// Check if a specific phase can be executed
pub fn can_execute_phase(session: InterviewSession, phase_number: Int) -> Bool {
  // Phase 1 can always be executed
  // Other phases require all prior phases to be completed
  case phase_number {
    1 -> True
    _ -> {
      // Check that all phases before this one are completed
      list.range(1, phase_number - 1)
      |> list.all(fn(p) { list.contains(session.completed_phases, p) })
    }
  }
}

/// Get the next unlockable phase
pub fn get_next_phase(session: InterviewSession) -> Int {
  session.current_phase
}

/// Format progress summary
pub fn format_progress(session: InterviewSession) -> String {
  let stage_str = case session.stage {
    Discovery -> "Discovery"
    Refinement -> "Refinement"
    Validation -> "Validation"
    Complete -> "Complete"
    Paused -> "Paused"
  }

  let profile_str = profile_to_string(session.profile)
  let answer_count = list.length(session.answers)
  let gap_count = list.length(session.gaps)
  let conflict_count = list.length(session.conflicts)

  "Profile: "
  <> profile_str
  <> " | Stage: "
  <> stage_str
  <> " | Answers: "
  <> string.inspect(answer_count)
  <> " | Gaps: "
  <> string.inspect(gap_count)
  <> " | Conflicts: "
  <> string.inspect(conflict_count)
}

/// Convert Profile to string
pub fn profile_to_string(profile: Profile) -> String {
  case profile {
    Api -> "api"
    Cli -> "cli"
    Event -> "event"
    Data -> "data"
    Workflow -> "workflow"
    UI -> "ui"
  }
}

/// Convert InterviewStage to string
pub fn stage_to_string(stage: InterviewStage) -> String {
  case stage {
    Discovery -> "discovery"
    Refinement -> "refinement"
    Validation -> "validation"
    Complete -> "complete"
    Paused -> "paused"
  }
}

/// Convert string to Profile
pub fn string_to_profile(s: String) -> Result(Profile, String) {
  case string.lowercase(s) {
    "api" -> Ok(Api)
    "cli" -> Ok(Cli)
    "event" -> Ok(Event)
    "data" -> Ok(Data)
    "workflow" -> Ok(Workflow)
    "ui" -> Ok(UI)
    _ -> Error("Unknown profile: " <> s)
  }
}
