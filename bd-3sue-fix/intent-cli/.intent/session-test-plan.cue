package testplan

import "schema/intent.cue"

session: #InterviewSession & {
  id: "test-plan"
  profile: #Profile.api
  created_at: "2025-02-09T00:00:00Z"
  updated_at: "2025-02-09T00:00:00Z"
  completed_at: ""
  stage: #InterviewStage.discovery
  rounds_completed: 1
  answers: [
    {
      question_id: "q1"
      question_text: "What are you building?"
      perspective: #Perspective.developer
      round: 1
      response: "User management API"
      extracted: {}
      confidence: 0.9
      notes: ""
      timestamp: "2025-02-09T00:00:00Z"
    }
  ]
  gaps: []
  conflicts: []
  raw_notes: ""
  current_phase: 0
  completed_phases: []
}

// Beads for testing plan-next
beads: [
  {
    id: "bead-1"
    title: "Design user schema"
    requires: []
    effort: "5min"
    status: "pending"
  },
  {
    id: "bead-2"
    title: "Implement user endpoints"
    requires: ["bead-1"]
    effort: "30min"
    status: "pending"
  },
  {
    id: "bead-3"
    title: "Add authentication"
    requires: ["bead-1"]
    effort: "20min"
    status: "pending"
  },
  {
    id: "bead-4"
    title: "Write tests"
    requires: ["bead-2", "bead-3"]
    effort: "15min"
    status: "pending"
  },
  {
    id: "bead-5"
    title: "Documentation"
    requires: ["bead-4"]
    effort: "10min"
    status: "pending"
  }
]
