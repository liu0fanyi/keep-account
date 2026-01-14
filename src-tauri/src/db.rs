//! Database Connection and Setup
//!
//! Manages SQLite database connection and migrations.

use std::sync::Arc;
use libsql::{Connection, Builder};
use std::path::PathBuf;

use tauri_plugin_http::reqwest;

// Import shared sync types and functions from tauri-sync-db
pub use tauri_sync_db_backend::{
    DbState, 
    SyncConfig, 
    configure_sync as configure_sync_backend, 
    get_sync_config,
    validate_cloud_connection
};

// DbState, validate_cloud_connection, get/load config are now provided by tauri-sync-db

/// Initialize database and run migrations
pub async fn init_db(db_path: &PathBuf) -> Result<DbState, String> {
    // Use shared crate's init_db with our migration callback
    tauri_sync_db_backend::init_db(db_path, |conn| {
        Box::pin(async {
            run_migrations(conn).await
        })
    }).await
}

/// Check if a column exists in a table
async fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let query = format!("PRAGMA table_info({})", table);
    if let Ok(mut rows) = conn.query(&query, ()).await {
        while let Ok(Some(row)) = rows.next().await {
            if let Ok(name) = row.get::<String>(1) {
                if name == column {
                    return true;
                }
            }
        }
    }
    false
}

