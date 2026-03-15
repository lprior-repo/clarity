// Examples demonstrating the ported shadcn UI components
// Run with: cargo run --example ui_components_demo

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use chrono::NaiveDate;
use clarity_web::ui::{
    Accordion, AccordionItem, AccordionTrigger, AccordionContent,
    AlertDialog, AlertDialogTrigger, AlertDialogContent, AlertDialogHeader,
    AlertDialogFooter, AlertDialogTitle, AlertDialogDescription,
    AlertDialogAction, AlertDialogCancel,
    AspectRatio,
    Avatar, AvatarImage, AvatarFallback,
    Calendar,
    Button,
    Card, CardHeader, CardTitle, CardContent,
};
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
struct AppState {
    show_dialog: bool,
    selected_date: Option<NaiveDate>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            show_dialog: false,
            selected_date: None,
        }
    }
}

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    let mut state = use_signal(AppState::default);

    rsx! {
        style { {include_str!("../assets/tailwind.css")} }
        div {
            class: "min-h-screen bg-background p-8 space-y-8",

            // Title
            h1 {
                class: "text-3xl font-bold mb-8",
                "Dioxus UI Components Demo"
            }

            // Accordion Demo
            AccordionSection {}

            // AlertDialog Demo
            AlertDialogSection { state: state.clone() }

            // AspectRatio Demo
            AspectRatioSection {}

            // Avatar Demo
            AvatarSection {}

            // Calendar Demo
            CalendarSection { state: state.clone() }
        }
    }
}

// Accordion Section
#[component]
fn AccordionSection() -> Element {
    rsx! {
        div {
            class: "space-y-4",
            h2 {
                class: "text-2xl font-semibold mb-4",
                "Accordion"
            }

            Accordion {
                class: "w-full",
                AccordionItem {
                    value: "item-1".into(),
                    AccordionTrigger {
                        value: "item-1".into(),
                        "Is it accessible?"
                    }
                    AccordionContent {
                        value: "item-1".into(),
                        "Yes. It adheres to the WAI-ARIA design pattern."
                    }
                }

                AccordionItem {
                    value: "item-2".into(),
                    AccordionTrigger {
                        value: "item-2".into(),
                        "Is it styled?"
                    }
                    AccordionContent {
                        value: "item-2".into(),
                        "Yes. It comes with default styles that match the other components."
                    }
                }

                AccordionItem {
                    value: "item-3".into(),
                    AccordionTrigger {
                        value: "item-3".into(),
                        "Is it animated?"
                    }
                    AccordionContent {
                        value: "item-3".into(),
                        "Yes. It's animated by default, but you can disable it if you prefer."
                    }
                }
            }
        }
    }
}

// AlertDialog Section
#[component]
fn AlertDialogSection(state: Signal<AppState>) -> Element {
    let show_dialog = state.read().show_dialog;

    rsx! {
        div {
            class: "space-y-4",
            h2 {
                class: "text-2xl font-semibold mb-4",
                "Alert Dialog"
            }

            p {
                class: "text-sm text-muted-foreground mb-4",
                "A modal dialog that interrupts the user with important content and expects a response."
            }

            AlertDialog {
                open: show_dialog,
                AlertDialogTrigger {
                    class: "",
                    Button {
                        onclick: move |_| {
                            let mut s = state.write();
                            s.show_dialog = true;
                        },
                        "Show Dialog"
                    }
                }

                if show_dialog {
                    AlertDialogContent {
                        class: "",
                        AlertDialogHeader {
                            class: "",
                            AlertDialogTitle {
                                class: "",
                                "Are you sure?"
                            }
                            AlertDialogDescription {
                                class: "",
                                "This action cannot be undone. This will permanently delete your account and remove your data from our servers."
                            }
                        }
                        AlertDialogFooter {
                            class: "",
                            AlertDialogCancel {
                                class: "",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.show_dialog = false;
                                },
                                "Cancel"
                            }
                            AlertDialogAction {
                                class: "",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.show_dialog = false;
                                },
                                "Continue"
                            }
                        }
                    }
                }
            }
        }
    }
}

