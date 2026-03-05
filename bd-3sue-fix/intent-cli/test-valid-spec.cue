package test

import "github.com/intent-cli/intent/schema:intent"

spec: intent.#Spec & {
	name: "Test Spec"
	description: "A minimal valid spec"
	audience: "Test users"
	version: "1.0.0"

	success_criteria: [
		"Test passes",
	]

	features: [{
		name: "Test Feature"
		description: "A test feature"
		behaviors: [{
			name: "test-behavior"
			intent: "Test intent"
		}]
	}]

	invariants: []

	anti_patterns: []

	ai_hints: {}
}
