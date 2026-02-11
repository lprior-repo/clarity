"use client"

import React from "react"
import { useState, useRef, useEffect, useCallback } from "react"
import { getStepsForPhase } from "@/lib/prompts"
import type { PromptStep } from "@/lib/prompts"

export interface Answer {
  stepId: string
  value: string
  timestamp: number
}

// ── Terminal commands generated per step ──
function getCommandsForStep(
  stepId: string,
  value: string,
): { agent: string; cmd: string; output: string }[] {
  const v = value.slice(0, 60)
  switch (stepId) {
    case "problem":
      return [
        {
          agent: "planner",
          cmd: `bd init --project beads-plan`,
          output: "Initialized .beads/ in current directory",
        },
        {
          agent: "planner",
          cmd: `bd create --type epic --title "Problem: ${v}..."`,
          output: "Created bd-a1f0  Problem Statement",
        },
      ]
    case "antithesis":
      return [
        {
          agent: "planner",
          cmd: `bd update bd-a1f0 --label antithesis --note "${v}..."`,
          output: "Updated bd-a1f0  +label:antithesis",
        },
      ]
    case "solution":
      return [
        {
          agent: "planner",
          cmd: `bd create --type epic --title "Solution: ${v}..."`,
          output: "Created bd-b2e1  Solution",
        },
        {
          agent: "planner",
          cmd: "bd dep add bd-b2e1 --blocks bd-a1f0 --type discovered-from",
          output: "Linked bd-b2e1 -> bd-a1f0 (discovered-from)",
        },
      ]
    case "persona":
      return [
        {
          agent: "planner",
          cmd: `bd create --type task --parent bd-b2e1 --title "Persona: ${v}..."`,
          output: "Created bd-b2e1.1  Persona definition",
        },
      ]
    case "scenario":
      return [
        {
          agent: "planner",
          cmd: `bd create --type task --parent bd-b2e1 --title "North Star Scenario"`,
          output: "Created bd-b2e1.2  North Star Scenario",
        },
        {
          agent: "planner",
          cmd: "bd dep add bd-b2e1.2 --related bd-b2e1.1",
          output: "Linked bd-b2e1.2 -> bd-b2e1.1 (related)",
        },
      ]
    case "use-cases": {
      const lines = value
        .split("\n")
        .map((l) => l.trim())
        .filter(Boolean)
      const cmds: { agent: string; cmd: string; output: string }[] = []
      lines.forEach((uc, i) => {
        cmds.push({
          agent: "planner",
          cmd: `bd create --type feature --title "${uc.slice(0, 55)}..."`,
          output: `Created bd-c${i}d${i}  ${uc.slice(0, 30)}...`,
        })
      })
      return cmds
    }
    case "constraints":
      return [
        {
          agent: "planner",
          cmd: `bd update bd-b2e1 --label stack --note "${v}..."`,
          output: "Updated bd-b2e1  +label:stack",
        },
      ]
    case "tasks": {
      const lines = value
        .split("\n")
        .map((l) => l.trim())
        .filter(Boolean)
      const cmds: { agent: string; cmd: string; output: string }[] = []
      lines.forEach((t, i) => {
        const parts = t.split(":")
        const mod = parts.length > 1 ? parts[0].trim() : "core"
        const title = parts.length > 1 ? parts.slice(1).join(":").trim() : t
        cmds.push({
          agent: "claude-code",
          cmd: `bd create --type task --title "${title.slice(0, 50)}" --label "${mod}" --priority P2`,
          output: `Created bd-d${i}e${i}  [${mod}] ${title.slice(0, 25)}`,
        })
      })
      cmds.push({
        agent: "claude-code",
        cmd: "bd ready --json",
        output: `[${lines.length} task(s) ready for execution]`,
      })
      return cmds
    }
    default:
      return []
  }
}

