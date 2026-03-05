"use client"

// Minimal-React: all animation via rAF + CSS custom properties
// Architecture mirrors Dioxus signal-driven components for portability
import { useState, useRef, useEffect, useCallback, useMemo } from "react"
import { getStepsForPhase } from "@/lib/prompts"

export interface Answer {
  stepId: string
  value: string
  timestamp: number
}

// ─────────────────────────────────────────────────────────────
// Command generator – pure function, no React deps
// ─────────────────────────────────────────────────────────────
interface TermCmd { agent: "planner" | "claude-code" | "opencode"; cmd: string; out: string }

function cmdsForStep(id: string, val: string): TermCmd[] {
  const v = val.slice(0, 55)
  switch (id) {
    case "problem": return [
      { agent: "planner",     cmd: `bd init --project beads-plan`,                                           out: "Initialized .beads/ — SQLite + JSONL ready" },
      { agent: "planner",     cmd: `bd create -t epic --title "Problem: ${v}..."`,                           out: "Created bd-a1f0  [epic] Problem Statement" },
    ]
    case "antithesis": return [
      { agent: "planner",     cmd: `bd update bd-a1f0 --label antithesis --note "${v}..."`,                  out: "Updated bd-a1f0  +label:antithesis" },
    ]
    case "solution": return [
      { agent: "planner",     cmd: `bd create -t epic --title "Solution: ${v}..."`,                          out: "Created bd-b2e1  [epic] Solution" },
      { agent: "planner",     cmd: "bd dep add bd-b2e1 --blocks bd-a1f0 --type discovered-from",             out: "Edge: bd-b2e1 -[discovered-from]-> bd-a1f0" },
    ]
    case "persona": return [
      { agent: "planner",     cmd: `bd create -t task --parent bd-b2e1 --title "Persona: ${v}..."`,          out: "Created bd-b2e1.1  [task] Persona" },
    ]
    case "scenario": return [
      { agent: "planner",     cmd: `bd create -t task --parent bd-b2e1 --title "North Star Scenario"`,       out: "Created bd-b2e1.2  [task] North Star" },
      { agent: "planner",     cmd: "bd dep add bd-b2e1.2 --related bd-b2e1.1",                               out: "Edge: bd-b2e1.2 -[related]-> bd-b2e1.1" },
      { agent: "planner",     cmd: "bd show bd-b2e1 --graph",                                                out: "Graph: 2 nodes, 3 edges, 0 cycles  [valid]" },
    ]
    case "use-cases": {
      const lines = val.split("\n").map(l => l.trim()).filter(Boolean)
      return [
        ...lines.map((uc, i): TermCmd => ({
          agent: "planner",
          cmd: `bd create -t feature --title "${uc.slice(0, 48)}..."`,
          out: `Created bd-c${i}d${i}  [feature] ${uc.slice(0, 28)}...`,
        })),
        { agent: "planner", cmd: "bd list --status open --json | jq length",     out: `${lines.length + 2}` },
      ]
    }
    case "constraints": return [
      { agent: "planner",     cmd: `bd update bd-b2e1 --label stack --note "${v}..."`,                       out: "Updated bd-b2e1  +label:stack" },
      { agent: "claude-code", cmd: "bd ready --assignee claude-code --json",                                  out: "[] — no tasks claimed yet" },
    ]
    case "tasks": {
      const lines = val.split("\n").map(l => l.trim()).filter(Boolean)
      return [
        ...lines.flatMap((t, i): TermCmd[] => {
          const [mod, ...rest] = t.includes(":") ? t.split(":") : ["core", t]
          const title = rest.join(":").trim() || t
          return [
            { agent: "claude-code", cmd: `bd create -t task --title "${title.slice(0,44)}" --label "${mod.trim()}" -p P2`, out: `Created bd-d${i}e${i}  [${mod.trim()}]` },
            ...(i > 0 ? [{ agent: "claude-code" as const, cmd: `bd dep add bd-d${i}e${i} --related bd-d${i-1}e${i-1}`, out: `Edge: bd-d${i}e${i} -[related]-> bd-d${i-1}e${i-1}` }] : []),
          ]
        }),
        { agent: "opencode",    cmd: "bd ready --json",                                                       out: `[${lines.length} task(s) on execution frontier]` },
        { agent: "opencode",    cmd: `bd list --status open --fmt table`,                                     out: `${lines.length} open  0 in-progress  0 done` },
      ]
    }
    default: return []
  }
}

