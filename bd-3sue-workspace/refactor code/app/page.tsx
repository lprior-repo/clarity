"use client"

import { useState, useCallback } from "react"
import { PlanningCoach } from "@/components/planning-coach"
import type { Answer } from "@/components/planning-coach"
import { ArtifactPanel } from "@/components/artifact-panel"
import { GraphVisualizer } from "@/components/graph-visualizer"
import { StateMachine } from "@/components/state-machine"
import { PROMPT_STEPS, getStepsForPhase } from "@/lib/prompts"

const PHASES = [
  { key: "discover", label: "Discover" },
  { key: "define", label: "Define" },
  { key: "develop", label: "Develop" },
  { key: "deliver", label: "Deliver" },
] as const

type RightTab = "plan" | "graph" | "state"

const TABS: { key: RightTab; label: string }[] = [
  { key: "plan", label: "Plan" },
  { key: "graph", label: "Graph" },
  { key: "state", label: "State" },
]

function phaseDone(key: string, answers: Answer[]) {
  const steps = getStepsForPhase(key).filter((s) => s.required)
  if (steps.length === 0) return false
  return steps.every((s) => answers.some((a) => a.stepId === s.id))
}

export default function Page() {
  const [activePhase, setActivePhase] = useState("discover")
  const [answers, setAnswers] = useState<Answer[]>([])
  const [rightTab, setRightTab] = useState<RightTab>("plan")

  const handleAnswer = useCallback((stepId: string, value: string) => {
    setAnswers((prev) => [
      ...prev.filter((a) => a.stepId !== stepId),
      { stepId, value, timestamp: Date.now() },
    ])
  }, [])

  const totalRequired = PROMPT_STEPS.filter((s) => s.required).length
  const totalDone = PROMPT_STEPS.filter(
    (s) => s.required && answers.some((a) => a.stepId === s.id),
  ).length

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background">
      {/* Top bar */}
      <header className="flex shrink-0 items-center justify-between border-b border-border px-5 py-2">
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-2">
            <div className="flex h-6 w-6 items-center justify-center rounded-md bg-primary">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" className="text-primary-foreground">
                <circle cx="4" cy="4" r="2" fill="currentColor" />
                <circle cx="10" cy="4" r="2" fill="currentColor" />
                <circle cx="7" cy="10" r="2" fill="currentColor" />
                <path d="M4 4L10 4M4 4L7 10M10 4L7 10" stroke="currentColor" strokeWidth="1" opacity="0.5" />
              </svg>
            </div>
            <span className="text-sm font-bold tracking-tight text-foreground">Beads Planner</span>
          </div>

          <nav className="flex items-center" aria-label="Planning phases">
            {PHASES.map((phase, i) => {
              const done = phaseDone(phase.key, answers)
              const active = activePhase === phase.key
              return (
                <button
                  key={phase.key}
                  type="button"
                  onClick={() => setActivePhase(phase.key)}
                  className={`relative flex items-center gap-1.5 px-3 py-2 text-sm transition-colors ${active ? "text-foreground" : "text-muted-foreground hover:text-foreground/70"}`}
                >
                  {done ? (
                    <svg width="14" height="14" viewBox="0 0 14 14" fill="none" className="text-chart-2">
                      <path d="M3.5 7L6 9.5L10.5 4.5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
                    </svg>
                  ) : (
                    <span className={`flex h-4 w-4 items-center justify-center rounded-full text-xs ${active ? "bg-primary/20 text-primary" : "bg-secondary text-muted-foreground"}`}>
                      {i + 1}
                    </span>
                  )}
                  <span className={active ? "font-medium" : ""}>{phase.label}</span>
                  {active && <span className="absolute inset-x-0 -bottom-[9px] h-0.5 bg-primary" />}
                </button>
              )
            })}
          </nav>
        </div>

        <span className="font-mono text-xs text-muted-foreground">{totalDone}/{totalRequired}</span>
      </header>

      {/* Main content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left: Coach with inline terminal */}
        <main className="flex-1 overflow-hidden border-r border-border">
          <PlanningCoach
            activePhase={activePhase}
            answers={answers}
            onAnswer={handleAnswer}
            onPhaseChange={setActivePhase}
          />
        </main>

        {/* Right: Tabbed panel */}
        <div className="flex w-[440px] shrink-0 flex-col lg:w-[500px]">
          <div className="flex shrink-0 items-center border-b border-border">
            {TABS.map((tab) => (
              <button
                key={tab.key}
                type="button"
                onClick={() => setRightTab(tab.key)}
                className={`relative flex items-center gap-1.5 px-4 py-2.5 text-xs font-medium transition-colors ${rightTab === tab.key ? "text-foreground" : "text-muted-foreground hover:text-foreground/70"}`}
              >
                {tab.key === "graph" ? (
                  <svg width="12" height="12" viewBox="0 0 16 16" fill="none" className="shrink-0">
                    <circle cx="4" cy="4" r="2" stroke="currentColor" strokeWidth="1.2" />
                    <circle cx="12" cy="4" r="2" stroke="currentColor" strokeWidth="1.2" />
                    <circle cx="8" cy="12" r="2" stroke="currentColor" strokeWidth="1.2" />
                    <path d="M5.5 5.5L7 10.5M10.5 5.5L9 10.5" stroke="currentColor" strokeWidth="1" opacity="0.5" />
                  </svg>
                ) : tab.key === "state" ? (
                  <svg width="12" height="12" viewBox="0 0 16 16" fill="none" className="shrink-0">
                    <rect x="2" y="2" width="5" height="5" rx="1" stroke="currentColor" strokeWidth="1.2" />
                    <rect x="9" y="9" width="5" height="5" rx="1" stroke="currentColor" strokeWidth="1.2" />
                    <path d="M7 4.5H9.5V9.5H11.5" stroke="currentColor" strokeWidth="1" strokeLinecap="round" />
                  </svg>
                ) : (
                  <svg width="12" height="12" viewBox="0 0 16 16" fill="none" className="shrink-0">
                    <rect x="2" y="2" width="12" height="12" rx="2" stroke="currentColor" strokeWidth="1.2" />
                    <path d="M5 6H11M5 8.5H9M5 11H7" stroke="currentColor" strokeWidth="1" strokeLinecap="round" opacity="0.6" />
                  </svg>
                )}
                {tab.label}
                {rightTab === tab.key && <span className="absolute inset-x-0 -bottom-px h-0.5 bg-primary" />}
              </button>
            ))}
          </div>

          <div className="flex-1 overflow-hidden">
            {rightTab === "plan" && <ArtifactPanel answers={answers} activePhase={activePhase} />}
            {rightTab === "graph" && <GraphVisualizer answers={answers} />}
            {rightTab === "state" && <StateMachine answers={answers} activePhase={activePhase} />}
          </div>
        </div>
      </div>
    </div>
  )
}