// ── Inline terminal block ──
function InlineTerminal({
  commands,
}: {
  commands: { agent: string; cmd: string; output: string }[]
}) {
  const [visibleCount, setVisibleCount] = useState(0)

  useEffect(() => {
    if (visibleCount >= commands.length * 2) return
    const delay = visibleCount % 2 === 0 ? 300 : 150
    const timer = setTimeout(() => setVisibleCount((c) => c + 1), delay)
    return () => clearTimeout(timer)
  }, [visibleCount, commands.length])

  const isRunning = visibleCount < commands.length * 2

  return (
    <div className="mx-2 my-1.5 overflow-hidden rounded-lg border border-border/60 bg-[hsl(0,0%,3%)]">
      {/* Mini header bar */}
      <div className="flex items-center gap-2 border-b border-border/40 px-3 py-1.5">
        <div className="flex gap-1">
          <span className="h-2 w-2 rounded-full bg-chart-4/60" />
          <span className="h-2 w-2 rounded-full bg-chart-3/60" />
          <span className="h-2 w-2 rounded-full bg-chart-2/60" />
        </div>
        <span className="font-mono text-[10px] text-muted-foreground/40">
          beads-cli
        </span>
        {isRunning && (
          <span className="ml-auto flex items-center gap-1">
            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-chart-2" />
            <span className="font-mono text-[10px] text-chart-2/70">
              running
            </span>
          </span>
        )}
      </div>
      {/* Command lines */}
      <div className="px-3 py-2 font-mono text-xs leading-relaxed">
        {commands.map((entry, i) => {
          const cmdVisible = visibleCount > i * 2
          const outVisible = visibleCount > i * 2 + 1
          if (!cmdVisible) return null
          return (
            <React.Fragment key={i}>
              <div className="flex items-start gap-1.5 animate-fade-up">
                <span
                  className={`mt-px shrink-0 rounded px-1 py-px text-[10px] font-medium ${
                    entry.agent === "claude-code"
                      ? "bg-chart-3/15 text-chart-3"
                      : "bg-primary/15 text-primary"
                  }`}
                >
                  {entry.agent}
                </span>
                <span className="text-chart-2">{"$"}</span>
                <span className="text-foreground/90">{entry.cmd}</span>
              </div>
              {outVisible && (
                <div className="animate-fade-up pl-4 text-muted-foreground/50 pb-1">
                  {entry.output}
                </div>
              )}
            </React.Fragment>
          )
        })}
        {/* Blinking cursor at end */}
        {!isRunning && (
          <div className="flex items-center gap-1 pt-0.5">
            <span className="text-chart-2">{"$"}</span>
            <span className="inline-block h-3 w-1.5 animate-terminal-blink bg-foreground/60" />
          </div>
        )}
      </div>
    </div>
  )
}

// ── Chat bubbles ──
function CoachBubble({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex gap-3 animate-fade-up">
      <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-primary/15 text-xs font-bold text-primary">
        B
      </div>
      <div className="max-w-lg text-sm leading-relaxed text-foreground">
        {children}
      </div>
    </div>
  )
}

function UserBubble({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex justify-end animate-fade-up">
      <div className="max-w-lg rounded-lg bg-primary/10 px-4 py-2.5 text-sm leading-relaxed text-foreground">
        {children}
      </div>
    </div>
  )
}

