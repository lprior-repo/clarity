import "github.com/intent-cli/intent/schema:intent"

// Test: Missing required 'description' field
spec: intent.#Spec & {
	name: "Test Spec Missing Description"
	audience: "Test audience"
	version: "1.0.0"

	success_criteria: ["Test criterion"]

