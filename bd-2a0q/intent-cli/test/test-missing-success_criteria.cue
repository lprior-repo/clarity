import "github.com/intent-cli/intent/schema:intent"

spec: intent.#Spec & {
	name: "Test Spec"
	description: "Test description"
	audience: "Test audience"
	version: "1.0.0"

