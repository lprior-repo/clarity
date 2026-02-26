# Progressive Discover - Atomic Task Definitions

## Overview

This directory contains atomic task definitions for the Progressive Discover Phase implementation. Each task is defined as a JSON file following the planner's task format, with complete acceptance criteria, dependencies, and implementation details.

## Task Files (11 core tasks + 3 supporting tasks)

### Foundation Tasks

| File | ID | Title | Hours | Complexity |
|------|----|-------|-------|------------|
| `progressive-discover-state-machine.json` | `progressive-discover-state-machine` | Create ProgressiveDiscover state machine types | 4 | Medium |
| `storage-integration.json` | `storage-integration` | Add ProgressiveDiscover state persistence to storage layer | 6 | Medium |

### UI Component Tasks

| File | ID | Title | Hours | Complexity |
|------|----|-------|-------|------------|
| `prompt-phase-component.json` | `prompt-phase-component` | Create PromptPhase component | 6 | Low |
| `extracting-phase-component.json` | `extracting-phase-component` | Create ExtractingPhase component | 4 | Low |
| `confirm-phase-component.json` | `confirm-phase-component` | Create ConfirmPhase component | 16 | High |
| `preview-phase-component.json` | `preview-phase-component` | Create PreviewPhase component | 8 | Medium |
| `kirk-compilation-phase-component.json` | `kirk-compilation-phase-component` | Create KirkCompilationPhase component | 10 | Medium |
| `locked-phase-component.json` | `locked-phase-component` | Create LockedPhase component | 6 | Low |
| `progressive-discover-main-container.json` | `progressive-discover-main-container` | Create ProgressiveDiscover main container | 10 | Medium |

### Server Function Tasks

| File | ID | Title | Hours | Complexity |
|------|----|-------|-------|------------|
| `validation-server-functions.json` | `validation-server-functions` | Create adversarial validation server functions | 8 | Medium |
| `compile-to-kirk-server-function.json` | `compile-to-kirk-server-function` | Create compile_to_kirk server function | 12 | High |

### Cleanup & Testing Tasks

| File | ID | Title | Hours | Complexity |
|------|----|-------|-------|------------|
| `delete-old-express-guided-components.json` | `delete-old-express-guided-components` | Delete old Express/Guided components | 2 | Low |
| `e2e-tests.json` | `e2e-tests` | Create end-to-end tests for Progressive Discover flow | 16 | Medium |

## Task JSON Schema

Each task file contains:

- **id**: Unique task identifier
- **title**: Human-readable task title
- **phase**: Which phase this task belongs to (discover)
- **priority**: P0 (critical), P1 (important), P2 (nice-to-have)
- **status**: pending, in_progress, completed
- **description**: Detailed description of what needs to be done
- **acceptance_criteria**: Checklist of requirements for task completion
- **dependencies**: List of task IDs that must be completed first
- **blocks**: List of task IDs that are blocked by this task
- **files**: Files to create/modify with descriptions
- **implementation_details**: Specific implementation guidance
- **testing_requirements**: Testing requirements for the task
- **references**: Links to relevant documentation
- **estimated_hours**: Time estimate for completion
- **complexity**: low, medium, or high
- **tags**: Searchable tags for categorization

## Dependency Graph

```
progressive-discover-state-machine (Foundation)
├── prompt-phase-component
│   └── extracting-phase-component
│       └── confirm-phase-component
│           └── preview-phase-component
│
validation-server-functions
└── confirm-phase-component
    └── preview-phase-component
        └── kirk-compilation-phase-component
            └── locked-phase-component
                └── progressive-discover-main-container
                    └── delete-old-express-guided-components

compile-to-kirk-server-function
└── kirk-compilation-phase-component

storage-integration (parallel, no blocking)

e2e-tests (can start after main-container and key phases complete)
```

## Recommended Implementation Order

### Week 1: Foundation & Core Phases
1. `progressive-discover-state-machine` - Foundation types (4h)
2. `prompt-phase-component` - User input (6h)
3. `extracting-phase-component` - Extraction loading (4h)
4. `validation-server-functions` - Adversarial validation (8h)
5. `confirm-phase-component` - Core adversarial coaching (16h)

### Week 2: Completion & Integration
6. `preview-phase-component` - Summary & brutal truths (8h)
7. `compile-to-kirk-server-function` - KIRK compilation (12h)
8. `kirk-compilation-phase-component` - Compilation UI (10h)
9. `locked-phase-component` - Completion & navigation (6h)
10. `progressive-discover-main-container` - Orchestration (10h)
11. `storage-integration` - Persistence (6h)

### Week 3: Cleanup & Testing
12. `delete-old-express-guided-components` - Remove old code (2h)
13. `e2e-tests` - Comprehensive testing (16h)

## Usage

### Reading Task Definitions

```bash
# View a specific task
cat docs/planner/tasks/progressive-discover-state-machine.json

# View all tasks
ls -la docs/planner/tasks/*.json

# View task index
cat docs/planner/tasks/INDEX.json
```

### Integration with Planner

These task definitions are structured to integrate with a planner system. To add them to a planner session:

```bash
# Load all tasks into planner
for task in docs/planner/tasks/*.json; do
    planner add-task "$task"
done
```

### Tracking Progress

Update task status in JSON files as you work:

```json
{
  "status": "in_progress",  // or "completed"
  "completed_at": "2026-02-25T10:30:00Z"
}
```

## Task Index

See `INDEX.json` for:
- Complete task dependency graph
- Critical path analysis
- Complexity breakdown
- Risk factors
- Success metrics
- Total estimated hours (108)

## Reference Documents

- `/docs/VISION-ProgressiveDiscover.md` - Complete vision and architecture
- `/docs/architecture/discover-phase.md` - Current discover phase architecture
- `/docs/architecture/mental-lattice.md` - Mental lattice analysis framework
- `/docs/architecture/storage-layer.md` - Storage and persistence layer

## Tags for Filtering

- `foundation` - Foundational types and infrastructure
- `ui` - User interface components
- `server` - Server-side functions
- `state-machine` - State machine and flow control
- `adversarial-coaching` - AI adversarial validation patterns
- `kirk` - KIRK contract generation
- `storage` - Persistence and recovery
- `testing` - Quality assurance and testing
- `cleanup` - Code removal and migration

## Statistics

- **Total Tasks**: 11 core + 3 supporting = 14 tasks
- **Total Estimated Hours**: 108 hours
- **High Complexity**: 2 tasks
- **Medium Complexity**: 6 tasks
- **Low Complexity**: 6 tasks
- **Critical Path Tasks**: 10 tasks

## Contributing

When adding new tasks or updating existing ones:

1. Follow the task JSON schema defined in `INDEX.json`
2. Include all required fields
3. Add dependencies to `INDEX.json` if applicable
4. Update estimated hours based on actual implementation time
5. Add relevant tags for discoverability
6. Update this README if task structure changes

---

*Task definitions created: 2026-02-25*
*Progressive Discover Phase - Clarity Project*
