export interface PromptStep {
  id: string
  phase: "discover" | "define" | "develop" | "deliver"
  title: string
  question: string
  hint: string
  required: boolean
  followUp?: string
}

export const PROMPT_STEPS: PromptStep[] = [
  // ── Discover ──────────────────────────
  {
    id: "problem",
    phase: "discover",
    title: "The Problem",
    question: "What specific problem are your users facing right now?",
    hint: "Be concrete. Not 'auth is hard' but 'developers manually copy API tokens and manage expiry, causing production auth errors'.",
    required: true,
    followUp:
      "Good start. Now let's stress-test that -- why might this NOT actually be a problem worth solving?",
  },
  {
    id: "antithesis",
    phase: "discover",
    title: "The Antithesis",
    question:
      "Why might the current situation actually be fine? What's the strongest argument against building this?",
    hint: "This is the null hypothesis. If you can't argue against yourself, you haven't thought hard enough.",
    required: true,
    followUp:
      "That tension is exactly what shapes a sharp product. Now -- what are you building to resolve it?",
  },
  {
    id: "solution",
    phase: "discover",
    title: "Your Solution",
    question:
      "Given that tension between the problem and the antithesis, what are you actually building? One clear sentence.",
    hint: "Focus on what the software DOES, not what it IS. 'Automatically rotates and injects API tokens at deploy time' not 'A token management platform'.",
    required: true,
    followUp: "Clear. Now who is this for?",
  },
  {
    id: "persona",
    phase: "discover",
    title: "Target User",
    question:
      "Who is the primary person using this? Role, daily workflow, technical level, and what tools they already use.",
    hint: "Think role, not name. 'Solo dev shipping 3 side projects, uses Next.js + Vercel, comfortable with CLI but forgets to rotate keys.'",
    required: true,
    followUp:
      "Good. Let's ground all of this in a concrete story before we move to requirements.",
  },
  {
    id: "scenario",
    phase: "discover",
    title: "North Star Scenario",
    question:
      "Write a short story: your user hits the problem and uses your software to solve it. Walk through their exact steps.",
    hint: "Make it a narrative. Include the trigger, the friction, them using your tool, and the outcome. Flag any steps that feel hand-wavy -- those are your unknowns.",
    required: true,
    followUp:
      "You've got a thesis, a user, and a story. Ready to extract the actual requirements from this.",
  },

  // ── Define ────────────────────────────
  {
    id: "use-cases",
    phase: "define",
    title: "Use Cases",
    question:
      "Let's extract capabilities from your scenario. For each one, write: '[User] can [action] so that [outcome]'. One per line.",
    hint: "Walk through your scenario line by line. Every time the user DOES something, that's a use case. Start with must-haves, then nice-to-haves.",
    required: true,
    followUp:
      "Now let's give the AI agent the technical context it needs to actually build these.",
  },
  {
    id: "constraints",
    phase: "define",
    title: "Constraints & Stack",
    question:
      "What's the tech stack, architecture constraints, and anything the agent must know? Languages, frameworks, existing code patterns, deployment target.",
    hint: "The more specific you are, the better the agent performs. 'Next.js 16, TypeScript strict, Supabase for auth, deployed on Vercel, monorepo with pnpm.'",
    required: true,
    followUp:
      "Requirements locked. Time to break this into tasks an agent can execute.",
  },

  // ── Develop ───────────────────────────
  {
    id: "tasks",
    phase: "develop",
    title: "Task Breakdown",
    question:
      "Break your use cases into implementation tasks. Each should be completable in one agent session. Format: 'module: what it does'. One per line.",
    hint: "Think in small, testable units. 'auth: implement login endpoint', 'auth: add token refresh middleware', 'ui: build login form'. Mark dependencies with 'blocks:' if needed.",
    required: true,
    followUp:
      "Good decomposition. You can click any task in the sidebar to add acceptance criteria, edge cases, and test specs before handing off.",
  },

  // ── Deliver ───────────────────────────
  {
    id: "review",
    phase: "deliver",
    title: "Final Review",
    question:
      "Look at your plan in the sidebar. Any tasks missing acceptance criteria? Any unclear dependencies? Any edge cases you haven't considered?",
    hint: "This is your last checkpoint. The quality of your task specs directly determines agent output quality.",
    required: false,
    followUp: "Plan looks solid. Ready for agent handoff.",
  },
]

export function getStepsForPhase(phase: string) {
  return PROMPT_STEPS.filter((s) => s.phase === phase)
}
