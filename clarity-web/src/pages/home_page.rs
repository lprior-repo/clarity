#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use dioxus::prelude::*;

/// HomePage - Main landing page with Discover phase
///
/// TODO: Replace with ProgressiveDiscover main container component
/// See: docs/PROGRESSIVE_DISCOVER_PLAN.md
#[component]
pub fn HomePage() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-background text-foreground",
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
                class: "container mx-auto px-4 py-8",
                div {
                    class: "max-w-4xl mx-auto",
                    // Phase header
                    div {
                        class: "mb-8",
                        h2 {
                            class: "text-xl font-semibold text-foreground mb-2",
                            "Discover Phase"
                        }
                        p {
                            class: "text-muted-foreground",
                            "Define your problem, users, context, constraints, and goals."
                        }
                    }

                    // TODO: Replace with ProgressiveDiscover main container
                    // See bead: progressive-discover-main-container
                    div {
                        class: "p-8 border border-dashed border-border rounded-lg text-center",
                        div {
                            class: "text-muted-foreground mb-4",
                            "Progressive Discover component is being implemented."
                        }
                        div {
                            class: "text-sm text-muted-foreground",
                            "The old Express/Guided flow components have been removed."
                        }
                        p {
                            class: "text-xs text-muted-foreground mt-4",
                            "See: docs/PROGRESSIVE_DISCOVER_PLAN.md"
                        }
                    }
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
