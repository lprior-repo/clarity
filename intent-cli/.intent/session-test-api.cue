package testapi

import "schema/intent.cue"

session: #InterviewSession & {
  id: "test-api"
  profile: #Profile.api
  created_at: "2025-02-09T00:00:00Z"
  updated_at: "2025-02-09T00:00:00Z"
  completed_at: ""
  stage: #InterviewStage.discovery
  rounds_completed: 1
  answers: [
    {
      question_id: "q1"
      question_text: "What endpoint are you building?"
      perspective: #Perspective.developer
      round: 1
      response: "GET /api/users"
      extracted: {"endpoint": "/api/users"}
      confidence: 0.9
      notes: ""
      timestamp: "2025-02-09T00:00:00Z"
    },
    {
      question_id: "q2"
      question_text: "What should it return?"
      perspective: #Perspective.developer
      round: 1
      response: "List of users with id and name"
      extracted: {}
      confidence: 0.8
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
