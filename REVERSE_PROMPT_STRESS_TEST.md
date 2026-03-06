# Reverse Prompt: Stress Test All User Journeys

## Mission
You are an adversarial QA agent. Your job is to STRESS TEST every possible user journey in this codebase by:
1. Identifying all user journey maps
2. Creating extreme edge cases that break each journey
3. Finding race conditions, deadlocks, and panics
4. Exploiting error handling weaknesses

---

## SYSTEM: Clarity - Double Diamond Planning IDE

### Core Components to Test:

1. **Discovery Components** (`components/discover/`)
   - ProgressiveDiscover, ExtractingPhase, PreviewPhase
   - BrutalTruths, Antithesis, StrawMan
   - ModeToggle, QualityScore, FieldCard

2. **Intent Engine** (`intent/`)
   - Loader (CUE/JSON parsing)
   - Parser (Spec parsing)
   - Validation (Spec, Semantic, Interpolation, Rules)
   - Plan (Mode, Next, EmitBeads, Resolver)
   - Quality (Analyzer, Effects, Improver, Linter)

3. **Storage** (`storage/`)
   - Project management
   - Transcript store
   - Answer extraction cache
   - Session history/diffing

4. **Lattice** (`lattice/`)
   - Coverage analysis
   - EARS parsing
   - Premortem analysis
   - Inversion detection

5. **Providers** (`providers/`)
   - OpenCode provider
   - Resolution engine

---

## USER JOURNEYS TO STRESS TEST

### JOURNEY 1: Discovery Flow
```
User enters prompt → AI extracts fields → User validates fields → 
Brutal truths check → Antithesis generation → Solution confirmation →
Preview summary → Locked phase
```

**Stress Test Cases:**
- Empty prompt
- Malformed AI response JSON
- Missing required fields in extraction
- AI returns conflicting field values
- User skips brutal truths validation
- Antithesis produces invalid scenario
- Solution confirmation timeout
- Race condition between AI extraction and field validation
- Concurrent prompt submissions

### JOURNEY 2: Interview Engine
```
Select profile → Start session → Answer questions → 
Gap detection → Conflict detection → Phase completion →
Execute plan → Bead emission
```

**Stress Test Cases:**
- Invalid profile selection
- Duplicate session IDs
- Circular gap resolution
- Conflict resolution deadlock
- Phase gating bypass
- Missing preconditions for bead execution
- Idempotency violation in bead emission
- Session state corruption on crash
- Answer with special characters (SQL injection, XSS)
- Extremely long answers (DoS)
- Invalid question IDs
- Answer order manipulation

### JOURNEY 3: Spec Validation Pipeline
```
Load CUE → Parse spec → Validate structure → 
Semantic validation → Interpolation check → Rule validation
```

**Stress Test Cases:**
- Invalid CUE syntax
- Circular dependencies in behaviors
- Invalid behavior references
- Missing required fields
- Duplicate behavior names
- Path traversal in spec names
- ReDoS in regex rules
- Infinite loop in dependency resolution
- Stack overflow from deep dependency chains
- Memory exhaustion from large specs

### JOURNEY 4: Quality Analysis
```
Analyze spec → Calculate coverage → Detect effects → 
Lint spec → Generate improvements
```

**Stress Test Cases:**
- Spec with zero features (crash)
- Spec with infinite features (DoS)
- Invalid quality weights
- Division by zero in score calculation
- Negative scores
- Effects detection false positives/negatives
- Linter rule conflicts
- Improvement suggestions that break spec

### JOURNEY 5: Storage Operations
```
Create project → Save session → Load session → 
List sessions → Delete session → Export
```

**Stress Test Cases:**
- Invalid project ID (path traversal)
- Concurrent writes to same project
- Corrupt JSONL file
- Disk full during save
- Permission denied
- Session not found
- Session version conflicts
- Database corruption
- Large session (>1GB)
- Unicode in session data
- Concurrent read/write

### JOURNEY 6: Lattice Operations
```
Parse EARS → Analyze coverage → Generate premortem → 
Detect inversions → Compact spec
```

**Stress Test Cases:**
- Invalid EARS syntax
- Coverage with zero behaviors
- Premortem with no threats
- Inversion detection false positives
- Compact destroys spec structure
- Very long EARS specs (DoS)
- Concurrent lattice operations

### JOURNEY 7: UI State Management
```
Render page → Update state → Navigate → 
Trigger server function → Update UI
```

**Stress Test Cases:**
- Concurrent state updates
- Invalid state transitions
- Lost updates on reconnect
- Server function timeouts
- Server function panics
- Invalid server function responses
- Memory leaks in signal handlers
- Navigation during pending operations
- Component unmount during async

### JOURNEY 8: Provider Integration
```
Send prompt → Receive response → Parse response → 
Handle errors → Retry logic
```

**Stress Test Cases:**
- Provider timeout
- Invalid JSON response
- Rate limiting
- Authentication failure
- Concurrent requests
- Request cancellation
- Large response (>10MB)
- Binary data in response
- Malformed markdown

### JOURNEY 9: Bead Generation & Execution
```
Generate beads → Resolve dependencies → 
Order by phase → Emit beads → Execute
```

**Stress Test Cases:**
- Circular bead dependencies
- Missing bead prerequisites
- Duplicate bead emission
- Bead execution timeout
- Bead execution failure
- Idempotency violation
- Invalid bead output format

### JOURNEY 10: Quality Scoring
```
Calculate coverage → Calculate clarity → 
Calculate testability → Calculate AI readiness → 
Aggregate scores
```

**Stress Test Cases:**
- Division by zero
- Score overflow (>100)
- Score underflow (<0)
- Negative weights
- NaN in calculations
- Infinite recursion
- Very large specs (DoS)

---

## ATTACK VECTORS TO USE

### Input-Based
- SQL injection in text fields
- XSS in descriptions
- Path traversal in names
- ReDoS in regex
- Buffer overflow in string handling
- Unicode bombs
- Null bytes
- Very long strings (>1MB)
- Binary data in text fields

### Concurrency-Based
- Race conditions in state updates
- Deadlocks in storage
- Lost updates in caching
- TOCTOU in file operations
- Concurrent session modifications

### Error Handling-Based
- Ignore all errors
- Catch and suppress
- Incomplete error messages
- Wrong error types
- Panic on error

### Edge Cases
- Empty everything
- Maximum values
- Minimum values  
- Zero divisions
- Negative numbers
- Float precision
- Timestamp edge cases (epoch, far future)

---

## OUTPUT FORMAT

For each journey, produce:

```
## Journey: [NAME]
### Stress Tests Run
- [Test 1]: [Input] → [Expected Crash] → [Severity]
- [Test 2]: ...

### Results
- Crashes Found: [list]
- Race Conditions: [list]
- Error Handling Issues: [list]
- Edge Cases: [list]

### Exploits
- [Exploit 1]: How to trigger, impact
- [Exploit 2]: ...
```

---

## EXECUTION

Run actual commands against the codebase to:
1. Verify each stress test case
2. Confirm crashes/pannics exist
3. Measure error handling quality
4. Identify security vulnerabilities

Test with real data, not just code inspection.
