//! Category Repository
//!
//! Manages category (消费项目) CRUD operations.

use crate::db::DbState;
use crate::models::{Category, NewCategory};
use libsql::Connection;

/// Create a new category
pub async fn create_category(
    conn: &Connection,
    new_category: NewCategory,
) -> Result<Category, String> {
    conn.execute(
        "INSERT INTO categories (name, icon) VALUES (?, ?)",
        libsql::params![new_category.name, new_category.icon],
    )
    .await
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    let category = get_category_by_id(conn, id).await?;
    Ok(category)
}

/// Get category by ID
pub async fn get_category_by_id(conn: &Connection, id: i64) -> Result<Category, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, icon, created_at, updated_at FROM categories WHERE id = ?")
        .await
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query(libsql::params![id])
        .await
        .map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        Ok(Category {
            id: row.get(0).map_err(|e| e.to_string())?,
            name: row.get(1).map_err(|e| e.to_string())?,
            icon: row.get(2).ok(),
            created_at: row.get(3).map_err(|e| e.to_string())?,
            updated_at: row.get(4).map_err(|e| e.to_string())?,
        })
    } else {
        Err("Category not found".to_string())
    }
}

/// Get all categories
pub async fn get_all_categories(conn: &Connection) -> Result<Vec<Category>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, icon, created_at, updated_at FROM categories ORDER BY name")
        .await
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query(()).await.map_err(|e| e.to_string())?;

    let mut categories = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        categories.push(Category {
            id: row.get(0).map_err(|e| e.to_string())?,
            name: row.get(1).map_err(|e| e.to_string())?,
            icon: row.get(2).ok(),
            created_at: row.get(3).map_err(|e| e.to_string())?,
            updated_at: row.get(4).map_err(|e| e.to_string())?,
        });
    }

    Ok(categories)
}

/// Update category
pub async fn update_category(
    conn: &Connection,
    id: i64,
    name: Option<String>,
    icon: Option<String>,
) -> Result<Category, String> {
    if let Some(name) = name {
        conn.execute(
            "UPDATE categories SET name = ?, updated_at = datetime('now') WHERE id = ?",
            libsql::params![name, id],
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    if let Some(icon) = icon {
        conn.execute(
            "UPDATE categories SET icon = ?, updated_at = datetime('now') WHERE id = ?",
            libsql::params![icon, id],
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    get_category_by_id(conn, id).await
}

/// Delete category
pub async fn delete_category(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM categories WHERE id = ?", libsql::params![id])
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
