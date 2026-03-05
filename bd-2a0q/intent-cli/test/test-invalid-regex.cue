// Test case: Invalid regex patterns
name: "invalid-regex-spec"
description: "Spec with invalid regex patterns"
audience: "developers"
version: "1.0.0"
success_criteria: ["API works"]
features: [
  {
    name: "feature1"
    description: "A test feature with invalid regex"
    behaviors: [
      {
        name: "behavior1"
        intent: "Test behavior with invalid regex"
        preconditions: ["System ready"]
        postconditions: ["Behavior completed"]
        verifications: [
          {
            description: "Verify with invalid regex"
            criteria: ["body.message matches [a-z"]
          },
        ]
        notes: ""
        requires: []
        tags: ["test"]
      },
    ]
  },
]
invariants: []
anti_patterns: []
ai_hints: {}
