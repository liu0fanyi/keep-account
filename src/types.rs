//! Shared data types for the keep-accounts application.
//!
//! These types are used by both desktop and mobile views.

use serde::{Deserialize, Serialize};

/// Category for transactions (消费类型)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Transaction with its category information
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TransactionWithCategory {
    pub id: i64,
    pub category_id: i64,
    pub category_name: String,
    pub category_icon: Option<String>,
    pub amount: f64,
    pub transaction_date: String,
    pub note: Option<String>,
    pub created_at: String,
}

/// Monthly summary statistics
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MonthlySummary {
    pub year: i32,
    pub month: i32,
    pub total_income: f64,
    pub total_expense: f64,
    pub net_amount: f64,
    pub transaction_count: i32,
}

/// Installment with its category information
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InstallmentWithCategory {
    pub id: i64,
    pub category_id: i64,
    pub category_name: String,
    pub category_icon: Option<String>,
    pub total_amount: f64,
    pub installment_count: i32,
    pub start_date: String,
    pub note: Option<String>,
    pub created_at: String,
}

/// Individual installment payment detail
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InstallmentDetail {
    pub id: i64,
    pub installment_id: i64,
    pub sequence_number: i32,
    pub amount: f64,
    pub due_date: String,
    pub is_paid: bool,
    pub paid_date: Option<String>,
}
