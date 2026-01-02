//! Transaction Repository
//!
//! Manages transaction (交易记录) CRUD operations.

use crate::db::DbState;
use crate::models::{NewTransaction, Transaction, TransactionWithCategory};
use libsql::Connection;

/// Create a new transaction
pub async fn create_transaction(
    conn: &Connection,
    new_transaction: NewTransaction,
) -> Result<Transaction, String> {
    conn.execute(
        "INSERT INTO transactions (category_id, amount, transaction_date, note) VALUES (?, ?, ?, ?)",
        libsql::params![
            new_transaction.category_id,
            new_transaction.amount,
            new_transaction.transaction_date,
            new_transaction.note
        ],
    )
    .await
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    let transaction = get_transaction_by_id(conn, id).await?;
    Ok(transaction)
}

/// Get transaction by ID
pub async fn get_transaction_by_id(conn: &Connection, id: i64) -> Result<Transaction, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, category_id, amount, transaction_date, note, created_at
             FROM transactions WHERE id = ?"
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query(libsql::params![id])
        .await
        .map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        Ok(Transaction {
            id: row.get(0).map_err(|e| e.to_string())?,
            category_id: row.get(1).map_err(|e| e.to_string())?,
            amount: row.get(2).map_err(|e| e.to_string())?,
            transaction_date: row.get(3).map_err(|e| e.to_string())?,
            note: row.get(4).ok(),
            created_at: row.get(5).map_err(|e| e.to_string())?,
        })
    } else {
        Err("Transaction not found".to_string())
    }
}

/// Get all transactions
pub async fn get_all_transactions(conn: &Connection) -> Result<Vec<Transaction>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, category_id, amount, transaction_date, note, created_at
             FROM transactions
             ORDER BY transaction_date DESC"
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query(()).await.map_err(|e| e.to_string())?;

    let mut transactions = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        transactions.push(Transaction {
            id: row.get(0).map_err(|e| e.to_string())?,
            category_id: row.get(1).map_err(|e| e.to_string())?,
            amount: row.get(2).map_err(|e| e.to_string())?,
            transaction_date: row.get(3).map_err(|e| e.to_string())?,
            note: row.get(4).ok(),
            created_at: row.get(5).map_err(|e| e.to_string())?,
        });
    }

    Ok(transactions)
}

/// Get transactions with category information
pub async fn get_transactions_with_category(
    conn: &Connection,
) -> Result<Vec<TransactionWithCategory>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.category_id, c.name, c.icon, t.amount, t.transaction_date, t.note, t.created_at
             FROM transactions t
             INNER JOIN categories c ON t.category_id = c.id
             ORDER BY t.transaction_date DESC"
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query(()).await.map_err(|e| e.to_string())?;

    let mut transactions = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        transactions.push(TransactionWithCategory {
            id: row.get(0).map_err(|e| e.to_string())?,
            category_id: row.get(1).map_err(|e| e.to_string())?,
            category_name: row.get(2).map_err(|e| e.to_string())?,
            category_icon: row.get(3).ok(),
            amount: row.get(4).map_err(|e| e.to_string())?,
            transaction_date: row.get(5).map_err(|e| e.to_string())?,
            note: row.get(6).ok(),
            created_at: row.get(7).map_err(|e| e.to_string())?,
        });
    }

    Ok(transactions)
}

/// Get transactions by month (YYYY-MM format)
pub async fn get_transactions_by_month(
    conn: &Connection,
    year: i32,
    month: i32,
) -> Result<Vec<TransactionWithCategory>, String> {
    let month_start = format!("{:04}-{:02}-01", year, month);
    let next_month = if month == 12 {
        format!("{:04}-01-01", year + 1)
    } else {
        format!("{:04}-{:02}-01", year, month + 1)
    };

    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.category_id, c.name, c.icon, t.amount, t.transaction_date, t.note, t.created_at
             FROM transactions t
             INNER JOIN categories c ON t.category_id = c.id
             WHERE t.transaction_date >= ? AND t.transaction_date < ?
             ORDER BY t.transaction_date DESC"
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query(libsql::params![month_start, next_month])
        .await
        .map_err(|e| e.to_string())?;

    let mut transactions = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        transactions.push(TransactionWithCategory {
            id: row.get(0).map_err(|e| e.to_string())?,
            category_id: row.get(1).map_err(|e| e.to_string())?,
            category_name: row.get(2).map_err(|e| e.to_string())?,
            category_icon: row.get(3).ok(),
            amount: row.get(4).map_err(|e| e.to_string())?,
            transaction_date: row.get(5).map_err(|e| e.to_string())?,
            note: row.get(6).ok(),
            created_at: row.get(7).map_err(|e| e.to_string())?,
        });
    }

    Ok(transactions)
}

/// Delete transaction
pub async fn delete_transaction(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM transactions WHERE id = ?", libsql::params![id])
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Get monthly summary
pub async fn get_monthly_summary(
    conn: &Connection,
    year: i32,
    month: i32,
) -> Result<crate::models::MonthlySummary, String> {
    let month_start = format!("{:04}-{:02}-01", year, month);
    let next_month = if month == 12 {
        format!("{:04}-01-01", year + 1)
    } else {
        format!("{:04}-{:02}-01", year, month + 1)
    };

    let mut stmt = conn
        .prepare(
            "SELECT
                CAST(COALESCE(SUM(CASE WHEN amount >= 0 THEN amount ELSE 0 END), 0) AS REAL) as income,
                CAST(COALESCE(SUM(CASE WHEN amount < 0 THEN ABS(amount) ELSE 0 END), 0) AS REAL) as expense,
                CAST(COALESCE(SUM(amount), 0) AS REAL) as net,
                COUNT(*) as count
             FROM transactions
             WHERE transaction_date >= ? AND transaction_date < ?"
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query(libsql::params![month_start, next_month])
        .await
        .map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        // Get values with proper type handling
        let income: f64 = row.get(0).map_err(|e| e.to_string())?;
        let expense: f64 = row.get(1).map_err(|e| e.to_string())?;
        let net: f64 = row.get(2).map_err(|e| e.to_string())?;
        let count: i64 = row.get(3).map_err(|e| e.to_string())?;

        Ok(crate::models::MonthlySummary {
            year,
            month,
            total_income: income,
            total_expense: expense,
            net_amount: net,
            transaction_count: count as i32,
        })
    } else {
        Ok(crate::models::MonthlySummary {
            year,
            month,
            total_income: 0.0,
            total_expense: 0.0,
            net_amount: 0.0,
            transaction_count: 0,
        })
    }
}
