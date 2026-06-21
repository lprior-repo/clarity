// Clarity reviewer report schema.
// Source of truth: MASTER_DOC.md section 14A.

package schema

#ReviewerName: "sre" | "architect" | "wardley" | "munger" | "security" | "test"

#ReviewerVerdict: "Pass" | "Fail"

#ReviewerFinding: {
	finding_id: string & =~"^[a-z0-9][a-z0-9-]{2,80}$"
	claim: string
	gate_ids: [...string]
	evidence_event_ids: [...string]
	required_questions: [...string] | *[]
	unrecoverable: bool | *false
}

#ReviewerWarning: {
	warning_id: string & =~"^[a-z0-9][a-z0-9-]{2,80}$"
	claim: string
	evidence_event_ids: [...string]
}

#ReviewerPassClaim: {
	gate_id: string
	claim: string
	evidence_event_ids: [...string] & [_, ...]
}

#ReviewerReport: {
	reviewer: #ReviewerName
	verdict: #ReviewerVerdict
	blocking_findings: [...#ReviewerFinding]
	warnings: [...#ReviewerWarning]
	pass_claims: [...#ReviewerPassClaim]
	required_questions: [...string]
	required_gate_rechecks: [...string]
	confidence: number & >=0 & <=1
	reviewed_event_prefix_hash: string & =~"^sha256:[a-f0-9]{64}$"
}

#PassingReviewerReport: #ReviewerReport & {
	verdict: "Pass"
	confidence: >=0.75
	blocking_findings: []
	pass_claims: [_, ...]
}
