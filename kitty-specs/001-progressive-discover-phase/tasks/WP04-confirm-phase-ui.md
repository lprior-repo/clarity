# WP04: Confirm Phase UI

---
work_package_id: "WP04"
title: "Confirm Phase UI"
lane: "planned"
dependencies: ["WP01", "WP02"]
beads: ["bd-1hkh", "bd-24dz", "bd-36lz", "bd-fskg", "bd-2smr", "bd-2jjb"]
---

## Objective

Build the Confirm phase components for reviewing and validating extracted fields with adversarial coaching.

## Context

The Confirm phase is where users review each extracted field (Problem, Persona, Solution, Nonpersona, Scenario) and provide adversarial validation. Each field has specific validation requirements.

**Key Files**:
- `clarity-web/src/components/discover/confirm/` (new directory)
- `clarity-web/src/components/discover/confirm/problem_confirm.rs`
- `clarity-web/src/components/discover/confirm/persona_confirm.rs`
- `clarity-web/src/components/discover/confirm/solution_confirm.rs`
- `clarity-web/src/components/discover/confirm/nonpersona_confirm.rs`
- `clarity-web/src/components/discover/confirm/scenario_confirm.rs`

## Beads in This Package

| Bead ID | Title | File |
|---------|-------|------|
| bd-1hkh | ProblemDisplay component | problem_confirm.rs |
| bd-24dz | AntithesisInput component | problem_confirm.rs |
| bd-36lz | AntithesisQuality indicator | problem_confirm.rs |
| bd-fskg | StrawManTrap checklist | persona_confirm.rs |
| bd-2smr | PersonaDisplay component | persona_confirm.rs |
| bd-2jjb | ProblemConfirm composition | problem_confirm.rs |

## Implementation Guidance

### bd-1hkh: ProblemDisplay Component

**Purpose**: Display extracted problem statement in editable textarea.

```rust
#[component]
pub fn ProblemDisplay(
    problem: String,
    on_change: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "space-y-2",
            label { class: "text-sm font-medium",
                "Problem Statement"
            }
            textarea {
                class: "w-full min-h-[100px] p-3 border rounded-md",
                value: problem,
                oninput: move |e| on_change.call(e.value()),
            }
        }
    }
}
```

### bd-24dz: AntithesisInput Component

**Purpose**: Input for 3 null hypothesis points.

**Requirements**:
- 3 text inputs for antithesis points
- Each point validates specificity
- Call validate_antithesis server function

```rust
#[component]
pub fn AntithesisInput(
    points: Signal<[String; 3]>,
    on_validate: EventHandler<AntithesisValidation>,
) -> Element {
    rsx! {
        div { class: "space-y-4",
            h3 { "Provide 3 null hypothesis points" }
            for i in 0..3 {
                textarea {
                    key: "{i}",
                    class: "w-full p-2 border rounded",
                    placeholder: "Point {i + 1}...",
                    value: points.read()[i].clone(),
                    oninput: move |e| {
                        let mut pts = points.write();
                        pts[i] = e.value();
                    },
                }
            }
        }
    }
}
```

### bd-36lz: AntithesisQuality Indicator

**Purpose**: Show quality score (0.0-1.0) for antithesis.

**Requirements**:
- Visual quality bar
- Score display
- Block progression if score < 0.5

```rust
#[component]
pub fn AntithesisQuality(score: f64) -> Element {
    let color = if score >= 0.7 { "bg-green-500" }
                else if score >= 0.5 { "bg-yellow-500" }
                else { "bg-red-500" };

    rsx! {
        div { class: "space-y-2",
            div { class: "flex justify-between text-sm",
                span { "Quality Score" }
                span { "{(score * 100.0) as u8}%" }
            }
            div { class: "h-2 bg-muted rounded-full overflow-hidden",
                div {
                    class: "h-full {color} transition-all",
                    style: "width: {(score * 100.0)}%",
                }
            }
            if score < 0.5 {
                p { class: "text-sm text-destructive",
                    "Quality too low. Please add more specific points."
                }
            }
        }
    }
}
```

### bd-fskg: StrawManTrap Checklist

**Purpose**: Display detected straw man traps with acknowledgment.

**Trap Types**:
- IrrationalActor: User acts against self-interest
- ManicPixieDreamUser: Idealized user with no friction
- StoicMonk: Unrealistic self-discipline
- YourClone: User thinks exactly like designer

```rust
#[component]
pub fn StrawManTrapChecklist(
    traps: Vec<StrawManTrap>,
    acknowledged: Signal<Vec<StrawManTrap>>,
) -> Element {
    rsx! {
        div { class: "space-y-3",
            h3 { "Persona Validation" }
            for trap in traps {
                div {
                    key: "{trap:?}",
                    class: "flex items-start gap-2 p-3 border rounded",
                    input {
                        r#type: "checkbox",
                        checked: acknowledged.read().contains(&trap),
                        onchange: move |_| {
                            // Toggle acknowledgment
                        },
                    }
                    div {
                        strong { "{trap.title()}" }
                        p { class: "text-sm text-muted-foreground",
                            "{trap.description()}"
                        }
                    }
                }
            }
        }
    }
}
```

### bd-2smr: PersonaDisplay Component

**Purpose**: Display extracted persona with straw man trap validation.

### bd-2jjb: ProblemConfirm Composition

**Purpose**: Compose all problem confirmation components.

## Definition of Done

- [ ] All 6 beads complete
- [ ] Antithesis validation works
- [ ] Quality indicator works
- [ ] Straw man trap detection works
- [ ] Progress blocking on low quality

## Workflow

```bash
br claim bd-1hkh bd-24dz bd-36lz bd-fskg bd-2smr bd-2jjb
# Implement components
br close bd-1hkh bd-24dz bd-36lz bd-fskg bd-2smr bd-2jjb
```
