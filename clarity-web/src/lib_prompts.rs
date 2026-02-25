#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_panics_doc)]

use serde::{Deserialize, Serialize};

/// A single prompt step in the planning process
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PromptStep {
    pub id: String,
    #[serde(rename = "phase")]
    pub phase_val: String,
    pub title: String,
    pub question: String,
    pub hint: String,
    pub required: bool,
    #[serde(rename = "followUp", skip_serializing_if = "Option::is_none")]
    pub follow_up: Option<String>,
}

impl PromptStep {
    /// Get the phase as a static string
    pub fn phase(&self) -> &'static str {
        match self.phase_val.as_str() {
            "discover" => "discover",
            "define" => "define",
            "develop" => "develop",
            "deliver" => "deliver",
            _ => "discover",
        }
    }
}

/// Lazy-initialized prompt steps
pub fn prompt_steps() -> &'static [PromptStep] {
    use std::sync::OnceLock;
    static STEPS: OnceLock<Vec<PromptStep>> = OnceLock::new();
    STEPS.get_or_init(|| {
        vec![
            // ── Discover ──────────────────────────
            PromptStep {
                id: "problem".to_string(),
                phase_val: "discover".to_string(),
                title: "The Problem".to_string(),
                question: "What specific problem are your users facing right now?".to_string(),
                hint: "Be concrete. Not 'auth is hard' but 'developers manually copy API tokens and manage expiry, causing production auth errors'.".to_string(),
                required: true,
                follow_up: Some("Good start. Now let's stress-test that -- why might this NOT actually be a problem worth solving?".to_string()),
            },
            PromptStep {
                id: "antithesis".to_string(),
                phase_val: "discover".to_string(),
                title: "The Antithesis".to_string(),
                question: "Why might the current situation actually be fine? What's the strongest argument against building this?".to_string(),
                hint: "This is the null hypothesis. If you can't argue against yourself, you haven't thought hard enough.".to_string(),
                required: true,
                follow_up: Some("That tension is exactly what shapes a sharp product. Now -- what are you building to resolve it?".to_string()),
            },
            PromptStep {
                id: "solution".to_string(),
                phase_val: "discover".to_string(),
                title: "Your Solution".to_string(),
                question: "Given that tension between the problem and the antithesis, what are you actually building? One clear sentence.".to_string(),
                hint: "Focus on what the software DOES, not what it IS. 'Automatically rotates and injects API tokens at deploy time' not 'A token management platform'.".to_string(),
                required: true,
                follow_up: Some("Clear. Now who is this for?".to_string()),
            },
            PromptStep {
                id: "persona".to_string(),
                phase_val: "discover".to_string(),
                title: "Target User".to_string(),
                question: "Who is the primary person using this? Role, daily workflow, technical level, and what tools they already use.".to_string(),
                hint: "Think role, not name. 'Solo dev shipping 3 side projects, uses Next.js + Vercel, comfortable with CLI but forgets to rotate keys.'.".to_string(),
                required: true,
                follow_up: Some("Good. Let's ground all of this in a concrete story before we move to requirements.".to_string()),
            },
            PromptStep {
                id: "scenario".to_string(),
                phase_val: "discover".to_string(),
                title: "North Star Scenario".to_string(),
                question: "Write a short story: your user hits the problem and uses your software to solve it. Walk through their exact steps.".to_string(),
                hint: "Make it a narrative. Include the trigger, the friction, them using your tool, and the outcome. Flag any steps that feel hand-wavy -- those are your unknowns.".to_string(),
                required: true,
                follow_up: Some("You've got a thesis, a user, and a story. Ready to extract the actual requirements from this.".to_string()),
            },

            // ── Define ────────────────────────────
            PromptStep {
                id: "use-cases".to_string(),
                phase_val: "define".to_string(),
                title: "Use Cases".to_string(),
                question: "Let's extract capabilities from your scenario. For each one, write: '[User] can [action] so that [outcome]'. One per line.".to_string(),
                hint: "Walk through your scenario line by line. Every time the user DOES something, that's a use case. Start with must-haves, then nice-to-haves.".to_string(),
                required: true,
                follow_up: Some("Now let's give the AI agent the technical context it needs to actually build these.".to_string()),
            },
            PromptStep {
                id: "constraints".to_string(),
                phase_val: "define".to_string(),
                title: "Constraints & Stack".to_string(),
                question: "What's the tech stack, architecture constraints, and anything the agent must know? Languages, frameworks, existing code patterns, deployment target.".to_string(),
                hint: "The more specific you are, the better the agent performs. 'Next.js 16, TypeScript strict, Supabase for auth, deployed on Vercel, monorepo with pnpm.'.".to_string(),
                required: true,
                follow_up: Some("Requirements locked. Time to break this into tasks an agent can execute.".to_string()),
            },

            // ── Develop ───────────────────────────
            PromptStep {
                id: "tasks".to_string(),
                phase_val: "develop".to_string(),
                title: "Task Breakdown".to_string(),
                question: "Break your use cases into implementation tasks. Each should be completable in one agent session. Format: 'module: what it does'. One per line.".to_string(),
                hint: "Think in small, testable units. 'auth: implement login endpoint', 'auth: add token refresh middleware', 'ui: build login form'. Mark dependencies with 'blocks:' if needed.".to_string(),
                required: true,
                follow_up: Some("Good decomposition. You can click any task in the sidebar to add acceptance criteria, edge cases, and test specs before handing off.".to_string()),
            },

            // ── Deliver ───────────────────────────
            PromptStep {
                id: "review".to_string(),
                phase_val: "deliver".to_string(),
                title: "Final Review".to_string(),
                question: "Look at your plan in the sidebar. Any tasks missing acceptance criteria? Any unclear dependencies? Any edge cases you haven't considered?".to_string(),
                hint: "This is your last checkpoint. The quality of your task specs directly determines agent output quality.".to_string(),
                required: false,
                follow_up: Some("Plan looks solid. Ready for agent handoff.".to_string()),
            },
        ]
    })
}

/// Get all steps for a specific phase
pub fn get_steps_for_phase(phase: &str) -> Vec<&'static PromptStep> {
    prompt_steps()
        .iter()
        .filter(|s| s.phase_val == phase)
        .collect()
}

/// Get a step by its ID
pub fn get_step_by_id(id: &str) -> Option<&'static PromptStep> {
    prompt_steps().iter().find(|s| s.id == id)
}
