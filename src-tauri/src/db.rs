//! Database Connection and Setup
//!
//! Manages SQLite database connection and migrations.

use std::sync::Arc;
use libsql::{Builder, Connection, Database};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::Mutex;

/// Sync configuration for Turso cloud database
#[derive(Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub url: String,
    pub token: String,
}

/// Database state wrapper
pub struct DbState {
    db: Arc<Mutex<Option<Arc<Database>>>>,
    conn: Arc<Mutex<Option<Connection>>>,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            db: Arc::new(Mutex::new(None)),
            conn: Arc::new(Mutex::new(None)),
        }
    }

    /// Get a connection, initializing if necessary
    pub async fn get_connection(&self) -> Result<Connection, String> {
        let guard = self.conn.lock().await;
        if let Some(conn) = &*guard {
            return Ok(conn.clone());
        }
        Err("Database not initialized".to_string())
    }

    /// Manually trigger database sync (for cloud-synced databases)
    pub async fn sync(&self) -> Result<(), String> {
        let guard = self.db.lock().await;
        if let Some(db) = &*guard {
            db.sync().await.map_err(|e| {
                let err_str = format!("{}", e);
                if err_str.contains("File mode") || err_str.contains("not supported") {
                    "云同步未启用。请先配置云同步并重启应用。".to_string()
                } else {
                    format!("同步失败: {}", e)
                }
            })?;
            Ok(())
        } else {
            Err("数据库未初始化".to_string())
        }
    }

    /// Close all connections and drop database
    pub async fn close(&self) {
        let mut db_guard = self.db.lock().await;
        let mut conn_guard = self.conn.lock().await;
        *conn_guard = None;
        *db_guard = None;
    }
}

/// Get sync configuration file path
fn get_config_path(db_path: &PathBuf) -> PathBuf {
    db_path.parent().unwrap().join("sync_config.json")
}

/// Load sync configuration from file
fn load_config(db_path: &PathBuf) -> Option<SyncConfig> {
    let path = get_config_path(db_path);
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            return serde_json::from_str(&content).ok();
        }
    }
    None
}

/// Initialize database with path
pub async fn init_db(db_path: &PathBuf) -> Result<DbState, String> {
    let db_path_str = db_path.to_str().ok_or("Invalid DB path")?;

    let config = load_config(db_path);

    let (db, conn) = if let Some(conf) = config {
        // Cloud sync mode
        eprintln!("Initializing Synced DB: {}, token len: {}", conf.url, conf.token.len());

        let db = Builder::new_synced_database(db_path_str, conf.url, conf.token)
            .build()
            .await
            .map_err(|e| format!("Build failed: {}", e))?;
        let conn = db.connect().map_err(|e| format!("Connect failed: {}", e))?;
        (db, conn)
    } else {
        // Local only mode
        let db = Builder::new_local(db_path_str)
            .build()
            .await
            .map_err(|e| format!("Failed to build local db: {}", e))?;
        let conn = db.connect().map_err(|e| format!("Failed to connect: {}", e))?;
        (db, conn)
    };

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", ())
        .await
        .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;

    // Run migrations
    run_migrations(&conn).await?;

    let state = DbState::new();
    *state.db.lock().await = Some(Arc::new(db));
    *state.conn.lock().await = Some(conn);

    Ok(state)
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

/// Configure cloud sync with Turso database
pub async fn configure_sync(db_path: &PathBuf, url: String, token: String) -> Result<(), String> {
    let config = SyncConfig { url, token };
    let config_path = get_config_path(db_path);
    std::fs::write(config_path, serde_json::to_string(&config).unwrap())
        .map_err(|e| e.to_string())?;

    eprintln!("Sync config saved");
    Ok(())
}

/// Get current sync configuration
pub fn get_sync_config(db_path: &PathBuf) -> Option<SyncConfig> {
    load_config(db_path)
}
