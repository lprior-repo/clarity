// Clarity canonical Rust question bank.
// Source of truth: MASTER_DOC.md sections 7, 10, and 17A.

package intent

#RustProfile: "rust-cli" | "rust-library" | "rust-web-service" | "rust-async-service" | "rust-storage" | "rust-ui" | "rust-refactor"

#GateRef: {
	gate_id: "required-fields" | "vorp" | "straw-man" | "antithesis" | "hole-punching" | "ears" | "kirk16" | "cue-spec" | "enhanced-bead" | "security" | "nfr" | "reviewer"
	required: bool | *true
}

#Question: {
	id: string & =~"^q-[a-z0-9][a-z0-9-]{4,96}$"
	profile: #RustProfile
	round: 1 | 2 | 3 | 4 | 5
	perspective: "user" | "developer" | "operator" | "security" | "business" | "reviewer"
	category: "happy_path" | "error_case" | "edge_case" | "constraint" | "dependency" | "nonfunctional" | "security" | "verification" | "recovery"
	priority: "critical" | "important" | "nice_to_have"
	question: string
	context: string
	example: string
	expected_type: "text" | "list" | "json" | *"text"
	extract_into: [...string] | *[]
	depends_on: [...string] | *[]
	blocks: [...string] | *[]
	gates: [...#GateRef] & [_, ...]
}

