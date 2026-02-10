"use client"

import React from "react"

import { useState } from "react"
import type { PlanTask } from "@/lib/types"
import { validateTask } from "@/lib/types"

type Tab = "basic" | "ears" | "contracts" | "tests" | "research"

function TabButton({ active, label, hasErrors, onClick }: { active: boolean; label: string; hasErrors?: boolean; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`relative px-3 py-2 text-sm transition-colors ${active ? "text-foreground" : "text-muted-foreground hover:text-foreground"}`}
    >
      <span className="flex items-center gap-1.5">
        {hasErrors && <span className="inline-block h-1.5 w-1.5 rounded-full bg-chart-4" />}
        {label}
      </span>
      {active && <span className="absolute inset-x-0 -bottom-px h-px bg-primary" />}
    </button>
  )
}

function ListEditor({
  items,
  onChange,
  placeholder,
  addLabel,
}: {
  items: string[]
  onChange: (items: string[]) => void
  placeholder: string
  addLabel: string
}) {
  return (
    <div className="space-y-1.5">
      {items.map((item, i) => (
        <div key={i} className="group flex items-start gap-2">
          <span className="mt-2 text-xs text-muted-foreground">{i + 1}.</span>
          <input
            value={item}
            onChange={(e) => {
              const next = [...items]
              next[i] = e.target.value
              onChange(next)
            }}
            className="flex-1 rounded border border-border bg-secondary/50 px-2 py-1.5 text-sm text-foreground focus:border-primary focus:outline-none"
            placeholder={placeholder}
          />
          <button
            type="button"
            onClick={() => onChange(items.filter((_, j) => j !== i))}
            className="mt-1.5 text-xs text-muted-foreground opacity-0 hover:text-destructive group-hover:opacity-100"
          >
            x
          </button>
        </div>
      ))}
      <button
        type="button"
        onClick={() => onChange([...items, ""])}
        className="text-xs text-muted-foreground hover:text-primary"
      >
        + {addLabel}
      </button>
    </div>
  )
}

function SectionLabel({ children }: { children: React.ReactNode }) {
  return <h4 className="mb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">{children}</h4>
}

// ── Tab: Basic ──────────────────────────────────────

function BasicTab({ task, onChange }: { task: PlanTask; onChange: (t: PlanTask) => void }) {
  return (
    <div className="space-y-4">
      <div>
        <SectionLabel>Title</SectionLabel>
        <input
          value={task.title}
          onChange={(e) => onChange({ ...task, title: e.target.value })}
          className="w-full rounded border border-border bg-secondary/50 px-3 py-2 text-sm font-medium text-foreground focus:border-primary focus:outline-none"
        />
        <p className="mt-1 text-xs text-muted-foreground">Format: "component: action description"</p>
      </div>
      <div className="flex gap-4">
        <div>
          <SectionLabel>Type</SectionLabel>
          <select
            value={task.type}
            onChange={(e) => onChange({ ...task, type: e.target.value as PlanTask["type"] })}
            className="rounded border border-border bg-secondary px-2 py-1.5 text-sm text-foreground"
          >
            {(["feature", "bug", "task", "epic", "chore"] as const).map((t) => (
              <option key={t} value={t}>{t}</option>
            ))}
          </select>
        </div>
        <div>
          <SectionLabel>Priority</SectionLabel>
          <select
            value={task.priority}
            onChange={(e) => onChange({ ...task, priority: Number(e.target.value) as PlanTask["priority"] })}
            className="rounded border border-border bg-secondary px-2 py-1.5 text-sm text-foreground"
          >
            {[0, 1, 2, 3, 4].map((p) => (
              <option key={p} value={p}>P{p}</option>
            ))}
          </select>
        </div>
        <div>
          <SectionLabel>Effort</SectionLabel>
          <select
            value={task.effort}
            onChange={(e) => onChange({ ...task, effort: e.target.value as PlanTask["effort"] })}
            className="rounded border border-border bg-secondary px-2 py-1.5 text-sm text-foreground"
          >
            {(["15min", "30min", "1hr", "2hr", "4hr"] as const).map((e) => (
              <option key={e} value={e}>{e}</option>
            ))}
          </select>
        </div>
      </div>
      <div>
        <SectionLabel>Description</SectionLabel>
        <textarea
          value={task.description}
          onChange={(e) => onChange({ ...task, description: e.target.value })}
          rows={3}
          className="w-full resize-none rounded border border-border bg-secondary/50 px-3 py-2 text-sm text-foreground focus:border-primary focus:outline-none"
        />
      </div>
    </div>
  )
}

