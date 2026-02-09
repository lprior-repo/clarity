# Clarity Architecture

This document provides comprehensive architecture diagrams and system design documentation for the Clarity fullstack application.

## Table of Contents

- [System Overview](#system-overview)
- [Three-Crate Architecture](#three-crate-architecture)
- [Component Interactions](#component-interactions)
- [Data Flow](#data-flow)
- [Technology Stack](#technology-stack)
- [Design Principles](#design-principles)
- [Deployment Architecture](#deployment-architecture)

## System Overview

Clarity is a modern fullstack web application built with Rust, following functional programming principles and test-driven development. The system is designed around a three-crate architecture that separates concerns between frontend, backend, and shared business logic.

```
┌─────────────────────────────────────────────────────────────────┐
│                         Clarity System                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────┐         ┌──────────────┐         ┌──────────┐ │
│  │    Client    │◄────────┤    Server    │◄────────┤   Core   │ │
│  │   (Dioxus)   │  WS/HTTP │   (Axum)     │  Calls  │ (Shared) │ │
│  └──────────────┘         └──────────────┘         └──────────┘ │
│       │                         │                         │      │
│       │                         │                         │      │
│       ▼                         ▼                         ▼      │
│   Browser                  WebSocket               PostgreSQL  │
│   (UI)                     /REST API                Database   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### High-Level Architecture

The Clarity system consists of three main components:

1. **Client Layer** (`clarity-client`): Dioxus-based frontend application
2. **Server Layer** (`clarity-server`): Axum-based backend with REST and WebSocket support
3. **Core Layer** (`clarity-core`): Shared business logic, types, and database operations

## Three-Crate Architecture

### Crate Structure Diagram

```
clarity/
│
├── clarity-client/          # Frontend Application
│   ├── src/
│   │   ├── main.rs          # Dioxus app entry point
│   │   ├── components/      # Reusable UI components
│   │   ├── routes/          # Route handlers
│   │   └── hooks/           # Custom React-like hooks
│   └── Cargo.toml
│
├── clarity-server/          # Backend Application
│   ├── src/
│   │   ├── main.rs          # Axum server entry point
│   │   ├── handlers/        # HTTP request handlers
│   │   ├── routes/          # Route definitions
│   │   ├── websocket/       # WebSocket handlers
│   │   └── middleware/      # Custom middleware
│   └── Cargo.toml
│
├── clarity-core/            # Shared Business Logic
│   ├── src/
│   │   ├── lib.rs           # Library entry point
│   │   ├── models/          # Domain data models
│   │   ├── validation/      # Input validation
│   │   ├── db/              # Database operations
│   │   └── error.rs         # Error types
│   ├── migrations/          # SQLx migrations
│   └── Cargo.toml
│
└── Cargo.toml               # Workspace configuration
```

### Dependency Graph

```
┌─────────────────┐
│ clarity-client  │
│  (Dioxus UI)    │
└────────┬────────┘
         │ depends on
         ▼
┌─────────────────┐     ┌─────────────────┐
│ clarity-server  │────►│  clarity-core   │
│   (Axum API)    │     │ (Business Logic)│
└─────────────────┘     └────────┬────────┘
                                 │ depends on
                                 ▼
                        ┌─────────────────┐
                        │   PostgreSQL    │
                        │    Database     │
                        └─────────────────┘
```

### Crate Responsibilities

#### clarity-client
**Purpose**: User interface and client-side logic

**Responsibilities**:
- Render UI components using Dioxus
- Manage client-side state
- Handle user interactions
- Communicate with backend via WebSocket/HTTP
- Implement responsive design

**Key Technologies**:
- Dioxus 0.7 (React-like framework for Rust)
- WebSocket client for real-time updates
- HTTP client for REST API calls

**Does NOT**:
- Access database directly
- Implement business logic
- Handle authentication server-side

#### clarity-server
**Purpose**: Backend API and WebSocket server

**Responsibilities**:
- Serve REST API endpoints
- Handle WebSocket connections
- Process incoming requests
- Coordinate business logic via clarity-core
- Manage server-side state
- Implement authentication/authorization

**Key Technologies**:
- Axum 0.8 (web framework)
- Tokio (async runtime)
- Tower middleware
- WebSocket support

**Does NOT**:
- Implement business logic (delegates to clarity-core)
- Access database directly (uses clarity-core)
- Render UI

#### clarity-core
**Purpose**: Shared business logic and data layer

**Responsibilities**:
- Define domain models and types
- Implement business rules
- Handle database operations with SQLx
- Provide validation logic
- Define error types
- Expose reusable utilities

**Key Technologies**:
- SQLx 0.8 (compile-time checked queries)
- PostgreSQL driver
- Validation libraries
- Serde (serialization)

**Dependencies**:
- No framework-specific code
- Can be used independently by both client and server

## Component Interactions

### Request-Response Flow

```
┌─────────┐                    ┌─────────┐                    ┌──────────┐
│ Client  │                    │ Server  │                    │  Core    │
│(Dioxus) │                    │ (Axum)  │                    │          │
└────┬────┘                    └────┬────┘                    └────┬─────┘
     │                              │                              │
     │ 1. User Action               │                              │
     ├─────────────────────────────►│                              │
     │   HTTP/WebSocket Request     │                              │
     │                              │                              │
     │                              │ 2. Route Request             │
     │                              ├─────────────────────────────►│
     │                              │   to Business Logic          │
     │                              │                              │
     │                              │                              │ 3. Query
     │                              │                              ├───────►
     │                              │                              │ Database
     │                              │                              │
     │                              │                              │ 4. Result
     │                              │                              ◄───────┤
     │                              │                              │
     │                              │ 5. Return Data               │
     │                              ◄─────────────────────────────┤
     │                              │                              │
     │ 6. Send Response             │                              │
     ◄─────────────────────────────┤                              │
     │   (JSON / WebSocket msg)     │                              │
     │                              │                              │
     │ 7. Update UI                 │                              │
     ├──────────────────────────────                            │
     │                              │                              │
```

### Sequence Diagram: User Request

```
User          Client           Server          Core           Database
 │              │                │               │                │
 │ 1. Click     │                │               │                │
 ├─────────────►│                │               │                │
 │              │                │               │                │
 │              │ 2. HTTP GET    │               │                │
 │              ├───────────────►│               │                │
 │              │    /api/data    │               │                │
 │              │                │               │                │
 │              │                │ 3. validate() │                │
 │              │                ├─────────────► │                │
 │              │                │               │                │
 │              │                │               │ 4. SQL Query   │
 │              │                │               ├───────────────►│
 │              │                │               │                │
 │              │                │               │ 5. Rows        │
 │              │                │               ◄───────────────┤
 │              │                │               │                │
 │              │                │ 6. Result     │                │
 │              │                ◄───────────────┤                │
 │              │                │               │                │
 │              │ 7. JSON        │               │                │
 │              │    Response    │               │                │
 │              ◄───────────────┤               │                │
 │              │                │               │                │
 │ 8. Render    │                │               │                │
 ◄──────────────┤                │               │                │
```

### WebSocket Real-Time Communication

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│   Client     │         │   Server     │         │    Core      │
│  (Dioxus)    │         │   (Axum)     │         │              │
└──────┬───────┘         └──────┬───────┘         └──────┬───────┘
       │                        │                        │
       │ 1. WebSocket Connect   │                        │
       ├───────────────────────►│                        │
       │                        │                        │
       │ 2. Init Session        │                        │
       │◄───────────────────────┤                        │
       │                        │                        │
       │ 3. Subscribe Events    │                        │
       ├───────────────────────►│                        │
       │                        │                        │
       │                        │ 4. Register Listener   │
       │                        ├───────────────────────►│
       │                        │                        │
       │                        │ 5. Ack Subscription    │
       │                        ◄───────────────────────┤
       │                        │                        │
       │ 6. Receive Updates     │                        │
       │◄───────────────────────┤                        │
       │    (Server Push)       │                        │
       │                        │                        │
       │ 7. Update UI           │                        │
       ├─────────────────────┐  │                        │
       │                        │                        │
       │ [Continuous Bidirectional Communication]         │
       │◄──────────────────────►│                        │
       │                        │                        │
```

## Data Flow

### Application Data Flow

```
┌────────────────────────────────────────────────────────────┐
│                      Data Flow Diagram                     │
└────────────────────────────────────────────────────────────┘

User Input
    │
    ▼
┌─────────────┐
│  Client     │  Validate UI Input
│  Component  │─────────────┐
└──────┬──────┘             │
       │                    ▼
       │              ┌─────────────┐
       │              │ Validation  │  Check format, type, constraints
       │              │   Layer     │─────────────┐
       │              └─────────────┘             │
       │                                        ▼
       │                                  ┌─────────────┐
       │                                  │ Is Valid?   │
       │                                  └──────┬──────┘
       │                                         │
       │                    ┌────────────────────┴────────────────────┐
       │                    │                                         │
       │                    ▼ No                                     ▼ Yes
       │              ┌─────────────┐                          ┌─────────────┐
       │              │ Show Error  │                          │ HTTP/WS     │
       │              │   to User   │                          │ Request     │
       │              └─────────────┘                          └──────┬──────┘
       │                                                         │
       │                                                         ▼
       │                                                   ┌─────────────┐
       │                                                   │   Server    │
       │                                                   │   Handler   │
       │                                                   └──────┬──────┘
       │                                                          │
       │                                                          ▼
       │                                                   ┌─────────────┐
       │                                                   │   Core      │
       │                                                   │  Business   │
       │                                                   │   Logic     │
       │                                                   └──────┬──────┘
       │                                                          │
       │                                                          ▼
       │                                                   ┌─────────────┐
       │                                                   │  Database   │
       │                                                   │  Query      │
       │                                                   └──────┬──────┘
       │                                                          │
       │                                                          ▼
       │                                                   ┌─────────────┐
       │                                                   │ PostgreSQL │
       │                                                   └──────┬──────┘
       │                                                          │
       │                    ┌─────────────────────────────────────┘
       │                    │
       ▼                    ▼
┌─────────────┐      ┌─────────────┐
│  Result     │      │    Error    │
│  Display    │      │  Handling   │
└─────────────┘      └─────────────┘
```

### Database Interaction Flow

```
┌────────────────────────────────────────────────────────────┐
│              Database Layer (clarity-core)                 │
└────────────────────────────────────────────────────────────┘

Application Request
    │
    ▼
┌─────────────────┐
│  Repository     │  Generic CRUD operations
│    Trait        │──────────────┐
└────────┬────────┘              │
         │                       ▼
         │                 ┌─────────────────┐
         │                 │ SQLx Query      │  Compile-time
         │                 │   Builder       │  checked queries
         │                 └────────┬────────┘
         │                          │
         │                          ▼
         │                 ┌─────────────────┐
         │                 │ Connection Pool │  deadpool-postgres
         │                 │   Manager       │
         │                 └────────┬────────┘
         │                          │
         │                          ▼
         │                 ┌─────────────────┐
         │                 │  PostgreSQL     │
         │                 │    Database     │
         │                 └────────┬────────┘
         │                          │
         │                          ▼
         │                 ┌─────────────────┐
         │                 │ Query Result    │
         │                 │   (Rows)        │
         │                 └────────┬────────┘
         │                          │
         ▼                          ▼
┌─────────────────┐      ┌─────────────────┐
│  Result<T, E>   │      │    Error        │
│  (Typed Data)   │      │  Handling       │
└─────────────────┘      └─────────────────┘
```

## Technology Stack

### Frontend Stack

```
┌────────────────────────────────────────────────────────────┐
│                    Frontend Stack                           │
├────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Dioxus     │  │   Web APIs   │  │   CSS/       │     │
│  │  Framework   │  │  (Fetch/WS)  │  │  Styling     │     │
│  │   (0.7)      │  │              │  │              │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│         │                  │                  │              │
│         └──────────────────┴──────────────────┘              │
│                            │                                 │
│                            ▼                                 │
│                   ┌──────────────┐                          │
│                   │   Browser    │                          │
│                   │  Rendering   │                          │
│                   └──────────────┘                          │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### Backend Stack

```
┌────────────────────────────────────────────────────────────┐
│                    Backend Stack                            │
├────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │    Axum      │  │    Tokio     │  │    Tower     │     │
│  │   (0.8)      │  │   Runtime    │  │  Middleware  │     │
│  │              │  │              │  │              │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│         │                  │                  │              │
│         └──────────────────┴──────────────────┘              │
│                            │                                 │
│                            ▼                                 │
│                   ┌──────────────┐                          │
│                   │  HTTP/WS     │                          │
│                   │   Server     │                          │
│                   └──────────────┘                          │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### Data Layer

```
┌────────────────────────────────────────────────────────────┐
│                    Data Layer                               │
├────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │    SQLx      │  │ PostgreSQL   │  │  Connection  │     │
│  │   (0.8)      │  │   Driver     │  │    Pool      │     │
│  │              │  │              │  │  (deadpool)  │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│         │                  │                  │              │
│         └──────────────────┴──────────────────┘              │
│                            │                                 │
│                            ▼                                 │
│                   ┌──────────────┐                          │
│                   │   Database   │                          │
│                   │   Queries    │                          │
│                   └──────────────┘                          │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### Full Stack Integration

```
┌──────────────────────────────────────────────────────────────┐
│                   Full Stack Architecture                    │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────┐         ┌─────────────────┐            │
│  │  Frontend       │         │  Backend        │            │
│  │  - Dioxus 0.7   │◄────────│  - Axum 0.8     │            │
│  │  - Web APIs     │  WS/HT  │  - Tokio        │            │
│  │  - Components   │         │  - Tower        │            │
│  └─────────────────┘         └────────┬────────┘            │
│                                       │                      │
│                                       ▼                      │
│                              ┌─────────────────┐             │
│                              │  Shared Core    │             │
│                              │  - SQLx 0.8     │             │
│                              │  - Models       │             │
│                              │  - Validation   │             │
│                              │  - Business     │             │
│                              │    Logic        │             │
│                              └────────┬────────┘             │
│                                       │                      │
│                                       ▼                      │
│                              ┌─────────────────┐             │
│                              │  PostgreSQL     │             │
│                              │  Database       │             │
│                              └─────────────────┘             │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## Design Principles

### Functional Programming Principles

```
┌────────────────────────────────────────────────────────────┐
│            Functional Programming in Clarity               │
└────────────────────────────────────────────────────────────┘

1. IMMUTABILITY
   ┌─────────────────┐
   │  Data           │  All data structures immutable by default
   │  Structures     │  Use `&` references for read-only access
   └─────────────────┘

2. PURE FUNCTIONS
   ┌─────────────────┐
   │  Functions      │  No side effects
   │                 │  Same input → Same output
   │                 │  Easy to test and reason about
   └─────────────────┘

3. EXPLICIT ERROR HANDLING
   ┌─────────────────┐
   │  Result<T, E>   │  No exceptions or panics
   │                 │  Errors are values
   │                 │  Forced handling with `?` operator
   └─────────────────┘

4. FUNCTION COMPOSITION
   ┌─────────────────┐
   │  Combinators    │  `map()`, `and_then()`, `filter()`
   │                 │  Build complex operations from simple ones
   │                 │  Iterator-based transformations
   └─────────────────┘

5. TYPE SAFETY
   ┌─────────────────┐
   │  Type System    │  Leverage Rust's type system
   │                 │  Compile-time guarantees
   │                 │  Prevent runtime errors
   └─────────────────┘
```

### Zero-Panic Architecture

```
┌────────────────────────────────────────────────────────────┐
│              Zero-Panic Error Handling                     │
└────────────────────────────────────────────────────────────┘

FORBIDDEN (compile errors):
    ❌ unwrap()
    ❌ expect()
    ❌ panic!()
    ❌ todo!()
    ❌ unimplemented!()

REQUIRED:
    ✅ Result<T, E>
    ✅ Option<T>
    ✅ `?` operator
    ✅ Proper error propagation

Example Pattern:

    ❌ BAD:
        let user = get_user(id).unwrap();

    ✅ GOOD:
        let user = get_user(id)
            .map_err(|e| ApiError::NotFound)?;

Error Flow:
    ┌──────────┐     ┌──────────┐     ┌──────────┐
    │  Domain  │────►│  Core    │────►│ Server   │
    │  Error   │     │  Error   │     │  Error   │
    └──────────┘     └──────────┘     └──────────┘
         │                │                │
         │                │                │
         ▼                ▼                ▼
    ┌──────────────────────────────────────┐
    │         Client Error Response        │
    │    (HTTP status + JSON error body)   │
    └──────────────────────────────────────┘
```

## Deployment Architecture

### Development Environment

```
┌────────────────────────────────────────────────────────────┐
│              Development Deployment                        │
└────────────────────────────────────────────────────────────┘

┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│  Terminal 1 │         │  Terminal 2 │         │  Terminal 3 │
│             │         │             │         │             │
│  $ moon     │         │  $ moon     │         │  $ psql     │
│     :server │         │     :client │         │     clarity │
│             │         │             │         │             │
│  Axum       │         │  Dioxus     │         │  PostgreSQL │
│  :3000      │         │  :8080      │         │  :5432      │
└─────────────┘         └─────────────┘         └─────────────┘
       │                       │                       │
       └───────────────────────┴───────────────────────┘
                               │
                               ▼
                    ┌─────────────────┐
                    │  Localhost      │
                    │  Development    │
                    │  Environment   │
                    └─────────────────┘
```

### Production Architecture (Future)

```
┌────────────────────────────────────────────────────────────┐
│              Production Deployment (Planned)               │
└────────────────────────────────────────────────────────────┘

                    ┌─────────────────┐
                    │   Load Balancer │
                    │   (Nginx/Caddy) │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
              ▼                             ▼
     ┌─────────────┐                ┌─────────────┐
     │  Instance 1 │                │  Instance 2 │
     │  Clarity    │                │  Clarity    │
     │  Server     │                │  Server     │
     └──────┬──────┘                └──────┬──────┘
            │                              │
            └──────────┬───────────────────┘
                       │
                       ▼
              ┌─────────────────┐
              │  PostgreSQL     │
              │  Primary        │
              │  (with replica) │
              └─────────────────┘

Features:
    - Horizontal scaling
    - Database replication
    - Connection pooling
    - Health checks
    - Graceful shutdown
    - Log aggregation
```

## Key Architectural Decisions

### Why Three Crates?

1. **Separation of Concerns**: Each crate has a single, well-defined responsibility
2. **Reusability**: clarity-core can be used independently by other tools
3. **Testing**: Core logic can be tested without frontend/backend dependencies
4. **Compilation**: Faster incremental compilation with clear boundaries
5. **Deployment**: Frontend and backend can be deployed independently

### Why Axum?

1. **Type Safety**: Leverages Rust's type system for route handlers
2. **Performance**: Built on Tokio for high-performance async I/O
3. **WebSocket Support**: First-class WebSocket implementation
4. **Ecosystem**: Part of Tokio ecosystem, excellent middleware support
5. **Modern**: Active development, Rust async ecosystem

### Why Dioxus?

1. **Rust Native**: No JavaScript required, full type safety
2. **React-Like**: Familiar component model and hooks
3. **Performance**: Compiled to WebAssembly, near-native speed
4. **Fullstack**: Can run on server and client with same code
5. **Type Safety**: Share types with backend via clarity-core

### Why SQLx?

1. **Compile-Time Checking**: Queries verified at compile time
2. **Async**: First-class async support with Tokio
3. **Type Safety**: Maps SQL types directly to Rust types
4. **Performance**: Zero-cost abstraction over database driver
5. **Flexibility**: Write raw SQL with full database feature support

## Security Considerations

### Current Security Measures

```
┌────────────────────────────────────────────────────────────┐
│                 Security Architecture                       │
└────────────────────────────────────────────────────────────┘

1. INPUT VALIDATION
   ┌─────────────────┐
   │  Client Side    │  Validate format, type, constraints
   │  Validation     │  First line of defense
   └─────────────────┘
           │
           ▼
   ┌─────────────────┐
   │  Server Side    │  Validate all inputs (never trust client)
   │  Validation     │  Business logic enforcement
   └─────────────────┘

2. DATABASE SECURITY
   ┌─────────────────┐
   │  SQLx           │  Compile-time query validation
   │  Prepared       │  Prevents SQL injection
   │  Statements     │  Type-safe parameter binding
   └─────────────────┘

3. ERROR HANDLING
   ┌─────────────────┐
   │  No Panics      │  Zero-panic architecture
   │  Result Types   │  Graceful error handling
   └─────────────────┘
```

### Future Security Enhancements

1. **Authentication/Authorization**: User identity and permissions
2. **Rate Limiting**: Prevent abuse and DoS attacks
3. **CORS**: Proper cross-origin resource sharing
4. **HTTPS**: TLS/SSL for all connections
5. **Secrets Management**: Environment variables for sensitive data
6. **Input Sanitization**: Prevent XSS attacks
7. **CSRF Protection**: Cross-site request forgery tokens

## Performance Considerations

### Current Optimizations

```
┌────────────────────────────────────────────────────────────┐
│                Performance Architecture                     │
└────────────────────────────────────────────────────────────┘

1. BUILD PERFORMANCE
   ┌─────────────────┐
   │  MoonRepo       │  Aggressive caching (6-7ms vs 450ms)
   │  Caching        │  Parallel crate builds
   └─────────────────┘

2. RUNTIME PERFORMANCE
   ┌─────────────────┐
   │  Axum/Tokio     │  Async I/O, non-blocking
   │  Zero-Copy      │  Minimize data copying
   └─────────────────┘

3. DATABASE PERFORMANCE
   ┌─────────────────┐
   │  Connection     │  Reuse database connections
   │  Pooling        │  Reduce overhead
   └─────────────────┘
```

### Future Performance Enhancements

1. **Caching**: Redis for frequently accessed data
2. **CDN**: Static asset delivery
3. **Compression**: Response body compression
4. **Lazy Loading**: Load data on demand
5. **Pagination**: Limit query result sets
6. **Indexing**: Optimize database indexes
7. **Monitoring**: Performance metrics and profiling

## Monitoring and Observability

### Planned Monitoring Stack

```
┌────────────────────────────────────────────────────────────┐
│            Monitoring and Observability                     │
└────────────────────────────────────────────────────────────┘

1. LOGGING
   ┌─────────────────┐
   │  Tracing        │  Structured logging
   │  Subscriber     │  Multiple log levels
   └─────────────────┘

2. METRICS (Future)
   ┌─────────────────┐
   │  Prometheus     │  Request latency, error rates
   │  Metrics        │  Resource utilization
   └─────────────────┘

3. TRACING (Future)
   ┌─────────────────┐
   │  Distributed    │  Request flow across services
   │  Tracing        │  Performance bottlenecks
   └─────────────────┘

4. HEALTH CHECKS (Future)
   ┌─────────────────┐
   │  HTTP Endpoint  │  `/health` route
   │  Status         │  Database connectivity
   └─────────────────┘
```

## Extension Points

### Adding New Features

```
┌────────────────────────────────────────────────────────────┐
│                 Extension Architecture                      │
└────────────────────────────────────────────────────────────┘

1. NEW API ENDPOINT
   ├── Add handler in clarity-server/src/handlers/
   ├── Add route in clarity-server/src/routes/
   ├── Add business logic in clarity-core/src/
   └── Add tests for all layers

2. NEW DATABASE MODEL
   ├── Add struct in clarity-core/src/models/
   ├── Add migration in clarity-core/migrations/
   ├── Implement Repository trait
   └── Add validation rules

3. NEW UI COMPONENT
   ├── Add component in clarity-client/src/components/
   ├── Add hook in clarity-client/src/hooks/
   ├── Add route in clarity-client/src/routes/
   └── Wire up to backend API

4. NEW BUSINESS LOGIC
   ├── Add function in clarity-core/src/
   ├── Implement validation
   ├── Add error types
   └── Write unit tests
```

## Conclusion

The Clarity architecture is designed around:
- **Simplicity**: Clear separation of concerns
- **Type Safety**: Leverage Rust's type system
- **Performance**: Async, zero-copy, connection pooling
- **Maintainability**: Functional patterns, zero-panic
- **Testability**: Pure functions, dependency injection
- **Scalability**: Modular design, independent deployment

For implementation details, see:
- [README.md](../README.md) - Project overview and setup
- [AGENTS.md](../AGENTS.md) - Development guidelines
- [docs/zero-unwrap-philosophy.md](./zero-unwrap-philosophy.md) - Error handling philosophy
- [docs/workspace-setup-summary.md](./workspace-setup-summary.md) - Development environment
