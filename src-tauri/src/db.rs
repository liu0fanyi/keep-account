//! Database Connection and Setup
//!
//! Manages SQLite database connection and migrations.

use std::sync::Arc;
use libsql::{Builder, Connection, Database};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::Mutex;

use tauri_plugin_http::reqwest;

/// Sync configuration for Turso cloud database
#[derive(Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub url: String,
    pub token: String,
}

/// Database state wrapper
#[derive(Clone)]
pub struct DbState {
    db: Arc<Mutex<Option<Arc<Database>>>>,
    conn: Arc<Mutex<Option<Connection>>>,
    /// Whether cloud sync is enabled for this session
    is_sync_enabled: Arc<Mutex<bool>>,
    /// Current sync URL (for logging)
    sync_url: Arc<Mutex<String>>,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            db: Arc::new(Mutex::new(None)),
            conn: Arc::new(Mutex::new(None)),
            is_sync_enabled: Arc::new(Mutex::new(false)),
            sync_url: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Check if cloud sync is enabled for this session
    pub async fn is_cloud_sync_enabled(&self) -> bool {
        *self.is_sync_enabled.lock().await
    }

    /// Set sync enabled status and URL
    pub async fn set_sync_config(&self, enabled: bool, url: String) {
        *self.is_sync_enabled.lock().await = enabled;
        *self.sync_url.lock().await = url;
    }

    /// Get current sync URL
    pub async fn get_sync_url(&self) -> String {
        self.sync_url.lock().await.clone()
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

pub(crate) async fn validate_cloud_connection(url: String, token: String) -> Result<(), String> {
    log::info!("Validating cloud connection: url={}", url);

    // Basic format check
    if !url.starts_with("libsql://") && !url.starts_with("https://") {
        log::error!("Invalid URL format: {}", url);
        return Err("URL must start with libsql:// or https://".to_string());
    }

    // Convert libsql:// to https:// for HTTP check
    let http_url = if url.starts_with("libsql://") {
        url.replace("libsql://", "https://")
    } else {
        url.clone()
    };

    log::info!("HTTP URL: {}", http_url);
    log::info!("Token length: {}", token.len());

    // Use reqwest to check connectivity AND authentication
    // We must send a query to trigger actual token validation.
    // Just checking GET / might return 200 OK (welcome page) even with bad token.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            log::error!("Failed to build HTTP client: {}", e);
            format!("Client build failed: {}", e)
        })?;

    // Standard LibSQL/Turso HTTP API expects POST with JSON statements
    let query_body = serde_json::json!({
        "statements": ["SELECT 1"]
    });

    log::info!("Sending validation request to: {}", http_url);

    let body_str = serde_json::to_string(&query_body).map_err(|e| e.to_string())?;

    let res = client.post(&http_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(body_str)
        .send()
        .await;

    let res = match res {
        Ok(r) => {
            log::info!("Request sent successfully");
            r
        }
        Err(e) => {
            log::error!("Request failed: {}", e);
            return Err(format!("Connection failed: {}", e));
        }
    };

    let status = res.status();
    log::info!("Response status: {}", status);

    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        log::error!("Authentication failed");
        return Err("Authentication failed (Invalid Token)".to_string());
    }

    if !status.is_success() {
        log::error!("Server returned error status: {}", status);
        return Err(format!("Server returned error: {}", status));
    }

    log::info!("Cloud connection validated successfully");
    Ok(())
}

