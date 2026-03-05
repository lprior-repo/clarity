import gleam/list
import gleam/option.{None, Some}
import gleam/string
import gleeunit
import gleeunit/should
import intent/kirk/ears_parser.{
  Complex, EventDriven, Optional, StateDriven, Ubiquitous, Unwanted,
}

pub fn main() -> Nil {
  gleeunit.main()
}

// ============================================================================
// EARS Parser Tests
// ============================================================================

pub fn parse_ubiquitous_requirement_test() {
  let input = "THE SYSTEM SHALL validate all API inputs"
  let result = ears_parser.parse(input)

  result.requirements
  |> list.length
  |> should.equal(1)

  case result.requirements {
    [req] -> {
      req.pattern |> should.equal(Ubiquitous)
      req.system_shall |> should.equal("validate all API inputs")
      req.trigger |> should.equal(None)
      req.state |> should.equal(None)
      req.condition |> should.equal(None)
    }
    _ -> should.fail()
  }
}

pub fn parse_ubiquitous_lowercase_test() {
  let input = "the system shall log all requests"
  let result = ears_parser.parse(input)

  result.requirements
  |> list.length
  |> should.equal(1)

  case result.requirements {
    [req] -> {
      req.pattern |> should.equal(Ubiquitous)
      req.system_shall |> should.equal("log all requests")
    }
    _ -> should.fail()
  }
}

pub fn parse_event_driven_requirement_test() {
  let input = "WHEN user submits form THE SYSTEM SHALL validate email"
  let result = ears_parser.parse(input)

  result.requirements
  |> list.length
  |> should.equal(1)

  case result.requirements {
    [req] -> {
      req.pattern |> should.equal(EventDriven)
      req.system_shall |> should.equal("validate email")
      req.trigger |> should.equal(Some("user submits form"))
      req.state |> should.equal(None)
    }
    _ -> should.fail()
  }
}

pub fn parse_state_driven_requirement_test() {
  let input = "WHILE user is authenticated THE SYSTEM SHALL allow access"
  let result = ears_parser.parse(input)

  result.requirements
  |> list.length
  |> should.equal(1)

  case result.requirements {
    [req] -> {
      req.pattern |> should.equal(StateDriven)
      req.system_shall |> should.equal("allow access")
      req.state |> should.equal(Some("user is authenticated"))
      req.trigger |> should.equal(None)
    }
    _ -> should.fail()
  }
}

pub fn parse_optional_requirement_test() {
  let input = "WHERE user has admin role THE SYSTEM SHALL show admin panel"
  let result = ears_parser.parse(input)

  result.requirements
  |> list.length
  |> should.equal(1)

  case result.requirements {
    [req] -> {
      req.pattern |> should.equal(Optional)
      req.system_shall |> should.equal("show admin panel")
      req.condition |> should.equal(Some("user has admin role"))
    }
    _ -> should.fail()
  }
}

pub fn parse_unwanted_requirement_test() {
  let input = "IF user is banned THEN THE SYSTEM SHALL NOT allow login"
  let result = ears_parser.parse(input)

  result.requirements
  |> list.length
  |> should.equal(1)

  case result.requirements {
    [req] -> {
      req.pattern |> should.equal(Unwanted)
      req.system_shall_not |> should.equal(Some("allow login"))
      req.condition |> should.equal(Some("user is banned"))
    }
    _ -> should.fail()
  }
}

pub fn parse_complex_requirement_test() {
  let input =
    "WHILE user is logged in WHEN session expires THE SYSTEM SHALL redirect to login"
  let result = ears_parser.parse(input)

  result.requirements
  |> list.length
  |> should.equal(1)

  case result.requirements {
    [req] -> {
      req.pattern |> should.equal(Complex)
      req.system_shall |> should.equal("redirect to login")
      req.state |> should.equal(Some("user is logged in"))
      req.trigger |> should.equal(Some("session expires"))
    }
    _ -> should.fail()
  }
}

pub fn parse_multiple_requirements_test() {
  let input =
    "THE SYSTEM SHALL validate inputs\nWHEN user clicks submit THE SYSTEM SHALL save data\nWHILE loading THE SYSTEM SHALL show spinner"
  let result = ears_parser.parse(input)

  result.requirements
  |> list.length
  |> should.equal(3)

  // Verify patterns in order
  case result.requirements {
    [r1, r2, r3] -> {
      r1.pattern |> should.equal(Ubiquitous)
      r2.pattern |> should.equal(EventDriven)
      r3.pattern |> should.equal(StateDriven)
    }
    _ -> should.fail()
  }
}

pub fn parse_ignores_comments_test() {
  let input = "# This is a comment\nTHE SYSTEM SHALL work\n# Another comment"
  let result = ears_parser.parse(input)

  result.requirements
  |> list.length
  |> should.equal(1)
}

pub fn parse_ignores_empty_lines_test() {
  let input = "\n\nTHE SYSTEM SHALL work\n\n"
  let result = ears_parser.parse(input)

  result.requirements
  |> list.length
  |> should.equal(1)
}

