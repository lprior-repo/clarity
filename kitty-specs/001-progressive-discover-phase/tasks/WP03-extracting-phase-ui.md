---
lane: "doing"
shell_pid: "576257"
agent: "claude"
---
# WP03: Extracting Phase UI

---
work_package_id: "WP03"
title: "Extracting Phase UI"
lane: "planned"
dependencies: []
beads: ["bd-23qy", "bd-xz68"]
---

## Objective

Build the Extracting phase component that shows progress while AI extracts fields from user input.

## Context

The Extracting phase displays an animated progress bar and status messages while the AI extracts the 5 fields (Problem, Persona, Solution, Nonpersona, Scenario) from the user's prompt.

**Key Files**:
- `clarity-web/src/components/discover/progressive_discover.rs` (existing)
- `clarity-web/src/components/discover/phases/extracting_phase.rs` (new)

## Beads in This Package

| Bead ID | Title | File |
|---------|-------|------|
| bd-23qy | progress animation | extracting_phase.rs |
| bd-xz68 | ExtractingPhase component | extracting_phase.rs |

## Implementation Guidance

### bd-23qy: Progress Animation

**Purpose**: Create animated progress bar with status messages.

**Requirements**:
- Animated progress bar (0-100%)
- Status messages that update as extraction progresses
- Smooth CSS transitions

**Implementation**:
```rust
const EXTRACTION_STEPS: &[&str] = &[
    "Parsing problem statement...",
    "Identifying target users...",
    "Extracting solution details...",
    "Analyzing scenario context...",
    "Validating extraction quality...",
];

#[component]
pub fn ExtractionProgress(progress: u8) -> Element {
    let step_index = (progress as usize / 20).min(EXTRACTION_STEPS.len() - 1);

    rsx! {
        div { class: "space-y-4",
            // Progress bar
            div { class: "h-2 bg-muted rounded-full overflow-hidden",
                div {
                    class: "h-full bg-primary transition-all duration-500",
                    style: "width: {progress}%",
                }
            }

            // Status message
            p { class: "text-sm text-muted-foreground text-center",
                "{EXTRACTION_STEPS[step_index]}"
            }
        }
    }
}
```

### bd-xz68: ExtractingPhase Component

**Purpose**: Main component that orchestrates extraction and auto-transitions.

**Requirements**:
- Display progress animation
- Trigger extraction on mount
- Auto-transition to ConfirmingFields on completion
- Handle extraction errors

**Implementation**:
```rust
#[component]
pub fn ExtractingPhase(
    state: Signal<ProgressiveDiscoverState>,
    actions: ProgressiveDiscoverActions,
) -> Element {
    let progress = use_signal(|| 0u8);

    // Simulate extraction (replace with actual server call)
    use_effect(move || {
        // In real implementation, call extract_fields server function
        // and update progress based on response
    });

    rsx! {
        div { class: "space-y-6 p-6",
            h2 { class: "text-lg font-semibold",
                "Extracting Your Ideas..."
            }

            ExtractionProgress { progress: *progress.read() }

            // Auto-transition when complete
            if *progress.read() >= 100 {
                {actions.advance_phase();}
            }
        }
    }
}
```

## Test Strategy

Manual testing:
1. Verify progress bar animates smoothly
2. Verify status messages update correctly
3. Verify auto-transition to Confirm phase

## Definition of Done

- [ ] Both beads complete
- [ ] Progress animation works
- [ ] Auto-transition works
- [ ] No panics or unwraps

## Workflow

```bash
br claim bd-23qy bd-xz68
# Implement components
br close bd-23qy bd-xz68
```

## Activity Log

- 2026-02-26T16:29:07Z – claude – shell_pid=576257 – lane=doing – Assigned agent via workflow command
