// Clarity closed event envelope and payload schema.
// Source of truth: MASTER_DOC.md sections 18, 19, and 19A.

package schema

#Actor: "User" | "AiProvider" | "Reviewer" | "System" | "Bd"

#EventType:
	"InterviewStarted" |
	"UserAnswerRecorded" |
	"SkipAttempted" |
	"PhaseCompleted" |
	"NormalQuestioningFrozen" |
	"InterviewExhausted" |
	"SpecCompleted" |
	"InterviewAborted" |
	"RecoveredDegraded" |
	"ArtifactCompileRequested" |
	"BeadGenerationRequested" |
	"SessionLockAcquired" |
	"SessionLockRefreshed" |
	"SessionLockReleased" |
	"SessionLockStolenAfterExpiry" |
	"SessionLockRejected" |
	"AiCallRequested" |
	"AiQuestionRecorded" |
	"AiExtractionSucceeded" |
	"AiReviewSucceeded" |
	"AiSummarySucceeded" |
	"AiJsonRepairSucceeded" |
	"AiJsonRepairFailed" |
	"AiCallFailed" |
	"AiEffectAmbiguous" |
	"GatePassed" |
	"GateFailed" |
	"GateRecheckRequested" |
	"PmeVorpScored" |
	"PmeFailureCategoriesMapped" |
	"PmeHumanLimitationsModeled" |
	"PmeCdiEvidenceRecorded" |
	"PmeNfrTradeoffsRecorded" |
	"ReviewerPanelStarted" |
	"ReviewerOutputRecorded" |
	"ReviewerPassed" |
	"ReviewerFailed" |
	"ReviewerOutputInvalid" |
	"ReviewerRepairQuestionRecorded" |
	"KirkCompiled" |
	"CueSpecCompiled" |
	"CueSpecValidated" |
	"EnhancedBeadsGenerated" |
	"EnhancedBeadsValidated" |
	"JsonlProjectionWritten" |
	"JsonlProjectionFailed" |
	"RawExportRequested" |
	"PrivacyConsentRecorded" |
	"RawExportWritten" |
	"SanitizedExportWritten" |
	"BdEmitStarted" |
	"BdCreateRequested" |
	"BdBeadCreated" |
	"BdBeadSkippedExisting" |
	"BdEmitPartial" |
	"BdEmitCompleted" |
	"BdEmitFailed"

#BaseEventEnvelope: {
	session_id: string
	seq: int & >=1
	event_id: string
	event_type: #EventType
	created_at: string
	idempotency_key: string
	schema_version: string
	actor: #Actor
	prev_event_hash: string | null
	event_hash?: string
	...
}

#SessionEventType:
	"InterviewStarted" |
	"SkipAttempted" |
	"PhaseCompleted" |
	"NormalQuestioningFrozen" |
	"InterviewExhausted" |
	"SpecCompleted" |
	"InterviewAborted" |
	"RecoveredDegraded" |
	"ArtifactCompileRequested" |
	"BeadGenerationRequested"

#LockEventType:
	"SessionLockAcquired" |
	"SessionLockRefreshed" |
	"SessionLockReleased" |
	"SessionLockStolenAfterExpiry" |
	"SessionLockRejected"

#AiTerminalEventType:
	"AiQuestionRecorded" |
	"AiExtractionSucceeded" |
	"AiReviewSucceeded" |
	"AiSummarySucceeded" |
	"AiJsonRepairSucceeded" |
	"AiJsonRepairFailed" |
	"AiCallFailed" |
	"AiEffectAmbiguous"

#GateEventType:
	"GatePassed" |
	"GateFailed" |
	"GateRecheckRequested"

#PmeEventType:
	"PmeVorpScored" |
	"PmeFailureCategoriesMapped" |
	"PmeHumanLimitationsModeled" |
	"PmeCdiEvidenceRecorded" |
	"PmeNfrTradeoffsRecorded"

#ReviewerEventType:
	"ReviewerPanelStarted" |
	"ReviewerOutputRecorded" |
	"ReviewerPassed" |
	"ReviewerFailed" |
	"ReviewerOutputInvalid" |
	"ReviewerRepairQuestionRecorded"

#ArtifactEventType:
	"KirkCompiled" |
	"CueSpecCompiled" |
	"CueSpecValidated" |
	"EnhancedBeadsGenerated" |
	"EnhancedBeadsValidated"

#ProjectionEventType:
	"JsonlProjectionWritten" |
	"JsonlProjectionFailed" |
	"RawExportRequested" |
	"PrivacyConsentRecorded" |
	"RawExportWritten" |
	"SanitizedExportWritten"

#BdTerminalEventType:
	"BdBeadCreated" |
	"BdBeadSkippedExisting" |
	"BdEmitPartial" |
	"BdEmitCompleted" |
	"BdEmitFailed"

