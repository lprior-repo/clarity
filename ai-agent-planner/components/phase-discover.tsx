"use client"

import React from "react"

import { useState } from "react"
import type { ProductThesis, Persona, NorthStarScenario } from "@/lib/types"

function SectionLabel({ children }: { children: React.ReactNode }) {
  return <h3 className="mb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">{children}</h3>
}

function FieldLabel({ children, hint }: { children: React.ReactNode; hint?: string }) {
  return (
    <label className="mb-1.5 block text-sm font-medium text-foreground">
      {children}
      {hint && <span className="ml-2 font-normal text-muted-foreground">{hint}</span>}
    </label>
  )
}

function TextArea({
  value,
  onChange,
  placeholder,
  rows = 3,
}: {
  value: string
  onChange: (v: string) => void
  placeholder: string
  rows?: number
}) {
  return (
    <textarea
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      rows={rows}
      className="w-full resize-none rounded-md border border-border bg-secondary/50 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
    />
  )
}

// ── Thesis Section ──────────────────────────────────

function ThesisEditor({
  thesis,
  onChange,
}: {
  thesis: ProductThesis
  onChange: (t: ProductThesis) => void
}) {
  return (
    <div className="space-y-4">
      <div>
        <FieldLabel hint="What specific user problem are you solving?">Problem</FieldLabel>
        <TextArea
          value={thesis.problem}
          onChange={(v) => onChange({ ...thesis, problem: v })}
          placeholder="Describe the specific pain point users face today..."
        />
      </div>
      <div>
        <FieldLabel hint="How does your software fix it?">Solution</FieldLabel>
        <TextArea
          value={thesis.solution}
          onChange={(v) => onChange({ ...thesis, solution: v })}
          placeholder="Describe your proposed solution..."
        />
      </div>
      <div className="rounded-md border border-chart-4/20 bg-chart-4/5 p-3">
        <FieldLabel hint="Why might the user NOT need this?">Antithesis (Null Hypothesis)</FieldLabel>
        <TextArea
          value={thesis.antithesis}
          onChange={(v) => onChange({ ...thesis, antithesis: v })}
          placeholder="Be skeptical -- why might the existing solution be good enough?"
        />
        <p className="mt-1.5 text-xs text-muted-foreground">
          Forces scientific thinking. If you can't articulate why this might fail, you haven't thought deeply enough.
        </p>
      </div>
    </div>
  )
}

// ── Persona Card ────────────────────────────────────

function PersonaCard({
  persona,
  onChange,
  onRemove,
}: {
  persona: Persona
  onChange: (p: Persona) => void
  onRemove: () => void
}) {
  return (
    <div
      className={`rounded-md border p-3 ${
        persona.isNonpersona ? "border-chart-4/30 bg-chart-4/5" : "border-border bg-card"
      }`}
    >
      <div className="mb-2 flex items-center justify-between">
        <input
          value={persona.name}
          onChange={(e) => onChange({ ...persona, name: e.target.value })}
          className="bg-transparent text-sm font-semibold text-foreground focus:outline-none"
          placeholder="Persona name"
        />
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => onChange({ ...persona, isNonpersona: !persona.isNonpersona })}
            className={`rounded px-2 py-0.5 text-xs font-medium ${
              persona.isNonpersona
                ? "bg-chart-4/20 text-chart-4"
                : "bg-chart-2/20 text-chart-2"
            }`}
          >
            {persona.isNonpersona ? "Non-persona" : "Target"}
          </button>
          <button type="button" onClick={onRemove} className="text-xs text-muted-foreground hover:text-destructive">
            Remove
          </button>
        </div>
      </div>
      <TextArea
        value={persona.description}
        onChange={(v) => onChange({ ...persona, description: v })}
        placeholder="Who is this person? What do they do?"
        rows={2}
      />
      <div className="mt-2">
        <span className="text-xs text-muted-foreground">Means:</span>
        <input
          value={persona.means}
          onChange={(e) => onChange({ ...persona, means: e.target.value })}
          className="mt-1 w-full rounded border border-border bg-secondary/50 px-2 py-1 text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
          placeholder="Skills, resources, constraints..."
        />
      </div>
    </div>
  )
}

// ── Scenario Card ───────────────────────────────────

function ScenarioCard({
  scenario,
  onChange,
  onRemove,
  personaNames,
}: {
  scenario: NorthStarScenario
  onChange: (s: NorthStarScenario) => void
  onRemove: () => void
  personaNames: string[]
}) {
  return (
    <div className="rounded-md border border-border bg-card p-3">
      <div className="mb-2 flex items-center justify-between">
        <input
          value={scenario.title}
          onChange={(e) => onChange({ ...scenario, title: e.target.value })}
          className="bg-transparent text-sm font-semibold text-foreground focus:outline-none"
          placeholder="Scenario title"
        />
        <button type="button" onClick={onRemove} className="text-xs text-muted-foreground hover:text-destructive">
          Remove
        </button>
      </div>
      <div className="mb-2">
        <select
          value={scenario.persona}
          onChange={(e) => onChange({ ...scenario, persona: e.target.value })}
          className="rounded border border-border bg-secondary px-2 py-1 text-xs text-foreground"
        >
          <option value="">Select persona...</option>
          {personaNames.map((n) => (
            <option key={n} value={n}>
              {n}
            </option>
          ))}
        </select>
      </div>
      <TextArea
        value={scenario.story}
        onChange={(v) => onChange({ ...scenario, story: v })}
        placeholder="Write a narrative: the character encounters a problem and uses your software to solve it. Include specific actions, not abstractions."
        rows={4}
      />
    </div>
  )
}

