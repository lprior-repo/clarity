"use client"

import { useMemo, useRef, useEffect, useState, useCallback } from "react"
import type { Answer } from "@/components/planning-coach"
import { useOpenCode } from "@/hooks/use-opencode"
import type { ConnectionStatus } from "@/lib/opencode-client"

interface TerminalLine {
  type: "cmd" | "output" | "comment" | "separator" | "error"
  text: string
  agent?: string
  timestamp?: number
  executed?: boolean
}

function parseLines(text: string | null) {
  if (!text) return []
  return text.split("\n").map((l) => l.trim()).filter(Boolean)
}

function getVal(answers: Answer[], id: string) {
  const v = answers.find((a) => a.stepId === id)?.value
  return v && v !== "(skipped)" ? v : null
}

function StatusIndicator({ status, isMockMode }: { status: ConnectionStatus; isMockMode: boolean }) {
  const statusConfig = {
    connected: { color: "bg-chart-2", text: "Connected" },
    connecting: { color: "bg-yellow-500 animate-pulse", text: "Connecting..." },
    disconnected: { color: "bg-muted-foreground/50", text: "Disconnected" },
    error: { color: "bg-red-500", text: "Error" },
  }

  const config = isMockMode
    ? { color: "bg-yellow-500/70", text: "Demo Mode" }
    : statusConfig[status]

  return (
    <div className="flex items-center gap-1.5 px-2 py-1">
      <span className={`h-2 w-2 rounded-full ${config.color}`} />
      <span className="text-[10px] font-medium text-muted-foreground">{config.text}</span>
    </div>
  )
}

export function TerminalFeed({ answers }: { answers: Answer[] }) {
  const scrollRef = useRef<HTMLDivElement>(null)
  const [executedCommands, setExecutedCommands] = useState<Set<string>>(new Set())

  const { status, isMockMode, sendCommand, lines: liveLines, isStreaming, reconnect } = useOpenCode()

  // Generate mock command preview from answers
  const previewLines = useMemo(() => {
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

  // Execute a command
  const executeCommand = useCallback(
    async (cmd: string, agent?: string) => {
      const key = `${cmd}:${agent}`
      if (executedCommands.has(key)) return

      setExecutedCommands((prev) => new Set(prev).add(key))
      await sendCommand(cmd, agent)
    },
    [executedCommands, sendCommand]
  )

  // Execute all commands
  const executeAll = useCallback(async () => {
    const cmdLines = previewLines.filter((l) => l.type === "cmd")
    for (const line of cmdLines) {
      if (!executedCommands.has(`${line.text}:${line.agent}`)) {
        await executeCommand(line.text, line.agent)
      }
    }
  }, [previewLines, executedCommands, executeCommand])

  // Combine preview lines with live output
  const displayLines = useMemo(() => {
    if (liveLines.length > 0) {
      // When we have live output, show that instead
      return liveLines.map((l) => ({
        ...l,
        executed: true,
      }))
    }
    return previewLines
  }, [previewLines, liveLines])

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [displayLines.length])

  return (
    <div className="flex h-full flex-col">
      {/* Header with status */}
      <div className="flex shrink-0 items-center justify-between border-b border-border/50 px-3 py-1.5">
        <StatusIndicator status={status} isMockMode={isMockMode} />
        <div className="flex items-center gap-2">
          {status === "disconnected" && (
            <button
              type="button"
              onClick={reconnect}
              className="rounded px-2 py-0.5 text-[10px] font-medium text-muted-foreground hover:bg-secondary"
            >
              Reconnect
            </button>
          )}
          {!isMockMode && previewLines.some((l) => l.type === "cmd") && (
            <button
              type="button"
              onClick={executeAll}
              disabled={isStreaming}
              className="rounded bg-primary/10 px-2 py-0.5 text-[10px] font-medium text-primary hover:bg-primary/20 disabled:opacity-50"
            >
              {isStreaming ? "Running..." : "Run All"}
            </button>
          )}
        </div>
      </div>

      {/* Terminal content */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto bg-background p-4 font-mono text-xs leading-relaxed">
        {displayLines.map((line, i) => {
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
          if (line.type === "error") {
            return (
              <div key={i} className="animate-fade-up text-red-500" style={{ animationDelay: `${i * 30}ms` }}>
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
                {!line.executed && !isMockMode && (
                  <button
                    type="button"
                    onClick={() => executeCommand(line.text, line.agent)}
                    disabled={isStreaming}
                    className="ml-auto shrink-0 rounded bg-secondary px-1.5 py-px text-[9px] text-muted-foreground hover:bg-secondary/80 disabled:opacity-50"
                  >
                    run
                  </button>
                )}
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
          <span className={`inline-block h-3.5 w-1.5 bg-foreground/70 ${isStreaming ? "" : "animate-terminal-blink"}`} />
        </div>
      </div>
    </div>
  )
}
