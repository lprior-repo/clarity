use dioxus::prelude::*;

/// Main application component with responsive design
/// Follows mobile-first approach with fluid typography and spacing
pub fn app() -> Element {
    rsx! {
        div { class: "container",
            // Skip to content link for accessibility
            a { href: "#main-content", class: "skip-to-content", "Skip to main content" }

            // Header with responsive navigation
            header { class: "nav",
                div { class: "flex justify-between items-center",
                    h1 { class: "text-wrap", "Clarity Application" }
                    nav {
                        ul { class: "flex flex-wrap gap-md",
                            li { class: "nav-item", a { href: "#home", "Home" } }
                            li { class: "nav-item", a { href: "#features", "Features" } }
                            li { class: "nav-item", a { href: "#about", "About" } }
                        }
                    }
                }
            }

            // Hero section with responsive layout
            main { id: "main-content",
                section { class: "hero",
                    h2 { class: "text-wrap", "Welcome to Clarity" }
                    p { class: "text-wrap",
                        "A modern fullstack Dioxus application with responsive design, \
                        built using functional Rust patterns and zero-unwrap philosophy."
                    }
                }

                // Feature cards with responsive grid
                section { class: "grid grid-cols-1 grid-cols-md-2 grid-cols-lg-3 gap-lg",
                    // Card 1
                    div { class: "card",
                        h3 { "Responsive Design" }
                        p { "Mobile-first approach with fluid typography and spacing" }
                        button { class: "btn btn-primary touch-target", "Learn More" }
                    }

                    // Card 2
                    div { class: "card",
                        h3 { "Functional Rust" }
                        p { "Zero unwrap philosophy with Result-based error handling" }
                        button { class: "btn btn-primary touch-target", "Explore" }
                    }

                    // Card 3
                    div { class: "card",
                        h3 { "Type Safety" }
                        p { "Leverage Rust's type system for compile-time guarantees" }
                        button { class: "btn btn-primary touch-target", "Discover" }
                    }
                }
            }

            // Footer
            footer { class: "text-wrap",
                p { "Built with Dioxus, Rust, and functional programming principles" }
                p { class: "text-sm",
                    "Supports dark mode, reduced motion, and screen readers"
                }
            }
        }
    }
}

// Note: App renamed to app() to follow Rust naming conventions
// Use app() instead of App()
