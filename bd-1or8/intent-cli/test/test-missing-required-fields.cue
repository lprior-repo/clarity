// Test case: Missing required fields
package test

import "github.com/intent-cli/intent/schema:intent"

spec: intent.#Spec & {
  name: "Test Spec"
  description: "Spec missing verifications in behavior"
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
          intent: "Test behavior - this spec intentionally has minimal fields"
          notes: "Testing optional fields"
          requires: []
          tags: ["test"]
        }
      ]
    }
  ]
  invariants: []
  anti_patterns: []
  ai_hints: {}
}
