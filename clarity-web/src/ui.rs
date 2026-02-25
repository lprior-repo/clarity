#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

// UI components module - shadcn-inspired components for Dioxus
pub mod badge;
pub mod button;
pub mod card;
pub mod input;
pub mod label;
pub mod scroll_area;
pub mod separator;
pub mod textarea;

pub use badge::Badge;
pub use button::Button;
pub use card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
pub use input::Input;
pub use label::Label;
pub use scroll_area::ScrollArea;
pub use separator::Separator;
pub use textarea::Textarea;