pub fn parse_reports_errors_for_invalid_lines_test() {
  let input = "This is not a valid EARS requirement"
  let result = ears_parser.parse(input)

  result.errors
  |> list.length
  |> should.equal(1)

  result.requirements
  |> list.length
  |> should.equal(0)
}

pub fn parse_generates_warnings_for_missing_unwanted_test() {
  // A spec without any IF...SHALL NOT patterns should generate a warning
  let input = "THE SYSTEM SHALL do something"
  let result = ears_parser.parse(input)

  result.warnings
  |> list.any(fn(w) {
    w == "Consider adding IF...SHALL NOT patterns for unwanted behaviors"
  })
  |> should.be_true()
}

// ============================================================================
// Behavior Conversion Tests
// ============================================================================

pub fn to_behaviors_generates_names_test() {
  let input = "WHEN user submits THE SYSTEM SHALL create record"
  let result = ears_parser.parse(input)
  let behaviors = ears_parser.to_behaviors(result)

  behaviors
  |> list.length
  |> should.equal(1)

  case behaviors {
    [b] -> {
      // Name should contain "create" based on the behavior
      b.name
      |> should.not_equal("")
    }
    _ -> should.fail()
  }
}

pub fn to_behaviors_infers_http_methods_test() {
  let create_input = "THE SYSTEM SHALL create a new user"
  let create_result = ears_parser.parse(create_input)
  let create_behaviors = ears_parser.to_behaviors(create_result)

  case create_behaviors {
    [b] -> {
      b.method |> should.equal("POST")
      b.status |> should.equal(201)
    }
    _ -> should.fail()
  }

  let delete_input = "THE SYSTEM SHALL delete the user record"
  let delete_result = ears_parser.parse(delete_input)
  let delete_behaviors = ears_parser.to_behaviors(delete_result)

  case delete_behaviors {
    [b] -> {
      b.method |> should.equal("DELETE")
      b.status |> should.equal(204)
    }
    _ -> should.fail()
  }
}

pub fn to_behaviors_extracts_preconditions_test() {
  let input = "WHILE user is authenticated THE SYSTEM SHALL access resource"
  let result = ears_parser.parse(input)
  let behaviors = ears_parser.to_behaviors(result)

  case behaviors {
    [b] -> {
      b.preconditions
      |> list.any(fn(p) { p == "user is authenticated" })
      |> should.be_true()
    }
    _ -> should.fail()
  }
}

// ============================================================================
// CUE Output Tests
// ============================================================================

pub fn to_cue_generates_valid_structure_test() {
  let input = "THE SYSTEM SHALL validate inputs"
  let result = ears_parser.parse(input)
  let cue_output = ears_parser.to_cue(result, "TestSpec")

  // Should contain spec declaration
  cue_output
  |> should_contain("spec: intent.#Spec")

  // Should contain the spec name
  cue_output
  |> should_contain("name: \"TestSpec\"")

  // Should contain behaviors section
  cue_output
  |> should_contain("behaviors:")
}

pub fn to_cue_escapes_special_characters_test() {
  let input = "THE SYSTEM SHALL handle \"quoted\" values"
  let result = ears_parser.parse(input)
  let cue_output = ears_parser.to_cue(result, "TestSpec")

  // Quotes should be escaped
  cue_output
  |> should_contain("\\\"quoted\\\"")
}

// ============================================================================
// Format Output Tests
// ============================================================================

pub fn format_result_shows_pattern_icons_test() {
  let input =
    "THE SYSTEM SHALL work\nWHEN event THE SYSTEM SHALL respond\nIF bad THEN THE SYSTEM SHALL NOT fail"
  let result = ears_parser.parse(input)
  let formatted = ears_parser.format_result(result)

  // Should show the Ubiquitous icon
  formatted
  |> should_contain("Ubiquitous")

  // Should show the Event-Driven icon
  formatted
  |> should_contain("Event-Driven")

  // Should show the Unwanted icon
  formatted
  |> should_contain("Unwanted")
}

pub fn pattern_to_string_test() {
  ears_parser.pattern_to_string(Ubiquitous)
  |> should.equal("Ubiquitous")

  ears_parser.pattern_to_string(EventDriven)
  |> should.equal("Event-Driven")

  ears_parser.pattern_to_string(StateDriven)
  |> should.equal("State-Driven")

  ears_parser.pattern_to_string(Optional)
  |> should.equal("Optional")

  ears_parser.pattern_to_string(Unwanted)
  |> should.equal("Unwanted")

  ears_parser.pattern_to_string(Complex)
  |> should.equal("Complex")
}

// ============================================================================
// Helper Functions
// ============================================================================

fn should_contain(haystack: String, needle: String) {
  case string.contains(haystack, needle) {
    True -> Nil
    False -> {
      // For better error messages
      should.equal(haystack, "Expected to contain: " <> needle)
    }
  }
}
