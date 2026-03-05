import gleam/dict
import gleam/list
import gleam/option
import gleeunit/should
import intent/semantic_validator

// Helper function to create a valid spec for testing
fn valid_spec() -> semantic_validator.Spec {
  semantic_validator.Spec(
    name: "test-api",
    description: "Test API description",
    audience: "developers",
    version: "1.0.0",
    success_criteria: ["Users can CRUD"],
    config: semantic_validator.Config(
      base_url: "http://localhost:8080",
      timeout_ms: 5000,
      headers: dict.new(),
    ),
    features: [
      semantic_validator.Feature(
        name: "test-feature",
        description: "Test feature",
        behaviors: [
          semantic_validator.Behavior(
            name: "test-behavior",
            intent: "Test intent",
            notes: "",
            requires: [],
            tags: [],
            request: semantic_validator.Request(
              method: "GET",
              path: "/api/test",
              headers: dict.new(),
              query: dict.new(),
              body: option.None,
            ),
            response: semantic_validator.Response(
              status: 200,
              example: option.None,
              checks: dict.from_list([
                #(
                  "test",
                  semantic_validator.Check(rule: "present", why: "Test check"),
                ),
              ]),
              headers: option.None,
            ),
            captures: dict.new(),
          ),
        ],
      ),
    ],
    rules: [],
    anti_patterns: [],
    ai_hints: semantic_validator.AIHints(
      implementation: option.None,
      entities: option.None,
      security: option.None,
      pitfalls: option.None,
    ),
  )
}

// Test: Duplicate behavior names across features fail validation
pub fn duplicate_behavior_names_fail_validation_test() {
  let spec = valid_spec()

  // Create two features with duplicate behavior names
  let feature1 =
    semantic_validator.Feature(
      name: "feature1",
      description: "First feature",
      behaviors: [
        semantic_validator.Behavior(
          name: "duplicate_behavior",
          intent: "Test intent",
          notes: "",
          requires: [],
          tags: [],
          request: semantic_validator.Request(
            method: "GET",
            path: "/api/test1",
            headers: dict.new(),
            query: dict.new(),
            body: option.None,
          ),
          response: semantic_validator.Response(
            status: 200,
            example: option.None,
            checks: dict.from_list([
              #(
                "test",
                semantic_validator.Check(rule: "present", why: "Test check"),
              ),
            ]),
            headers: option.None,
          ),
          captures: dict.new(),
        ),
      ],
    )

  let feature2 =
    semantic_validator.Feature(
      name: "feature2",
      description: "Second feature",
      behaviors: [
        semantic_validator.Behavior(
          name: "duplicate_behavior",
          intent: "Test intent",
          notes: "",
          requires: [],
          tags: [],
          request: semantic_validator.Request(
            method: "GET",
            path: "/api/test2",
            headers: dict.new(),
            query: dict.new(),
            body: option.None,
          ),
          response: semantic_validator.Response(
            status: 200,
            example: option.None,
            checks: dict.from_list([
              #(
                "test",
                semantic_validator.Check(rule: "present", why: "Test check"),
              ),
            ]),
            headers: option.None,
          ),
          captures: dict.new(),
        ),
      ],
    )

  let spec = semantic_validator.Spec(..spec, features: [feature1, feature2])

  let result = semantic_validator.validate_spec_semantics(spec)

  case result {
    semantic_validator.Valid -> should.fail()
    semantic_validator.Invalid(errors) -> {
      let has_unique_names_error =
        errors
        |> list.any(fn(e) { e.field == "behaviors" && e.rule == "unique_names" })
      has_unique_names_error
      |> should.be_true
    }
  }
}

// Test: Unique behavior names pass validation
pub fn unique_behavior_names_pass_validation_test() {
  let spec = valid_spec()

  // Create two features with unique behavior names
  let feature1 =
    semantic_validator.Feature(
      name: "feature1",
      description: "First feature",
      behaviors: [
        semantic_validator.Behavior(
          name: "behavior1",
          intent: "Test intent",
          notes: "",
          requires: [],
          tags: [],
          request: semantic_validator.Request(
            method: "GET",
            path: "/api/test1",
            headers: dict.new(),
            query: dict.new(),
            body: option.None,
          ),
          response: semantic_validator.Response(
            status: 200,
            example: option.None,
            checks: dict.from_list([
              #(
                "test",
                semantic_validator.Check(rule: "present", why: "Test check"),
              ),
            ]),
            headers: option.None,
          ),
          captures: dict.new(),
        ),
      ],
    )

  let feature2 =
    semantic_validator.Feature(
      name: "feature2",
      description: "Second feature",
      behaviors: [
        semantic_validator.Behavior(
          name: "behavior2",
          intent: "Test intent",
          notes: "",
          requires: [],
          tags: [],
          request: semantic_validator.Request(
            method: "GET",
            path: "/api/test2",
            headers: dict.new(),
            query: dict.new(),
            body: option.None,
          ),
          response: semantic_validator.Response(
            status: 200,
            example: option.None,
            checks: dict.from_list([
              #(
                "test",
                semantic_validator.Check(rule: "present", why: "Test check"),
              ),
            ]),
            headers: option.None,
          ),
          captures: dict.new(),
        ),
      ],
    )

  let spec = semantic_validator.Spec(..spec, features: [feature1, feature2])

  let result = semantic_validator.validate_spec_semantics(spec)

  result
  |> should.equal(semantic_validator.Valid)
}
