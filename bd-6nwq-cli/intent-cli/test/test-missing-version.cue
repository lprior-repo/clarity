import "github.com/intent-cli/intent/schema:intent"

spec: intent.#Spec & {
	name: "Test Spec"
	description: "Test description"
	audience: "Test audience"

	success_criteria: ["Test criterion"]

