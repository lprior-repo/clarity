#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

use crate::components::discover::ProgressiveDiscover;

/// HomePage - Main landing page with Discover phase
///
/// Renders the ProgressiveDiscover component which orchestrates the full
/// flow: Prompt -> Extracting -> ConfirmingFields -> Preview -> KirkCompilation -> Locked
#[component]
pub fn HomePage() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-background text-foreground flex flex-col",
            // Header
            header {
                class: "border-b border-border bg-card",
                div {
                    class: "container mx-auto px-4 py-4",
                    div {
                        class: "flex items-center justify-between",
                        h1 {
                            class: "text-2xl font-bold text-foreground",
                            "Clarity Planner"
                        }
                        span {
                            class: "text-sm text-muted-foreground",
                            "Double Diamond Planning IDE"
                        }
                    }
                }
            }

            // Main content
            main {
                class: "container mx-auto px-4 py-8 flex-1",
                ProgressiveDiscover {
                    extraction_provider: None,
                    initial_prompt: None,
                    on_complete: None,
                    on_refine: None,
                }
            }

            // Footer
            footer {
                class: "border-t border-border bg-card mt-auto",
                div {
                    class: "container mx-auto px-4 py-4",
                    p {
                        class: "text-sm text-muted-foreground text-center",
                        "Clarity Planner - Intent-Driven Development"
                    }
                }
            }
        }
    }
}