#ProfileQuestions: {
	round_1: [...#Question]
	round_2: [...#Question]
	round_3: [...#Question]
	round_4: [...#Question]
	round_5: [...#Question]
}

#QuestionBank: {
	schema_version: string | *"1.0.0"
	profiles: {
		"rust-cli": #ProfileQuestions
		"rust-library": #ProfileQuestions
		"rust-web-service": #ProfileQuestions
		"rust-async-service": #ProfileQuestions
		"rust-storage": #ProfileQuestions
		"rust-ui": #ProfileQuestions
		"rust-refactor": #ProfileQuestions
	}
}

_baseProfile: #ProfileQuestions & {
	round_1: [{
		id: "q-rust-cli-problem" | "q-rust-library-problem" | "q-rust-web-service-problem" | "q-rust-async-service-problem" | "q-rust-storage-problem" | "q-rust-ui-problem" | "q-rust-refactor-problem"
		profile: #RustProfile
		round: 1
		perspective: "user"
		category: "happy_path"
		priority: "critical"
		question: "What exact Rust work scope are we planning?"
		context: "Name the concrete behavior, boundary, and success condition before implementation."
		example: "Implement a bounded CLI command that validates input and returns stable JSON."
		extract_into: ["problem", "solution"]
		gates: [{gate_id: "required-fields"}]
	}]
	round_2: [{
		id: "q-rust-cli-ears" | "q-rust-library-ears" | "q-rust-web-service-ears" | "q-rust-async-service-ears" | "q-rust-storage-ears" | "q-rust-ui-ears" | "q-rust-refactor-ears"
		profile: #RustProfile
		round: 2
		perspective: "developer"
		category: "verification"
		priority: "critical"
		question: "State at least one ubiquitous, event-driven, and unwanted EARS requirement."
		context: "The plan cannot proceed on vague verbs like handle, manage, or process."
		example: "WHEN stdin closes, THE SYSTEM SHALL return StdinClosed and preserve committed events."
		extract_into: ["ears_requirements"]
		gates: [{gate_id: "ears"}]
	}]
	round_3: [{
		id: "q-rust-cli-failure" | "q-rust-library-failure" | "q-rust-web-service-failure" | "q-rust-async-service-failure" | "q-rust-storage-failure" | "q-rust-ui-failure" | "q-rust-refactor-failure"
		profile: #RustProfile
		round: 3
		perspective: "operator"
		category: "recovery"
		priority: "critical"
		question: "Name the most likely crash, retry, corruption, or partial-side-effect failure for this scope and the exact recovery behavior."
		context: "Recovery is part of the product contract, not cleanup work."
		example: "Crash after request event but before terminal event appends AiEffectAmbiguous on resume."
		extract_into: ["failure_modes"]
		gates: [{gate_id: "hole-punching"}]
	}]
	round_4: [{
		id: "q-rust-cli-security" | "q-rust-library-security" | "q-rust-web-service-security" | "q-rust-async-service-security" | "q-rust-storage-security" | "q-rust-ui-security" | "q-rust-refactor-security"
		profile: #RustProfile
		round: 4
		perspective: "security"
		category: "security"
		priority: "critical"
		question: "What secret, path, injection, data leak, or authorization boundary must this scope never violate?"
		context: "Security questions may not harvest secrets; they must define forbidden outcomes."
		example: "Provider prompts shall not include raw API keys and shall fail with SecretDetectedInProviderRequest."
		extract_into: ["security_requirements"]
		gates: [{gate_id: "security"}]
	}]
	round_5: [{
		id: "q-rust-cli-beads" | "q-rust-library-beads" | "q-rust-web-service-beads" | "q-rust-async-service-beads" | "q-rust-storage-beads" | "q-rust-ui-beads" | "q-rust-refactor-beads"
		profile: #RustProfile
		round: 5
		perspective: "reviewer"
		category: "verification"
		priority: "critical"
		question: "What is the smallest one-behavior bead boundary and required evidence for implementation?"
		context: "A generated bead must be molecular, test-first, and evidence-carrying."
		example: "One bead adds the pure reducer transition and unit/property tests only."
		extract_into: ["bead_boundary", "verification"]
		gates: [{gate_id: "enhanced-bead"}]
	}]
}

question_bank: #QuestionBank & {
	profiles: {
		"rust-cli": _baseProfile & {
			round_1: [{id: "q-rust-cli-problem", profile: "rust-cli"}]
			round_2: [{id: "q-rust-cli-ears", profile: "rust-cli"}]
			round_3: [{id: "q-rust-cli-failure", profile: "rust-cli"}]
			round_4: [{id: "q-rust-cli-security", profile: "rust-cli"}]
			round_5: [{id: "q-rust-cli-beads", profile: "rust-cli"}]
		}
		"rust-library": _baseProfile & {
			round_1: [{id: "q-rust-library-problem", profile: "rust-library"}]
			round_2: [{id: "q-rust-library-ears", profile: "rust-library"}]
			round_3: [{id: "q-rust-library-failure", profile: "rust-library"}]
			round_4: [{id: "q-rust-library-security", profile: "rust-library"}]
			round_5: [{id: "q-rust-library-beads", profile: "rust-library"}]
		}
		"rust-web-service": _baseProfile & {
			round_1: [{id: "q-rust-web-service-problem", profile: "rust-web-service"}]
			round_2: [{id: "q-rust-web-service-ears", profile: "rust-web-service"}]
			round_3: [{id: "q-rust-web-service-failure", profile: "rust-web-service"}]
			round_4: [{id: "q-rust-web-service-security", profile: "rust-web-service"}]
			round_5: [{id: "q-rust-web-service-beads", profile: "rust-web-service"}]
		}
		"rust-async-service": _baseProfile & {
			round_1: [{id: "q-rust-async-service-problem", profile: "rust-async-service"}]
			round_2: [{id: "q-rust-async-service-ears", profile: "rust-async-service"}]
			round_3: [{id: "q-rust-async-service-failure", profile: "rust-async-service"}]
			round_4: [{id: "q-rust-async-service-security", profile: "rust-async-service"}]
			round_5: [{id: "q-rust-async-service-beads", profile: "rust-async-service"}]
		}
		"rust-storage": _baseProfile & {
			round_1: [{id: "q-rust-storage-problem", profile: "rust-storage"}]
			round_2: [{id: "q-rust-storage-ears", profile: "rust-storage"}]
			round_3: [{id: "q-rust-storage-failure", profile: "rust-storage"}]
			round_4: [{id: "q-rust-storage-security", profile: "rust-storage"}]
			round_5: [{id: "q-rust-storage-beads", profile: "rust-storage"}]
		}
		"rust-ui": _baseProfile & {
			round_1: [{id: "q-rust-ui-problem", profile: "rust-ui"}]
			round_2: [{id: "q-rust-ui-ears", profile: "rust-ui"}]
			round_3: [{id: "q-rust-ui-failure", profile: "rust-ui"}]
			round_4: [{id: "q-rust-ui-security", profile: "rust-ui"}]
			round_5: [{id: "q-rust-ui-beads", profile: "rust-ui"}]
		}
		"rust-refactor": _baseProfile & {
			round_1: [{id: "q-rust-refactor-problem", profile: "rust-refactor"}]
			round_2: [{id: "q-rust-refactor-ears", profile: "rust-refactor"}]
			round_3: [{id: "q-rust-refactor-failure", profile: "rust-refactor"}]
			round_4: [{id: "q-rust-refactor-security", profile: "rust-refactor"}]
			round_5: [{id: "q-rust-refactor-beads", profile: "rust-refactor"}]
		}
	}
}
