---
lane: "doing"
shell_pid: "749044"
---
# WP06: Kirk Compilation + Locked Phase

---
work_package_id: "WP06"
title: "Kirk Compilation + Locked Phase"
lane: "planned"
dependencies: ["WP01", "WP05"]
beads: ["bd-1jie", "bd-3cpp", "bd-1le0", "bd-3nia"]
---

## Objective

Build the Kirk Compilation phase (shows 16-section compilation progress) and Locked phase (completion summary).

## Context

After the user locks in their plan, the system compiles it into a 16-section KIRK contract. Once complete, the Locked phase shows the completion summary and navigation options.

**Key Files**:
- `clarity-web/src/components/discover/phases/kirk_compilation_phase.rs` (new)
- `clarity-web/src/components/discover/phases/locked_phase.rs` (new)

## Beads in This Package

| Bead ID | Title | File |
|---------|-------|------|
| bd-1jie | compilation progress | kirk_compilation_phase.rs |
| bd-3cpp | KirkCompilationPhase component | kirk_compilation_phase.rs |
| bd-1le0 | completion summary | locked_phase.rs |
| bd-3nia | LockedPhase component | locked_phase.rs |

## Implementation Guidance

### bd-1jie: Compilation Progress

**Purpose**: Display 16-section compilation progress.

**Requirements**:
- Show all 16 KIRK sections
- Animate completion of each section
- Show which section is currently being compiled

```rust
const KIRK_SECTIONS: &[&str] = &[
    "Problem Definition",
    "Target Users",
    "User Personas",
    "Non-Users",
    "Solution Overview",
    "Key Features",
    "User Scenarios",
    "Success Criteria",
    "Technical Constraints",
    "Dependencies",
    "Risks",
    "Mitigations",
    "Timeline",
    "Resources",
    "Milestones",
    "Deliverables",
];

#[component]
pub fn KirkProgress(current_section: usize) -> Element {
    rsx! {
        div { class: "space-y-2",
            for (i, section) in KIRK_SECTIONS.iter().enumerate() {
                div {
                    key: "{i}",
                    class: format!(
                        "flex items-center gap-2 p-2 rounded {}",
                        if i < current_section { "bg-green-100 text-green-800" }
                        else if i == current_section { "bg-primary/10 text-primary animate-pulse" }
                        else { "bg-muted text-muted-foreground" }
                    ),
                    if i < current_section {
                        "✓"
                    } else if i == current_section {
                        "⏳"
                    } else {
                        "○"
                    }
                    span { "{section}" }
                }
            }
        }
    }
}
```

### bd-3cpp: KirkCompilationPhase Component

**Purpose**: Orchestrate compilation and auto-transition.

```rust
#[component]
pub fn KirkCompilationPhase(
    transcript: InterrogationTranscript,
    on_complete: EventHandler<KirkContract>,
) -> Element {
    let current_section = use_signal(|| 0usize);

    // Simulate compilation (replace with actual server call)
    use_effect(move || {
        // Call compile_to_kirk server function
        // Update current_section based on progress
    });

    rsx! {
        div { class: "space-y-6 p-6",
            h2 { class: "text-xl font-bold",
                "Compiling Your Plan..."
            }

            KirkProgress { current_section: *current_section.read() }

            // Auto-transition when complete
            if *current_section.read() >= KIRK_SECTIONS.len() {
                {on_complete.call(KirkContract::default());}
            }
        }
    }
}
```

### bd-1le0: Completion Summary

**Purpose**: Display completion status and generated bead count.

```rust
#[component]
pub fn CompletionSummary(contract: KirkContract) -> Element {
    rsx! {
        div { class: "space-y-4 text-center",
            div { class: "text-6xl",
                "🎉"
            }
            h2 { class: "text-2xl font-bold",
                "Plan Locked!"
            }
            p { class: "text-muted-foreground",
                "Your plan has been compiled into a {contract.sections.len()}-section KIRK contract."
            }
            div { class: "p-4 bg-muted rounded-lg",
                p { class: "text-2xl font-bold text-primary",
                    "{contract.beads.len()}"
                }
                p { class: "text-sm text-muted-foreground",
                    "Implementation beads generated"
                }
            }
        }
    }
}
```

### bd-3nia: LockedPhase Component

**Purpose**: Show completion and provide navigation options.

```rust
#[component]
pub fn LockedPhase(
    contract: KirkContract,
    on_view_plan: EventHandler<()>,
    on_view_graph: EventHandler<()>,
    on_view_state: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "space-y-6 p-6",
            CompletionSummary { contract: contract.clone() }

            div { class: "flex gap-4 justify-center",
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: on_view_plan,
                    "View Plan"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: on_view_graph,
                    "View Graph"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: on_view_state,
                    "View State"
                }
            }
        }
    }
}
```

## Definition of Done

- [ ] All 4 beads complete
- [ ] Compilation progress animates
- [ ] Auto-transition to Locked works
- [ ] Locked phase displays correctly
- [ ] Navigation buttons work

## Workflow

```bash
br claim bd-1jie bd-3cpp bd-1le0 bd-3nia
# Implement components
br close bd-1jie bd-3cpp bd-1le0 bd-3nia
```
