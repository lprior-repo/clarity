import "github.com/intent-cli/intent/schema:intent"

// Test: Missing required 'name' field
spec: intent.#Spec & {
	description: "Test spec missing name field"
	audience: "Test audience"
	version: "1.0.0"

	success_criteria: ["Test criterion"]

