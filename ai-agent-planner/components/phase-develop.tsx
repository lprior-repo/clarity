"use client"

import { useState } from "react"
import type { PlanTask } from "@/lib/types"
import { validateTask, isTaskReady } from "@/lib/types"
import { TaskDetailEditor } from "@/components/task-detail-editor"

const EFFORT_COLORS: Record<string, string> = {
  "15min": "text-chart-2",
  "30min": "text-chart-2",
  "1hr": "text-foreground",
  "2hr": "text-chart-3",
  "4hr": "text-chart-4",
}

function TaskCard({
  task,
  isSelected,
  onClick,
}: {
  task: PlanTask
  isSelected: boolean
  onClick: () => void
}) {
  const checks = validateTask(task)
  const errors = checks.filter((c) => c.severity === "error" && !c.passed).length
  const warnings = checks.filter((c) => c.severity === "warning" && !c.passed).length
  const ready = isTaskReady(task)

  return (
    <button
      type="button"
      onClick={onClick}
      className={`w-full rounded-md border p-3 text-left transition-colors ${
        isSelected
          ? "border-primary bg-primary/5"
          : "border-border bg-card hover:border-muted-foreground/30"
      }`}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="font-mono text-xs text-muted-foreground">{task.id}</span>
            <span className={`rounded px-1.5 py-0.5 text-xs font-medium ${EFFORT_COLORS[task.effort]}`}>
              {task.effort}
            </span>
          </div>
          <p className="mt-1 truncate text-sm font-medium text-foreground">{task.title}</p>
          {task.dependsOn.length > 0 && (
            <p className="mt-1 text-xs text-muted-foreground">
              depends on {task.dependsOn.join(", ")}
            </p>
          )}
        </div>
        <div className="shrink-0">
          {ready ? (
            <span className="inline-flex h-5 items-center rounded-full bg-chart-2/15 px-2 text-xs font-medium text-chart-2">
              Ready
            </span>
          ) : (
            <span className="inline-flex h-5 items-center rounded-full bg-chart-4/15 px-2 text-xs font-medium text-chart-4">
              {errors}e {warnings > 0 ? `${warnings}w` : ""}
            </span>
          )}
        </div>
      </div>

      {/* Mini quality bar */}
      <div className="mt-2 flex gap-0.5">
        {checks.map((c, i) => (
          <div
            key={i}
            className={`h-1 flex-1 rounded-full ${
              c.passed ? "bg-chart-2/40" : c.severity === "error" ? "bg-chart-4/40" : "bg-chart-3/40"
            }`}
          />
        ))}
      </div>
    </button>
  )
}

export function PhaseDevelop({
  tasks,
  onTasksChange,
}: {
  tasks: PlanTask[]
  onTasksChange: (t: PlanTask[]) => void
}) {
  const [selectedId, setSelectedId] = useState<string | null>(tasks[0]?.id ?? null)
  const selectedTask = tasks.find((t) => t.id === selectedId) ?? null

  const readyCount = tasks.filter(isTaskReady).length

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="shrink-0 border-b border-border px-6 py-4">
        <div className="flex items-baseline gap-3">
          <h2 className="text-lg font-semibold text-foreground">Develop</h2>
          <span className="text-sm text-muted-foreground">Task decomposition and specification</span>
        </div>
        <p className="mt-1 text-xs leading-relaxed text-muted-foreground">
          Each task becomes a Bead -- an atomic unit of work with EARS requirements, contracts, tests, and implementation phases.
          All quality gates must pass before handoff.
        </p>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Task list */}
        <div className="w-72 shrink-0 overflow-y-auto border-r border-border p-3">
          <div className="mb-3 flex items-center justify-between">
            <span className="text-xs text-muted-foreground">
              {readyCount}/{tasks.length} ready
            </span>
          </div>
          <div className="space-y-2">
            {tasks.map((task) => (
              <TaskCard
                key={task.id}
                task={task}
                isSelected={selectedId === task.id}
                onClick={() => setSelectedId(task.id)}
              />
            ))}
          </div>
        </div>

        {/* Detail editor */}
        <div className="flex-1 overflow-hidden">
          {selectedTask ? (
            <TaskDetailEditor
              task={selectedTask}
              onChange={(updated) => {
                onTasksChange(tasks.map((t) => (t.id === updated.id ? updated : t)))
              }}
            />
          ) : (
            <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
              Select a task to edit
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