// AspectRatio Section
#[component]
fn AspectRatioSection() -> Element {
    rsx! {
        div {
            class: "space-y-4",
            h2 {
                class: "text-2xl font-semibold mb-4",
                "Aspect Ratio"
            }

            p {
                class: "text-sm text-muted-foreground mb-4",
                "Displays content within a container that maintains a specific aspect ratio."
            }

            div {
                class: "grid w-full max-w-sm gap-4",
                // 16:9 (widescreen)
                div {
                    class: "space-y-2",
                    div {
                        class: "text-sm font-medium",
                        "16:9 (Widescreen)"
                    }
                    AspectRatio {
                        ratio: 16.0 / 9.0,
                        class: "bg-muted rounded-md",
                        div {
                            class: "flex h-full w-full items-center justify-center text-sm",
                            "Video Content"
                        }
                    }
                }

                // 4:3 (standard)
                div {
                    class: "space-y-2",
                    div {
                        class: "text-sm font-medium",
                        "4:3 (Standard)"
                    }
                    AspectRatio {
                        ratio: 4.0 / 3.0,
                        class: "bg-muted rounded-md",
                        div {
                            class: "flex h-full w-full items-center justify-center text-sm",
                            "Image Content"
                        }
                    }
                }

                // 1:1 (square)
                div {
                    class: "space-y-2",
                    div {
                        class: "text-sm font-medium",
                        "1:1 (Square)"
                    }
                    AspectRatio {
                        ratio: 1.0,
                        class: "bg-muted rounded-md",
                        div {
                            class: "flex h-full w-full items-center justify-center text-sm",
                            "Square Content"
                        }
                    }
                }
            }
        }
    }
}

// Avatar Section
#[component]
fn AvatarSection() -> Element {
    rsx! {
        div {
            class: "space-y-4",
            h2 {
                class: "text-2xl font-semibold mb-4",
                "Avatar"
            }

            p {
                class: "text-sm text-muted-foreground mb-4",
                "Circular image component for displaying user profiles and identities."
            }

            div {
                class: "flex items-center space-x-4",
                // With image
                Avatar {
                    class: "",
                    alt: "User Avatar".into(),
                    AvatarImage {
                        src: "https://github.com/shadcn.png".into(),
                        alt: "@shadcn".into(),
                    }
                    AvatarFallback {
                        class: "",
                        delay_ms: 0,
                        "CN"
                    }
                }

                // Fallback only
                Avatar {
                    class: "",
                    alt: "JD".into(),
                    AvatarFallback {
                        class: "",
                        delay_ms: 0,
                        "JD"
                    }
                }

                // Larger size
                Avatar {
                    class: "h-16 w-16",
                    alt: "Large Avatar".into(),
                    AvatarFallback {
                        class: "",
                        delay_ms: 0,
                        "AB"
                    }
                }
            }
        }
    }
}

// Calendar Section
#[component]
fn CalendarSection(state: Signal<AppState>) -> Element {
    let selected_date = state.read().selected_date;

    rsx! {
        div {
            class: "space-y-4",
            h2 {
                class: "text-2xl font-semibold mb-4",
                "Calendar"
            }

            p {
                class: "text-sm text-muted-foreground mb-4",
                "Date picker with month navigation and day selection."
            }

            div {
                class: "flex flex-col sm:flex-row gap-4",
                Calendar {
                    class: "rounded-lg border",
                    selected: selected_date,
                    on_select: Some(Callback::from(move |date: NaiveDate| {
                        let mut s = state.write();
                        s.selected_date = Some(date);
                    })),
                    show_outside_days: true,
                }

                Card {
                    class: "w-full sm:w-64",
                    CardHeader {
                        class: "",
                        CardTitle {
                            class: "",
                            "Selected Date"
                        }
                    }
                    CardContent {
                        class: "",
                        match selected_date {
                            Some(date) => {
                                rsx! {
                                    p {
                                        class: "text-2xl font-bold",
                                        {date.format("%B %d, %Y").to_string()}
                                    }
                                    p {
                                        class: "text-sm text-muted-foreground mt-2",
                                        {date.format("%A").to_string()}
                                    }
                                }
                            }
                            None => rsx! {
                                p {
                                    class: "text-sm text-muted-foreground",
                                    "No date selected"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