// ─────────────────────────────────────────────────────────────
// InlineTerminalStream – rAF-driven, paced char-by-char renderer
// Stages: typing cmd → pause → output scrolls → next pair
// Pure CSS scanlines + CRT vignette, no extra libraries
// ─────────────────────────────────────────────────────────────

const AGENT_COLOR: Record<string, { badge: string; glow: string }> = {
  planner:      { badge: "bg-blue-500/20 text-blue-400 ring-1 ring-blue-500/30",   glow: "#3b82f6" },
  "claude-code":{ badge: "bg-amber-500/20 text-amber-400 ring-1 ring-amber-500/30", glow: "#f59e0b" },
  opencode:     { badge: "bg-emerald-500/20 text-emerald-400 ring-1 ring-emerald-500/30", glow: "#10b981" },
}

// Each "frame" in the timeline
interface TermFrame {
  lineIdx: number       // which TermLine this belongs to
  kind: "cmd" | "out"
  agent?: string
  text: string
  ts: string            // timestamp string pre-computed
  delayAfterPrev: number // ms to pause before starting to type this line
}

function buildFrames(cmds: TermCmd[]): TermFrame[] {
  let time = Date.now()
  const frames: TermFrame[] = []
  cmds.forEach((c, i) => {
    const ts = new Date(time).toISOString().slice(11, 19)
    frames.push({ lineIdx: i * 2, kind: "cmd", agent: c.agent, text: c.cmd, ts, delayAfterPrev: i === 0 ? 0 : 90 })
    time += 120
    const tsOut = new Date(time).toISOString().slice(11, 19)
    frames.push({ lineIdx: i * 2 + 1, kind: "out", text: c.out, ts: tsOut, delayAfterPrev: 160 })
    time += 80
  })
  return frames
}

interface StreamState {
  frameIdx: number     // which frame we are currently typing
  charIdx: number      // chars revealed in current frame
  pausing: number      // ms remaining in inter-line pause
  visible: string[]    // fully-or-partially revealed text per frame
}

