//! Quick layout template engine.

pub mod diff;
pub mod engine;
pub mod error;
pub mod size;
pub mod template;

pub use size::parse_byte_size;

pub use diff::build_diff_report;
pub use engine::generate_layout;
pub use error::{LayoutError, LayoutResult};
pub use template::{load_template, LayoutTemplate, TemplateRegistry};
