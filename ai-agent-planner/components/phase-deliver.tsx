"use client"

import React from "react"

import { useState } from "react"
import type { PlanTask, PlanSession } from "@/lib/types"
import { validateTask, isTaskReady, getGraphHealth } from "@/lib/types"

function SectionLabel({ children }: { children: React.ReactNode }) {
  return <h3 className="mb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">{children}</h3>
}

// ── Validation Card ─────────────────────────────────

function TaskValidationCard({ task }: { task: PlanTask }) {
  const checks = validateTask(task)
  const ready = isTaskReady(task)

  return (
    <div className={`rounded-md border p-3 ${ready ? "border-chart-2/30 bg-chart-2/5" : "border-chart-4/30 bg-chart-4/5"}`}>
      <div className="flex items-center justify-between">
        <div>
          <span className="font-mono text-xs text-muted-foreground">{task.id}</span>
          <p className="text-sm font-medium text-foreground">{task.title}</p>
        </div>
        <span className={`rounded-full px-2.5 py-0.5 text-xs font-semibold ${ready ? "bg-chart-2/20 text-chart-2" : "bg-chart-4/20 text-chart-4"}`}>
          {ready ? "READY" : "NEEDS WORK"}
        </span>
      </div>
      <div className="mt-2 grid grid-cols-2 gap-x-4 gap-y-0.5">
        {checks.map((c, i) => (
          <span
            key={i}
            className={`text-xs ${c.passed ? "text-chart-2/70" : c.severity === "error" ? "text-chart-4" : "text-chart-3"}`}
          >
            {c.passed ? "+" : c.severity === "error" ? "x" : "!"} {c.label}
          </span>
        ))}
      </div>
    </div>
  )
}

// ── Dependency Graph (ASCII-style) ──────────────────

