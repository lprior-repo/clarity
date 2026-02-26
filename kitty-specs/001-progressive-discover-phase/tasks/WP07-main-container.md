---
lane: "doing"
shell_pid: "788107"
agent: "claude"
---
# WP07: Main Container

---
work_package_id: "WP07"
title: "Main Container"
lane: "planned"
dependencies: ["WP02", "WP03", "WP04", "WP05", "WP06"]
beads: ["bd-1vgj", "bd-3ja1"]
---

## Objective

Build the main ProgressiveDiscover container component that orchestrates all phases and manages state transitions.

## Context

The main container is the root component for the Progressive Discover wizard. It holds the state machine, renders the appropriate phase component, and handles navigation between phases.

**Key Files**:
- `clarity-web/src/components/discover/progressive_discover.rs` (existing, enhance)
- `clarity-web/src/hooks/progressive_discover.rs` (existing, enhance)

## Beads in This Package

| Bead ID | Title | File |
|---------|-------|------|
| bd-1vgj | state machine hook | progressive_discover.rs |
| bd-3ja1 | ProgressiveDiscover component | progressive_discover.rs |

## Implementation Guidance

### bd-1vgj: State Machine Hook

**Purpose**: Enhance the existing hook with full state machine logic.

**Requirements**:
- Use existing ProgressiveDiscoverState and ProgressiveDiscoverActions
- Add phase-specific state (e.g., antithesis points, straw man traps)
- Add navigation methods
- Add validation state

The hook already exists in `clarity-web/src/hooks/progressive_discover.rs`. Enhance it with:

```rust
// Add to ProgressiveDiscoverState
pub struct ProgressiveDiscoverState {
    // ... existing fields ...
    pub antithesis_points: [String; 3],
    pub antithesis_score: Option<f64>,
    pub detected_traps: Vec<StrawManTrap>,
    pub acknowledged_traps: Vec<StrawManTrap>,
    pub vorp_validation: Option<VorpValidation>,
    pub hole_punching: HolePunchingResults,
    pub brutal_truths: [bool; 4],
}

// Add methods to ProgressiveDiscoverActions
impl ProgressiveDiscoverActions {
    pub fn set_antithesis(&mut self, points: [String; 3], score: f64) { ... }
    pub fn acknowledge_trap(&mut self, trap: StrawManTrap) { ... }
    pub fn set_brutal_truth(&mut self, index: usize, value: bool) { ... }
    pub fn can_advance_from_subphase(&self) -> bool { ... }
}
```

### bd-3ja1: ProgressiveDiscover Component

**Purpose**: Main component that renders the appropriate phase.

**Requirements**:
- Render current phase component based on state
- Pass state and actions to each phase
- Handle phase transitions
- Integrate with crash recovery

```rust
#[component]
pub fn ProgressiveDiscover() -> Element {
    // Check for crash recovery
    let recovery = use_signal(|| load_session_from_local_storage());
    let state = use_signal(|| {
        match recovery.read().clone() {
            CrashRecoveryResult::SessionLoaded { transcript, .. } => {
                ProgressiveDiscoverState::from_transcript(transcript)
            }
            _ => ProgressiveDiscoverState::default(),
        }
    });
    let actions = use_progressive_discover_actions(state);

    // Auto-save on state change
    use_effect({
        let state = state.clone();
        move || {
            let session_id = generate_session_id();
            save_transcript_to_local_storage(&session_id, &state.read().transcript);
        }
    });

    rsx! {
        div { class: "max-w-2xl mx-auto",
            match state.read().phase {
                ProgressiveDiscoverPhase::Prompt => rsx! {
                    PromptPhase {
                        state: state,
                        actions: actions,
                    }
                },
                ProgressiveDiscoverPhase::Extracting => rsx! {
                    ExtractingPhase {
                        state: state,
                        actions: actions,
                    }
                },
                ProgressiveDiscoverPhase::ConfirmingFields => rsx! {
                    ConfirmPhase {
                        state: state,
                        actions: actions,
                    }
                },
                ProgressiveDiscoverPhase::Preview => rsx! {
                    PreviewPhase {
                        state: state,
                        actions: actions,
                    }
                },
                ProgressiveDiscoverPhase::KirkCompilation => rsx! {
                    KirkCompilationPhase {
                        state: state,
                        actions: actions,
                    }
                },
                ProgressiveDiscoverPhase::Locked => rsx! {
                    LockedPhase {
                        state: state,
                        actions: actions,
                    }
                },
            }
        }
    }
}
```

## Definition of Done

- [ ] Both beads complete
- [ ] State machine handles all transitions
- [ ] All phases render correctly
- [ ] Crash recovery works
- [ ] Auto-save works

## Workflow

```bash
br claim bd-1vgj bd-3ja1
# Implement components
br close bd-1vgj bd-3ja1
```

## Activity Log

- 2026-02-26T16:48:13Z – claude – shell_pid=788107 – lane=doing – Assigned agent via workflow command
