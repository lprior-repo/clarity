# Clarity System Diagrams

Visual representations of the Clarity system architecture and component interactions.

## Table of Contents

- [High-Level System Architecture](#high-level-system-architecture)
- [Component Interaction Diagram](#component-interaction-diagram)
- [Data Flow Diagram](#data-flow-diagram)
- [Request Lifecycle](#request-lifecycle)
- [WebSocket Communication Flow](#websocket-communication-flow)
- [Error Handling Flow](#error-handling-flow)
- [Development Workflow](#development-workflow)

## High-Level System Architecture

```
╔═══════════════════════════════════════════════════════════════════╗
║                        CLARITY SYSTEM                             ║
╠═══════════════════════════════════════════════════════════════════╣
║                                                                   ║
║   ┌──────────────┐         ┌──────────────┐         ┌─────────┐  ║
║   │              │         │              │         │         │  ║
║   │   Dioxus     │◄────────┤    Axum      │◄────────┤  Core   │  ║
║   │   Client     │  WS/HT  │   Server     │  Calls  │ Shared  │  ║
║   │              │         │              │         │         │  ║
║   │  Frontend    │         │   Backend    │         │   DB    │  ║
║   │              │         │              │         │  Layer  │  ║
║   └──────┬───────┘         └──────┬───────┘         └────┬────┘  ║
║          │                        │                      │       ║
║          │                        │                      │       ║
║          ▼                        ▼                      ▼       ║
║   ┌──────────────┐         ┌──────────────┐         ┌─────────┐  ║
║   │              │         │              │         │         │  ║
║   │   Browser    │         │  WebSocket   │         │  PostgreSQL║
║   │   (UI)       │         │  + REST API  │         │ Database ║
║   │              │         │              │         │         │  ║
║   │  :8080       │         │  :3000       │         │  :5432  │  ║
║   └──────────────┘         └──────────────┘         └─────────┘  ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
```

## Component Interaction Diagram

```
╔════════════════════════════════════════════════════════════════════╗
║                    COMPONENT INTERACTIONS                          ║
╚════════════════════════════════════════════════════════════════════╝

    ┌──────────────────────────────────────────────────────────┐
    │                      USER LAYER                           │
    │  ┌────────────────────────────────────────────────────┐  │
    │  │           User Interactions (Browser)              │  │
    │  │  • Click events  • Form submissions  • Navigation │  │
    │  └────────────────────────────────────────────────────┘  │
    └────────────────────┬───────────────────────────────────────┘
                         │
                         ▼
    ┌──────────────────────────────────────────────────────────┐
    │                   PRESENTATION LAYER                      │
    │  ┌────────────────────────────────────────────────────┐  │
    │  │           clarity-client (Dioxus)                  │  │
    │  │  ┌────────────┐  ┌────────────┐  ┌─────────────┐ │  │
    │  │  │ Components │  │   Routes   │  │    Hooks    │ │  │
    │  │  │    UI      │  │ Navigation │  │   State     │ │  │
    │  │  └────────────┘  └────────────┘  └─────────────┘ │  │
    │  └────────────────────────────────────────────────────┘  │
    └────────────────────┬───────────────────────────────────────┘
                         │
                         ▼
    ┌──────────────────────────────────────────────────────────┐
    │                    API LAYER                              │
    │  ┌────────────────────────────────────────────────────┐  │
    │  │           clarity-server (Axum)                    │  │
    │  │  ┌────────────┐  ┌────────────┐  ┌─────────────┐ │  │
    │  │  │ Handlers   │  │   Routes   │  │ Middleware  │ │  │
    │  │  │  HTTP/WS   │  │  Endpoints │  │   Auth      │ │  │
    │  │  └────────────┘  └────────────┘  └─────────────┘ │  │
    │  └────────────────────────────────────────────────────┘  │
    └────────────────────┬───────────────────────────────────────┘
                         │
                         ▼
    ┌──────────────────────────────────────────────────────────┐
    │                   BUSINESS LAYER                          │
    │  ┌────────────────────────────────────────────────────┐  │
    │  │           clarity-core (Shared)                    │  │
    │  │  ┌────────────┐  ┌────────────┐  ┌─────────────┐ │  │
    │  │  │  Models    │  │ Validation │  │   Error     │ │  │
    │  │  │  Domain    │  │   Logic    │  │  Handling   │ │  │
    │  │  └────────────┘  └────────────┘  └─────────────┘ │  │
    │  └────────────────────────────────────────────────────┘  │
    └────────────────────┬───────────────────────────────────────┘
                         │
                         ▼
    ┌──────────────────────────────────────────────────────────┐
    │                    DATA LAYER                             │
    │  ┌────────────────────────────────────────────────────┐  │
    │  │           Database Operations                       │  │
    │  │  ┌────────────┐  ┌────────────┐  ┌─────────────┐ │  │
    │  │  │   SQLx     │  │ Connection │  │  Migrations │ │  │
    │  │  │  Queries   │  │   Pool     │  │   Schema    │ │  │
    │  │  └────────────┘  └────────────┘  └─────────────┘ │  │
    │  └────────────────────────────────────────────────────┘  │
    └────────────────────┬───────────────────────────────────────┘
                         │
                         ▼
    ┌──────────────────────────────────────────────────────────┐
    │                   STORAGE LAYER                           │
    │  ┌────────────────────────────────────────────────────┐  │
    │  │           PostgreSQL Database                       │  │
    │  │  • Tables  • Indexes  • Constraints  • Data        │  │
    │  └────────────────────────────────────────────────────┘  │
    └──────────────────────────────────────────────────────────┘
```

## Data Flow Diagram

```
╔════════════════════════════════════════════════════════════════════╗
║                      DATA FLOW (Read Path)                        ║
╚════════════════════════════════════════════════════════════════════╝

  Browser               Dioxus Client         Axum Server         Core         PostgreSQL
    │                        │                    │                │                │
    │  1. User Action        │                    │                │                │
    │  (Click, Type, etc)    │                    │                │                │
    ├───────────────────────►│                    │                │                │
    │                        │                    │                │                │
    │                        │  2. Event Handler  │                │                │
    │                        │    Triggers        │                │                │
    │                        │    Request         │                │                │
    │                        ├───────────────────►│                │                │
    │                        │  HTTP GET/POST     │                │                │
    │                        │                    │                │                │
    │                        │                    │  3. Route       │                │
    │                        │                    │  Matched        │                │
    │                        │                    │                │                │
    │                        │                    │  4. Validate    │                │
    │                        │                    ├──────────────►│                │
    │                        │                    │  Input         │                │
    │                        │                    │                │                │
    │                        │                    │                │  5. Build      │
    │                        │                    │                │  Query         │
    │                        │                    │                ├──────────────►│
    │                        │                    │                │                │
    │                        │                    │                │                │ 6. Execute
    │                        │                    │                │                ├──────►
    │                        │                    │                │                │ SQL
    │                        │                    │                │                │
    │                        │                    │                │                │ 7. Return
    │                        │                    │                │                │◄──────┤
    │                        │                    │                │   Rows        │
    │                        │                    │                │                │
    │                        │                    │                │  8. Map to    │
    │                        │                    │                │  Domain Type  │
    │                        │                    │◄───────────────┤                │
    │                        │                    │   Result<T,E>  │                │
    │                        │                    │                │                │
    │                        │  9. JSON Response  │                │                │
    │                        │◄───────────────────┤                │                │
    │                        │                    │                │                │
    │  10. Update UI        │                    │                │                │
    │  Re-render            │                    │                │                │
    │◄──────────────────────┤                    │                │                │
```

## Request Lifecycle

```
╔════════════════════════════════════════════════════════════════════╗
║                 HTTP REQUEST LIFECYCLE                            ║
╚════════════════════════════════════════════════════════════════════╝

1. REQUEST INITIATION
   ┌──────────────────────────────────────────────────────────────┐
   │  Browser: User clicks button / submits form                 │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼
2. CLIENT PROCESSING
   ┌──────────────────────────────────────────────────────────────┐
   │  Dioxus Client:                                             │
   │    • Capture event                                          │
   │    • Client-side validation                                 │
   │    • Build HTTP request                                     │
   │    • Add authentication headers (if needed)                 │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼
3. NETWORK TRANSMISSION
   ┌──────────────────────────────────────────────────────────────┐
   │  HTTP Request sent to :3000                                 │
   │    • Method: GET/POST/PUT/DELETE                            │
   │    • Path: /api/resource                                    │
   │    • Headers: Content-Type, Authorization                   │
   │    • Body: JSON payload                                     │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼
4. SERVER RECEIVAL
   ┌──────────────────────────────────────────────────────────────┐
   │  Axum Server:                                               │
   │    • Receive request on Tokio runtime                       │
   │    • Parse HTTP method and path                             │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼
5. ROUTE MATCHING
   ┌──────────────────────────────────────────────────────────────┐
   │  Axum Router:                                               │
   │    • Match route pattern                                    │
   │    • Extract path parameters                               │
   │    • Call handler function                                 │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼
6. REQUEST HANDLING
   ┌──────────────────────────────────────────────────────────────┐
   │  Handler Function:                                          │
   │    • Extract request data                                   │
   │    • Validate input                                         │
   │    • Call business logic (clarity-core)                     │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼
7. RESPONSE TRANSMISSION
   ┌──────────────────────────────────────────────────────────────┐
   │  Server sends HTTP response                                 │
   │    • Status code (200, 400, 500, etc)                       │
   │    • Headers (Content-Type, etc)                            │
   │    • Body (JSON data or error message)                      │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼
8. CLIENT RECEIVAL
   ┌──────────────────────────────────────────────────────────────┐
   │  Dioxus receives response:                                  │
   │    • Parse JSON payload                                     │
   │    • Update component state                                 │
   │    • Trigger re-render                                      │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼
9. UI UPDATE
   ┌──────────────────────────────────────────────────────────────┐
   │  Browser displays updated interface to user                 │
   └──────────────────────────────────────────────────────────────┘
```

## WebSocket Communication Flow

```
╔════════════════════════════════════════════════════════════════════╗
║              WEBSOCKET REAL-TIME COMMUNICATION                      ║
╚════════════════════════════════════════════════════════════════════╝

    Dioxus Client              Axum Server              clarity-Core
         │                         │                         │
         │  1. WebSocket           │                         │
         │     Upgrade Request     │                         │
         ├────────────────────────►│                         │
         │   GET /ws               │                         │
         │                         │                         │
         │                         │  2. Accept Connection   │
         │                         │    Create WebSocket     │
         │                         │    Task                 │
         │                         │                         │
         │  3. Connection          │                         │
         │     Established         │                         │
         │◄────────────────────────┤                         │
         │   (101 Switching        │                         │
         │    Protocols)           │                         │
         │                         │                         │
         │  4. Subscribe           │  5. Register            │
         │     to Events           ├────────────────────────►│  Listener
         ├────────────────────────►│    Subscribe            │
         │   {                    │    {                    │  Topic
         │     "action": "sub",   │      "topic": "updates" │
         │     "topic": "updates"│    }                    │
         │   }                    │                         │
         │                         │  6. Ack Subscription    │
         │  7. Subscribed          │◄────────────────────────┤
         │◄────────────────────────┤    {                    │
         │   {                    │      "status": "ok",     │
         │     "status": "ok",    │      "topic": "updates"  │
         │     "topic": "updates" │    }                    │
         │   }                    │                         │
         │                         │                         │
         │  ═════════════════════════════════════════════════│
         │  CONTINUOUS BIDIRECTIONAL COMMUNICATION            │
         │  ═════════════════════════════════════════════════│
         │                         │                         │
         │  8. Server Push         │  9. Broadcast           │  10. Get Data
         │◄────────────────────────┤    to Subscribers  ────►│    from DB
         │   {                    │                         │
         │     "type": "update",  │                         │
         │     "data": {...}      │                         │
         │   }                    │                         │
         │                         │                         │
         │  11. Update UI         │                         │
         │  Re-render Component   │                         │
         ├─────────────────────┐  │                         │
         │                     │  │                         │
         │  12. Client Message │  │                         │
         ├────────────────────►│  │                         │
         │   {                 │  │                         │
         │     "action": "..", │  │                         │
         │     "payload": {}   │  │                         │
         │   }                 │  │                         │
         │                     │  │                         │
         │                     │  13. Process Message       │
         │                     ├───────────────────────────►│
         │                     │                         │
         │                     │  14. Result              │
         │                     │◄──────────────────────────┤
         │                     │                         │
         │                     │  15. Broadcast Response  │
         │  16. Response       │◄────────────────────────┤
         │◄────────────────────┤                         │
         │                     │                         │
         │  [Loop Continues...]│                         │
         │                     │  17. Close               │
         │  18. Close           ├────────────────────────►│
         ├────────────────────►│  Cleanup                 │
         │                     │  Unsubscribe             │
         │  19. Closed          │                         │
         │◄────────────────────┤                         │
```

## Error Handling Flow

```
╔════════════════════════════════════════════════════════════════════╗
║                   ZERO-PANIC ERROR HANDLING                         ║
╚════════════════════════════════════════════════════════════════════╝

    Client Request
          │
          ▼
    ┌─────────────┐
    │  Handler    │
    └──────┬──────┘
           │
           ▼
    ┌─────────────┐
    │  Validate   │
    └──────┬──────┘
           │
           ├─ Invalid ───────────────────────┐
           │                                 │
           ▼ Valid                           ▼
    ┌─────────────┐                 ┌─────────────────┐
    │  Call Core  │                 │ Return 400 Bad  │
    └──────┬──────┘                 │    Request      │
           │                         └────────┬────────┘
           ▼                                  │
    ┌─────────────┐                          │
    │  Business   │                          │
    │  Logic      │                          │
    └──────┬──────┘                          │
           │                                  │
           ├─ Error ──────────────────┐       │
           │                          │       │
           ▼ OK                       ▼       ▼
    ┌─────────────┐          ┌─────────────────┐
    │  Return OK  │          │ Error Handling  │
    └─────────────┘          │  Chain          │
           │                 └────────┬────────┘
           │                          │
           │          ┌───────────────┴───────────────┐
           │          │                              │
           │          ▼                              ▼
           │    ┌─────────────┐              ┌─────────────┐
           │    │   Domain    │              │   Database  │
           │    │   Error     │              │    Error    │
           │    └──────┬──────┘              └──────┬──────┘
           │           │                            │
           └───────────┴────────────┬───────────────┘
                                    │
                                    ▼
                             ┌─────────────┐
                             │  Map to     │
                             │  HTTP       │
                             │  Response   │
                             └──────┬──────┘
                                    │
               ┌────────────────────┴────────────────────┐
               │                                          │
               ▼                                          ▼
        ┌───────────────┐                          ┌───────────────┐
        │ Client Error  │                          │ Server Error  │
        │ 4xx           │                          │ 5xx           │
        │ {             │                          │ {             │
        │   "error": {  │                          │   "error": {  │
        │     "code":   │                          │     "code":   │
        │       "VALIDATION_ERROR",               │       "INTERNAL_ERROR",
        │     "message": "..."                    │     "message": "..."
        │   }            │                          │   }            │
        │ }              │                          │ }              │
        └───────┬────────┘                          └───────┬────────┘
                │                                          │
                └────────────────────┬─────────────────────┘
                                     │
                                     ▼
                              ┌─────────────┐
                              │   Client    │
                              │   Display   │
                              │   Error     │
                              └─────────────┘
```

## Development Workflow

```
╔════════════════════════════════════════════════════════════════════╗
║                    TDD15 DEVELOPMENT WORKFLOW                     ║
╚════════════════════════════════════════════════════════════════════╝

1. TRIAGE (Phase 0)
   ┌──────────────────────────────────────────────────────────────┐
   │  Assess Complexity:                                          │
   │    • SIMPLE   → [0,4,5,6,14,15]  (~60% faster)              │
   │    • MEDIUM   → [0,1,2,4,5,6,7,9,11,15]  (~35% faster)      │
   │    • COMPLEX  → All 16 phases                               │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼

2. RESEARCH (Phase 1) - Skipped for SIMPLE
                           │
                           ▼

3. PLAN (Phase 2) - Skipped for SIMPLE
                           │
                           ▼

4. RED (Phase 4)
   ┌──────────────────────────────────────────────────────────────┐
   │  Write Failing Test:                                         │
   │    moon run :test -- my_test                                │
   │    ✓ Test MUST fail (RED)                                   │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼

5. GREEN (Phase 5)
   ┌──────────────────────────────────────────────────────────────┐
   │  Implement Minimal Code:                                     │
   │    • Write just enough to pass                              │
   │    moon run :test -- my_test                                │
   │    ✓ Test passes (GREEN)                                    │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼

6. REFACTOR (Phase 6)
   ┌──────────────────────────────────────────────────────────────┐
   │  Improve Code:                                               │
   │    • Clean up implementation                                │
   │    • Apply functional patterns                              │
   │    moon run :test -- my_test                                │
   │    ✓ Tests still pass                                       │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼

7. CODE QUALITY (Quick Loop)
   ┌──────────────────────────────────────────────────────────────┐
   │  moon run :quick  (6-7ms with cache)                        │
   │    ✓ Format check                                           │
   │    ✓ Clippy check                                           │
   └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼

8. MF#1 / VERIFY (Skipped for SIMPLE)
                           │
                           ▼

9. IMPLEMENTATION (Skipped for SIMPLE)
                           │
                           ▼

10. FULL PIPELINE
    ┌──────────────────────────────────────────────────────────────┐
    │  moon run :ci                                               │
    │    ✓ Format check (rustfmt)                                 │
    │    ✓ Lint check (clippy)                                    │
    │    ✓ All tests (unit, integration, doc)                     │
    └────────────────────────┬─────────────────────────────────────┘
                           │
                           ▼

11. LAND (Phase 14-15)
    ┌──────────────────────────────────────────────────────────────┐
    │  jj commit -m "feat: description"                           │
    │  jj git push                                               │
    │    ✓ Work complete and pushed                              │
    └──────────────────────────────────────────────────────────────┘


Key Commands:
    moon run :quick     → Fast format + lint (cached, 6-7ms)
    moon run :fmt-fix   → Auto-fix formatting
    moon run :ci        → Full CI pipeline
    moon run :test      → Run all tests
    moon run :server    → Start backend
    moon run :client    → Start frontend
```

---

**Note**: For implementation details and code examples, see [architecture.md](./architecture.md)
