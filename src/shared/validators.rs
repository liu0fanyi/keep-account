//! Input validation utilities.

/// Validate and parse amount string
pub fn validate_amount(amount_str: &str) -> Result<f64, &'static str> {
    if amount_str.is_empty() {
        return Err("请输入金额");
    }
    
    amount_str.parse::<f64>()
        .map_err(|_| "金额格式错误，请输入有效数字")
}

/// Validate category selection
pub fn validate_category_id(id: i64) -> Result<(), &'static str> {
    if id == 0 {
        Err("请选择分类")
    } else {
        Ok(())
    }
}

/// Validate non-empty string
pub fn validate_not_empty(value: &str, field_name: &'static str) -> Result<(), &'static str> {
    if value.trim().is_empty() {
        Err(field_name)
    } else {
        Ok(())
    }
}

/// Parse positive integer
pub fn parse_positive_int(value: &str) -> Result<i32, &'static str> {
    match value.parse::<i32>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err("请输入有效的正整数"),
    }
}
