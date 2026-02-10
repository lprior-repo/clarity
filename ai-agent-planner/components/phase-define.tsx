"use client"

import React from "react"

import type { UseCase, NorthStarScenario, Persona } from "@/lib/types"

function SectionLabel({ children }: { children: React.ReactNode }) {
  return <h3 className="mb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">{children}</h3>
}

const PRIORITY_STYLES = {
  must: "bg-chart-4/15 text-chart-4 border-chart-4/30",
  should: "bg-chart-3/15 text-chart-3 border-chart-3/30",
  could: "bg-muted text-muted-foreground border-border",
}

function UseCaseRow({
  uc,
  onChange,
  onRemove,
  personaNames,
  scenarioTitles,
}: {
  uc: UseCase
  onChange: (u: UseCase) => void
  onRemove: () => void
  personaNames: string[]
  scenarioTitles: string[]
}) {
  return (
    <div className="group rounded-md border border-border bg-card p-3">
      <div className="mb-2 flex items-start justify-between gap-2">
        <span className="shrink-0 font-mono text-xs text-muted-foreground">{uc.id}</span>
        <div className="flex items-center gap-1.5">
          {(["must", "should", "could"] as const).map((p) => (
            <button
              key={p}
              type="button"
              onClick={() => onChange({ ...uc, priority: p })}
              className={`rounded border px-2 py-0.5 text-xs font-medium capitalize transition-colors ${
                uc.priority === p ? PRIORITY_STYLES[p] : "border-transparent text-muted-foreground/50 hover:text-muted-foreground"
              }`}
            >
              {p}
            </button>
          ))}
          <button type="button" onClick={onRemove} className="ml-2 text-xs text-muted-foreground opacity-0 hover:text-destructive group-hover:opacity-100">
            Remove
          </button>
        </div>
      </div>

      {/* The sentence builder */}
      <div className="flex flex-wrap items-baseline gap-1 text-sm">
        <select
          value={uc.persona}
          onChange={(e) => onChange({ ...uc, persona: e.target.value })}
          className="rounded border border-border bg-secondary px-1.5 py-0.5 text-xs text-foreground"
        >
          {personaNames.map((n) => (
            <option key={n} value={n}>{n}</option>
          ))}
        </select>
        <span className="text-muted-foreground">can</span>
        <input
          value={uc.action}
          onChange={(e) => onChange({ ...uc, action: e.target.value })}
          className="flex-1 border-b border-dashed border-border bg-transparent px-1 text-sm text-foreground focus:border-primary focus:outline-none"
          placeholder="perform action..."
        />
        <span className="text-muted-foreground">so that</span>
        <input
          value={uc.motivation}
          onChange={(e) => onChange({ ...uc, motivation: e.target.value })}
          className="flex-1 border-b border-dashed border-border bg-transparent px-1 text-sm text-foreground focus:border-primary focus:outline-none"
          placeholder="motivation..."
        />
      </div>

      <div className="mt-2">
        <select
          value={uc.northStar}
          onChange={(e) => onChange({ ...uc, northStar: e.target.value })}
          className="rounded border border-border bg-secondary px-1.5 py-0.5 text-xs text-muted-foreground"
        >
          <option value="">Link to scenario...</option>
          {scenarioTitles.map((t) => (
            <option key={t} value={t}>{t}</option>
          ))}
        </select>
      </div>
    </div>
  )
}

export function PhaseDefine({
  useCases,
  context,
  personas,
  scenarios,
  onUseCasesChange,
  onContextChange,
}: {
  useCases: UseCase[]
  context: string
  personas: Persona[]
  scenarios: NorthStarScenario[]
  onUseCasesChange: (u: UseCase[]) => void
  onContextChange: (c: string) => void
}) {
  const personaNames = personas.filter((p) => !p.isNonpersona).map((p) => p.name).filter(Boolean)
  const scenarioTitles = scenarios.map((s) => s.title).filter(Boolean)

  const musts = useCases.filter((u) => u.priority === "must")
  const shoulds = useCases.filter((u) => u.priority === "should")
  const coulds = useCases.filter((u) => u.priority === "could")

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="shrink-0 border-b border-border px-6 py-4">
        <div className="flex items-baseline gap-3">
          <h2 className="text-lg font-semibold text-foreground">Define</h2>
          <span className="text-sm text-muted-foreground">The Great Reindexing: stories to requirements</span>
        </div>
        <p className="mt-1 text-xs leading-relaxed text-muted-foreground">
          Convert your North Star Scenarios into a Use Case Compendium. Each use case follows:
          "[Persona] can [action] so that [motivation]". Prioritize ruthlessly -- must/should/could.
        </p>
      </div>

      <div className="flex-1 overflow-y-auto px-6 py-5">
        {/* Summary bar */}
        <div className="mb-4 flex items-center gap-4 rounded-md border border-border bg-secondary/30 px-4 py-2.5 text-xs">
          <span className="text-muted-foreground">
            <span className="font-semibold text-chart-4">{musts.length}</span> must-have
          </span>
          <span className="text-muted-foreground">
            <span className="font-semibold text-chart-3">{shoulds.length}</span> should-have
          </span>
          <span className="text-muted-foreground">
            <span className="font-semibold text-foreground">{coulds.length}</span> could-have
          </span>
          <span className="ml-auto text-muted-foreground">{useCases.length} total use cases</span>
        </div>

        {/* Context */}
        <div className="mb-5">
          <SectionLabel>Technical Context</SectionLabel>
          <textarea
            value={context}
            onChange={(e) => onContextChange(e.target.value)}
            placeholder="Existing tech stack, architectural constraints, related files..."
            rows={2}
            className="w-full resize-none rounded-md border border-border bg-secondary/50 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
          />
        </div>

        {/* Use cases */}
        <SectionLabel>Use Case Compendium</SectionLabel>
        <div className="space-y-2">
          {useCases.map((uc, i) => (
            <UseCaseRow
              key={uc.id}
              uc={uc}
              onChange={(updated) => {
                const next = [...useCases]
                next[i] = updated
                onUseCasesChange(next)
              }}
              onRemove={() => onUseCasesChange(useCases.filter((_, j) => j !== i))}
              personaNames={personaNames}
              scenarioTitles={scenarioTitles}
            />
          ))}
        </div>

        <button
          type="button"
          onClick={() =>
            onUseCasesChange([
              ...useCases,
              {
                id: `uc-${useCases.length + 1}`,
                persona: personaNames[0] || "",
                action: "",
                motivation: "",
                northStar: "",
                priority: "should",
              },
            ])
          }
          className="mt-3 rounded border border-dashed border-border px-3 py-2 text-xs text-muted-foreground hover:border-primary hover:text-primary"
        >
          + Add use case
        </button>
      </div>
    </div>
  )
}