// ── Main Discover Phase ─────────────────────────────

export function PhaseDiscover({
  thesis,
  personas,
  scenarios,
  onThesisChange,
  onPersonasChange,
  onScenariosChange,
}: {
  thesis: ProductThesis
  personas: Persona[]
  scenarios: NorthStarScenario[]
  onThesisChange: (t: ProductThesis) => void
  onPersonasChange: (p: Persona[]) => void
  onScenariosChange: (s: NorthStarScenario[]) => void
}) {
  const [expanded, setExpanded] = useState<"thesis" | "personas" | "scenarios">("thesis")

  const targetPersonas = personas.filter((p) => !p.isNonpersona)
  const personaNames = targetPersonas.map((p) => p.name).filter(Boolean)

  const sections = [
    { key: "thesis" as const, label: "Thesis & Antithesis", filled: thesis.problem.length > 0 && thesis.antithesis.length > 0 },
    { key: "personas" as const, label: "Personas", filled: personas.length >= 2 },
    { key: "scenarios" as const, label: "North Star Scenarios", filled: scenarios.length >= 1 && scenarios[0]?.story.length > 0 },
  ]

  return (
    <div className="flex h-full flex-col overflow-hidden">
      {/* Phase header */}
      <div className="shrink-0 border-b border-border px-6 py-4">
        <div className="flex items-baseline gap-3">
          <h2 className="text-lg font-semibold text-foreground">Discover</h2>
          <span className="text-sm text-muted-foreground">Why does this software exist?</span>
        </div>
        <p className="mt-1 text-xs leading-relaxed text-muted-foreground">
          Articulate the problem, define who you're building for, and write the story of how they'll use it.
          You cannot proceed without a thesis, at least two personas, and one North Star scenario.
        </p>
      </div>

      {/* Section tabs */}
      <div className="flex shrink-0 gap-1 border-b border-border px-6 pt-2">
        {sections.map((s) => (
          <button
            key={s.key}
            type="button"
            onClick={() => setExpanded(s.key)}
            className={`relative px-3 py-2 text-sm transition-colors ${
              expanded === s.key
                ? "text-foreground"
                : "text-muted-foreground hover:text-foreground"
            }`}
          >
            <span className="flex items-center gap-1.5">
              <span
                className={`inline-block h-1.5 w-1.5 rounded-full ${
                  s.filled ? "bg-chart-2" : "bg-muted-foreground/30"
                }`}
              />
              {s.label}
            </span>
            {expanded === s.key && (
              <span className="absolute inset-x-0 -bottom-px h-px bg-primary" />
            )}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-6 py-5">
        {expanded === "thesis" && (
          <div>
            <SectionLabel>Product Thesis</SectionLabel>
            <ThesisEditor thesis={thesis} onChange={onThesisChange} />
          </div>
        )}

        {expanded === "personas" && (
          <div className="space-y-3">
            <SectionLabel>Who are you building for?</SectionLabel>
            {personas.map((p, i) => (
              <PersonaCard
                key={i}
                persona={p}
                onChange={(updated) => {
                  const next = [...personas]
                  next[i] = updated
                  onPersonasChange(next)
                }}
                onRemove={() => onPersonasChange(personas.filter((_, j) => j !== i))}
              />
            ))}
            <button
              type="button"
              onClick={() =>
                onPersonasChange([...personas, { name: "", description: "", means: "" }])
              }
              className="rounded border border-dashed border-border px-3 py-2 text-xs text-muted-foreground hover:border-primary hover:text-primary"
            >
              + Add persona
            </button>
          </div>
        )}

        {expanded === "scenarios" && (
          <div className="space-y-3">
            <SectionLabel>North Star Scenarios</SectionLabel>
            <p className="text-xs text-muted-foreground">
              Write a story where a persona encounters a problem and uses your software to solve it.
              Include specific actions -- flag "plot holes" where steps are missing.
            </p>
            {scenarios.map((s, i) => (
              <ScenarioCard
                key={i}
                scenario={s}
                onChange={(updated) => {
                  const next = [...scenarios]
                  next[i] = updated
                  onScenariosChange(next)
                }}
                onRemove={() => onScenariosChange(scenarios.filter((_, j) => j !== i))}
                personaNames={personaNames}
              />
            ))}
            <button
              type="button"
              onClick={() =>
                onScenariosChange([...scenarios, { title: "", story: "", persona: "" }])
              }
              className="rounded border border-dashed border-border px-3 py-2 text-xs text-muted-foreground hover:border-primary hover:text-primary"
            >
              + Add scenario
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