// ── Main coach ──
export function PlanningCoach({
  activePhase,
  answers,
  onAnswer,
  onPhaseChange,
}: {
  activePhase: string
  answers: Answer[]
  onAnswer: (stepId: string, value: string) => void
  onPhaseChange: (phase: string) => void
}) {
  const [draft, setDraft] = useState("")
  const scrollRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLTextAreaElement>(null)

  const completedIds = answers.map((a) => a.stepId)
  const phaseSteps = getStepsForPhase(activePhase)
  const currentStep = phaseSteps.find((s) => !completedIds.includes(s.id))

  // Build conversation thread with terminal blocks interleaved
  const thread: (
    | { type: "coach"; content: string; stepTitle?: string }
    | { type: "user"; content: string }
    | { type: "terminal"; commands: { agent: string; cmd: string; output: string }[] }
  )[] = []

  for (const step of phaseSteps) {
    const answer = answers.find((a) => a.stepId === step.id)
    thread.push({
      type: "coach",
      content: step.question,
      stepTitle: step.title,
    })
    if (answer) {
      thread.push({ type: "user", content: answer.value })

      // Insert terminal block showing the commands that fired
      const cmds = getCommandsForStep(step.id, answer.value)
      if (cmds.length > 0) {
        thread.push({ type: "terminal", commands: cmds })
      }

      if (step.followUp) {
        thread.push({ type: "coach", content: step.followUp })
      }
    } else {
      break
    }
  }

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [thread.length, activePhase])

  useEffect(() => {
    inputRef.current?.focus()
  }, [currentStep?.id])

  const handleSubmit = useCallback(() => {
    if (!draft.trim() || !currentStep) return
    onAnswer(currentStep.id, draft.trim())
    setDraft("")
  }, [draft, currentStep, onAnswer])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        handleSubmit()
      }
    },
    [handleSubmit],
  )

  const phaseComplete = phaseSteps.every(
    (s) => !s.required || completedIds.includes(s.id),
  )
  const phases = ["discover", "define", "develop", "deliver"]
  const nextPhase = phases[phases.indexOf(activePhase) + 1]

  return (
    <div className="flex h-full flex-col">
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-6 py-6">
        <div className="mx-auto max-w-xl space-y-4">
          {thread.map((entry, i) => {
            if (entry.type === "terminal") {
              return <InlineTerminal key={`t-${i}`} commands={entry.commands} />
            }
            if (entry.type === "coach") {
              return (
                <div key={i} className="space-y-1">
                  {entry.stepTitle && (
                    <span className="ml-10 text-[10px] font-medium uppercase tracking-widest text-muted-foreground/50">
                      {entry.stepTitle}
                    </span>
                  )}
                  <CoachBubble>
                    <p>{entry.content}</p>
                  </CoachBubble>
                </div>
              )
            }
            return <UserBubble key={i}>{entry.content}</UserBubble>
          })}

          {/* Hint */}
          {currentStep && !phaseComplete && (
            <div className="ml-10 rounded-md border border-dashed border-border px-3 py-2 text-xs leading-relaxed text-muted-foreground animate-fade-up">
              {currentStep.hint}
            </div>
          )}

          {/* Phase complete */}
          {phaseComplete && (
            <div className="space-y-3 pt-2">
              <CoachBubble>
                <p>
                  {nextPhase
                    ? "This phase is locked in. Ready to continue?"
                    : "Your plan is fully specified. Review the tasks in the sidebar, then hand off to agents."}
                </p>
              </CoachBubble>
              {nextPhase && (
                <div className="ml-10">
                  <button
                    type="button"
                    onClick={() => onPhaseChange(nextPhase)}
                    className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90"
                  >
                    {"Continue to "}
                    {nextPhase.charAt(0).toUpperCase() + nextPhase.slice(1)}
                  </button>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Input */}
      {currentStep && !phaseComplete && (
        <div className="shrink-0 border-t border-border px-6 py-4">
          <div className="mx-auto max-w-xl">
            <div className="overflow-hidden rounded-lg border border-border bg-card transition-colors focus-within:border-primary/50">
              <textarea
                ref={inputRef}
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder={`${currentStep.title}...`}
                rows={3}
                className="w-full resize-none bg-transparent px-4 py-3 text-sm text-foreground placeholder:text-muted-foreground/50 focus:outline-none"
              />
              <div className="flex items-center justify-between px-4 py-2">
                <div className="flex items-center gap-2">
                  {!currentStep.required && (
                    <button
                      type="button"
                      onClick={() => {
                        onAnswer(currentStep.id, "(skipped)")
                        setDraft("")
                      }}
                      className="text-xs text-muted-foreground hover:text-foreground"
                    >
                      Skip
                    </button>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  <kbd className="hidden rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground sm:inline">
                    Cmd+Enter
                  </kbd>
                  <button
                    type="button"
                    onClick={handleSubmit}
                    disabled={!draft.trim()}
                    className="rounded-md bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground transition-opacity disabled:opacity-30"
                  >
                    Send
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
