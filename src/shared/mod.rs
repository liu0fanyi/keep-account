//! Shared utilities module.
//! 
//! Contains common code used by both mobile and desktop components.

pub mod icons;
pub mod validators;
pub mod api_helpers;

// Re-exports for convenience
pub use icons::{COMMON_ICONS, DEFAULT_ICON};
pub use validators::{validate_amount, validate_category_id, validate_not_empty, parse_positive_int};
pub use api_helpers::*;
