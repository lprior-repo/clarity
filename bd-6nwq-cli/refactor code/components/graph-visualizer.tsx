"use client"

import { useMemo, useEffect, useLayoutEffect, useRef, useState, useCallback } from "react"
import type { Answer } from "@/components/planning-coach"

interface GraphNode {
  id: string
  label: string
  group: "thesis" | "persona" | "scenario" | "usecase" | "task"
  x: number
  y: number
}

interface GraphEdge {
  from: string
  to: string
}

const GROUP_COLORS: Record<string, string> = {
  thesis: "hsl(221, 83%, 53%)",
  persona: "hsl(262, 83%, 58%)",
  scenario: "hsl(142, 71%, 45%)",
  usecase: "hsl(38, 92%, 50%)",
  task: "hsl(0, 72%, 51%)",
}

const GROUP_BG: Record<string, string> = {
  thesis: "hsl(221, 83%, 53%, 0.15)",
  persona: "hsl(262, 83%, 58%, 0.15)",
  scenario: "hsl(142, 71%, 45%, 0.15)",
  usecase: "hsl(38, 92%, 50%, 0.15)",
  task: "hsl(0, 72%, 51%, 0.15)",
}

function parseLines(text: string | null) {
  if (!text) return []
  return text.split("\n").map((l) => l.trim()).filter(Boolean)
}

function getVal(answers: Answer[], id: string) {
  const v = answers.find((a) => a.stepId === id)?.value
  return v && v !== "(skipped)" ? v : null
}

