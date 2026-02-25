#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

// UI components module - shadcn-inspired components for Dioxus
pub mod button;
pub mod card;
pub mod badge;
pub mod scroll_area;
pub mod separator;

pub use button::Button;
pub use card::{Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter};
pub use badge::Badge;
pub use scroll_area::ScrollArea;
pub use separator::Separator;
