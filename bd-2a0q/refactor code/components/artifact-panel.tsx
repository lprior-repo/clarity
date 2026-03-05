"use client"

import { useState } from "react"
import type { Answer } from "@/components/planning-coach"
import { PROMPT_STEPS } from "@/lib/prompts"

interface ArtifactPanelProps {
  answers: Answer[]
  activePhase: string
}

function getVal(answers: Answer[], id: string) {
  const v = answers.find((a) => a.stepId === id)?.value
  return v && v !== "(skipped)" ? v : null
}

function parseLines(text: string | null) {
  if (!text) return []
  return text.split("\n").map((l) => l.trim()).filter(Boolean)
}

function SectionHeader({ label, count }: { label: string; count?: number }) {
  return (
    <div className="flex items-center gap-2 pb-2 pt-5 first:pt-0">
      <h4 className="text-xs font-semibold uppercase tracking-widest text-muted-foreground/70">
        {label}
      </h4>
      {count !== undefined && (
        <span className="rounded-full bg-secondary px-1.5 py-0.5 text-xs tabular-nums text-muted-foreground">
          {count}
        </span>
      )}
    </div>
  )
}

function ThesisCard({
  label,
  value,
  accent,
}: {
  label: string
  value: string | null
  accent?: string
}) {
  if (!value) return null
  return (
    <div
      className={`animate-fade-up rounded-lg border px-3 py-2.5 ${accent ?? "border-border bg-card"}`}
    >
      <span className="mb-1 block text-xs font-medium uppercase tracking-wider text-muted-foreground">
        {label}
      </span>
      <p className="text-sm leading-relaxed text-foreground">{value}</p>
    </div>
  )
}

function UseCaseRow({ text, index }: { text: string; index: number }) {
  const match = text.match(/(.+?)\s+can\s+(.+?)\s+so that\s+(.+)/i)
  return (
    <div className="animate-fade-up flex items-start gap-2.5 rounded-md px-2 py-2 transition-colors hover:bg-secondary/50">
      <span className="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded bg-secondary font-mono text-xs text-muted-foreground">
        {index + 1}
      </span>
      {match ? (
        <p className="min-w-0 text-sm leading-relaxed">
          <span className="font-medium text-primary">{match[1]}</span>
          <span className="text-muted-foreground">{" can "}</span>
          <span className="text-foreground">{match[2]}</span>
          <span className="text-muted-foreground">{" so that "}</span>
          <span className="text-foreground/70">{match[3]}</span>
        </p>
      ) : (
        <p className="min-w-0 text-sm text-foreground">{text}</p>
      )}
    </div>
  )
}

function TaskRow({
  text,
  index,
  selected,
  onSelect,
}: {
  text: string
  index: number
  selected: boolean
  onSelect: () => void
}) {
  const parts = text.split(":")
  const module = parts.length > 1 ? parts[0].trim() : null
  const title = parts.length > 1 ? parts.slice(1).join(":").trim() : text

  return (
    <button
      type="button"
      onClick={onSelect}
      className={`animate-fade-up flex w-full items-start gap-2.5 rounded-md px-2 py-2 text-left transition-all ${
        selected
          ? "bg-primary/10 ring-1 ring-primary/30"
          : "hover:bg-secondary/50"
      }`}
    >
      <span
        className={`mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded font-mono text-xs transition-colors ${
          selected
            ? "bg-primary text-primary-foreground"
            : "bg-secondary text-muted-foreground"
        }`}
      >
        {index + 1}
      </span>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-1.5">
          {module && (
            <span className="rounded bg-chart-5/15 px-1.5 py-0.5 font-mono text-xs text-chart-5">
              {module}
            </span>
          )}
        </div>
        <p className="mt-0.5 text-sm text-foreground">{title}</p>
      </div>
      <svg
        width="14"
        height="14"
        viewBox="0 0 14 14"
        fill="none"
        className="mt-1 shrink-0 text-muted-foreground/40"
      >
        <path
          d="M5 3L9 7L5 11"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    </button>
  )
}

function TaskDetail({
  task,
  index,
  onClose,
}: {
  task: string
  index: number
  onClose: () => void
}) {
  const parts = task.split(":")
  const module = parts.length > 1 ? parts[0].trim() : null
  const title = parts.length > 1 ? parts.slice(1).join(":").trim() : task

  return (
    <div className="animate-fade-up rounded-lg border border-primary/20 bg-primary/5">
      <div className="flex items-center justify-between border-b border-primary/10 px-3 py-2">
        <div className="flex items-center gap-2">
          <span className="flex h-5 w-5 items-center justify-center rounded bg-primary font-mono text-xs text-primary-foreground">
            {index + 1}
          </span>
          {module && (
            <span className="rounded bg-chart-5/15 px-1.5 py-0.5 font-mono text-xs text-chart-5">
              {module}
            </span>
          )}
          <span className="text-sm font-medium text-foreground">{title}</span>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="rounded p-0.5 text-muted-foreground hover:text-foreground"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path
              d="M4 4L10 10M10 4L4 10"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
            />
          </svg>
        </button>
      </div>
      <div className="space-y-3 px-3 py-3">
        <div>
          <span className="mb-1 block text-xs font-medium uppercase tracking-wider text-muted-foreground">
            Acceptance Criteria
          </span>
          <div className="rounded border border-dashed border-border/50 px-3 py-3 text-center text-xs text-muted-foreground/40">
            Add criteria in the Develop phase
          </div>
        </div>
        <div>
          <span className="mb-1 block text-xs font-medium uppercase tracking-wider text-muted-foreground">
            Edge Cases
          </span>
          <div className="rounded border border-dashed border-border/50 px-3 py-3 text-center text-xs text-muted-foreground/40">
            The coach will prompt you for these
          </div>
        </div>
      </div>
    </div>
  )
}

