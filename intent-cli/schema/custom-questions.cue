// Custom Interview Questions Schema
// Place a file named custom-questions.cue in your .intent/ directory
// to add or override interview questions for your project.
//
// Usage: Copy this file to .intent/custom-questions.cue and customize.
//
// Questions with the same ID as built-in questions will override them.
// Questions with new IDs will be added to the appropriate profile/round.

package intent

// Re-use the question schema from the main questions file
#Question: {
	id:            string
	round:         1 | 2 | 3 | 4 | 5
	perspective:   "user" | "developer" | "ops" | "security" | "business"
	category:      "happy_path" | "error_case" | "edge_case" | "constraint" | "dependency" | "nonfunctional"
	priority:      "critical" | "important" | "nice_to_have"
	question:      string
	context:       string
	example:       string
	expected_type: string | *"text"
	extract_into:  [...string] | *[]
	depends_on:    [...string] | *[]
	blocks:        [...string] | *[]
}

// Custom questions structure - simpler than the full questions database
// Only include the profiles and rounds you want to customize
#CustomQuestions: {
	// Optional: Add or override questions for specific profiles
	api?:      #ProfileCustom
	cli?:      #ProfileCustom
	event?:    #ProfileCustom
	data?:     #ProfileCustom
	workflow?: #ProfileCustom
	ui?:       #ProfileCustom

	// Optional: Add or override common questions (rounds 3-5)
	common?: #CommonCustom
}

#ProfileCustom: {
	round_1?: [...#Question]
	round_2?: [...#Question]
}

#CommonCustom: {
	round_3?: [...#Question]
	round_4?: [...#Question]
	round_5?: [...#Question]
}

// The exported custom questions
// Users should define this in their .intent/custom-questions.cue
custom_questions: #CustomQuestions & {
	// Example: Override an API question
	// api: {
	// 	round_1: [
	// 		{
	// 			id:          "r1-user-api-1"  // Same ID = override
	// 			round:       1
	// 			perspective: "user"
	// 			category:    "happy_path"
	// 			priority:    "critical"
	// 			question:    "What is the PRIMARY purpose of this API?"
	// 			context:     "Customized for our project"
	// 			example:     "Our custom example"
	// 		},
	// 	]
	// }

	// Example: Add a new question
	// api: {
	// 	round_1: [
	// 		{
	// 			id:          "custom-api-1"  // New ID = add
	// 			round:       1
	// 			perspective: "developer"
	// 			category:    "constraint"
	// 			priority:    "important"
	// 			question:    "What external services does this API depend on?"
	// 			context:     "Company-specific integration question"
	// 			example:     "Auth0, Stripe, SendGrid"
	// 			extract_into: ["dependencies"]
	// 		},
	// 	]
	// }
}