export function GraphVisualizer({ answers }: { answers: Answer[] }) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const animFrameRef = useRef<number>(0)
  const particlesRef = useRef<{ edge: number; t: number; speed: number }[]>([])
  const [hoveredNode, setHoveredNode] = useState<string | null>(null)
  const [dimensions, setDimensions] = useState({ w: 600, h: 400 })

  const { nodes, edges } = useMemo(() => {
    const n: GraphNode[] = []
    const e: GraphEdge[] = []
    const cx = dimensions.w / 2
    const cy = dimensions.h / 2

    // Thesis nodes in top center cluster
    const problem = getVal(answers, "problem")
    const antithesis = getVal(answers, "antithesis")
    const solution = getVal(answers, "solution")

    if (problem) n.push({ id: "problem", label: "Problem", group: "thesis", x: cx - 80, y: 60 })
    if (antithesis) {
      n.push({ id: "antithesis", label: "Antithesis", group: "thesis", x: cx + 80, y: 60 })
      e.push({ from: "problem", to: "antithesis" })
    }
    if (solution) {
      n.push({ id: "solution", label: "Solution", group: "thesis", x: cx, y: 130 })
      if (problem) e.push({ from: "problem", to: "solution" })
      if (antithesis) e.push({ from: "antithesis", to: "solution" })
    }

    // Persona
    const persona = getVal(answers, "persona")
    if (persona) {
      n.push({ id: "persona", label: "User", group: "persona", x: cx - 160, y: 200 })
      if (solution) e.push({ from: "solution", to: "persona" })
    }

    // Scenario
    const scenario = getVal(answers, "scenario")
    if (scenario) {
      n.push({ id: "scenario", label: "North Star", group: "scenario", x: cx + 160, y: 200 })
      if (persona) e.push({ from: "persona", to: "scenario" })
      if (solution) e.push({ from: "solution", to: "scenario" })
    }

    // Use cases -- fan out below
    const useCases = parseLines(getVal(answers, "use-cases"))
    const ucStartX = cx - ((useCases.length - 1) * 70) / 2
    useCases.forEach((uc, i) => {
      const id = `uc-${i}`
      const short = uc.length > 20 ? uc.slice(0, 18) + ".." : uc
      n.push({ id, label: short, group: "usecase", x: ucStartX + i * 70, y: 300 })
      if (scenario) e.push({ from: "scenario", to: id })
    })

    // Tasks -- fan out at bottom
    const tasks = parseLines(getVal(answers, "tasks"))
    const tStartX = cx - ((tasks.length - 1) * 60) / 2
    tasks.forEach((t, i) => {
      const id = `task-${i}`
      const parts = t.split(":")
      const short = parts.length > 1 ? parts[0].trim() : t.slice(0, 14)
      n.push({ id, label: short, group: "task", x: tStartX + i * 60, y: 400 })
      // Link to nearest use case or scenario
      if (useCases.length > 0) {
        const ucIdx = Math.min(i, useCases.length - 1)
        e.push({ from: `uc-${ucIdx}`, to: id })
      } else if (scenario) {
        e.push({ from: "scenario", to: id })
      }
    })

    return { nodes: n, edges: e }
  }, [answers, dimensions])

  // Capture dimensions before first paint to prevent flash, then keep synced
  useLayoutEffect(() => {
    const container = containerRef.current
    if (!container) return
    const { width, height } = container.getBoundingClientRect()
    if (width > 0 && height > 0) setDimensions({ w: width, h: height })
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setDimensions({ w: entry.contentRect.width, h: entry.contentRect.height })
      }
    })
    observer.observe(container)
    return () => observer.disconnect()
  }, [])

  // Spawn particles
  useEffect(() => {
    if (edges.length === 0) return
    const interval = setInterval(() => {
      const edgeIdx = Math.floor(Math.random() * edges.length)
      particlesRef.current.push({ edge: edgeIdx, t: 0, speed: 0.008 + Math.random() * 0.006 })
      if (particlesRef.current.length > 30) particlesRef.current.shift()
    }, 400)
    return () => clearInterval(interval)
  }, [edges.length])

  // Canvas render loop
  const render = useCallback(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const ctx = canvas.getContext("2d")
    if (!ctx) return
    const dpr = window.devicePixelRatio || 1
    canvas.width = dimensions.w * dpr
    canvas.height = dimensions.h * dpr
    ctx.scale(dpr, dpr)
    ctx.clearRect(0, 0, dimensions.w, dimensions.h)

    // Draw edges
    edges.forEach((edge) => {
      const fromNode = nodes.find((n) => n.id === edge.from)
      const toNode = nodes.find((n) => n.id === edge.to)
      if (!fromNode || !toNode) return
      ctx.beginPath()
      ctx.moveTo(fromNode.x, fromNode.y)
      ctx.lineTo(toNode.x, toNode.y)
      ctx.strokeStyle = "hsl(0, 0%, 20%)"
      ctx.lineWidth = 1
      ctx.stroke()
    })

    // Draw particles
    particlesRef.current = particlesRef.current.filter((p) => p.t <= 1)
    particlesRef.current.forEach((p) => {
      p.t += p.speed
      const edge = edges[p.edge]
      if (!edge) return
      const fromNode = nodes.find((n) => n.id === edge.from)
      const toNode = nodes.find((n) => n.id === edge.to)
      if (!fromNode || !toNode) return
      const x = fromNode.x + (toNode.x - fromNode.x) * p.t
      const y = fromNode.y + (toNode.y - fromNode.y) * p.t
      const alpha = p.t < 0.1 ? p.t / 0.1 : p.t > 0.9 ? (1 - p.t) / 0.1 : 1
      ctx.beginPath()
      ctx.arc(x, y, 2, 0, Math.PI * 2)
      ctx.fillStyle = `hsl(221, 83%, 53%, ${alpha * 0.8})`
      ctx.fill()
      // Glow
      ctx.beginPath()
      ctx.arc(x, y, 6, 0, Math.PI * 2)
      ctx.fillStyle = `hsl(221, 83%, 53%, ${alpha * 0.15})`
      ctx.fill()
    })

    // Draw nodes
    nodes.forEach((node) => {
      const isHovered = hoveredNode === node.id
      const radius = isHovered ? 22 : 18
      const color = GROUP_COLORS[node.group]
      const bg = GROUP_BG[node.group]

      // Glow
      if (isHovered) {
        ctx.beginPath()
        ctx.arc(node.x, node.y, 30, 0, Math.PI * 2)
        ctx.fillStyle = bg
        ctx.fill()
      }

      // Circle
      ctx.beginPath()
      ctx.arc(node.x, node.y, radius, 0, Math.PI * 2)
      ctx.fillStyle = bg
      ctx.fill()
      ctx.strokeStyle = color
      ctx.lineWidth = isHovered ? 2 : 1.5
      ctx.stroke()

      // Label
      ctx.font = `${isHovered ? 11 : 10}px var(--font-inter), system-ui, sans-serif`
      ctx.fillStyle = "hsl(0, 0%, 80%)"
      ctx.textAlign = "center"
      ctx.fillText(node.label, node.x, node.y + radius + 14)
    })

    animFrameRef.current = requestAnimationFrame(render)
  }, [nodes, edges, dimensions, hoveredNode])

  useEffect(() => {
    animFrameRef.current = requestAnimationFrame(render)
    return () => cancelAnimationFrame(animFrameRef.current)
  }, [render])

  // Hit detection for hover
  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const rect = canvasRef.current?.getBoundingClientRect()
      if (!rect) return
      const mx = e.clientX - rect.left
      const my = e.clientY - rect.top
      const hit = nodes.find((n) => Math.hypot(n.x - mx, n.y - my) < 22)
      setHoveredNode(hit?.id ?? null)
    },
    [nodes],
  )

  if (nodes.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-sm text-muted-foreground/40">
          Answer questions to see your plan graph grow
        </p>
      </div>
    )
  }

  return (
    <div ref={containerRef} className="relative h-full w-full">
      <canvas
        ref={canvasRef}
        style={{ width: dimensions.w, height: dimensions.h }}
        className="cursor-crosshair"
        onMouseMove={handleMouseMove}
        onMouseLeave={() => setHoveredNode(null)}
      />
      {/* Legend */}
      <div className="absolute bottom-3 left-3 flex flex-wrap gap-3">
        {Object.entries(GROUP_COLORS).map(([key, color]) => (
          <div key={key} className="flex items-center gap-1.5">
            <span className="inline-block h-2.5 w-2.5 rounded-full" style={{ backgroundColor: color }} />
            <span className="font-mono text-xs capitalize text-muted-foreground/60">{key}</span>
          </div>
        ))}
      </div>
    </div>
  )
}