export function ArtifactPanel({ answers, activePhase }: ArtifactPanelProps) {
  const [selectedTask, setSelectedTask] = useState<number | null>(null)

  const problem = getVal(answers, "problem")
  const antithesis = getVal(answers, "antithesis")
  const solution = getVal(answers, "solution")
  const persona = getVal(answers, "persona")
  const scenario = getVal(answers, "scenario")
  const useCases = parseLines(getVal(answers, "use-cases"))
  const constraints = getVal(answers, "constraints")
  const tasks = parseLines(getVal(answers, "tasks"))

  const required = PROMPT_STEPS.filter((s) => s.required)
  const done = required.filter((s) =>
    answers.some((a) => a.stepId === s.id && a.value !== "(skipped)"),
  )
  const progress =
    required.length > 0 ? Math.round((done.length / required.length) * 100) : 0

  const hasAnything = answers.length > 0

  return (
    <div className="flex h-full flex-col">
      {/* Progress bar */}
      <div className="shrink-0 px-4 pt-3 pb-1">
        <div className="flex items-center gap-2">
          <div className="h-1 flex-1 rounded-full bg-secondary">
            <div
              className="h-full rounded-full bg-primary transition-all duration-700 ease-out"
              style={{ width: `${progress}%` }}
            />
          </div>
          <span className="font-mono text-xs text-muted-foreground">{progress}%</span>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-4 py-2">
        {!hasAnything ? (
          <div className="flex h-full items-center justify-center">
            <p className="max-w-xs text-center text-sm leading-relaxed text-muted-foreground/40">
              {activePhase === "discover"
                ? "Answer the coach to build your thesis, persona, and north star scenario."
                : "Your plan will build up here as you answer."}
            </p>
          </div>
        ) : (
          <div className="space-y-1 pb-4">
            {(problem || antithesis || solution) && (
              <>
                <SectionHeader label="Thesis" />
                <div className="space-y-2">
                  <ThesisCard label="Problem" value={problem} />
                  <ThesisCard
                    label="Antithesis"
                    value={antithesis}
                    accent="border-chart-4/20 bg-chart-4/5"
                  />
                  <ThesisCard label="Solution" value={solution} />
                </div>
              </>
            )}

            {persona && (
              <>
                <SectionHeader label="User" />
                <div className="animate-fade-up rounded-lg border border-chart-5/20 bg-chart-5/5 px-3 py-2.5">
                  <p className="text-sm leading-relaxed text-foreground">{persona}</p>
                </div>
              </>
            )}

            {scenario && (
              <>
                <SectionHeader label="North Star" />
                <div className="animate-fade-up rounded-lg border border-chart-2/20 bg-chart-2/5 px-3 py-2.5">
                  <p className="text-sm leading-relaxed text-foreground/80">{scenario}</p>
                </div>
              </>
            )}

            {useCases.length > 0 && (
              <>
                <SectionHeader label="Use Cases" count={useCases.length} />
                <div className="space-y-0.5">
                  {useCases.map((uc, i) => (
                    <UseCaseRow key={i} text={uc} index={i} />
                  ))}
                </div>
              </>
            )}

            {constraints && (
              <>
                <SectionHeader label="Stack" />
                <div className="animate-fade-up rounded-lg border border-border bg-card px-3 py-2.5">
                  <p className="font-mono text-xs leading-relaxed text-foreground/80">{constraints}</p>
                </div>
              </>
            )}

            {tasks.length > 0 && (
              <>
                <SectionHeader label="Tasks" count={tasks.length} />
                {selectedTask !== null && tasks[selectedTask] && (
                  <div className="mb-2">
                    <TaskDetail
                      task={tasks[selectedTask]}
                      index={selectedTask}
                      onClose={() => setSelectedTask(null)}
                    />
                  </div>
                )}
                <div className="space-y-0.5">
                  {tasks.map((t, i) => (
                    <TaskRow
                      key={i}
                      text={t}
                      index={i}
                      selected={selectedTask === i}
                      onSelect={() =>
                        setSelectedTask(selectedTask === i ? null : i)
                      }
                    />
                  ))}
                </div>
              </>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
