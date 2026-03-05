// AI Interview Protocol Schema
// This schema defines the CUE output format that controls the AI's behavior
// The CLI outputs these schemas; the AI parses and follows them exactly

package ai_interview

// =============================================================================
// INTERVIEW STATE MACHINE
// =============================================================================

// The CLI outputs this to tell the AI what to do next
#AIDirective: {
	// Current state of the interview
	state: #InterviewState

	// What the AI should do right now
	action: #AIAction

	// The exact question to ask (if action is "ask")
	question?: #Question

	// Progress through the interview
	progress: {
		current_round:    int
		total_rounds:     int
		questions_asked:  int
		questions_remain: int
		percent_complete: float
	}

	// Collected requirements so far (for AI context)
	collected: #CollectedRequirements

	// Validation errors from last answer (if any)
	errors?: [...#ValidationError]
}

#InterviewState:
	"not_started" |
	"round_ubiquitous" |
	"round_event_driven" |
	"round_state_driven" |
	"round_optional" |
	"round_unwanted" |
	"round_complex" |
	"round_inversion" |
	"round_premortem" |
	"clarification_needed" |
	"complete"

#AIAction:
	"ask_question" |      // AI should ask the human this question
	"clarify_answer" |    // AI should ask for clarification
	"confirm_understanding" | // AI should confirm what was understood
	"present_summary" |   // AI should present collected requirements
	"await_approval" |    // AI should wait for human to approve
	"generate_beads" |    // AI should request bead generation
	"done"                // Interview complete, beads ready

// =============================================================================
// QUESTIONS (CLI tells AI exactly what to ask)
// =============================================================================

#Question: {
	// Unique ID for this question
	id: string

	// The EARS pattern this question is gathering
	pattern: #EARSPattern

	// The exact text the AI should ask
	text: string

	// Context to help the AI understand why
	context: string

	// Examples of good answers (AI can show these)
	examples: [...string]

	// Expected answer format
	expected_format: #AnswerFormat

	// Is this question required or optional?
	required: bool

	// Follow-up questions if answer matches certain patterns
	follow_ups?: [...#ConditionalFollowUp]
}

#EARSPattern:
	"ubiquitous" |     // THE SYSTEM SHALL
	"event_driven" |   // WHEN ... THE SYSTEM SHALL
	"state_driven" |   // WHILE ... THE SYSTEM SHALL
	"optional" |       // WHERE ... THE SYSTEM SHALL
	"unwanted" |       // IF ... THE SYSTEM SHALL NOT
	"complex" |        // WHILE ... WHEN ... THE SYSTEM SHALL
	"inversion" |      // What could fail?
	"premortem"        // Why did this fail?

#AnswerFormat:
	"ears_statement" |  // Expects EARS-formatted requirement
	"free_text" |       // Free-form text answer
	"yes_no" |          // Boolean answer
	"multiple_choice" | // Pick from options
	"list"              // Multiple items

#ConditionalFollowUp: {
	condition: string   // Pattern to match in answer
	question:  #Question
}

// =============================================================================
// ANSWERS (AI feeds these back to CLI)
// =============================================================================

#AnswerSubmission: {
	// Which question this answers
	question_id: string

	// The human's raw answer
	raw_answer: string

	// AI's parsed interpretation (optional, CLI validates)
	parsed?: {
		pattern?:   #EARSPattern
		trigger?:   string
		state?:     string
		condition?: string
		shall?:     string
		shall_not?: string
	}

	// Confidence level (AI can indicate uncertainty)
	confidence: "high" | "medium" | "low"

	// If low confidence, why?
	uncertainty_reason?: string
}

#ValidationError: {
	field:      string
	message:    string
	suggestion: string
}

// =============================================================================
// COLLECTED REQUIREMENTS (running state)
// =============================================================================

#CollectedRequirements: {
	// EARS requirements by pattern
	ubiquitous: [...#EARSRequirement]
	event_driven: [...#EARSRequirement]
	state_driven: [...#EARSRequirement]
	optional: [...#EARSRequirement]
	unwanted: [...#EARSRequirement]
	complex: [...#EARSRequirement]

	// KIRK additions
	inversions: {
		security: [...string]
		usability: [...string]
		integration: [...string]
	}
	premortem: {
		assumed_failure: string
		likely_causes: [...#LikelyCause]
	}
}

#EARSRequirement: {
	id:        string
	pattern:   #EARSPattern
	trigger?:  string
	state?:    string
	condition?: string
	shall:     string
	shall_not?: string
	raw_text:  string
}

#LikelyCause: {
	cause:       string
	probability: "high" | "medium" | "low"
	mitigation:  string
}

// =============================================================================
// INTERVIEW SESSION
// =============================================================================

#InterviewSession: {
	// Session identifier
	id: string

	// When started
	started_at: string

	// Current directive (what AI should do now)
	current: #AIDirective

	// History of Q&A
	history: [...#QAPair]

	// Session metadata
	metadata: {
		project_name: string
		api_name:     string
		description:  string
	}
}

#QAPair: {
	question:   #Question
	answer:     #AnswerSubmission
	asked_at:   string
	answered_at: string
}