/// Initialize database (async, populates existing state)
pub async fn init_db(db_path: &PathBuf, state: Arc<DbState>) -> Result<(), String> {
    let db_path_str = db_path.to_str().ok_or("Invalid DB path")?;

    let config = load_config(db_path);

    let (db, conn, is_cloud_sync, sync_url) = if let Some(conf) = config {
        // Only use cloud sync if BOTH url and token are non-empty
        if conf.url.is_empty() || conf.token.is_empty() {
            eprintln!("Sync config has empty URL or token, falling back to local mode");
            let db = Builder::new_local(db_path_str)
                .build()
                .await
                .map_err(|e| format!("Failed to build local db: {}", e))?;
            let conn = db.connect().map_err(|e| format!("Failed to connect: {}", e))?;
            (db, conn, false, String::new())
        } else {
            // Cloud sync mode
            let msg = format!("Initializing Synced DB: {}, token len: {}", conf.url, conf.token.len());
            eprintln!("{}", msg);
            let _ = rolling_logger::info(&msg);

            // Validate connection first!
            let validation_result = validate_cloud_connection(conf.url.clone(), conf.token.clone()).await;

            if let Err(e) = validation_result {
                eprintln!("Cloud connection validation failed: {}", e);
                eprintln!("Falling back to local mode due to invalid configuration.");
                let db = Builder::new_local(db_path_str)
                    .build()
                    .await
                    .map_err(|e| format!("Failed to build local db: {}", e))?;
                let conn = db.connect().map_err(|e| format!("Failed to connect: {}", e))?;
                (db, conn, false, String::new())
            } else {
                // Validation passed, try cloud connection
                let sync_url = conf.url.clone();

                async fn try_build_connect(path: &str, url: String, token: String) -> Result<(Database, Connection), String> {
                    let https = hyper_rustls::HttpsConnectorBuilder::new()
                        .with_webpki_roots()
                        .https_or_http()
                        .enable_http1()
                        .build();

                    let db = Builder::new_synced_database(path, url, token)
                        .connector(https)
                        .build()
                        .await
                        .map_err(|e| format!("Build failed: {}", e))?;
                    let conn = db.connect().map_err(|e| format!("Connect failed: {}", e))?;

                    // Force initial sync to detect conflicts immediately
                    db.sync().await.map_err(|e| format!("Initial sync failed: {}", e))?;

                    Ok((db, conn))
                }

                let (db, conn, _success, _url) = match try_build_connect(db_path_str, conf.url.clone(), conf.token.clone()).await {
                    Ok((db, conn)) => (db, conn, true, sync_url.clone()),
                    Err(e) => {
                        eprintln!("Synced DB init failed: {}", e);

                        // Check for various sync conflict conditions
                        let should_recover = e.contains("local state is incorrect")
                            || e.contains("invalid local state")
                            || e.contains("server returned a conflict")
                            || e.contains("Generation ID mismatch")
                            || e.contains("mismatch")
                            || e.contains("metadata file does not");

                        eprintln!("Should auto-recover: {}", should_recover);

                        if should_recover {
                            eprintln!("Detected conflicting local DB state. Auto-recovering by wiping local DB...");

                            // Backup conflicting database
                            let conflict_path = db_path.with_extension("db.legacy");
                            if conflict_path.exists() {
                                eprintln!("Removing old legacy backup: {:?}", conflict_path);
                                let _ = std::fs::remove_file(&conflict_path);
                            }
                            if let Err(e) = std::fs::rename(&db_path, &conflict_path) {
                                eprintln!("Rename to legacy failed: {} - removing instead", e);
                                let _ = std::fs::remove_file(&db_path);
                            } else {
                                eprintln!("Backed up old DB to: {:?}", conflict_path);
                            }

                            // Clean up sync metadata
                            eprintln!("Cleaning up sync metadata...");
                            let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
                            let _ = std::fs::remove_file(db_path.with_extension("db-shm"));

                            let sync_dir = db_path.parent().unwrap().join(format!("{}-sync", db_path.file_name().unwrap().to_str().unwrap()));
                            if sync_dir.exists() {
                                eprintln!("Removing sync directory: {:?}", sync_dir);
                                if sync_dir.is_dir() {
                                    let _ = std::fs::remove_dir_all(&sync_dir);
                                } else {
                                    let _ = std::fs::remove_file(&sync_dir);
                                }
                            }

                            eprintln!("Retrying with clean state...");
                            // Retry with clean state
                            match try_build_connect(db_path_str, conf.url, conf.token).await {
                                Ok((db, conn)) => (db, conn, true, sync_url.clone()),
                                Err(e) => {
                                     eprintln!("Retry failed after recovery: {}", e);
                                     let _ = rolling_logger::error(&format!("Retry failed after recovery: {}", e));

                                      eprintln!("Falling back to local mode...");
                                      let db = Builder::new_local(db_path_str)
                                        .build()
                                        .await
                                        .map_err(|e| format!("Failed to build local db: {}", e))?;
                                      let conn = db.connect().map_err(|e| format!("Failed to connect: {}", e))?;
                                      (db, conn, false, String::new())
                                }
                            }
                        } else {
                            eprintln!("Cloud init failed (non-recoverable): {}", e);
                            let _ = rolling_logger::warn(&format!("Cloud init failed, falling back to local: {}", e));
                            eprintln!("Falling back to local mode...");

                            let db = Builder::new_local(db_path_str)
                                .build()
                                .await
                                .map_err(|e| format!("Failed to build local db: {}", e))?;
                            let conn = db.connect().map_err(|e| format!("Failed to connect: {}", e))?;
                            (db, conn, false, String::new())
                        }
                    }
                };

                (db, conn, true, _url)
            }
        }
    } else {
        // Local only mode
        let db = Builder::new_local(db_path_str)
            .build()
            .await
            .map_err(|e| format!("Failed to build local db: {}", e))?;
        let conn = db.connect().map_err(|e| format!("Failed to connect: {}", e))?;
        (db, conn, false, String::new())
    };

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", ())
        .await
        .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;

    // Run migrations
    run_migrations(&conn).await?;

    // Populate the existing state
    *state.db.lock().await = Some(Arc::new(db));
    *state.conn.lock().await = Some(conn);
    state.set_sync_config(is_cloud_sync, sync_url).await;

    Ok(())
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

/// Configure cloud sync with Turso database
pub async fn configure_sync(db_path: &PathBuf, url: String, token: String) -> Result<(), String> {
    let _ = rolling_logger::info(&format!("Configuring sync with URL: {}", url));
    let config = SyncConfig { url, token };
    let config_path = get_config_path(db_path);
    std::fs::write(config_path, serde_json::to_string(&config).unwrap())
        .map_err(|e| {
            let _ = rolling_logger::error(&format!("Failed to write sync config: {}", e));
            e.to_string()
        })?;

    eprintln!("Sync config saved");
    let _ = rolling_logger::info("Sync config saved successfully");
    Ok(())
}

/// Get current sync configuration
pub fn get_sync_config(db_path: &PathBuf) -> Option<SyncConfig> {
    load_config(db_path)
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
