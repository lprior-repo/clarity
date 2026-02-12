"use client"

import { useState, useEffect, useCallback, useRef } from "react"
import {
  getOpenCodeClient,
  OpenCodeClient,
  TerminalLine,
  ConnectionStatus,
  Session,
} from "@/lib/opencode-client"

export interface UseOpenCodeOptions {
  autoConnect?: boolean
  sessionId?: string
}

export interface UseOpenCodeReturn {
  status: ConnectionStatus
  session: Session | null
  lines: TerminalLine[]
  isStreaming: boolean
  sendCommand: (cmd: string, agent?: string) => Promise<void>
  clearLines: () => void
  reconnect: () => Promise<void>
  client: OpenCodeClient
  isMockMode: boolean
}

export function useOpenCode(options: UseOpenCodeOptions = {}): UseOpenCodeReturn {
  const { autoConnect = true, sessionId: initialSessionId } = options

  const [status, setStatus] = useState<ConnectionStatus>("disconnected")
  const [session, setSession] = useState<Session | null>(null)
  const [lines, setLines] = useState<TerminalLine[]>([])
  const [isStreaming, setIsStreaming] = useState(false)
  const [isMockMode, setIsMockMode] = useState(false)

  const client = useRef(getOpenCodeClient())
  const abortController = useRef<AbortController | null>(null)

  // Subscribe to connection status
  useEffect(() => {
    const unsubscribe = client.current.subscribeStatus(setStatus)
    return unsubscribe
  }, [])

  // Auto-connect and create/get session
  useEffect(() => {
    if (!autoConnect) return

    const init = async () => {
      const healthy = await client.current.checkHealth()
      setIsMockMode(!healthy)

      if (healthy) {
        // Try to find or create a session
        if (initialSessionId) {
          // Use provided session ID
          setSession({ id: initialSessionId, title: null, createdAt: "", updatedAt: "" })
        } else {
          // Create a new session for beads planning
          const newSession = await client.current.createSession("Beads Planner")
          if (newSession) {
            setSession(newSession)
          }
        }
      }
    }

    init()
  }, [autoConnect, initialSessionId])

  const addLine = useCallback((line: TerminalLine) => {
    setLines((prev) => [...prev, line])
  }, [])

  const sendCommand = useCallback(
    async (cmd: string, agent?: string) => {
      // Add the command line immediately
      addLine({
        type: "cmd",
        text: cmd,
        agent,
        timestamp: Date.now(),
      })

      if (isMockMode || !session) {
        // Mock mode - generate fake output
        addLine({
          type: "output",
          text: `[mock] Command would execute: ${cmd}`,
          timestamp: Date.now(),
        })
        return
      }

      // Real mode - send to OpenCode server
      setIsStreaming(true)
      abortController.current = new AbortController()

      try {
        await client.current.sendMessage(
          session.id,
          cmd,
          (line) => {
            addLine(line)
          },
          abortController.current.signal
        )
      } finally {
        setIsStreaming(false)
      }
    },
    [isMockMode, session, addLine]
  )

  const clearLines = useCallback(() => {
    setLines([])
  }, [])

  const reconnect = useCallback(async () => {
    const healthy = await client.current.checkHealth()
    setIsMockMode(!healthy)

    if (healthy && !session) {
      const newSession = await client.current.createSession("Beads Planner")
      if (newSession) {
        setSession(newSession)
      }
    }
  }, [session])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (abortController.current) {
        abortController.current.abort()
      }
    }
  }, [])

  return {
    status,
    session,
    lines,
    isStreaming,
    sendCommand,
    clearLines,
    reconnect,
    client: client.current,
    isMockMode,
  }
}
