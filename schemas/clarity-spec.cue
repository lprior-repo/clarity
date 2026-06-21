// Clarity generated CUE spec schema.
// Source of truth: MASTER_DOC.md sections 15, 16, and 17A.

package schema

#RustProfile: "rust-cli" | "rust-library" | "rust-web-service" | "rust-async-service" | "rust-storage" | "rust-ui" | "rust-refactor"

#RustRequirement: {
	id: string
	kind: "ubiquitous" | "event-driven" | "state-driven" | "optional" | "unwanted" | "complex"
	text: string
	evidence_event_ids: [...string]
}

#GateResult: {
	gate_id: "required-fields" | "vorp" | "straw-man" | "antithesis" | "hole-punching" | "ears" | "kirk16" | "cue-spec" | "enhanced-bead" | string
	passed: bool
	score?: number
	threshold?: number
	evidence_event_ids: [...string]
}

#ClaritySpec: {
	schema_version: string
	session_id: string
	profile: #RustProfile
	requirements: [...#RustRequirement]
	gate_results: [...#GateResult]
	kirk_contract_ref?: string
	enhanced_bead_refs?: [...string]
	metadata: {
		generated_at: string
		source_event_prefix_hash: string & =~"^sha256:[a-f0-9]{64}$"
	}
}