/// Run database migrations
async fn run_migrations(conn: &Connection) -> Result<(), String> {
    // Categories table (消费项目/分类)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            icon TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Insert default categories if table is empty
    let mut count_rows = conn.query("SELECT COUNT(*) FROM categories", ())
        .await
        .map_err(|e| e.to_string())?;
    
    let is_empty = if let Ok(Some(row)) = count_rows.next().await {
        row.get::<i64>(0).unwrap_or(0) == 0
    } else {
        true
    };

    if is_empty {
        // Default categories: 食物, 交通, 日用品, 孩子, 学习, 其它
        let default_categories = vec![
            ("食物", "🍔"),
            ("交通", "🚗"),
            ("日用品", "🛒"),
            ("孩子", "👶"),
            ("学习", "📚"),
            ("其它", "📦"),
        ];

        for (name, icon) in default_categories {
            conn.execute(
                "INSERT INTO categories (name, icon) VALUES (?, ?)",
                (name, icon),
            )
            .await
            .map_err(|e| format!("Failed to insert default category: {}", e))?;
        }
        eprintln!("Initialized default categories");
    }

    // Transactions table (交易记录)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id INTEGER NOT NULL,
            amount REAL NOT NULL,
            transaction_date TEXT NOT NULL DEFAULT (datetime('now')),
            note TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY(category_id) REFERENCES categories(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Installments table (分期记录)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS installments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id INTEGER NOT NULL,
            total_amount REAL NOT NULL,
            installment_count INTEGER NOT NULL,
            start_date TEXT NOT NULL,
            note TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY(category_id) REFERENCES categories(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Installment details table (分期明细)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS installment_details (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            installment_id INTEGER NOT NULL,
            sequence_number INTEGER NOT NULL,
            amount REAL NOT NULL,
            due_date TEXT NOT NULL,
            is_paid INTEGER NOT NULL DEFAULT 0,
            paid_date TEXT,
            FOREIGN KEY(installment_id) REFERENCES installments(id) ON DELETE CASCADE
        )",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Create indexes for better query performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(transaction_date)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_installment_details_due_date ON installment_details(due_date)",
        (),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Configure cloud sync - wrapper around shared crate version
pub async fn configure_sync(db_path: &PathBuf, url: String, token: String) -> Result<(), String> {
    let _ = rolling_logger::info(&format!("Configuring sync with URL: {}", url));
    configure_sync_backend(db_path, url, token).await?;
    eprintln!("Sync config saved");
    Ok(())
}

/// Check if legacy database exists
pub fn has_legacy_db(db_path: &PathBuf) -> bool {
    let legacy_path = db_path.with_extension("db.legacy");
    legacy_path.exists()
}

/// Migrate data from legacy database to current database
pub async fn migrate_from_legacy(db_path: &PathBuf, current_conn: &Connection) -> Result<String, String> {
    let legacy_path = db_path.with_extension("db.legacy");
    
    if !legacy_path.exists() {
        return Err("没有找到旧数据备份文件".to_string());
    }
    
    eprintln!("Opening legacy database: {:?}", legacy_path);
    
    let legacy_path_str = legacy_path.to_str().ok_or("Invalid legacy path")?;
    let legacy_db = Builder::new_local(legacy_path_str)
        .build()
        .await
        .map_err(|e| format!("无法打开旧数据库: {}", e))?;
    let legacy_conn = legacy_db.connect()
        .map_err(|e| format!("无法连接旧数据库: {}", e))?;
    
    let mut migrated_categories = 0;
    let mut migrated_transactions = 0;
    let mut migrated_installments = 0;
    
    // Migrate categories
    eprintln!("Migrating categories...");
    let mut cat_stmt = legacy_conn.prepare(
        "SELECT id, name, icon, created_at, updated_at FROM categories"
    ).await.map_err(|e| e.to_string())?;
    let mut cat_rows = cat_stmt.query(()).await.map_err(|e| e.to_string())?;
    
    while let Ok(Some(row)) = cat_rows.next().await {
        let id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let name: String = row.get(1).map_err(|e| e.to_string())?;
        let icon: Option<String> = row.get(2).ok();
        let created_at: String = row.get(3).map_err(|e| e.to_string())?;
        let updated_at: String = row.get(4).map_err(|e| e.to_string())?;
        
        current_conn.execute(
            "INSERT OR REPLACE INTO categories (id, name, icon, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            libsql::params![id, name, icon, created_at, updated_at]
        ).await.map_err(|e| e.to_string())?;
        migrated_categories += 1;
    }
    
    // Migrate transactions
    eprintln!("Migrating transactions...");
    let mut tx_stmt = legacy_conn.prepare(
        "SELECT id, category_id, amount, transaction_date, note, created_at FROM transactions"
    ).await.map_err(|e| e.to_string())?;
    let mut tx_rows = tx_stmt.query(()).await.map_err(|e| e.to_string())?;
    
    while let Ok(Some(row)) = tx_rows.next().await {
        let id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let category_id: i64 = row.get(1).map_err(|e| e.to_string())?;
        let amount: f64 = row.get(2).map_err(|e| e.to_string())?;
        let transaction_date: String = row.get(3).map_err(|e| e.to_string())?;
        let note: Option<String> = row.get(4).ok();
        let created_at: String = row.get(5).map_err(|e| e.to_string())?;
        
        current_conn.execute(
            "INSERT OR REPLACE INTO transactions (id, category_id, amount, transaction_date, note, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            libsql::params![id, category_id, amount, transaction_date, note, created_at]
        ).await.map_err(|e| e.to_string())?;
        migrated_transactions += 1;
    }
    
    // Migrate installments
    eprintln!("Migrating installments...");
    let mut inst_stmt = legacy_conn.prepare(
        "SELECT id, category_id, total_amount, installment_count, start_date, note, created_at FROM installments"
    ).await.map_err(|e| e.to_string())?;
    let mut inst_rows = inst_stmt.query(()).await.map_err(|e| e.to_string())?;
    
    while let Ok(Some(row)) = inst_rows.next().await {
        let id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let category_id: i64 = row.get(1).map_err(|e| e.to_string())?;
        let total_amount: f64 = row.get(2).map_err(|e| e.to_string())?;
        let installment_count: i32 = row.get(3).map_err(|e| e.to_string())?;
        let start_date: String = row.get(4).map_err(|e| e.to_string())?;
        let note: Option<String> = row.get(5).ok();
        let created_at: String = row.get(6).map_err(|e| e.to_string())?;
        
        current_conn.execute(
            "INSERT OR REPLACE INTO installments (id, category_id, total_amount, installment_count, start_date, note, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            libsql::params![id, category_id, total_amount, installment_count, start_date, note, created_at]
        ).await.map_err(|e| e.to_string())?;
        migrated_installments += 1;
    }
    
    // Migrate installment details
    eprintln!("Migrating installment details...");
    let mut detail_stmt = legacy_conn.prepare(
        "SELECT id, installment_id, sequence_number, amount, due_date, is_paid, paid_date FROM installment_details"
    ).await.map_err(|e| e.to_string())?;
    let mut detail_rows = detail_stmt.query(()).await.map_err(|e| e.to_string())?;
    
    while let Ok(Some(row)) = detail_rows.next().await {
        let id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let installment_id: i64 = row.get(1).map_err(|e| e.to_string())?;
        let sequence_number: i32 = row.get(2).map_err(|e| e.to_string())?;
        let amount: f64 = row.get(3).map_err(|e| e.to_string())?;
        let due_date: String = row.get(4).map_err(|e| e.to_string())?;
        let is_paid: i32 = row.get(5).map_err(|e| e.to_string())?;
        let paid_date: Option<String> = row.get(6).ok();
        
        current_conn.execute(
            "INSERT OR REPLACE INTO installment_details (id, installment_id, sequence_number, amount, due_date, is_paid, paid_date) VALUES (?, ?, ?, ?, ?, ?, ?)",
            libsql::params![id, installment_id, sequence_number, amount, due_date, is_paid, paid_date]
        ).await.map_err(|e| e.to_string())?;
    }
    
    let summary = format!(
        "迁移完成！已导入 {} 个分类、{} 条交易记录、{} 个分期计划",
        migrated_categories, migrated_transactions, migrated_installments
    );
    eprintln!("{}", summary);
    
    Ok(summary)
}
