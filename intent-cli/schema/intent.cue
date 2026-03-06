// Intent v3.0 Schema
// Declarative specifications for planning and verification
package intent

// Main specification type
#Spec: {
	name!:        string
	description!: string
	audience!:    string
	version!:     string

	success_criteria!: [...string]

	features!: [...#Feature]
	invariants!: [...#Invariant]
	anti_patterns!: [...#AntiPattern]

	// AI implementation hints (required)
	ai_hints!: #AIHints
}

// Feature groups related behaviors
#Feature: {
	name!:        string
	description!: string
	behaviors!: [...#Behavior]
}

// A single behavior/test case
#Behavior: {
	name!:   #Identifier
	intent!: string // Plain English purpose

	// Additional context for humans/AI (optional)
	notes?: string

	// Dependencies - behaviors that must run first (optional)
	requires?: [...#Identifier]

	// Tags for filtering (optional)
	tags?: [...string]

	// Preconditions: What must be true before this behavior (optional)
	preconditions?: [...string]

	// Postconditions: What must be true after this behavior (optional)
	postconditions?: [...string]

	// Verifications: How to verify the behavior works (optional)
	verifications?: [...#Verification]
}

// Valid identifier pattern
#Identifier: =~"^[a-z][a-z0-9_-]*$"

// Verification of behavior correctness
#Verification: {
	description!: string
	criteria!: [...string]
	examples?: [...] // Generic JSON examples
}

// Global invariants that apply to all behaviors
#Invariant: {
	name!:        string
	description!: string
	criteria!: [...string] // What must always be true
}

// Anti-patterns with good/bad examples
#AntiPattern: {
	name!:        string
	description!: string

	// What NOT to do
	bad_example!: {...}

	// What TO do
	good_example!: {...}

	// Explanation (optional)
	why?: string
}

// AI implementation hints
#AIHints: {
	implementation?: {
		suggested_stack?: [...string]
	}

	entities?: [string]: #EntityHint

	security?: {
		password_hashing?: string
		jwt_algorithm?:    string
		jwt_expiry?:       string
		rate_limiting?:    string
	}

	pitfalls?: [...string]
}

#EntityHint: {
	fields?: [string]: string | #FieldHint
}

#FieldHint: {
	description?: string
	type?:        string
	validation?:  string
	example?:     string
	sensitive?:   bool | *false
}
