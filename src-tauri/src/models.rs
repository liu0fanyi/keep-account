//! Data Models
//!
//! Defines all data structures used in the application.

use serde::{Deserialize, Serialize};

/// 消费项目/分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 新建消费项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCategory {
    pub name: String,
    pub icon: Option<String>,
}

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub category_id: i64,
    pub amount: f64,
    pub transaction_date: String,
    pub note: Option<String>,
    pub created_at: String,
}

/// 新建交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTransaction {
    pub category_id: i64,
    pub amount: f64,
    pub transaction_date: String,
    pub note: Option<String>,
}

/// 分期计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installment {
    pub id: i64,
    pub category_id: i64,
    pub total_amount: f64,
    pub installment_count: i32,
    pub start_date: String,
    pub note: Option<String>,
    pub created_at: String,
}

/// 新建分期计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewInstallment {
    pub category_id: i64,
    pub total_amount: f64,
    pub installment_count: i32,
    pub start_date: String,
    pub note: Option<String>,
}

/// 分期明细
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallmentDetail {
    pub id: i64,
    pub installment_id: i64,
    pub sequence_number: i32,
    pub amount: f64,
    pub due_date: String,
    pub is_paid: bool,
    pub paid_date: Option<String>,
}

/// 带分类信息的交易记录（用于前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 带分类信息的分期计划（用于前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 按月份统计的交易汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySummary {
    pub year: i32,
    pub month: i32,
    pub total_income: f64,
    pub total_expense: f64,
    pub net_amount: f64,
    pub transaction_count: i32,
}
