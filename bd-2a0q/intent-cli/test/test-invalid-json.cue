// Test case: Invalid JSON in examples
name: "invalid-json-spec"
description: "Spec with invalid JSON in examples"
audience: "developers"
version: "1.0.0"
success_criteria: ["API works"]
features: [
  {
    name: "feature1"
    description: "A test feature"
    behaviors: [
      {
        name: "behavior1"
        intent: "Test the API"
        preconditions: ["System ready"]
        postconditions: ["Behavior completed"]
        verifications: [
          {
            description: "Verify behavior"
            criteria: ["Condition met"]
            examples: [{"message": "ok", "invalid": "json"}]
          },
        ]
        notes: "This is a test behavior"
        requires: []
        tags: ["test"]
      },
    ]
  },
]
invariants: []
anti_patterns: []
ai_hints: {}