function InlineTerminalStream({ cmds, stepId }: { cmds: TermCmd[]; stepId: string }) {
  const frames = useMemo(() => buildFrames(cmds), [cmds])

  const [state, setState] = useState<StreamState>({
    frameIdx: 0, charIdx: 0, pausing: 0,
    visible: new Array(frames.length).fill(""),
  })

  const rafRef  = useRef<number>(0)
  const lastRef = useRef<number>(0)
  const stateRef = useRef(state)
  stateRef.current = state

  // Reset fully when stepId changes
  useEffect(() => {
    cancelAnimationFrame(rafRef.current)
    setState({ frameIdx: 0, charIdx: 0, pausing: 0, visible: new Array(frames.length).fill("") })
    lastRef.current = 0
  }, [stepId, frames.length])

  useEffect(() => {
    const TICK = 18              // ms between rAF ticks we honour (≈55fps)
    const CMD_CHARS  = 4         // chars per tick while typing a command
    const OUT_CHARS  = 8         // chars per tick for output (feels like a flush)

    const tick = (now: number) => {
      const dt = now - (lastRef.current || now)
      if (dt < TICK) { rafRef.current = requestAnimationFrame(tick); return }
      lastRef.current = now

      setState(prev => {
        if (prev.frameIdx >= frames.length) return prev  // all done

        // Still in a pause between lines?
        if (prev.pausing > 0) {
          return { ...prev, pausing: Math.max(0, prev.pausing - dt) }
        }

        const frame  = frames[prev.frameIdx]
        const target = frame.text.length
        const rate   = frame.kind === "cmd" ? CMD_CHARS : OUT_CHARS
        const next   = Math.min(prev.charIdx + rate, target)
        const newVis = [...prev.visible]
        newVis[prev.frameIdx] = frame.text.slice(0, next)

        if (next >= target) {
          // Line done -- advance to next frame with its delay
          const nextIdx = prev.frameIdx + 1
          const pause   = nextIdx < frames.length ? frames[nextIdx].delayAfterPrev : 0
          return { frameIdx: nextIdx, charIdx: 0, pausing: pause, visible: newVis }
        }
        return { ...prev, charIdx: next, visible: newVis }
      })

      rafRef.current = requestAnimationFrame(tick)
    }

    rafRef.current = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(rafRef.current)
  }, [frames])

  const allDone = state.frameIdx >= frames.length
  const activeFrame = frames[state.frameIdx]

  return (
    <div className="relative my-3 overflow-hidden rounded-lg border border-white/[0.08] bg-[hsl(0,0%,2%)] shadow-xl">

      {/* Scanline texture */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 z-10 rounded-lg"
        style={{
          backgroundImage: "repeating-linear-gradient(0deg,transparent,transparent 3px,rgba(0,0,0,0.15) 3px,rgba(0,0,0,0.15) 4px)",
        }}
      />
      {/* CRT vignette */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 z-10 rounded-lg"
        style={{ background: "radial-gradient(ellipse at 50% 50%,transparent 55%,rgba(0,0,0,0.5) 100%)" }}
      />

      {/* Title bar */}
      <div className="relative z-20 flex items-center gap-2 border-b border-white/[0.06] px-3 py-1.5">
        <div className="flex gap-1">
          <span className="h-2.5 w-2.5 rounded-full bg-red-500/60" />
          <span className="h-2.5 w-2.5 rounded-full bg-amber-500/60" />
          <span className="h-2.5 w-2.5 rounded-full bg-emerald-500/60" />
        </div>
        <span className="flex-1 text-center font-mono text-[10px] text-white/20 select-none">
          beads-cli — agent session
        </span>
        {!allDone ? (
          <span className="flex items-center gap-1.5">
            <span
              className="h-1.5 w-1.5 rounded-full bg-emerald-400"
              style={{ boxShadow: "0 0 6px #10b981", animation: "pulse 1s ease-in-out infinite" }}
            />
            <span className="font-mono text-[10px] text-emerald-400/80">running</span>
          </span>
        ) : (
          <span className="flex items-center gap-1.5">
            <span className="h-1.5 w-1.5 rounded-full bg-white/15" />
            <span className="font-mono text-[10px] text-white/25">done</span>
          </span>
        )}
      </div>

      {/* Body */}
      <div className="relative z-20 space-y-0 px-3 pb-3 pt-2 font-mono text-xs leading-[1.7]">
        {frames.map((frame, i) => {
          const text = state.visible[i] ?? ""
          if (text.length === 0 && i >= state.frameIdx) return null

          const isActive = i === state.frameIdx && !allDone
          const agentColors = frame.agent ? (AGENT_COLOR[frame.agent] ?? AGENT_COLOR.planner) : null

          return (
            <div
              key={i}
              className="flex items-start gap-2 animate-term-line"
              style={{ animationDelay: "0ms", animationFillMode: "both" }}
            >
              {/* Timestamp */}
              <span className="mt-px shrink-0 select-none font-mono text-[9px] text-white/15 tabular-nums">
                {frame.ts}
              </span>

              {/* Agent badge (cmd lines only) */}
              {frame.kind === "cmd" && agentColors && (
                <span
                  className={`mt-px shrink-0 rounded px-1.5 py-px text-[9px] font-semibold leading-none tracking-wide ${agentColors.badge}`}
                  style={isActive ? { boxShadow: `0 0 8px ${agentColors.glow}55` } : undefined}
                >
                  {frame.agent}
                </span>
              )}

              {/* Prompt char */}
              {frame.kind === "cmd"
                ? <span className="shrink-0 select-none text-emerald-500/80">$</span>
                : <span className="shrink-0 select-none pl-[4.5rem] text-white/15">{"→"}</span>
              }

              {/* Text + cursor */}
              <span className={frame.kind === "cmd" ? "text-white/90" : "text-white/35"}>
                {text}
                {isActive && state.pausing === 0 && (
                  <span className="ml-px inline-block h-[0.85em] w-[6px] translate-y-[1px] bg-white/75 align-text-bottom animate-terminal-blink" />
                )}
              </span>
            </div>
          )
        })}

        {/* Idle prompt after completion */}
        {allDone && (
          <div className="flex items-center gap-2">
            <span className="shrink-0 select-none font-mono text-[9px] text-white/15">
              {frames[frames.length - 1]?.ts ?? ""}
            </span>
            <span className="select-none text-emerald-500/80">$</span>
            <span className="ml-px inline-block h-[0.85em] w-[6px] translate-y-[1px] bg-white/60 align-text-bottom animate-terminal-blink" />
          </div>
        )}
      </div>
    </div>
  )
}

// ─────────────────────────────────────────────────────────────
// Chat bubbles – minimal, no library deps
// ─────────────────────────────────────────────────────────────
function CoachBubble({ label, children }: { label?: string; children: React.ReactNode }) {
  return (
    <div className="flex gap-3 animate-fade-up">
      <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-primary/20 text-xs font-bold text-primary ring-1 ring-primary/30">
        B
      </div>
      <div className="flex-1 space-y-1">
        {label && (
          <span className="block text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/50">
            {label}
          </span>
        )}
        <div className="max-w-lg text-sm leading-relaxed text-foreground">
          {children}
        </div>
      </div>
    </div>
  )
}

function UserBubble({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex animate-fade-up justify-end">
      <div className="max-w-lg rounded-lg bg-secondary px-4 py-2.5 text-sm leading-relaxed text-foreground ring-1 ring-white/[0.06]">
        {children}
      </div>
    </div>
  )
}

// ─────────────────────────────────────────────────────────────
// HUD – live call counter strip
// ─────────────────────────────────────────────────────────────
function HUDStrip({ answers }: { answers: Answer[] }) {
  const total = useMemo(() =>
    answers.reduce((acc, a) => acc + cmdsForStep(a.stepId, a.value).length, 0),
  [answers])

  const agents = useMemo(() => {
    const counts: Record<string, number> = {}
    answers.forEach(a => cmdsForStep(a.stepId, a.value).forEach(c => { counts[c.agent] = (counts[c.agent] ?? 0) + 1 }))
    return counts
  }, [answers])

  if (total === 0) return null

  return (
    <div className="flex shrink-0 items-center gap-3 border-b border-white/[0.04] bg-[hsl(0,0%,3%)] px-6 py-1">
      <span className="font-mono text-[10px] text-white/20">API CALLS</span>
      <span className="font-mono text-[10px] font-bold text-primary">{total}</span>
      {Object.entries(agents).map(([agent, n]) => (
        <span key={agent} className={`font-mono text-[10px] ${AGENT_STYLE[agent]?.split(" ")[1] ?? "text-white/40"}`}>
          {agent}:{n}
        </span>
      ))}
      <span className="ml-auto h-1.5 w-1.5 animate-pulse rounded-full bg-[hsl(142,71%,45%)]" />
    </div>
  )
}

// ─────────────────────────────────────────────────────────────
// Main PlanningCoach
// ─────────────────────────────────────────────────────────────
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

  const completedIds = answers.map(a => a.stepId)
  const phaseSteps = getStepsForPhase(activePhase)
  const currentStep = phaseSteps.find(s => !completedIds.includes(s.id))

  // Build thread: coach Q → user A → terminal → followUp → next Q ...
  const thread = useMemo(() => {
    type Entry =
      | { type: "coach";    content: string; label?: string }
      | { type: "user";     content: string }
      | { type: "terminal"; cmds: TermCmd[];  stepId: string }

    const t: Entry[] = []
    for (const step of phaseSteps) {
      const answer = answers.find(a => a.stepId === step.id)
      t.push({ type: "coach", content: step.question, label: step.title })
      if (answer) {
        t.push({ type: "user", content: answer.value })
        const cmds = cmdsForStep(step.id, answer.value)
        if (cmds.length) t.push({ type: "terminal", cmds, stepId: step.id })
        if (step.followUp) t.push({ type: "coach", content: step.followUp })
      } else break
    }
    return t
  }, [phaseSteps, answers])

  // Auto-scroll on new entry
  useEffect(() => {
    const el = scrollRef.current
    if (!el) return
    el.scrollTo({ top: el.scrollHeight, behavior: "smooth" })
  }, [thread.length])

  useEffect(() => { inputRef.current?.focus() }, [currentStep?.id])

  const handleSubmit = useCallback(() => {
    if (!draft.trim() || !currentStep) return
    onAnswer(currentStep.id, draft.trim())
    setDraft("")
  }, [draft, currentStep, onAnswer])

  const handleKey = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) { e.preventDefault(); handleSubmit() }
  }, [handleSubmit])

  const phaseComplete = phaseSteps.length > 0 && phaseSteps.every(s => !s.required || completedIds.includes(s.id))
  const PHASES = ["discover", "define", "develop", "deliver"]
  const nextPhase = PHASES[PHASES.indexOf(activePhase) + 1]

  return (
    <div className="flex h-full flex-col">
      <HUDStrip answers={answers} />

      {/* Conversation scroll area */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-6 py-6 scroll-smooth">
        <div className="mx-auto max-w-xl space-y-4">
          {thread.map((entry, i) => {
            if (entry.type === "terminal") {
              return (
                <InlineTerminalStream
                  key={`term-${entry.stepId}`}
                  cmds={entry.cmds}
                  stepId={entry.stepId}
                />
              )
            }
            if (entry.type === "coach") {
              return (
                <CoachBubble key={i} label={entry.label}>
                  {entry.content}
                </CoachBubble>
              )
            }
            return <UserBubble key={i}>{entry.content}</UserBubble>
          })}

          {/* Hint card */}
          {currentStep && !phaseComplete && (
            <div
              className="ml-10 animate-fade-up rounded-md border border-dashed border-white/[0.08] px-3 py-2 text-xs leading-relaxed text-muted-foreground/50"
              style={{ animationDelay: "150ms", animationFillMode: "both" }}
            >
              {currentStep.hint}
            </div>
          )}

          {/* Phase complete CTA */}
          {phaseComplete && (
            <div className="space-y-3 pt-1">
              <CoachBubble>
                {nextPhase
                  ? "This phase is locked in. Ready to move forward?"
                  : "Plan fully specified. Review tasks in the sidebar, then hand off to agents."}
              </CoachBubble>
              {nextPhase && (
                <div className="ml-10">
                  <button
                    type="button"
                    onClick={() => onPhaseChange(nextPhase)}
                    className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground ring-1 ring-primary/40 transition-all hover:bg-primary/85 hover:ring-primary/70"
                  >
                    Continue to {nextPhase.charAt(0).toUpperCase() + nextPhase.slice(1)}
                  </button>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Input bar */}
      {currentStep && !phaseComplete && (
        <div className="shrink-0 border-t border-white/[0.06] px-6 py-4">
          <div className="mx-auto max-w-xl">
            <div className="overflow-hidden rounded-lg border border-white/[0.08] bg-card ring-0 transition-all focus-within:border-primary/40 focus-within:ring-1 focus-within:ring-primary/20">
              <textarea
                ref={inputRef}
                value={draft}
                onChange={e => setDraft(e.target.value)}
                onKeyDown={handleKey}
                placeholder={`${currentStep.title}...`}
                rows={3}
                className="w-full resize-none bg-transparent px-4 py-3 text-sm text-foreground placeholder:text-white/20 focus:outline-none"
              />
              <div className="flex items-center justify-between px-4 py-2 border-t border-white/[0.05]">
                <div className="flex items-center gap-3">
                  {!currentStep.required && (
                    <button
                      type="button"
                      onClick={() => { onAnswer(currentStep.id, "(skipped)"); setDraft("") }}
                      className="text-xs text-muted-foreground/50 hover:text-foreground"
                    >
                      Skip
                    </button>
                  )}
                  <span className="font-mono text-[10px] text-white/15">
                    {draft.length > 0 ? `${draft.length} chars` : ""}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <kbd className="hidden rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground/50 sm:inline">
                    ⌘↵
                  </kbd>
                  <button
                    type="button"
                    onClick={handleSubmit}
                    disabled={!draft.trim()}
                    className="rounded-md bg-primary px-3 py-1.5 text-xs font-semibold text-primary-foreground ring-1 ring-primary/40 transition-all disabled:opacity-25 hover:bg-primary/85"
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
