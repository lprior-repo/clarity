// ─── Double Diamond Phase Types ──────────────────────

export type DiamondPhase = "discover" | "define" | "develop" | "deliver"

// ─── Discover Phase ──────────────────────────────────

export interface ProductThesis {
  problem: string
  solution: string
  antithesis: string // why it might fail
}

export interface Persona {
  name: string
  description: string
  means: string // resources/skills they have
  isNonpersona?: boolean // who we're NOT building for
}

export interface NorthStarScenario {
  title: string
  story: string // the simulation narrative
  persona: string // name reference
}

// ─── Define Phase ────────────────────────────────────

export interface UseCase {
  id: string
  persona: string
  action: string
  motivation: string // "[persona] can [action] so that [motivation]"
  northStar: string // which scenario it came from
  priority: "must" | "should" | "could"
}

// ─── Planning Session Types ──────────────────────────

export type SessionStatus =
  | "discover"
  | "define"
  | "develop"
  | "deliver"

export type TaskType = "feature" | "bug" | "task" | "epic" | "chore"
export type TaskPriority = 0 | 1 | 2 | 3 | 4
export type Effort = "15min" | "30min" | "1hr" | "2hr" | "4hr"

// ─── EARS Requirements ───────────────────────────────

export interface EarsUbiquitous {
  text: string
}

export interface EarsEventDriven {
  trigger: string
  response: string
}

export interface EarsUnwanted {
  condition: string
  shallNot: string
  because: string
}

export interface EarsRequirements {
  ubiquitous: EarsUbiquitous[]
  eventDriven: EarsEventDriven[]
  unwanted: EarsUnwanted[]
}

// ─── KIRK Contracts ──────────────────────────────────

export interface Contracts {
  preconditions: string[]
  postconditions: string[]
  invariants: string[]
}

// ─── Tests ───────────────────────────────────────────

export interface Tests {
  happy: string[]
  error: string[]
  edge: string[]
}

// ─── Research ────────────────────────────────────────

export interface Research {
  files: string[]
  patterns: string[]
  questions: string[]
}

// ─── Implementation ──────────────────────────────────

export interface Implementation {
  phase0: string[] // research
  phase1: string[] // tests
  phase2: string[] // implementation
}

// ─── Task (a single bead to create) ──────────────────

export interface PlanTask {
  id: string
  title: string
  type: TaskType
  priority: TaskPriority
  effort: Effort
  description: string
  dependsOn: string[]
  ears: EarsRequirements
  contracts: Contracts
  tests: Tests
  research: Research
  implementation: Implementation
}

// ─── Validation ──────────────────────────────────────

export interface ValidationCheck {
  label: string
  passed: boolean
  severity: "error" | "warning"
}

export function validateTask(task: PlanTask): ValidationCheck[] {
  return [
    { label: "Title follows 'component: action' format", passed: task.title.includes(":"), severity: "warning" },
    { label: "Has ubiquitous requirements", passed: task.ears.ubiquitous.length >= 1, severity: "error" },
    { label: "Has event-driven requirements", passed: task.ears.eventDriven.length >= 1, severity: "error" },
    { label: "Has unwanted requirements", passed: task.ears.unwanted.length >= 1, severity: "error" },
    { label: "Preconditions defined", passed: task.contracts.preconditions.length >= 1, severity: "error" },
    { label: "Postconditions defined", passed: task.contracts.postconditions.length >= 1, severity: "error" },
    { label: "Invariants defined", passed: task.contracts.invariants.length >= 1, severity: "error" },
    { label: "Has happy path tests", passed: task.tests.happy.length >= 1, severity: "error" },
    { label: "Has error path tests", passed: task.tests.error.length >= 1, severity: "error" },
    { label: "Has edge case tests", passed: task.tests.edge.length >= 1, severity: "warning" },
    { label: "Research files identified", passed: task.research.files.length >= 1, severity: "warning" },
    { label: "Implementation phases defined", passed: task.implementation.phase0.length + task.implementation.phase1.length + task.implementation.phase2.length >= 1, severity: "warning" },
  ]
}

export function isTaskReady(task: PlanTask): boolean {
  return validateTask(task).filter((c) => c.severity === "error" && !c.passed).length === 0
}

// ─── Full Session ────────────────────────────────────

export interface PlanSession {
  id: string
  status: SessionStatus
  createdAt: string
  // Discover
  thesis: ProductThesis
  personas: Persona[]
  scenarios: NorthStarScenario[]
  // Define
  useCases: UseCase[]
  context: string
  // Develop
  tasks: PlanTask[]
}

// ─── Graph Health ────────────────────────────────────

export function getGraphHealth(tasks: PlanTask[]) {
  const total = tasks.length
  const ready = tasks.filter(isTaskReady).length
  const effortMap: Record<Effort, number> = { "15min": 15, "30min": 30, "1hr": 60, "2hr": 120, "4hr": 240 }
  const totalEffortMin = tasks.reduce((sum, t) => sum + effortMap[t.effort], 0)
  const hasCircular = false
  const allDepsValid = tasks.every((t) => t.dependsOn.every((d) => tasks.some((o) => o.id === d)))
  return { total, ready, totalEffortMin, hasCircular, allDepsValid }
}
