import "github.com/intent-cli/intent/schema:intent"

spec: intent.#Spec & {
	name: "Test Spec"
	description: "Test description"
	version: "1.0.0"

	success_criteria: ["Test criterion"]

