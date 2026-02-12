//! Planning prompts for the Diamond methodology
//!
//! Questions for each phase of the planning process.
//! Matches v2 `TypeScript` prompts exactly.

use crate::planner::types::DiamondPhase;
use crate::planner::types_coach::{CoachAnswer, CoachStep};

/// Prompt step data: (id, phase, title, question, hint, required, `follow_up`)
const STEPS: &[(&str, &str, &str, &str, &str, bool, &str)] = &[
  // ── DISCOVER ──────────────────────────
  (
    "problem",
    "discover",
    "The Problem",
    "What specific problem are your users facing right now?",
    "Be concrete. Not 'auth is hard' but 'developers manually copy API tokens and manage expiry, causing production auth errors'.",
    true,
    "Good start. Now let's stress-test that -- why might this NOT actually be a problem worth solving?",
  ),
  (
    "antithesis",
    "discover",
    "The Antithesis",
    "Why might the current situation actually be fine? What's the strongest argument against building this?",
    "This is the null hypothesis. If you can't argue against yourself, you haven't thought hard enough.",
    true,
    "That tension is exactly what shapes a sharp product. Now -- what are you building to resolve it?",
  ),
  (
    "solution",
    "discover",
    "Your Solution",
    "Given that tension between the problem and the antithesis, what are you actually building? One clear sentence.",
    "Focus on what the software DOES, not what it IS. 'Automatically rotates and injects API tokens at deploy time' not 'A token management platform'.",
    true,
    "Clear. Now who is this for?",
  ),
  (
    "persona",
    "discover",
    "Target User",
    "Who is the primary person using this? Role, daily workflow, technical level, and what tools they already use.",
    "Think role, not name. 'Solo dev shipping 3 side projects, uses Next.js + Vercel, comfortable with CLI but forgets to rotate keys.'",
    true,
    "Good. Let's ground all of this in a concrete story before we move to requirements.",
  ),
  (
    "scenario",
    "discover",
    "North Star Scenario",
    "Write a short story: your user hits the problem and uses your software to solve it. Walk through their exact steps.",
    "Make it a narrative. Include the trigger, the friction, them using your tool, and the outcome. Flag any steps that feel hand-wavy -- those are your unknowns.",
    true,
    "You've got a thesis, a user, and a story. Ready to extract the actual requirements from this.",
  ),
  // ── DEFINE ────────────────────────────
  (
    "use-cases",
    "define",
    "Use Cases",
    "Let's extract capabilities from your scenario. For each one, write: '[User] can [action] so that [outcome]'. One per line.",
    "Walk through your scenario line by line. Every time the user DOES something, that's a use case. Start with must-haves, then nice-to-haves.",
    true,
    "Now let's give the AI agent the technical context it needs to actually build these.",
  ),
  (
    "constraints",
    "define",
    "Constraints & Stack",
    "What's the tech stack, architecture constraints, and anything the agent must know? Languages, frameworks, existing code patterns, deployment target.",
    "The more specific you are, the better the agent performs. 'Next.js 16, TypeScript strict, Supabase for auth, deployed on Vercel, monorepo with pnpm.'",
    true,
    "Requirements locked. Time to break this into tasks an agent can execute.",
  ),
  // ── DEVELOP ───────────────────────────
  (
    "tasks",
    "develop",
    "Task Breakdown",
    "Break your use cases into implementation tasks. Each should be completable in one agent session. Format: 'module: what it does'. One per line.",
    "Think in small, testable units. 'auth: implement login endpoint', 'auth: add token refresh middleware', 'ui: build login form'. Mark dependencies with 'blocks:' if needed.",
    true,
    "Good decomposition. You can click any task in the sidebar to add acceptance criteria, edge cases, and test specs before handing off.",
  ),
  // ── DELIVER ───────────────────────────
  (
    "review",
    "deliver",
    "Final Review",
    "Look at your plan in the sidebar. Any tasks missing acceptance criteria? Any unclear dependencies? Any edge cases you haven't considered?",
    "This is your last checkpoint. The quality of your task specs directly determines agent output quality.",
    false,
    "Plan looks solid. Ready for agent handoff.",
  ),
];

/// Convert `DiamondPhase` to phase string
const fn phase_to_str(phase: DiamondPhase) -> &'static str {
  match phase {
    DiamondPhase::Top => "discover",
    DiamondPhase::Right => "define",
    DiamondPhase::Bottom => "develop",
    DiamondPhase::Left => "deliver",
  }
}

/// Get steps for a phase
#[must_use]
pub fn get_steps_for_phase(phase: DiamondPhase) -> Vec<CoachStep> {
  let phase_str = phase_to_str(phase);
  STEPS
    .iter()
    .filter(|(_, step_phase, _, _, _, _, _)| *step_phase == phase_str)
    .map(
      |(id, _, title, question, hint, required, follow_up)| CoachStep {
        id: (*id).to_string(),
        step_id: (*id).to_string(),
        title: (*title).to_string(),
        question: format!("{}{}", question, if *required { "" } else { " (optional)" }),
        hint: Some((*hint).to_string()),
        follow_up: Some((*follow_up).to_string()),
      },
    )
    .collect()
}

/// Get steps for phase string
#[must_use]
pub fn get_steps_for_phase_string(phase: &str) -> Vec<CoachStep> {
  STEPS
    .iter()
    .filter(|(_, step_phase, _, _, _, _, _)| *step_phase == phase)
    .map(
      |(id, _, title, question, hint, required, follow_up)| CoachStep {
        id: (*id).to_string(),
        step_id: (*id).to_string(),
        title: (*title).to_string(),
        question: format!("{}{}", question, if *required { "" } else { " (optional)" }),
        hint: Some((*hint).to_string()),
        follow_up: Some((*follow_up).to_string()),
      },
    )
    .collect()
}

/// Check if phase is complete
#[must_use]
pub fn phase_done(phase: &str, answers: &[CoachAnswer]) -> bool {
  let steps = get_steps_for_phase_string(phase);
  steps
    .iter()
    .all(|s| answers.iter().any(|a| a.step_id == s.id))
}

/// Count required steps
#[must_use]
pub fn total_required() -> usize {
  STEPS
    .iter()
    .filter(|(_, _, _, _, _, required, _)| *required)
    .count()
}

/// Count done required steps
#[must_use]
pub fn total_done(answers: &[CoachAnswer]) -> usize {
  STEPS
    .iter()
    .filter(|(id, _, _, _, _, required, _)| *required && answers.iter().any(|a| &a.step_id == *id))
    .count()
}
