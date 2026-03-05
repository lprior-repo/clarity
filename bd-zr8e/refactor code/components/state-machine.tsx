"use client"

import { useMemo } from "react"
import type { Answer } from "@/components/planning-coach"
import { PROMPT_STEPS, getStepsForPhase } from "@/lib/prompts"

const PHASES = ["discover", "define", "develop", "deliver"] as const

const PHASE_COLORS: Record<string, { ring: string; bg: string; text: string }> = {
  discover: { ring: "ring-chart-1/50", bg: "bg-chart-1/10", text: "text-chart-1" },
  define: { ring: "ring-chart-5/50", bg: "bg-chart-5/10", text: "text-chart-5" },
  develop: { ring: "ring-chart-3/50", bg: "bg-chart-3/10", text: "text-chart-3" },
  deliver: { ring: "ring-chart-2/50", bg: "bg-chart-2/10", text: "text-chart-2" },
}

export function StateMachine({
  answers,
  activePhase,
}: {
  answers: Answer[]
  activePhase: string
}) {
  const completedIds = answers.map((a) => a.stepId)

  const phaseStates = useMemo(() => {
    return PHASES.map((phase) => {
      const steps = getStepsForPhase(phase)
      const required = steps.filter((s) => s.required)
      const done = required.filter((s) => completedIds.includes(s.id))
      const currentStep = steps.find((s) => !completedIds.includes(s.id))
      const isComplete = required.length > 0 && required.every((s) => completedIds.includes(s.id))
      const isActive = activePhase === phase
      return {
        phase,
        steps,
        total: required.length,
        done: done.length,
        isComplete,
        isActive,
        currentStep: isActive ? currentStep : null,
      }
    })
  }, [answers, activePhase, completedIds])

  // Current global step index
  const allSteps = PROMPT_STEPS.filter((s) => s.required)
  const currentGlobalIdx = allSteps.findIndex((s) => !completedIds.includes(s.id))
  const totalSteps = allSteps.length
  const completedSteps = allSteps.filter((s) => completedIds.includes(s.id)).length

  return (
    <div className="flex h-full flex-col gap-6 p-4">
      {/* Overall progress */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-xs font-medium uppercase tracking-widest text-muted-foreground/70">
            Progress
          </span>
          <span className="font-mono text-xs text-muted-foreground">
            {completedSteps}/{totalSteps}
          </span>
        </div>
        <div className="flex gap-1">
          {allSteps.map((step, i) => (
            <div
              key={step.id}
              className={`h-1.5 flex-1 rounded-full transition-all duration-500 ${
                completedIds.includes(step.id)
                  ? "bg-primary"
                  : i === currentGlobalIdx
                    ? "animate-pulse-glow bg-primary/40"
                    : "bg-secondary"
              }`}
            />
          ))}
        </div>
      </div>

      {/* Phase state cards */}
      <div className="flex flex-1 flex-col gap-3">
        {phaseStates.map((ps, phaseIdx) => {
          const colors = PHASE_COLORS[ps.phase]
          return (
            <div key={ps.phase} className="animate-fade-up" style={{ animationDelay: `${phaseIdx * 80}ms` }}>
              <div
                className={`rounded-lg border p-3 transition-all duration-300 ${
                  ps.isActive
                    ? `ring-2 ${colors.ring} border-transparent ${colors.bg} animate-state-active`
                    : ps.isComplete
                      ? "border-border bg-card/50"
                      : "border-border/50 bg-transparent"
                }`}
              >
                {/* Phase header */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    {ps.isComplete ? (
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" className="text-chart-2">
                        <circle cx="8" cy="8" r="7" stroke="currentColor" strokeWidth="1.5" />
                        <path d="M5 8L7 10L11 6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
                      </svg>
                    ) : ps.isActive ? (
                      <span className={`flex h-4 w-4 items-center justify-center rounded-full ${colors.bg}`}>
                        <span className={`h-2 w-2 rounded-full bg-current ${colors.text} animate-pulse`} />
                      </span>
                    ) : (
                      <span className="flex h-4 w-4 items-center justify-center rounded-full bg-secondary">
                        <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground/30" />
                      </span>
                    )}
                    <span
                      className={`text-sm font-medium capitalize ${
                        ps.isActive ? colors.text : ps.isComplete ? "text-foreground/70" : "text-muted-foreground/50"
                      }`}
                    >
                      {ps.phase}
                    </span>
                  </div>
                  <span className="font-mono text-xs text-muted-foreground/50">
                    {ps.done}/{ps.total}
                  </span>
                </div>

                {/* Step sub-states */}
                {ps.isActive && ps.steps.length > 0 && (
                  <div className="mt-3 space-y-1.5 pl-6">
                    {ps.steps.map((step) => {
                      const isDone = completedIds.includes(step.id)
                      const isCurrent = ps.currentStep?.id === step.id
                      return (
                        <div key={step.id} className="flex items-center gap-2">
                          {isDone ? (
                            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" className="text-chart-2 shrink-0">
                              <path d="M3 6L5 8L9 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
                            </svg>
                          ) : isCurrent ? (
                            <span className="relative flex h-3 w-3 shrink-0">
                              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-primary/40" />
                              <span className="relative inline-flex h-3 w-3 rounded-full bg-primary" />
                            </span>
                          ) : (
                            <span className="h-3 w-3 shrink-0 rounded-full border border-border" />
                          )}
                          <span
                            className={`text-xs ${
                              isDone
                                ? "text-muted-foreground line-through"
                                : isCurrent
                                  ? "font-medium text-foreground"
                                  : "text-muted-foreground/40"
                            }`}
                          >
                            {step.title}
                          </span>
                        </div>
                      )
                    })}
                  </div>
                )}

                {/* Transition arrow */}
                {phaseIdx < PHASES.length - 1 && (
                  <div className="mt-2 flex justify-center">
                    <svg width="12" height="16" viewBox="0 0 12 16" fill="none" className="text-border">
                      <path d="M6 0V12M2 8L6 12L10 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
                    </svg>
                  </div>
                )}
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