function DepGraph({ tasks }: { tasks: PlanTask[] }) {
  const roots = tasks.filter((t) => t.dependsOn.length === 0)
  const children = (parentId: string) => tasks.filter((t) => t.dependsOn.includes(parentId))

  return (
    <div className="rounded-md border border-border bg-card p-4 font-mono text-xs">
      {roots.map((root) => {
        const deps = children(root.id)
        return (
          <div key={root.id} className="mb-3 last:mb-0">
            <div className="flex items-center gap-2">
              <span className="text-primary">{root.id}</span>
              <span className="text-muted-foreground">{root.title.split(":")[1]?.trim()}</span>
              <span className="text-muted-foreground">({root.effort})</span>
            </div>
            {deps.length > 0 && (
              <div className="ml-2 mt-1 border-l border-border pl-3">
                {deps.map((dep, i) => (
                  <div key={dep.id} className="flex items-center gap-2 py-0.5">
                    <span className="text-muted-foreground">{i === deps.length - 1 ? "└" : "├"}</span>
                    <span className={isTaskReady(dep) ? "text-chart-2" : "text-chart-4"}>{dep.id}</span>
                    <span className="text-muted-foreground">{dep.title.split(":")[1]?.trim()}</span>
                    <span className="text-muted-foreground">({dep.effort})</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )
      })}
    </div>
  )
}

// ── Command Preview ─────────────────────────────────

function CommandPreview({ session }: { session: PlanSession }) {
  const [copied, setCopied] = useState(false)
  const commands = [
    `# Initialize planning session`,
    `P="$HOME/.claude/skills/planner/planner.nu"`,
    `nu $P init --session-id ${session.id} \\`,
    `  --description "${session.thesis.solution.slice(0, 60)}..."`,
    ``,
    ...session.tasks.map(
      (t, i) =>
        `# Task ${i + 1}: ${t.title}\necho '${JSON.stringify({ id: t.id, title: t.title, type: t.type, priority: t.priority, effort: t.effort })}' | \\\n  nu $P add-task ${session.id} --task-json -`
    ),
    ``,
    `# Process all tasks (generate, validate, create beads)`,
    `nu $P process ${session.id}`,
    ``,
    `# Verify results`,
    `nu $P report ${session.id}`,
  ]

  const text = commands.join("\n")

  return (
    <div className="rounded-md border border-border bg-card">
      <div className="flex items-center justify-between border-b border-border px-3 py-2">
        <span className="text-xs font-medium text-muted-foreground">Command Preview</span>
        <button
          type="button"
          onClick={() => {
            navigator.clipboard.writeText(text)
            setCopied(true)
            setTimeout(() => setCopied(false), 2000)
          }}
          className="rounded px-2 py-0.5 text-xs text-muted-foreground hover:bg-secondary hover:text-foreground"
        >
          {copied ? "Copied" : "Copy"}
        </button>
      </div>
      <pre className="max-h-64 overflow-auto p-3 text-xs leading-relaxed text-muted-foreground">
        {text}
      </pre>
    </div>
  )
}

// ── Main Deliver Phase ──────────────────────────────

export function PhaseDeliver({ session }: { session: PlanSession }) {
  const health = getGraphHealth(session.tasks)
  const allReady = health.ready === health.total
  const [view, setView] = useState<"validation" | "graph" | "handoff">("validation")

  const effortHrs = Math.round(health.totalEffortMin / 60 * 10) / 10

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="shrink-0 border-b border-border px-6 py-4">
        <div className="flex items-baseline gap-3">
          <h2 className="text-lg font-semibold text-foreground">Deliver</h2>
          <span className="text-sm text-muted-foreground">Validate, visualize, hand off</span>
        </div>
        <p className="mt-1 text-xs leading-relaxed text-muted-foreground">
          Final quality gate before Beads are created. Every task must pass validation.
          Once confirmed, your planner skill generates the actual Beads for agent execution.
        </p>
      </div>

      {/* Summary bar */}
      <div className="flex shrink-0 items-center gap-6 border-b border-border px-6 py-3">
        <div className="flex items-center gap-2">
          <span className={`text-2xl font-bold ${allReady ? "text-chart-2" : "text-chart-4"}`}>{health.ready}</span>
          <span className="text-sm text-muted-foreground">/ {health.total} tasks ready</span>
        </div>
        <div className="text-xs text-muted-foreground">{effortHrs}hr estimated effort</div>
        <div className={`text-xs ${health.allDepsValid ? "text-chart-2" : "text-chart-4"}`}>
          {health.allDepsValid ? "+ No broken dependencies" : "x Broken dependencies"}
        </div>
        <div className={`text-xs ${health.hasCircular ? "text-chart-4" : "text-chart-2"}`}>
          {health.hasCircular ? "x Circular dependencies" : "+ No cycles"}
        </div>
      </div>

      {/* View tabs */}
      <div className="flex shrink-0 gap-1 border-b border-border px-6 pt-1">
        {(["validation", "graph", "handoff"] as const).map((v) => (
          <button
            key={v}
            type="button"
            onClick={() => setView(v)}
            className={`relative px-3 py-2 text-sm capitalize transition-colors ${view === v ? "text-foreground" : "text-muted-foreground hover:text-foreground"}`}
          >
            {v}
            {view === v && <span className="absolute inset-x-0 -bottom-px h-px bg-primary" />}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-6 py-5">
        {view === "validation" && (
          <div className="space-y-3">
            <SectionLabel>Task Quality Validation</SectionLabel>
            {session.tasks.map((t) => (
              <TaskValidationCard key={t.id} task={t} />
            ))}
          </div>
        )}

        {view === "graph" && (
          <div className="space-y-4">
            <SectionLabel>Dependency Structure</SectionLabel>
            <DepGraph tasks={session.tasks} />
            <div className="rounded-md border border-border bg-secondary/30 p-3 text-xs text-muted-foreground">
              <p>Parallelization: {session.tasks.filter((t) => t.dependsOn.length === 0).length} root tasks can start immediately.</p>
              <p className="mt-1">
                After root tasks complete, {session.tasks.filter((t) => t.dependsOn.length > 0).length} dependent tasks become unblocked.
              </p>
            </div>
          </div>
        )}

        {view === "handoff" && (
          <div className="space-y-5">
            <SectionLabel>Planning Summary</SectionLabel>
            <div className="rounded-md border border-border bg-card p-4 text-sm">
              <div className="space-y-1.5 text-muted-foreground">
                <p><span className="text-foreground">Session:</span> {session.id}</p>
                <p><span className="text-foreground">Tasks:</span> {health.total} ({health.ready} validated)</p>
                <p><span className="text-foreground">Total effort:</span> {effortHrs}hr</p>
                <p><span className="text-foreground">Ready to execute:</span> {session.tasks.filter((t) => t.dependsOn.length === 0 && isTaskReady(t)).length} task(s)</p>
              </div>
            </div>

            <CommandPreview session={session} />

            <div className="rounded-md border border-border p-4">
              <SectionLabel>Execution Options</SectionLabel>
              <div className="space-y-3">
                <label className="flex items-start gap-3 rounded-md border border-border p-3 hover:bg-secondary/30">
                  <input type="radio" name="exec" defaultChecked className="mt-1 accent-primary" />
                  <div>
                    <p className="text-sm font-medium text-foreground">Create Beads and start orchestrator</p>
                    <p className="text-xs text-muted-foreground">Agents begin executing via `bd ready` loop</p>
                  </div>
                </label>
                <label className="flex items-start gap-3 rounded-md border border-border p-3 hover:bg-secondary/30">
                  <input type="radio" name="exec" className="mt-1 accent-primary" />
                  <div>
                    <p className="text-sm font-medium text-foreground">Create Beads only</p>
                    <p className="text-xs text-muted-foreground">You manually start the orchestrator later</p>
                  </div>
                </label>
                <label className="flex items-start gap-3 rounded-md border border-border p-3 hover:bg-secondary/30">
                  <input type="radio" name="exec" className="mt-1 accent-primary" />
                  <div>
                    <p className="text-sm font-medium text-foreground">Dry run (preview only)</p>
                    <p className="text-xs text-muted-foreground">No Beads created -- just show what would happen</p>
                  </div>
                </label>
              </div>
            </div>

            <button
              type="button"
              disabled={!allReady}
              className={`w-full rounded-md px-4 py-3 text-sm font-semibold transition-colors ${
                allReady
                  ? "bg-primary text-primary-foreground hover:bg-primary/90"
                  : "cursor-not-allowed bg-muted text-muted-foreground"
              }`}
            >
              {allReady ? "Execute Planner Skill" : `${health.total - health.ready} task(s) need fixes before execution`}
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
