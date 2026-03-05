// AI Planning Protocol Schema
// Contract for machine-readable planning directives and execution guidance
package intent

// ============================================================================
// Shared Types
// ============================================================================

#Profile: "api" | "cli" | "event" | "data" | "workflow" | "ui"

#Stage: "discovery" | "refinement" | "validation" | "complete" | "paused"

#SessionRef: {
  id:         string
  profile:    #Profile
  created_at: string
  updated_at: string
  stage:      #Stage
}

#QuestionDirective: {
  id:          string
  round:       int & >=1 & <=5
  text:        string
  pattern:     "ubiquitous" | "event_driven" | "state_driven" | "optional" | "unwanted" | "complex"
  context:     string
  examples:    [...string]
  priority:    "critical" | "important" | "nice_to_have"
  perspective: "user" | "developer" | "ops" | "security" | "business"
  extract_into: [...string]
}

#RiskLevel: "low" | "medium" | "high"

#PlanSummary: {
  session_id:           string
  total_beads:          int & >=0
  total_effort:         string
  risk:                 #RiskLevel
  phase_count:          int & >=0
  critical_path_phases: int & >=0
  blockers:             [...string]
}

#Handoff: {
  why_this_plan:          string
  changed_since_last_run: string
}

// ============================================================================
// plan-work Contract
// ============================================================================

#PlanWorkAction:
  "ask_clarification" |
  "emit_plan" |
  "emit_beads" |
  "blocked" |
  "validation_error" |
  "write_error"

#PlanWorkDirective: {
  action:           #PlanWorkAction
  contract_version: string
  session:          #SessionRef
  planning_focus:   string
  assumptions:      [...string]
  open_questions:   [...string]
  risks:            [...string]
  acceptance_tests: [...string]
  handoff:          #Handoff
  next_commands:    [...string]

  // Present when action = ask_clarification
  question?: #QuestionDirective

  // Present when action = emit_plan
  plan?: #PlanSummary
}

#PlanWorkError: {
  action: #PlanWorkAction & ("validation_error" | "write_error")
  error: {
    message:    string
    suggestion: string
  }
}

// ============================================================================
// plan-next Contract
// ============================================================================

#PlanNextAction: "execute_bead" | "blocked" | "done"

#PlanBead: {
  id:       string
  title:    string
  requires: [...string]
  effort:   "5min" | "10min" | "15min" | "20min" | "30min"
  status:   "pending" | "in_progress" | "blocked" | "completed" | "failed"
}

#PlanNextDirective: {
  action:     #PlanNextAction
  session_id: string
  rationale:  string

  // Present for actionable/blocked directives
  phase?: int & >=1
  bead?:  #PlanBead

  // Present when action = execute_bead
  claim_command?: string

  // Present when action = blocked
  next_command?: string
}

// ============================================================================
// plan-emit-beads Contract
// ============================================================================

#PlanEmitBeadsDirective: {
  session_id:       string
  target:           "br"
  dry_run:          bool
  bead_count:       int & >=0
  commands:         [...string]
  executed_results: [...string]
}
