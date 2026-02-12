/**
 * OpenCode Server API Client
 *
 * Connects to a running OpenCode server (opencode serve) via HTTP API.
 * Falls back to mock mode when server is unavailable.
 */

const OPENCODE_DEFAULT_PORT = 4096
const OPENCODE_SERVER_URL = process.env.NEXT_PUBLIC_OPENCODE_URL || `http://localhost:${OPENCODE_DEFAULT_PORT}`

export interface OpenCodeConfig {
  url?: string
  password?: string
}

export interface Session {
  id: string
  title: string | null
  createdAt: string
  updatedAt: string
}

export interface MessagePart {
  type: "text"
  text: string
}

export interface SendMessageRequest {
  parts: MessagePart[]
}

export interface TerminalLine {
  type: "cmd" | "output" | "comment" | "separator" | "error"
  text: string
  agent?: string
  timestamp: number
}

export type ConnectionStatus = "connected" | "disconnected" | "connecting" | "error"

class OpenCodeClient {
  private url: string
  private password: string | null
  private status: ConnectionStatus = "disconnected"
  private listeners: Set<(status: ConnectionStatus) => void> = new Set()

  constructor(config: OpenCodeConfig = {}) {
    this.url = config.url || OPENCODE_SERVER_URL
    this.password = config.password || (typeof window !== "undefined" ? localStorage.getItem("opencode_password") : null)
  }

  private getAuthHeaders(): HeadersInit {
    const headers: HeadersInit = {
      "Content-Type": "application/json",
    }
    if (this.password) {
      // Basic auth: username "opencode", password from config
      const encoded = btoa(`opencode:${this.password}`)
      headers["Authorization"] = `Basic ${encoded}`
    }
    return headers
  }

  private setStatus(status: ConnectionStatus) {
    this.status = status
    this.listeners.forEach((fn) => fn(status))
  }

  subscribeStatus(fn: (status: ConnectionStatus) => void): () => void {
    this.listeners.add(fn)
    fn(this.status) // Immediate callback with current status
    return () => this.listeners.delete(fn)
  }

  async checkHealth(): Promise<boolean> {
    try {
      this.setStatus("connecting")
      const response = await fetch(`${this.url}/global/health`, {
        method: "GET",
        headers: this.getAuthHeaders(),
      })
      const ok = response.ok
      this.setStatus(ok ? "connected" : "error")
      return ok
    } catch {
      this.setStatus("disconnected")
      return false
    }
  }

  async listSessions(): Promise<Session[]> {
    try {
      const response = await fetch(`${this.url}/session`, {
        method: "GET",
        headers: this.getAuthHeaders(),
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      return await response.json()
    } catch {
      return []
    }
  }

  async createSession(title?: string): Promise<Session | null> {
    try {
      const response = await fetch(`${this.url}/session`, {
        method: "POST",
        headers: this.getAuthHeaders(),
        body: JSON.stringify({ title: title || "Beads Planner Session" }),
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      return await response.json()
    } catch {
      return null
    }
  }

  async sendMessage(
    sessionId: string,
    message: string,
    onLine: (line: TerminalLine) => void,
    signal?: AbortSignal
  ): Promise<void> {
    try {
      const response = await fetch(`${this.url}/session/${sessionId}/message`, {
        method: "POST",
        headers: this.getAuthHeaders(),
        body: JSON.stringify({ parts: [{ type: "text", text: message }] }),
        signal,
      })

      if (!response.ok) {
        onLine({
          type: "error",
          text: `Error: HTTP ${response.status}`,
          timestamp: Date.now(),
        })
        return
      }

      // Handle SSE streaming response
      const reader = response.body?.getReader()
      if (!reader) {
        onLine({
          type: "error",
          text: "Error: No response stream",
          timestamp: Date.now(),
        })
        return
      }

      const decoder = new TextDecoder()
      let buffer = ""

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split("\n")
        buffer = lines.pop() || ""

        for (const line of lines) {
          if (line.startsWith("data: ")) {
            try {
              const data = JSON.parse(line.slice(6))
              this.processSSEEvent(data, onLine)
            } catch {
              // Non-JSON SSE, emit as output
              onLine({
                type: "output",
                text: line.slice(6),
                timestamp: Date.now(),
              })
            }
          }
        }
      }
    } catch (err) {
      if ((err as Error).name === "AbortError") return
      onLine({
        type: "error",
        text: `Error: ${(err as Error).message}`,
        timestamp: Date.now(),
      })
    }
  }

  private processSSEEvent(data: unknown, onLine: (line: TerminalLine) => void): void {
    // Process different SSE event types from OpenCode
    if (typeof data === "object" && data !== null) {
      const event = data as Record<string, unknown>

      // Tool execution
      if (event.tool) {
        onLine({
          type: "cmd",
          text: `${event.tool}`,
          agent: typeof event.agent === "string" ? event.agent : undefined,
          timestamp: Date.now(),
        })
      }

      // Tool output
      if (event.output) {
        onLine({
          type: "output",
          text: String(event.output),
          timestamp: Date.now(),
        })
      }

      // Text content
      if (event.type === "text" && event.content) {
        onLine({
          type: "output",
          text: String(event.content),
          timestamp: Date.now(),
        })
      }

      // Error
      if (event.error) {
        onLine({
          type: "error",
          text: String(event.error),
          timestamp: Date.now(),
        })
      }
    }
  }

  setPassword(password: string) {
    this.password = password
    if (typeof window !== "undefined") {
      localStorage.setItem("opencode_password", password)
    }
  }

  getUrl(): string {
    return this.url
  }

  getStatus(): ConnectionStatus {
    return this.status
  }
}

// Singleton client
let clientInstance: OpenCodeClient | null = null

export function getOpenCodeClient(config?: OpenCodeConfig): OpenCodeClient {
  if (!clientInstance) {
    clientInstance = new OpenCodeClient(config)
  }
  return clientInstance
}

export { OpenCodeClient }
