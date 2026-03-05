// Test case: Duplicate behavior names in features
name: "duplicate-names-spec"
description: "Spec with duplicate behavior names"
audience: "developers"
version: "1.0.0"
success_criteria: ["API works"]
features: [
  {
    name: "feature1"
    description: "First feature"
    behaviors: [
      {
        name: "duplicate_behavior"
        intent: "Test behavior in first feature"
        preconditions: ["System ready"]
        postconditions: ["Behavior completed"]
        verifications: [
          {
            description: "Verify behavior"
            criteria: ["Condition met"]
          },
        ]
        notes: ""
        requires: []
        tags: ["test"]
      },
    ]
  },
  {
    name: "feature2"
    description: "Second feature with duplicate name"
    behaviors: [
      {
        name: "duplicate_behavior"
        intent: "Test behavior in second feature"
        preconditions: ["System ready"]
        postconditions: ["Behavior completed"]
        verifications: [
          {
            description: "Verify behavior"
            criteria: ["Condition met"]
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