// ── Tab: EARS ───────────────────────────────────────

function EarsTab({ task, onChange }: { task: PlanTask; onChange: (t: PlanTask) => void }) {
  const ears = task.ears
  return (
    <div className="space-y-5">
      <div>
        <SectionLabel>Ubiquitous (always true)</SectionLabel>
        <p className="mb-2 text-xs text-muted-foreground">THE SYSTEM SHALL ...</p>
        <ListEditor
          items={ears.ubiquitous.map((u) => u.text)}
          onChange={(items) => onChange({ ...task, ears: { ...ears, ubiquitous: items.map((text) => ({ text })) } })}
          placeholder="THE SYSTEM SHALL ..."
          addLabel="Add ubiquitous"
        />
      </div>
      <div>
        <SectionLabel>Event-Driven (trigger to response)</SectionLabel>
        {ears.eventDriven.map((ed, i) => (
          <div key={i} className="group mb-2 rounded border border-border bg-card p-2">
            <div className="flex items-start gap-2">
              <span className="mt-1.5 text-xs text-chart-3">WHEN</span>
              <input
                value={ed.trigger}
                onChange={(e) => {
                  const next = [...ears.eventDriven]
                  next[i] = { ...ed, trigger: e.target.value }
                  onChange({ ...task, ears: { ...ears, eventDriven: next } })
                }}
                className="flex-1 bg-transparent text-sm text-foreground focus:outline-none"
                placeholder="trigger..."
              />
              <button
                type="button"
                onClick={() => onChange({ ...task, ears: { ...ears, eventDriven: ears.eventDriven.filter((_, j) => j !== i) } })}
                className="text-xs text-muted-foreground opacity-0 hover:text-destructive group-hover:opacity-100"
              >
                x
              </button>
            </div>
            <div className="mt-1 flex items-start gap-2">
              <span className="mt-1.5 text-xs text-chart-2">THEN</span>
              <input
                value={ed.response}
                onChange={(e) => {
                  const next = [...ears.eventDriven]
                  next[i] = { ...ed, response: e.target.value }
                  onChange({ ...task, ears: { ...ears, eventDriven: next } })
                }}
                className="flex-1 bg-transparent text-sm text-foreground focus:outline-none"
                placeholder="THE SYSTEM SHALL ..."
              />
            </div>
          </div>
        ))}
        <button
          type="button"
          onClick={() => onChange({ ...task, ears: { ...ears, eventDriven: [...ears.eventDriven, { trigger: "", response: "" }] } })}
          className="text-xs text-muted-foreground hover:text-primary"
        >
          + Add event-driven
        </button>
      </div>
      <div>
        <SectionLabel>Unwanted (must never happen)</SectionLabel>
        {ears.unwanted.map((uw, i) => (
          <div key={i} className="group mb-2 rounded border border-chart-4/20 bg-chart-4/5 p-2">
            <div className="flex items-start gap-2">
              <span className="mt-1.5 text-xs text-chart-4">IF</span>
              <input
                value={uw.condition}
                onChange={(e) => {
                  const next = [...ears.unwanted]
                  next[i] = { ...uw, condition: e.target.value }
                  onChange({ ...task, ears: { ...ears, unwanted: next } })
                }}
                className="flex-1 bg-transparent text-sm text-foreground focus:outline-none"
                placeholder="condition..."
              />
              <button
                type="button"
                onClick={() => onChange({ ...task, ears: { ...ears, unwanted: ears.unwanted.filter((_, j) => j !== i) } })}
                className="text-xs text-muted-foreground opacity-0 hover:text-destructive group-hover:opacity-100"
              >
                x
              </button>
            </div>
            <div className="mt-1 flex items-start gap-2">
              <span className="mt-1.5 text-xs text-chart-4">SHALL NOT</span>
              <input
                value={uw.shallNot}
                onChange={(e) => {
                  const next = [...ears.unwanted]
                  next[i] = { ...uw, shallNot: e.target.value }
                  onChange({ ...task, ears: { ...ears, unwanted: next } })
                }}
                className="flex-1 bg-transparent text-sm text-foreground focus:outline-none"
                placeholder="THE SYSTEM SHALL NOT ..."
              />
            </div>
            <div className="mt-1 flex items-start gap-2">
              <span className="mt-1.5 text-xs text-muted-foreground">BECAUSE</span>
              <input
                value={uw.because}
                onChange={(e) => {
                  const next = [...ears.unwanted]
                  next[i] = { ...uw, because: e.target.value }
                  onChange({ ...task, ears: { ...ears, unwanted: next } })
                }}
                className="flex-1 bg-transparent text-sm text-muted-foreground focus:outline-none"
                placeholder="reason..."
              />
            </div>
          </div>
        ))}
        <button
          type="button"
          onClick={() => onChange({ ...task, ears: { ...ears, unwanted: [...ears.unwanted, { condition: "", shallNot: "", because: "" }] } })}
          className="text-xs text-muted-foreground hover:text-primary"
        >
          + Add unwanted
        </button>
      </div>
    </div>
  )
}

