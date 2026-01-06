//! API helper functions to reduce boilerplate in components.

use leptos::task::spawn_local;
use crate::api::{invoke, JsValue};
use crate::types::{Category, TransactionWithCategory, MonthlySummary, InstallmentWithCategory, InstallmentDetail};

/// Load categories from backend
pub async fn fetch_categories() -> Result<Vec<Category>, String> {
    let result = invoke("get_categories", JsValue::NULL).await;
    serde_wasm_bindgen::from_value::<Vec<Category>>(result)
        .map_err(|e| format!("Failed to parse categories: {:?}", e))
}

/// Load transactions for a specific month
pub async fn fetch_transactions(year: i32, month: i32) -> Result<Vec<TransactionWithCategory>, String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "year": year,
        "month": month,
    })).map_err(|e| format!("Failed to serialize args: {:?}", e))?;
    
    let result = invoke("get_transactions_by_month", args).await;
    serde_wasm_bindgen::from_value::<Vec<TransactionWithCategory>>(result)
        .map_err(|e| format!("Failed to parse transactions: {:?}", e))
}

/// Load monthly summary
pub async fn fetch_monthly_summary(year: i32, month: i32) -> Result<MonthlySummary, String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "year": year,
        "month": month,
    })).map_err(|e| format!("Failed to serialize args: {:?}", e))?;
    
    let result = invoke("get_monthly_summary", args).await;
    serde_wasm_bindgen::from_value::<MonthlySummary>(result)
        .map_err(|e| format!("Failed to parse summary: {:?}", e))
}

/// Create a new transaction
pub async fn create_transaction(
    category_id: i64,
    amount: f64,
    transaction_date: &str,
    note: Option<String>,
) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "categoryId": category_id,
        "amount": amount,
        "transactionDate": transaction_date,
        "note": note,
    })).map_err(|e| format!("Failed to serialize args: {:?}", e))?;
    
    let result = invoke("create_transaction", args).await;
    
    // Check if result contains error
    if let Some(error) = result.as_string() {
        if error.contains("Error") || error.contains("error") {
            return Err(error);
        }
    }
    Ok(())
}

/// Create a new category
pub async fn create_category(name: &str, icon: &str) -> Result<Category, String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "name": name,
        "icon": icon,
    })).map_err(|e| format!("Failed to serialize args: {:?}", e))?;
    
    let result = invoke("create_category", args).await;
    serde_wasm_bindgen::from_value::<Category>(result)
        .map_err(|e| format!("Failed to parse category: {:?}", e))
}

/// Delete a category
pub async fn delete_category(id: i64) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "id": id }))
        .map_err(|e| format!("Failed to serialize args: {:?}", e))?;
    
    let _ = invoke("delete_category", args).await;
    Ok(())
}

/// Delete a transaction
pub async fn delete_transaction(id: i64) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "id": id }))
        .map_err(|e| format!("Failed to serialize args: {:?}", e))?;
    
    let _ = invoke("delete_transaction", args).await;
    Ok(())
}

/// Load all installments
pub async fn fetch_installments() -> Result<Vec<InstallmentWithCategory>, String> {
    let result = invoke("get_installments", JsValue::NULL).await;
    serde_wasm_bindgen::from_value::<Vec<InstallmentWithCategory>>(result)
        .map_err(|e| format!("Failed to parse installments: {:?}", e))
}

/// Load due installments for a month
pub async fn fetch_due_installments(year: i32, month: u32) -> Result<Vec<InstallmentDetail>, String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "year": year,
        "month": month,
    })).map_err(|e| format!("Failed to serialize args: {:?}", e))?;
    
    let result = invoke("get_due_installments_by_month", args).await;
    serde_wasm_bindgen::from_value::<Vec<InstallmentDetail>>(result)
        .map_err(|e| format!("Failed to parse installment details: {:?}", e))
}

/// Create a new installment
pub async fn create_installment(
    category_id: i64,
    total_amount: f64,
    installment_count: i32,
    start_date: &str,
    note: Option<String>,
) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "categoryId": category_id,
        "totalAmount": total_amount,
        "installmentCount": installment_count,
        "startDate": start_date,
        "note": note,
    })).map_err(|e| format!("Failed to serialize args: {:?}", e))?;
    
    let _ = invoke("create_installment", args).await;
    Ok(())
}

/// Delete an installment
pub async fn delete_installment(id: i64) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "id": id }))
        .map_err(|e| format!("Failed to serialize args: {:?}", e))?;
    
    let _ = invoke("delete_installment", args).await;
    Ok(())
}

/// Get installment details by installment ID
pub async fn fetch_installment_details(installment_id: i64) -> Result<Vec<InstallmentDetail>, String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "installmentId": installment_id }))
        .map_err(|e| format!("Failed to serialize args: {:?}", e))?;
    
    let result = invoke("get_installment_details", args).await;
    serde_wasm_bindgen::from_value::<Vec<InstallmentDetail>>(result)
        .map_err(|e| format!("Failed to parse installment details: {:?}", e))
}
