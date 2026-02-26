#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

// UI components module - shadcn-inspired components for Dioxus
pub mod badge;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod confidence_badge;
pub mod dialog;
pub mod input;
pub mod label;
pub mod progress;
pub mod scroll_area;
pub mod select;
pub mod separator;
pub mod skeleton;
pub mod switch;
pub mod tabs;
pub mod textarea;

pub use badge::Badge;
pub use button::Button;
pub use card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
pub use checkbox::Checkbox;
pub use confidence_badge::ConfidenceBadge;
pub use dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
    DialogTrigger,
};
pub use input::Input;
pub use label::Label;
pub use progress::Progress;
pub use scroll_area::ScrollArea;
pub use select::{
    Select, SelectContent, SelectGroup, SelectIcon, SelectItem, SelectItemIndicator, SelectLabel,
    SelectScrollDownButton, SelectScrollUpButton, SelectSeparator, SelectTrigger, SelectValue,
};
pub use separator::Separator;
pub use skeleton::Skeleton;
pub use switch::Switch;
pub use tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
pub use textarea::Textarea;