// ── Tab: Contracts ──────────────────────────────────

function ContractsTab({ task, onChange }: { task: PlanTask; onChange: (t: PlanTask) => void }) {
  const c = task.contracts
  return (
    <div className="space-y-5">
      <div>
        <SectionLabel>Preconditions (must be true BEFORE execution)</SectionLabel>
        <ListEditor items={c.preconditions} onChange={(items) => onChange({ ...task, contracts: { ...c, preconditions: items } })} placeholder="Precondition..." addLabel="Add precondition" />
      </div>
      <div>
        <SectionLabel>Postconditions (must be true AFTER completion)</SectionLabel>
        <ListEditor items={c.postconditions} onChange={(items) => onChange({ ...task, contracts: { ...c, postconditions: items } })} placeholder="Postcondition..." addLabel="Add postcondition" />
      </div>
      <div>
        <SectionLabel>Invariants (always true throughout)</SectionLabel>
        <ListEditor items={c.invariants} onChange={(items) => onChange({ ...task, contracts: { ...c, invariants: items } })} placeholder="Invariant..." addLabel="Add invariant" />
      </div>
    </div>
  )
}

// ── Tab: Tests ──────────────────────────────────────

function TestsTab({ task, onChange }: { task: PlanTask; onChange: (t: PlanTask) => void }) {
  const t = task.tests
  return (
    <div className="space-y-5">
      <div>
        <SectionLabel>Happy Path (it works)</SectionLabel>
        <ListEditor items={t.happy} onChange={(items) => onChange({ ...task, tests: { ...t, happy: items } })} placeholder="Scenario..." addLabel="Add happy path" />
      </div>
      <div>
        <SectionLabel>Error Path (it fails gracefully)</SectionLabel>
        <ListEditor items={t.error} onChange={(items) => onChange({ ...task, tests: { ...t, error: items } })} placeholder="Scenario..." addLabel="Add error path" />
      </div>
      <div>
        <SectionLabel>Edge Cases (boundary conditions)</SectionLabel>
        <ListEditor items={t.edge} onChange={(items) => onChange({ ...task, tests: { ...t, edge: items } })} placeholder="Scenario..." addLabel="Add edge case" />
      </div>
    </div>
  )
}

// ── Tab: Research ───────────────────────────────────

