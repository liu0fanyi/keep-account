//! Mobile components module.

mod nav;
mod list;
mod form;
mod category_form;
mod installment_form;
mod view;
mod liquid_container;

pub use nav::{MobileView, MobileBottomNav};
pub use list::MobileTransactionList;
pub use form::MobileTransactionForm;
pub use category_form::MobileCategoryForm;
pub use installment_form::MobileInstallmentForm;
pub use view::MobileTransactionView;
pub use liquid_container::LiquidContainer;

// Import shared sync settings form from frontend crate
pub use tauri_sync_db_frontend::mobile::SyncSettingsForm;

