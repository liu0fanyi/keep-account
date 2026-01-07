//! Installment Repository
//!
//! Manages installment (分期) operations.

use crate::db::DbState;
use crate::models::{Installment, InstallmentDetail, InstallmentWithCategory, NewInstallment};
use libsql::Connection;
use chrono::Datelike;

/// Create a new installment plan with details
pub async fn create_installment(
    conn: &Connection,
    new_installment: NewInstallment,
) -> Result<Installment, String> {
    // Calculate monthly amount
    let monthly_amount = new_installment.total_amount / new_installment.installment_count as f64;

    // Create installment record
    conn.execute(
        "INSERT INTO installments (category_id, total_amount, installment_count, start_date, note)
         VALUES (?, ?, ?, ?, ?)",
        libsql::params![
            new_installment.category_id,
            new_installment.total_amount,
            new_installment.installment_count,
            new_installment.start_date.clone(),
            new_installment.note
        ],
    )
    .await
    .map_err(|e| e.to_string())?;

    let installment_id = conn.last_insert_rowid();

    // Create installment details
    for i in 0..new_installment.installment_count {
        let sequence_number = i + 1;
        let due_date = calculate_due_date(&new_installment.start_date, i)?;
        let amount = monthly_amount;

        conn.execute(
            "INSERT INTO installment_details (installment_id, sequence_number, amount, due_date)
             VALUES (?, ?, ?, ?)",
            libsql::params![installment_id, sequence_number, amount, due_date],
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    get_installment_by_id(conn, installment_id).await
}

/// Calculate due date for installment
fn calculate_due_date(start_date: &str, months_offset: i32) -> Result<String, String> {
    use chrono::NaiveDate;

    let date = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;

    // Proper month arithmetic
    let year = date.year();
    let month = date.month() as i32;
    let day = date.day();
    
    // Calculate new year and month
    let total_months = (year * 12 + month - 1) + months_offset;
    let new_year = total_months / 12;
    let new_month = (total_months % 12 + 1) as u32;
    
    // Handle day overflow (e.g., Jan 31 -> Feb 28)
    let days_in_month = days_in_month(new_year, new_month);
    let new_day = day.min(days_in_month);
    
    let due_date = NaiveDate::from_ymd_opt(new_year, new_month, new_day)
        .ok_or_else(|| format!("Invalid date: {}-{}-{}", new_year, new_month, new_day))?;

    Ok(due_date.format("%Y-%m-%d").to_string())
}

/// Get days in a month
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        },
        _ => 30,
    }
}

/// Get installment by ID
pub async fn get_installment_by_id(conn: &Connection, id: i64) -> Result<Installment, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, category_id, total_amount, installment_count, start_date, note, created_at
             FROM installments WHERE id = ?"
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query(libsql::params![id])
        .await
        .map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        Ok(Installment {
            id: row.get(0).map_err(|e| e.to_string())?,
            category_id: row.get(1).map_err(|e| e.to_string())?,
            total_amount: row.get(2).map_err(|e| e.to_string())?,
            installment_count: row.get(3).map_err(|e| e.to_string())?,
            start_date: row.get(4).map_err(|e| e.to_string())?,
            note: row.get(5).ok(),
            created_at: row.get(6).map_err(|e| e.to_string())?,
        })
    } else {
        Err("Installment not found".to_string())
    }
}

/// Get all installments with category info
pub async fn get_all_installments_with_category(
    conn: &Connection,
) -> Result<Vec<InstallmentWithCategory>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.category_id, c.name, c.icon, i.total_amount, i.installment_count,
                    i.start_date, i.note, i.created_at
             FROM installments i
             INNER JOIN categories c ON i.category_id = c.id
             ORDER BY i.start_date DESC"
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query(()).await.map_err(|e| e.to_string())?;

    let mut installments = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        installments.push(InstallmentWithCategory {
            id: row.get(0).map_err(|e| e.to_string())?,
            category_id: row.get(1).map_err(|e| e.to_string())?,
            category_name: row.get(2).map_err(|e| e.to_string())?,
            category_icon: row.get(3).ok(),
            total_amount: row.get(4).map_err(|e| e.to_string())?,
            installment_count: row.get(5).map_err(|e| e.to_string())?,
            start_date: row.get(6).map_err(|e| e.to_string())?,
            note: row.get(7).ok(),
            created_at: row.get(8).map_err(|e| e.to_string())?,
        });
    }

    Ok(installments)
}

/// Get installment details by installment ID
pub async fn get_installment_details(
    conn: &Connection,
    installment_id: i64,
) -> Result<Vec<InstallmentDetail>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, installment_id, sequence_number, amount, due_date, is_paid, paid_date
             FROM installment_details
             WHERE installment_id = ?
             ORDER BY sequence_number"
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query(libsql::params![installment_id])
        .await
        .map_err(|e| e.to_string())?;

    let mut details = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        details.push(InstallmentDetail {
            id: row.get(0).map_err(|e| e.to_string())?,
            installment_id: row.get(1).map_err(|e| e.to_string())?,
            sequence_number: row.get(2).map_err(|e| e.to_string())?,
            amount: row.get(3).map_err(|e| e.to_string())?,
            due_date: row.get(4).map_err(|e| e.to_string())?,
            is_paid: row.get::<i32>(5).map_err(|e| e.to_string())? != 0,
            paid_date: row.get(6).ok(),
        });
    }

    Ok(details)
}

/// Get due installments for a specific month
pub async fn get_due_installments_by_month(
    conn: &Connection,
    year: i32,
    month: i32,
) -> Result<Vec<InstallmentDetail>, String> {
    let month_start = format!("{:04}-{:02}-01", year, month);
    let next_month = if month == 12 {
        format!("{:04}-01-01", year + 1)
    } else {
        format!("{:04}-{:02}-01", year, month + 1)
    };

    let mut stmt = conn
        .prepare(
            "SELECT id, installment_id, sequence_number, amount, due_date, is_paid, paid_date
             FROM installment_details
             WHERE due_date >= ? AND due_date < ? AND is_paid = 0
             ORDER BY due_date"
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query(libsql::params![month_start, next_month])
        .await
        .map_err(|e| e.to_string())?;

    let mut details = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        details.push(InstallmentDetail {
            id: row.get(0).map_err(|e| e.to_string())?,
            installment_id: row.get(1).map_err(|e| e.to_string())?,
            sequence_number: row.get(2).map_err(|e| e.to_string())?,
            amount: row.get(3).map_err(|e| e.to_string())?,
            due_date: row.get(4).map_err(|e| e.to_string())?,
            is_paid: row.get::<i32>(5).map_err(|e| e.to_string())? != 0,
            paid_date: row.get(6).ok(),
        });
    }

    Ok(details)
}

/// Mark installment detail as paid
pub async fn mark_installment_paid(
    conn: &Connection,
    detail_id: i64,
    paid_date: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE installment_details SET is_paid = 1, paid_date = ? WHERE id = ?",
        libsql::params![paid_date, detail_id],
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Delete installment and its details
pub async fn delete_installment(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM installments WHERE id = ?", libsql::params![id])
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
