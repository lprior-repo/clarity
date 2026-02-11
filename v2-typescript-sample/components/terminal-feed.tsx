"use client"

import { useMemo, useRef, useEffect } from "react"
import type { Answer } from "@/components/planning-coach"

interface TerminalLine {
  type: "cmd" | "output" | "comment" | "separator"
  text: string
  agent?: string
}

function parseLines(text: string | null) {
  if (!text) return []
  return text.split("\n").map((l) => l.trim()).filter(Boolean)
}

function getVal(answers: Answer[], id: string) {
  const v = answers.find((a) => a.stepId === id)?.value
  return v && v !== "(skipped)" ? v : null
}

export function TerminalFeed({ answers }: { answers: Answer[] }) {
  const scrollRef = useRef<HTMLDivElement>(null)

  const lines = useMemo(() => {
    const l: TerminalLine[] = []

    l.push({ type: "comment", text: "# Beads Planner - Agent Command Preview" })
    l.push({ type: "comment", text: "# Commands generated from your planning session" })
    l.push({ type: "separator", text: "" })

    const problem = getVal(answers, "problem")
    if (problem) {
      l.push({ type: "comment", text: "# Phase: Discover" })
      l.push({ type: "cmd", text: "bd init --project beads-plan", agent: "planner" })
      l.push({ type: "output", text: "Initialized .beads/ in current directory" })
      l.push({
        type: "cmd",
        text: `bd create --type epic --title "Problem Statement" --desc "${problem.slice(0, 60)}..."`,
        agent: "planner",
      })
      l.push({ type: "output", text: "Created bd-a1f0  Problem Statement" })
    }

    const antithesis = getVal(answers, "antithesis")
    if (antithesis) {
      l.push({
        type: "cmd",
        text: `bd update bd-a1f0 --label antithesis --note "${antithesis.slice(0, 50)}..."`,
        agent: "planner",
      })
      l.push({ type: "output", text: "Updated bd-a1f0  +label:antithesis" })
    }

    const solution = getVal(answers, "solution")
    if (solution) {
      l.push({
        type: "cmd",
        text: `bd create --type epic --title "Solution" --desc "${solution.slice(0, 60)}..."`,
        agent: "planner",
      })
      l.push({ type: "output", text: "Created bd-b2e1  Solution" })
      l.push({ type: "cmd", text: "bd dep add bd-b2e1 --blocks bd-a1f0 --type discovered-from", agent: "planner" })
      l.push({ type: "output", text: "Linked bd-b2e1 -> bd-a1f0 (discovered-from)" })
    }

    const persona = getVal(answers, "persona")
    if (persona) {
      l.push({
        type: "cmd",
        text: `bd create --type task --parent bd-b2e1 --title "Persona: ${persona.slice(0, 40)}..."`,
        agent: "planner",
      })
      l.push({ type: "output", text: "Created bd-b2e1.1  Persona definition" })
    }

    const scenario = getVal(answers, "scenario")
    if (scenario) {
      l.push({ type: "separator", text: "" })
      l.push({
        type: "cmd",
        text: `bd create --type task --parent bd-b2e1 --title "North Star Scenario"`,
        agent: "planner",
      })
      l.push({ type: "output", text: "Created bd-b2e1.2  North Star Scenario" })
    }

    const useCases = parseLines(getVal(answers, "use-cases"))
    if (useCases.length > 0) {
      l.push({ type: "separator", text: "" })
      l.push({ type: "comment", text: "# Phase: Define - Use Cases" })
      useCases.forEach((uc, i) => {
        const id = `bd-c${i}d${i}`
        l.push({
          type: "cmd",
          text: `bd create --type feature --title "${uc.slice(0, 55)}..."`,
          agent: "planner",
        })
        l.push({ type: "output", text: `Created ${id}  ${uc.slice(0, 30)}...` })
      })
    }

    const constraints = getVal(answers, "constraints")
    if (constraints) {
      l.push({
        type: "cmd",
        text: `bd update bd-b2e1 --label "stack" --note "${constraints.slice(0, 50)}..."`,
        agent: "planner",
      })
      l.push({ type: "output", text: "Updated bd-b2e1  +label:stack" })
    }

    const tasks = parseLines(getVal(answers, "tasks"))
    if (tasks.length > 0) {
      l.push({ type: "separator", text: "" })
      l.push({ type: "comment", text: "# Phase: Develop - Task Decomposition" })
      tasks.forEach((task, i) => {
        const parts = task.split(":")
        const module = parts.length > 1 ? parts[0].trim() : "core"
        const title = parts.length > 1 ? parts.slice(1).join(":").trim() : task
        const id = `bd-d${i}e${i}`
        l.push({
          type: "cmd",
          text: `bd create --type task --title "${title.slice(0, 50)}" --label "${module}" --priority P2`,
          agent: "claude-code",
        })
        l.push({ type: "output", text: `Created ${id}  [${module}] ${title.slice(0, 30)}` })
        if (i > 0) {
          l.push({
            type: "cmd",
            text: `bd dep add ${id} --related bd-d${i - 1}e${i - 1}`,
            agent: "claude-code",
          })
          l.push({ type: "output", text: `Linked ${id} -> bd-d${i - 1}e${i - 1} (related)` })
        }
      })

      l.push({ type: "separator", text: "" })
      l.push({ type: "comment", text: "# Execution frontier" })
      l.push({ type: "cmd", text: "bd ready --json", agent: "claude-code" })
      l.push({
        type: "output",
        text: `[${tasks.length} task(s) ready for execution]`,
      })
    }

    if (answers.length === 0) {
      l.push({ type: "comment", text: "# Waiting for planning input..." })
      l.push({ type: "comment", text: "# Answer the coach's questions to generate agent commands" })
    }

    return l
  }, [answers])

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [lines.length])

  return (
    <div ref={scrollRef} className="h-full overflow-y-auto bg-background p-4 font-mono text-xs leading-relaxed">
      {lines.map((line, i) => {
        if (line.type === "separator") {
          return <div key={i} className="h-2" />
        }
        if (line.type === "comment") {
          return (
            <div key={i} className="animate-fade-up text-muted-foreground/40" style={{ animationDelay: `${i * 30}ms` }}>
              {line.text}
            </div>
          )
        }
        if (line.type === "cmd") {
          return (
            <div key={i} className="animate-fade-up flex items-start gap-1.5" style={{ animationDelay: `${i * 30}ms` }}>
              {line.agent && (
                <span
                  className={`mt-px shrink-0 rounded px-1 py-px text-[10px] font-medium ${
                    line.agent === "claude-code"
                      ? "bg-chart-3/15 text-chart-3"
                      : "bg-primary/15 text-primary"
                  }`}
                >
                  {line.agent}
                </span>
              )}
              <span className="text-chart-2">{"$"}</span>
              <span className="text-foreground">{line.text}</span>
            </div>
          )
        }
        // output
        return (
          <div key={i} className="animate-fade-up pl-4 text-muted-foreground/60" style={{ animationDelay: `${i * 30}ms` }}>
            {line.text}
          </div>
        )
      })}
      {/* Blinking cursor */}
      <div className="mt-1 flex items-center gap-1">
        <span className="text-chart-2">{"$"}</span>
        <span className="inline-block h-3.5 w-1.5 bg-foreground/70 animate-terminal-blink" />
      </div>
    </div>
  )
}
