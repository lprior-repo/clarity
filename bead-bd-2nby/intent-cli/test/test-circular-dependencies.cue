// Test case: Circular dependencies in behaviors
name: "circular-dependencies-spec"
description: "Spec with circular behavior dependencies"
audience: "developers"
version: "1.0.0"
success_criteria: ["API works"]
features: [
  {
    name: "feature1"
    description: "A test feature with circular deps"
    behaviors: [
      {
        name: "behavior1"
        intent: "Test behavior 1"
        preconditions: ["System is ready"]
        postconditions: ["Behavior 1 completed"]
        verifications: [
          {
            description: "Verify behavior 1"
            criteria: ["Condition met"]
          },
        ]
        notes: "Behavior 1 depends on behavior 2"
        requires: ["behavior2"]
        tags: ["test"]
      },
      {
        name: "behavior2"
        intent: "Test behavior 2"
        preconditions: ["System is ready"]
        postconditions: ["Behavior 2 completed"]
        verifications: [
          {
            description: "Verify behavior 2"
            criteria: ["Condition met"]
          },
        ]
        notes: "Behavior 2 depends on behavior 1"
        requires: ["behavior1"]
        tags: ["test"]
      },
    ]
  },
]
invariants: []
anti_patterns: []
ai_hints: {}
