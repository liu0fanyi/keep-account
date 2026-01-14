use std::path::PathBuf;
use tauri::State;
use tauri::Manager;
use tauri::Emitter;

mod db;
mod models;
mod repository;

use db::DbState;
use models::*;
use repository::{category_repo, installment_repo, transaction_repo};

/// Global database state
pub struct AppState {
    pub db: DbState,
    pub db_path: PathBuf,
}

// ============================================================================
// Category Commands
// ============================================================================

#[tauri::command]
async fn get_categories(
    state: State<'_, AppState>,
) -> Result<Vec<Category>, String> {
    let conn = state.db.get_connection().await?;
    category_repo::get_all_categories(&conn).await
}

#[tauri::command]
async fn create_category(
    state: State<'_, AppState>,
    name: String,
    icon: Option<String>,
) -> Result<Category, String> {
    let conn = state.db.get_connection().await?;
    let new_category = NewCategory { name, icon };
    category_repo::create_category(&conn, new_category).await
}

#[tauri::command]
async fn update_category(
    state: State<'_, AppState>,
    id: i64,
    name: Option<String>,
    icon: Option<String>,
) -> Result<Category, String> {
    let conn = state.db.get_connection().await?;
    category_repo::update_category(&conn, id, name, icon).await
}

#[tauri::command]
async fn delete_category(
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    let conn = state.db.get_connection().await?;
    category_repo::delete_category(&conn, id).await
}

// ============================================================================
// Transaction Commands
// ============================================================================

#[tauri::command]
async fn get_transactions(
    state: State<'_, AppState>,
) -> Result<Vec<TransactionWithCategory>, String> {
    let conn = state.db.get_connection().await?;
    transaction_repo::get_transactions_with_category(&conn).await
}

#[tauri::command]
async fn get_transactions_by_month(
    state: State<'_, AppState>,
    year: i32,
    month: i32,
) -> Result<Vec<TransactionWithCategory>, String> {
    let conn = state.db.get_connection().await?;
    transaction_repo::get_transactions_by_month(&conn, year, month).await
}

#[tauri::command]
async fn create_transaction(
    state: State<'_, AppState>,
    category_id: i64,
    amount: f64,
    transaction_date: String,
    note: Option<String>,
) -> Result<Transaction, String> {
    let conn = state.db.get_connection().await?;
    let new_transaction = NewTransaction {
        category_id,
        amount,
        transaction_date,
        note,
    };
    transaction_repo::create_transaction(&conn, new_transaction).await
}

#[tauri::command]
async fn delete_transaction(
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    let conn = state.db.get_connection().await?;
    transaction_repo::delete_transaction(&conn, id).await
}

#[tauri::command]
async fn get_monthly_summary(
    state: State<'_, AppState>,
    year: i32,
    month: i32,
) -> Result<MonthlySummary, String> {
    let conn = state.db.get_connection().await?;
    transaction_repo::get_monthly_summary(&conn, year, month).await
}

// ============================================================================
// Installment Commands
// ============================================================================

#[tauri::command]
async fn get_installments(
    state: State<'_, AppState>,
) -> Result<Vec<InstallmentWithCategory>, String> {
    let conn = state.db.get_connection().await?;
    installment_repo::get_all_installments_with_category(&conn).await
}

#[tauri::command]
async fn create_installment(
    state: State<'_, AppState>,
    category_id: i64,
    total_amount: f64,
    installment_count: i32,
    start_date: String,
    note: Option<String>,
) -> Result<Installment, String> {
    let conn = state.db.get_connection().await?;
    let new_installment = NewInstallment {
        category_id,
        total_amount,
        installment_count,
        start_date,
        note,
    };
    installment_repo::create_installment(&conn, new_installment).await
}

#[tauri::command]
async fn get_installment_details(
    state: State<'_, AppState>,
    installment_id: i64,
) -> Result<Vec<InstallmentDetail>, String> {
    let conn = state.db.get_connection().await?;
    installment_repo::get_installment_details(&conn, installment_id).await
}

#[tauri::command]
async fn get_due_installments_by_month(
    state: State<'_, AppState>,
    year: i32,
    month: i32,
) -> Result<Vec<InstallmentDetail>, String> {
    let conn = state.db.get_connection().await?;
    installment_repo::get_due_installments_by_month(&conn, year, month).await
}

#[tauri::command]
async fn mark_installment_paid(
    state: State<'_, AppState>,
    detail_id: i64,
    paid_date: String,
) -> Result<(), String> {
    let conn = state.db.get_connection().await?;
    installment_repo::mark_installment_paid(&conn, detail_id, &paid_date).await
}

#[tauri::command]
async fn delete_installment(
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    let conn = state.db.get_connection().await?;
    installment_repo::delete_installment(&conn, id).await
}

