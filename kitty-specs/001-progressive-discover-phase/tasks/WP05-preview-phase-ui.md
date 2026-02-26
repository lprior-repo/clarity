---
lane: "doing"
shell_pid: "581470"
agent: "claude"
---
# WP05: Preview Phase UI

---
work_package_id: "WP05"
title: "Preview Phase UI"
lane: "planned"
dependencies: ["WP04"]
beads: ["bd-3h2v", "bd-2k1q", "bd-3fz2"]
---

## Objective

Build the Preview phase component that displays a summary of all confirmed fields and the Four Brutal Truths checklist.

## Context

The Preview phase is the final review before locking in the plan. Users see all their confirmed data and must acknowledge the Four Brutal Truths.

**Key Files**:
- `clarity-web/src/components/discover/phases/preview_phase.rs` (new)
- `clarity-web/src/components/discover/preview_summary.rs` (may exist)

## Beads in This Package

| Bead ID | Title | File |
|---------|-------|------|
| bd-3h2v | summary display | preview_phase.rs |
| bd-2k1q | Four Brutal Truths checklist | preview_phase.rs |
| bd-3fz2 | PreviewPhase component | preview_phase.rs |

## Implementation Guidance

### bd-3h2v: Summary Display

**Purpose**: Display all confirmed fields in a readable summary.

**Requirements**:
- Show all 5 confirmed fields
- Allow editing (returns to confirm phase)
- Clean, readable layout

```rust
#[component]
pub fn TranscriptSummary(transcript: InterrogationTranscript) -> Element {
    rsx! {
        div { class: "space-y-4",
            SummaryField { label: "Problem", value: transcript.problem }
            SummaryField { label: "Persona", value: transcript.persona }
            SummaryField { label: "Solution", value: transcript.solution }
            SummaryField { label: "Nonpersona", value: transcript.nonpersona }
            SummaryField { label: "Scenario", value: transcript.scenario }
        }
    }
}
```

### bd-2k1q: Four Brutal Truths Checklist

**Purpose**: Display and require acknowledgment of Four Brutal Truths.

**The Four Brutal Truths**:
1. **Scale**: Is the problem big enough to matter?
2. **Back-loaded Value**: Is value delivered late in the user journey?
3. **VORP**: Is the solution Valuable, Obvious, Real, and Possible?
4. **Sustaining**: Will users keep coming back?

```rust
const BRUTAL_TRUTHS: &[(&str, &str)] = &[
    ("Scale", "Is the problem big enough to matter?"),
    ("Back-loaded Value", "Is value delivered early enough?"),
    ("VORP", "Is the solution Valuable, Obvious, Real, and Possible?"),
    ("Sustaining", "Will users keep coming back?"),
];

#[component]
pub fn BrutalTruthsChecklist(
    acknowledged: Signal<[bool; 4]>,
) -> Element {
    rsx! {
        div { class: "space-y-3",
            h3 { class: "text-lg font-semibold",
                "The Four Brutal Truths"
            }
            for (i, (title, question)) in BRUTAL_TRUTHS.iter().enumerate() {
                div {
                    key: "{i}",
                    class: "flex items-start gap-3 p-3 border rounded",
                    input {
                        r#type: "checkbox",
                        checked: acknowledged.read()[i],
                        onchange: move |_| {
                            let mut ack = acknowledged.write();
                            ack[i] = !ack[i];
                        },
                    }
                    div {
                        strong { "{title}" }
                        p { class: "text-sm text-muted-foreground",
                            "{question}"
                        }
                    }
                }
            }
        }
    }
}
```

### bd-3fz2: PreviewPhase Component

**Purpose**: Compose summary and brutal truths with navigation.

**Requirements**:
- Display summary
- Display brutal truths checklist
- "Refine" button returns to Prompt phase
- "Lock In" button proceeds to Kirk compilation
- Lock In only enabled when all truths acknowledged

```rust
#[component]
pub fn PreviewPhase(
    transcript: InterrogationTranscript,
    on_refine: EventHandler<()>,
    on_lock_in: EventHandler<()>,
) -> Element {
    let brutal_truths = use_signal(|| [false; 4]);
    let all_acknowledged = brutal_truths.read().iter().all(|&b| b);

    rsx! {
        div { class: "space-y-6 p-6",
            h2 { class: "text-xl font-bold",
                "Review Your Plan"
            }

            TranscriptSummary { transcript: transcript.clone() }

            BrutalTruthsChecklist { acknowledged: brutal_truths }

            div { class: "flex justify-between pt-4 border-t",
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: on_refine,
                    "Refine"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    disabled: !all_acknowledged,
                    onclick: on_lock_in,
                    "Lock In"
                }
            }
        }
    }
}
```

## Definition of Done

- [ ] All 3 beads complete
- [ ] Summary displays all fields
- [ ] Brutal truths checklist works
- [ ] Lock In button enables correctly

## Workflow

```bash
br claim bd-3h2v bd-2k1q bd-3fz2
# Implement components
br close bd-3h2v bd-2k1q bd-3fz2
```

## Activity Log

- 2026-02-26T16:30:19Z – claude – shell_pid=581470 – lane=doing – Assigned agent via workflow command
