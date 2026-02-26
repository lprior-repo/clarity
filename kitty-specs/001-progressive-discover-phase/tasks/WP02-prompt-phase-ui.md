---
lane: "for_review"
---
# WP02: Prompt Phase UI

---
work_package_id: "WP02"
title: "Prompt Phase UI"
lane: "planned"
dependencies: ["WP01"]
subtasks: ["T012", "T013", "T014", "T015", "T016", "T017"]
---

## Objective

Build the Prompt phase component - the entry point for the Progressive Discover wizard where users enter their initial problem description.

## Context

The Prompt phase is the first screen users see. It provides scaffolding prompts to help users get started, a large textarea for their input, and a button to trigger AI extraction.

**Key Files**:
- `clarity-web/src/components/discover/phases/prompt_phase.rs` (new)
- `clarity-web/src/components/discover/progressive_discover.rs`

## Implementation Guidance

### T012: Create Scaffolding Prompt Buttons

**Purpose**: Provide 3 example prompts to help users get started.

**Location**: `clarity-web/src/components/discover/phases/prompt_phase.rs`

**Implementation**:
```rust
const SCAFFOLDING_PROMPTS: &[(&str, &str)] = &[
    ("Problem-first", "I'm building [product] because [users] struggle with [problem]. The main pain point is..."),
    ("User-first", "My target users are [description]. They currently [behavior], but they want [outcome]."),
    ("Solution-first", "I want to create [solution] that helps [users] achieve [outcome]. The key insight is..."),
];

#[component]
pub fn ScaffoldingButton(prompt_template: String, label: String, on_click: EventHandler<String>) -> Element {
    rsx! {
        button {
            class: "px-4 py-2 text-sm bg-muted hover:bg-accent rounded-md transition-colors",
            onclick: move |_| on_click.call(prompt_template.clone()),
            "{label}"
        }
    }
}
```

### T013: Create Main Textarea

**Purpose**: Large input area for user's problem description (max 2000 chars).

**Requirements**:
- 2000 character limit
- Placeholder text guiding users
- Minimum 50 characters for extraction
- Auto-resize based on content

### T014: Add Live Character Count

**Purpose**: Show character count and progress toward minimum.

**Implementation**:
```rust
#[component]
pub fn CharacterCount(current: usize, minimum: usize, maximum: usize) -> Element {
    let progress = (current as f64 / minimum as f64).min(1.0);
    let color = if current < minimum { "text-muted-foreground" } else { "text-green-600" };

    rsx! {
        div { class: "flex justify-between text-xs {color}",
            span { "{current}/{maximum} characters" }
            span { "{if current < minimum { format!(\"Need {} more\", minimum - current) } else { \"Ready!\" }}" }
        }
    }
}
```

### T015: Create ExtractFieldsButton

**Purpose**: Button to trigger AI extraction, disabled until minimum chars.

**Requirements**:
- Disabled until 50 characters entered
- Shows "Extracting..." state during extraction
- Calls server function on click

### T016: Compose PromptPhase Component

**Purpose**: Main component combining all prompt phase elements.

**Implementation**:
```rust
#[component]
pub fn PromptPhase(
    transcript: Signal<InterrogationTranscript>,
    on_extract: EventHandler<()>,
) -> Element {
    // State for textarea content
    // Character count derived from content
    // Button enabled/disabled based on count

    rsx! {
        div { class: "space-y-6",
            // Header
            h2 { "Describe your idea" }
            p { "Tell me about the problem you're solving..." }

            // Scaffolding buttons
            div { class: "flex gap-2",
                for (label, template) in SCAFFOLDING_PROMPTS {
                    ScaffoldingButton { label, prompt_template: template, on_click }
                }
            }

            // Main textarea
            Textarea { ... }

            // Character count
            CharacterCount { current, minimum: 50, maximum: 2000 }

            // Extract button
            ExtractFieldsButton { enabled: current >= 50, on_click: on_extract }
        }
    }
}
```

### T017: Wire Up Extraction Trigger

**Purpose**: Connect button to server function for field extraction.

**Implementation**:
- Call `extract_fields` server function
- Pass transcript content
- Handle success/failure
- Transition to Extracting phase on success

## Test Strategy

Manual testing:
1. Verify scaffolding buttons populate textarea
2. Verify character count updates in real-time
3. Verify button is disabled under 50 chars
4. Verify extraction triggers on click

## Definition of Done

- [ ] All 6 subtasks complete
- [ ] Component renders correctly
- [ ] Character count works
- [ ] Button state toggles correctly
- [ ] Extraction triggers

## Risks

| Risk | Mitigation |
|------|------------|
| Extraction slow | Show loading state |
| Network failure | Show error message, allow retry |

## Implementation Command

```bash
spec-kitty implement WP02
```

## Activity Log

- 2026-02-26T17:50:26Z – unknown – lane=for_review – Already implemented - All components found: ScaffoldingPromptButton, PromptTextarea with char limit, CharacterCount, ExtractFieldsButton, and complete PromptPhase component