#EventEnvelope: #BaseEventEnvelope & ({
	event_type: #SessionEventType
	payload: #SessionPayload
} | {
	event_type: "UserAnswerRecorded"
	payload: #AnswerPayload
} | {
	event_type: #LockEventType
	payload: #LockPayload
} | {
	event_type: "AiCallRequested"
	payload: #AiCallRequestedPayload
} | {
	event_type: #AiTerminalEventType
	payload: #AiTerminalPayload
} | {
	event_type: #GateEventType
	payload: #GatePayload
} | {
	event_type: #PmeEventType
	payload: #GatePayload
} | {
	event_type: #ReviewerEventType
	payload: #ReviewerPayload
} | {
	event_type: #ArtifactEventType
	payload: #ArtifactPayload
} | {
	event_type: #ProjectionEventType
	payload: #ProjectionPayload
} | {
	event_type: "BdEmitStarted"
	payload: #SessionPayload
} | {
	event_type: "BdCreateRequested"
	payload: #BdCreateRequestedPayload
} | {
	event_type: #BdTerminalEventType
	payload: #BdTerminalPayload
})

#EventPayload:
	#SessionPayload |
	#AnswerPayload |
	#GatePayload |
	#ArtifactPayload |
	#AiCallRequestedPayload |
	#AiTerminalPayload |
	#ReviewerPayload |
	#ProjectionPayload |
	#LockPayload |
	#BdCreateRequestedPayload |
	#BdTerminalPayload

#SessionPayload: {
	kind: "session"
	command: string
	source_state?: string
	destination_state?: string
}

#AnswerPayload: {
	kind: "answer"
	question_id: string
	question_kind: "normal" | "reviewer-repair"
	normalized_answer_hash: string & =~"^sha256:[a-f0-9]{64}$"
	raw_answer_ref?: string
	supported_gate_ids: [...string]
}

#GatePayload: {
	kind: "gate"
	gate_id: string
	passed: bool
	threshold?: number
	score?: number
	evaluator_version: string
	input_event_ids: [...string]
	failure_reasons: [...string] | *[]
}

#ArtifactPayload: {
	kind: "artifact"
	artifact_kind: "kirk16" | "cue-spec" | "enhanced-beads" | "reviewer-output"
	artifact_id: string
	schema_hash: string & =~"^sha256:[a-f0-9]{64}$"
	payload_hash: string & =~"^sha256:[a-f0-9]{64}$"
	source_event_prefix_hash: string & =~"^sha256:[a-f0-9]{64}$"
	validated: bool
}

#AiCallRequestedPayload: {
	kind: "ai-call-requested"
	effect_id: string
	operation: "ask_question" | "extract_fields" | "review_artifact" | "repair_json" | "summarize" | "health_check"
	provider: string
	model?: string
	attempt_no: int & >=1
	max_attempts: int & >=1
	schema_hash: string
	prompt_hash: string & =~"^sha256:[a-f0-9]{64}$"
	redaction_policy_version: string
	secret_scan_result: "pass"
	input_event_ids: [...string]
	timeout_ms: int & >0
}

#AiTerminalPayload: {
	kind: "ai-terminal"
	effect_id: string
	request_event_id: string
	attempt_no: int & >=1
	response_hash?: string & =~"^sha256:[a-f0-9]{64}$"
	failure_category?: string
	validation_result: "pass" | "fail"
}

#ReviewerPayload: {
	kind: "reviewer"
	reviewer: "sre" | "architect" | "wardley" | "munger" | "security" | "test"
	reviewed_event_prefix_hash: string & =~"^sha256:[a-f0-9]{64}$"
	evidence_validation: "pass" | "fail" | "not-run"
}

#ProjectionPayload: {
	kind: "projection"
	destination_path_hash: string
	redaction_policy_version: string
	source_seq: int & >=0
	source_event_prefix_hash: string & =~"^sha256:[a-f0-9]{64}$"
	mode: "raw" | "sanitized"
}

#LockPayload: {
	kind: "lock"
	owner_token_hash: string
	expires_at: string
	compare_and_set_result: "accepted" | "rejected"
}

#BdCreateRequestedPayload: {
	kind: "bd-create-requested"
	local_bead_id: string
	canonical_task_id: string
	content_hash: string & =~"^sha256:[a-f0-9]{64}$"
	schema_hash: string & =~"^sha256:[a-f0-9]{64}$"
	bd_command_args_hash: string & =~"^sha256:[a-f0-9]{64}$"
	idempotency_key: string
	privacy_consent_event_id: string
	redaction_policy_version: string
}

#BdTerminalPayload: {
	kind: "bd-terminal"
	request_event_id: string
	local_bead_id: string
	bd_id?: string
	content_hash?: string & =~"^sha256:[a-f0-9]{64}$"
	result: "created" | "skipped" | "partial" | "failed"
}
