"use client"

import { useState } from "react"
import { DEMO_SESSION } from "@/lib/data"
import type { DiamondPhase, PlanSession } from "@/lib/types"
import { isTaskReady } from "@/lib/types"
import { PhaseDiscover } from "@/components/phase-discover"
import { PhaseDefine } from "@/components/phase-define"
import { PhaseDevelop } from "@/components/phase-develop"
import { PhaseDeliver } from "@/components/phase-deliver"

const PHASES: { key: DiamondPhase; label: string; subtitle: string }[] = [
  { key: "discover", label: "Discover", subtitle: "Why?" },
  { key: "define", label: "Define", subtitle: "What?" },
  { key: "develop", label: "Develop", subtitle: "How?" },
  { key: "deliver", label: "Deliver", subtitle: "Ship" },
]

function PhaseReadiness(session: PlanSession, phase: DiamondPhase): "complete" | "partial" | "empty" {
  switch (phase) {
    case "discover":
      if (session.thesis.problem && session.personas.length >= 2 && session.scenarios.length >= 1 && session.scenarios[0]?.story) return "complete"
      if (session.thesis.problem || session.personas.length > 0) return "partial"
      return "empty"
    case "define":
      if (session.useCases.length >= 3) return "complete"
      if (session.useCases.length > 0) return "partial"
      return "empty"
    case "develop":
      if (session.tasks.length > 0 && session.tasks.every(isTaskReady)) return "complete"
      if (session.tasks.length > 0) return "partial"
      return "empty"
    case "deliver":
      if (session.tasks.length > 0 && session.tasks.every(isTaskReady)) return "complete"
      return "empty"
  }
}

export default function Page() {
  const [session, setSession] = useState<PlanSession>(DEMO_SESSION)
  const [activePhase, setActivePhase] = useState<DiamondPhase>("discover")

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background">
      {/* Top bar */}
      <header className="flex shrink-0 items-center justify-between border-b border-border px-6 py-3">
        <div className="flex items-center gap-3">
          <h1 className="text-sm font-bold tracking-tight text-foreground">Beads Planner</h1>
          <span className="rounded bg-secondary px-2 py-0.5 font-mono text-xs text-muted-foreground">{session.id}</span>
        </div>
        <span className="text-xs text-muted-foreground">Double Diamond Planning IDE</span>
      </header>

      {/* Diamond stepper */}
      <nav className="flex shrink-0 items-center border-b border-border px-6" role="navigation" aria-label="Planning phases">
        {PHASES.map((phase, i) => {
          const readiness = PhaseReadiness(session, phase.key)
          const isActive = activePhase === phase.key
          return (
            <div key={phase.key} className="flex items-center">
              {i > 0 && (
                <div
                  className={`mx-1 h-px w-8 ${
                    readiness === "complete" ? "bg-chart-2/40" : "bg-border"
                  }`}
                />
              )}
              <button
                type="button"
                onClick={() => setActivePhase(phase.key)}
                className={`relative flex items-center gap-2 px-4 py-3 text-sm transition-colors ${
                  isActive ? "text-foreground" : "text-muted-foreground hover:text-foreground"
                }`}
              >
                <span
                  className={`inline-flex h-5 w-5 items-center justify-center rounded-full text-xs font-semibold ${
                    readiness === "complete"
                      ? "bg-chart-2/20 text-chart-2"
                      : readiness === "partial"
                        ? "bg-chart-3/20 text-chart-3"
                        : isActive
                          ? "bg-primary/20 text-primary"
                          : "bg-muted text-muted-foreground"
                  }`}
                >
                  {i + 1}
                </span>
                <span className="font-medium">{phase.label}</span>
                <span className="hidden text-xs text-muted-foreground sm:inline">{phase.subtitle}</span>
                {isActive && <span className="absolute inset-x-0 -bottom-px h-0.5 bg-primary" />}
              </button>
            </div>
          )
        })}

        {/* Next/prev in phase */}
        <div className="ml-auto flex gap-1">
          <button
            type="button"
            disabled={activePhase === "discover"}
            onClick={() => {
              const idx = PHASES.findIndex((p) => p.key === activePhase)
              if (idx > 0) setActivePhase(PHASES[idx - 1].key)
            }}
            className="rounded px-3 py-1.5 text-xs text-muted-foreground hover:bg-secondary hover:text-foreground disabled:opacity-30"
          >
            Back
          </button>
          <button
            type="button"
            disabled={activePhase === "deliver"}
            onClick={() => {
              const idx = PHASES.findIndex((p) => p.key === activePhase)
              if (idx < PHASES.length - 1) setActivePhase(PHASES[idx + 1].key)
            }}
            className="rounded bg-primary/10 px-3 py-1.5 text-xs font-medium text-primary hover:bg-primary/20 disabled:opacity-30"
          >
            Next Phase
          </button>
        </div>
      </nav>

      {/* Phase content */}
      <main className="flex-1 overflow-hidden">
        {activePhase === "discover" && (
          <PhaseDiscover
            thesis={session.thesis}
            personas={session.personas}
            scenarios={session.scenarios}
            onThesisChange={(thesis) => setSession({ ...session, thesis })}
            onPersonasChange={(personas) => setSession({ ...session, personas })}
            onScenariosChange={(scenarios) => setSession({ ...session, scenarios })}
          />
        )}
        {activePhase === "define" && (
          <PhaseDefine
            useCases={session.useCases}
            context={session.context}
            personas={session.personas}
            scenarios={session.scenarios}
            onUseCasesChange={(useCases) => setSession({ ...session, useCases })}
            onContextChange={(context) => setSession({ ...session, context })}
          />
        )}
        {activePhase === "develop" && (
          <PhaseDevelop
            tasks={session.tasks}
            onTasksChange={(tasks) => setSession({ ...session, tasks })}
          />
        )}
        {activePhase === "deliver" && <PhaseDeliver session={session} />}
      </main>
    </div>
  )
}