function ResearchTab({ task, onChange }: { task: PlanTask; onChange: (t: PlanTask) => void }) {
  const r = task.research
  const impl = task.implementation
  return (
    <div className="space-y-5">
      <div>
        <SectionLabel>Files to Read</SectionLabel>
        <ListEditor items={r.files} onChange={(items) => onChange({ ...task, research: { ...r, files: items } })} placeholder="path/to/file..." addLabel="Add file" />
      </div>
      <div>
        <SectionLabel>Patterns to Find</SectionLabel>
        <ListEditor items={r.patterns} onChange={(items) => onChange({ ...task, research: { ...r, patterns: items } })} placeholder="Pattern or question..." addLabel="Add pattern" />
      </div>
      <div>
        <SectionLabel>Open Questions</SectionLabel>
        <ListEditor items={r.questions} onChange={(items) => onChange({ ...task, research: { ...r, questions: items } })} placeholder="Question..." addLabel="Add question" />
      </div>
      <hr className="border-border" />
      <div>
        <SectionLabel>Phase 0: Research</SectionLabel>
        <ListEditor items={impl.phase0} onChange={(items) => onChange({ ...task, implementation: { ...impl, phase0: items } })} placeholder="Research step..." addLabel="Add step" />
      </div>
      <div>
        <SectionLabel>Phase 1: Tests</SectionLabel>
        <ListEditor items={impl.phase1} onChange={(items) => onChange({ ...task, implementation: { ...impl, phase1: items } })} placeholder="Test to write..." addLabel="Add step" />
      </div>
      <div>
        <SectionLabel>Phase 2: Implementation</SectionLabel>
        <ListEditor items={impl.phase2} onChange={(items) => onChange({ ...task, implementation: { ...impl, phase2: items } })} placeholder="Implementation step..." addLabel="Add step" />
      </div>
    </div>
  )
}

// ── Main Editor ─────────────────────────────────────

export function TaskDetailEditor({ task, onChange }: { task: PlanTask; onChange: (t: PlanTask) => void }) {
  const [tab, setTab] = useState<Tab>("basic")
  const checks = validateTask(task)

  const tabHasErrors = (tabName: Tab) => {
    const errorLabels: Record<Tab, string[]> = {
      basic: ["Title follows"],
      ears: ["ubiquitous", "event-driven", "unwanted"],
      contracts: ["Preconditions", "Postconditions", "Invariants"],
      tests: ["happy", "error", "edge"],
      research: ["Research", "Implementation"],
    }
    return checks.some((c) => !c.passed && errorLabels[tabName].some((l) => c.label.toLowerCase().includes(l.toLowerCase())))
  }

  return (
    <div className="flex h-full flex-col overflow-hidden">
      {/* Tabs */}
      <div className="flex shrink-0 gap-0.5 border-b border-border px-4">
        {(["basic", "ears", "contracts", "tests", "research"] as const).map((t) => (
          <TabButton key={t} active={tab === t} label={t.charAt(0).toUpperCase() + t.slice(1)} hasErrors={tabHasErrors(t)} onClick={() => setTab(t)} />
        ))}
      </div>

      {/* Quality check bar */}
      <div className="shrink-0 border-b border-border px-4 py-2">
        <div className="flex flex-wrap gap-x-3 gap-y-1">
          {checks.map((c, i) => (
            <span key={i} className={`text-xs ${c.passed ? "text-chart-2/70" : c.severity === "error" ? "text-chart-4" : "text-chart-3"}`}>
              {c.passed ? "+" : c.severity === "error" ? "x" : "!"} {c.label}
            </span>
          ))}
        </div>
      </div>

      {/* Tab content */}
      <div className="flex-1 overflow-y-auto p-4">
        {tab === "basic" && <BasicTab task={task} onChange={onChange} />}
        {tab === "ears" && <EarsTab task={task} onChange={onChange} />}
        {tab === "contracts" && <ContractsTab task={task} onChange={onChange} />}
        {tab === "tests" && <TestsTab task={task} onChange={onChange} />}
        {tab === "research" && <ResearchTab task={task} onChange={onChange} />}
      </div>
    </div>
  )
}