// ============================================================================
// Sync Commands
// ============================================================================

#[tauri::command]
async fn sync_database(state: State<'_, AppState>) -> Result<(), String> {
    state.db.sync().await
}

#[tauri::command]
async fn configure_sync(
    state: State<'_, AppState>,
    url: String,
    token: String,
) -> Result<(), String> {
    log::info!("configure_sync called: url={}, token_len={}", url, token.len());

    if !url.is_empty() && !token.is_empty() {
        db::validate_cloud_connection(url.clone(), token.clone()).await
            .map_err(|e| format!("验证连接失败: {}", e))?;
    }

    db::configure_sync(&state.db_path, url, token).await
}

#[tauri::command]
async fn get_sync_config(
    state: State<'_, AppState>,
) -> Result<Option<db::SyncConfig>, String> {
    Ok(db::get_sync_config(&state.db_path))
}

#[tauri::command]
async fn has_legacy_db(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    Ok(db::has_legacy_db(&state.db_path))
}

#[tauri::command]
async fn migrate_from_legacy(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let conn = state.db.get_connection().await?;
    db::migrate_from_legacy(&state.db_path, &conn).await
}

#[tauri::command]
async fn is_cloud_sync_enabled(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    Ok(state.db.is_cloud_sync_enabled().await)
}

#[tauri::command]
async fn get_app_logs() -> Result<String, String> {
    rolling_logger::read_logs()
}

// ============================================================================
// Application Entry Point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            // Get database path
            let db_path = app
                .path()
                .app_local_data_dir()
                .expect("Failed to get app local data dir")
                .join("accounts.db");
            
            eprintln!("Database path: {:?}", db_path);
            
            let app_handle = app.handle().clone();

            rolling_logger::init_logger(
                app.path()
                    .app_local_data_dir()
                    .expect("Failed to get app local data dir"),
            )?;
            rolling_logger::info("Application started");
            
            let db_path_for_init = db_path.clone();
            
            // Create initial empty state (for immediate command usage)
            let db_state = db::DbState::new();
            
            // Manage state IMMEDIATELY so app doesn't panic
            app.manage(AppState { 
                db: db_state.clone(), // Clone the DbState so we can update it later
                db_path: db_path.clone() 
            });

            // Initialize database asynchronously in bg
            tauri::async_runtime::spawn(async move {
                eprintln!("Initializing database asynchronously at: {:?}", db_path_for_init);
                match db::init_db(&db_path_for_init).await {
                    Ok(initialized_state) => {
                        eprintln!("Database initialized successfully (async)");
                        let _ = rolling_logger::info("Async DB init success");
                        
                        // Update the existing DbState with the initialized data
                        eprintln!("Calling db_state.update_from...");
                        db_state.update_from(&initialized_state).await;
                        eprintln!("db_state.update_from completed successfully");
                        
                        if let Err(e) = app_handle.emit("db-initialized", ()) {
                            eprintln!("Failed to emit event: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize database (async): {}", e);
                        let _ = rolling_logger::error(&format!("Async DB init failed: {}", e));
                    }
                }
            });
            

            // Set window size based on monitor DPI (desktop only)
            // Phone screen: 2400x1080 (height x width) = aspect ratio 2.22:1

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(scale_factor) = window.scale_factor() {
                    eprintln!("Monitor scale factor: {}", scale_factor);
                    
                    // Base logical dimensions (for 100% scaling)
                    // Target size: 540x1200 to match phone screen proportion
                    let base_width = 540.0;
                    let base_height = 1200.0;
                    
                    // Adjust for scale factor to get appropriate logical size
                    // Higher DPI = smaller logical size to maintain physical size
                    let logical_width = (base_width / scale_factor).round() as f64;
                    let logical_height = (base_height / scale_factor).round() as f64;
                    
                    eprintln!("Setting window size to: {}x{} (logical pixels)", logical_width, logical_height);
                    
                    // Set window size (desktop platforms only)
                    use tauri::Size;
                    let _ = window.set_size(Size::Logical(tauri::LogicalSize {
                        width: logical_width,
                        height: logical_height,
                    }));
                } else {
                    eprintln!("Could not get scale factor, using default size");
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Category commands
            get_categories,
            create_category,
            update_category,
            delete_category,
            // Transaction commands
            get_transactions,
            get_transactions_by_month,
            create_transaction,
            delete_transaction,
            get_monthly_summary,
            // Installment commands
            get_installments,
            create_installment,
            get_installment_details,
            get_due_installments_by_month,
            mark_installment_paid,
            delete_installment,
            // Sync commands
            sync_database,
            configure_sync,
            get_sync_config,
            is_cloud_sync_enabled,
            has_legacy_db,
            migrate_from_legacy,
            get_app_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
