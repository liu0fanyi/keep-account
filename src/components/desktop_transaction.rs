//! Desktop Transaction View component.

use leptos::prelude::*;
use leptos::task::spawn_local;
use chrono::Datelike;

use crate::types::{Category, TransactionWithCategory, MonthlySummary, InstallmentDetail};
use crate::api::{invoke, JsValue};

#[component]
pub fn DesktopTransactionView(
    categories: ReadSignal<Vec<Category>>,
    selected_year: ReadSignal<i32>,
    selected_month: ReadSignal<i32>,
    set_selected_year: WriteSignal<i32>,
    set_selected_month: WriteSignal<i32>,
) -> impl IntoView {
    let (transactions, set_transactions) = create_signal(Vec::<TransactionWithCategory>::new());
    let (monthly_summary, set_monthly_summary) = create_signal(None::<MonthlySummary>);
    let (installment_details, set_installment_details) = create_signal(Vec::<InstallmentDetail>::new());

    // Form state
    let (show_add_form, set_show_add_form) = create_signal(false);
    let (selected_category_id, set_selected_category_id) = create_signal(0i64);
    let (amount, set_amount) = create_signal(String::new());
    let (note, set_note) = create_signal(String::new());
    let (form_error, set_form_error) = create_signal(String::new());
    let (transaction_date, set_transaction_date) = create_signal({
        let now = chrono::Local::now();
        now.format("%Y-%m-%d").to_string()
    });

    // Load transactions for selected month
    let load_transactions = move || {
        let year = selected_year.get_untracked();
        let month = selected_month.get_untracked();
        let set_transactions = set_transactions.clone();
        let set_monthly_summary = set_monthly_summary.clone();
        let set_installment_details = set_installment_details.clone();

        spawn_local(async move {
            // Load transactions
            let tx_args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "year": year,
                "month": month,
            })).unwrap();

            let tx_result = invoke("get_transactions_by_month", tx_args).await;
            if let Ok(txs) = serde_wasm_bindgen::from_value::<Vec<TransactionWithCategory>>(tx_result) {
                set_transactions.set(txs);
            }

            // Load monthly summary
            let summary_args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "year": year,
                "month": month,
            })).unwrap();

            let summary_result = invoke("get_monthly_summary", summary_args).await;
            if let Ok(summary) = serde_wasm_bindgen::from_value::<MonthlySummary>(summary_result) {
                set_monthly_summary.set(Some(summary));
            }

            // Load installment details for the month
            let installment_args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "year": year,
                "month": month,
            })).unwrap();

            let installment_result = invoke("get_due_installments_by_month", installment_args).await;
            if let Ok(installments) = serde_wasm_bindgen::from_value::<Vec<InstallmentDetail>>(installment_result) {
                set_installment_details.set(installments);
            }
        });
    };

    // Reload when selected month changes
    create_effect(move |_| {
        let _year = selected_year.get();
        let _month = selected_month.get();
        load_transactions();
    });

    // Add transaction
    let add_transaction = move |_| {
        web_sys::console::log_1(&"=== 开始添加交易 ===".into());
        let cat_id = selected_category_id.get();
        let amt_str = amount.get();
        let dt = transaction_date.get();
        let nt = note.get();
        let set_amount = set_amount.clone();
        let set_note = set_note.clone();
        let set_show_add_form = set_show_add_form.clone();
        let selected_year = selected_year.clone();
        let selected_month = selected_month.clone();

        web_sys::console::log_1(&format!("输入数据: category={}, amount={}, date={:?}", cat_id, amt_str, dt).into());

        // Clear previous error
        set_form_error.set(String::new());

        if cat_id == 0 {
            set_form_error.set("请选择分类".to_string());
            web_sys::console::error_1(&"未选择分类".into());
            return;
        }

        if amt_str.is_empty() {
            set_form_error.set("请输入金额".to_string());
            web_sys::console::error_1(&"金额为空".into());
            return;
        }

        let amt: f64 = match amt_str.parse() {
            Ok(a) => a,
            Err(_) => {
                set_form_error.set("金额格式错误，请输入有效数字".to_string());
                web_sys::console::error_1(&"金额格式错误".into());
                eprintln!("Invalid amount");
                return;
            }
        };

        let note_val = if nt.is_empty() { None } else { Some(nt) };

        web_sys::console::log_1(&"准备调用后端API".into());

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "categoryId": cat_id,
                "amount": amt,
                "transactionDate": dt,
                "note": note_val,
            })).unwrap();

            web_sys::console::log_1(&format!("调用create_transaction: {:?}", args).into());

            // Create transaction
            let result = invoke("create_transaction", args).await;

            web_sys::console::log_1(&format!("create_transaction返回: {:?}", result).into());

            // Check if result is an error
            if let Some(error) = result.as_string() {
                if error.contains("Error") || error.contains("error") {
                    let set_form_error = set_form_error.clone();
                    set_form_error.set(format!("保存失败: {}", error));
                    web_sys::console::error_1(&format!("创建交易失败: {}", error).into());
                    return;
                }
            }

            // Reload transactions
            let year = selected_year.get_untracked();
            let month = selected_month.get_untracked();
            web_sys::console::log_1(&format!("准备重新加载交易，年月: {}-{}", year, month).into());

            let tx_args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "year": year,
                "month": month,
            })).unwrap();

            web_sys::console::log_1(&"调用get_transactions_by_month".into());
            let tx_result = invoke("get_transactions_by_month", tx_args).await;
            web_sys::console::log_1(&format!("get_transactions_by_month返回: {:?}", tx_result).into());

            if let Ok(txs) = serde_wasm_bindgen::from_value::<Vec<TransactionWithCategory>>(tx_result) {
                web_sys::console::log_1(&format!("解析成功，共{}条交易记录", txs.len()).into());
                web_sys::console::log_1(&format!("第一笔交易: {:?}", txs.first()).into());
                set_transactions.set(txs);
                web_sys::console::log_1(&"已更新transactions signal".into());
            } else {
                web_sys::console::error_1(&"解析交易列表失败".into());
            }

            // Reload monthly summary
            let summary_args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "year": year,
                "month": month,
            })).unwrap();
            let summary_result = invoke("get_monthly_summary", summary_args).await;
            if let Ok(summary) = serde_wasm_bindgen::from_value::<MonthlySummary>(summary_result) {
                set_monthly_summary.set(Some(summary));
            }

            // Reset form
            set_amount.set(String::new());
            set_note.set(String::new());
            set_form_error.set(String::new());
            set_show_add_form.set(false);
        });
    };

    view! {
        <div class="transaction-view">
            // Month selector
            <div class="month-selector">
                <button
                    on:click=move |_| {
                        let mut m = selected_month.get() - 1;
                        let mut y = selected_year.get();
                        if m < 1 {
                            m = 12;
                            y -= 1;
                        }
                        set_selected_month.set(m);
                        set_selected_year.set(y);
                    }
                >
                    "◀"
                </button>
                <span class="month-display">
                    {move || format!("{}年{:02}月", selected_year.get(), selected_month.get())}
                </span>
                <button
                    on:click=move |_| {
                        let mut m = selected_month.get() + 1;
                        let mut y = selected_year.get();
                        if m > 12 {
                            m = 1;
                            y += 1;
                        }
                        set_selected_month.set(m);
                        set_selected_year.set(y);
                    }
                >
                    "▶"
                </button>
            </div>

            // Monthly summary
            {move || {
                monthly_summary.get().map(|summary| {
                    view! {
                        <div class="monthly-summary">
                            <div class="summary-item income">
                                <span>"收入"</span>
                                <span class="amount-positive">{format!("{:.2}", summary.total_income)}</span>
                            </div>
                            <div class="summary-item expense">
                                <span>"支出"</span>
                                <span class="amount-negative">{format!("{:.2}", summary.total_expense)}</span>
                            </div>
                            <div class="summary-item net">
                                <span>"结余"</span>
                                <span class=move || {
                                    if summary.net_amount >= 0.0 { "amount-positive" } else { "amount-negative" }
                                }>
                                    {format!("{:.2}", summary.net_amount.abs())}
                                </span>
                            </div>
                        </div>
                    }
                })
            }}

            // Add transaction button
            <div class="section-header">
                <h2>"交易记录"</h2>
                <button class="btn-primary" on:click=move |_| {
                    let is_showing = show_add_form.get();
                    if !is_showing {
                        // Reset to today's date when opening the form
                        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                        set_transaction_date.set(today);
                        // Clear any previous error
                        set_form_error.set(String::new());
                    }
                    set_show_add_form.set(!is_showing);
                }>
                    {move || if show_add_form.get() { "取消" } else { "+ 记账" }}
                </button>
            </div>

            {move || if show_add_form.get() {
                Some(view! {
                    <div class="add-form">
                        {move || {
                            let error = form_error.get();
                            if !error.is_empty() {
                                Some(view! {
                                    <div class="error-message" style="background: #fee; color: #c33; padding: 10px; border-radius: 6px; margin-bottom: 12px; border: 1px solid #fcc;">
                                        {error}
                                    </div>
                                })
                            } else {
                                None
                            }
                        }}
                        <div class="icon-selector">
                            {categories.get().into_iter().map(|cat| {
                                let cat_id = cat.id;
                                let is_selected = selected_category_id.get() == cat_id;
                                view! {
                                    <button
                                        class=move || {
                                            if is_selected { "icon-btn selected" } else { "icon-btn" }
                                        }
                                        on:click=move |_| set_selected_category_id.set(cat_id)
                                        title=cat.name.clone()
                                    >
                                        {cat.icon.clone().unwrap_or("".to_string())}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <input
                            type="number"
                            placeholder="金额（正数=收入，负数=支出）"
                            value=amount
                            on:input=move |ev| set_amount.set(event_target_value(&ev))
                        />
                        <input
                            type="date"
                            value=transaction_date
                            on:input=move |ev| set_transaction_date.set(event_target_value(&ev))
                        />
                        <input
                            type="text"
                            placeholder="备注（可选）"
                            value=note
                            on:input=move |ev| set_note.set(event_target_value(&ev))
                        />
                        <button
                            class="btn-primary"
                            on:click=add_transaction
                        >
                            "保存"
                        </button>
                    </div>
                })
            } else {
                None
            }}

            // Transaction list
            <div class="transaction-list">
                <For
                    each=move || transactions.get()
                    key=|tx| tx.id
                    let:tx
                >
                    <div class="transaction-item">
                        <span class="tx-icon">
                            {tx.category_icon.clone().unwrap_or_else(|| "📦".to_string())}
                        </span>
                        <div class="tx-details">
                            <span class="tx-category">{tx.category_name}</span>
                            {tx.note.map(|n| view! { <span class="tx-note">{n}</span> })}
                        </div>
                        <span class=move || {
                            if tx.amount >= 0.0 { "tx-amount-positive" } else { "tx-amount-negative" }
                        }>
                            {format!("{:+.2}", tx.amount)}
                        </span>
                        <span class="tx-date">{tx.transaction_date}</span>
                        <button
                            class="btn-delete"
                            on:click=move |_| {
                                let tx_id = tx.id;
                                let set_transactions = set_transactions.clone();
                                let selected_year = selected_year.clone();
                                let selected_month = selected_month.clone();
                                let set_monthly_summary = set_monthly_summary.clone();

                                spawn_local(async move {
                                    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                                        "id": tx_id
                                    })).unwrap();

                                    let result = invoke("delete_transaction", args).await;

                                    // Check if result is an error
                                    if let Some(error) = result.as_string() {
                                        if error.contains("Error") || error.contains("error") {
                                            return;
                                        }
                                    }

                                    // Reload transactions
                                    let year = selected_year.get_untracked();
                                    let month = selected_month.get_untracked();
                                    let tx_args = serde_wasm_bindgen::to_value(&serde_json::json!({
                                        "year": year,
                                        "month": month,
                                    })).unwrap();

                                    if let Ok(txs) = serde_wasm_bindgen::from_value::<Vec<TransactionWithCategory>>(
                                        invoke("get_transactions_by_month", tx_args).await
                                    ) {
                                        set_transactions.set(txs);
                                    }

                                    // Reload monthly summary
                                    let summary_args = serde_wasm_bindgen::to_value(&serde_json::json!({
                                        "year": year,
                                        "month": month,
                                    })).unwrap();
                                    if let Ok(summary) = serde_wasm_bindgen::from_value::<MonthlySummary>(
                                        invoke("get_monthly_summary", summary_args).await
                                    ) {
                                        set_monthly_summary.set(Some(summary));
                                    }
                                });
                            }
                        >
                            "删除"
                        </button>
                    </div>
                </For>

                // Installment details for this month
                <For
                    each=move || installment_details.get()
                    key=|detail| detail.id
                    let:detail
                >
                    <div class="transaction-item">
                        <span class="tx-icon">
                            "💳"
                        </span>
                        <div class="tx-details">
                            <span class="tx-category">{format!("分期 (第{}期)", detail.sequence_number)}</span>
                            <span class="tx-note">{format!("到期日: {}", detail.due_date)}</span>
                        </div>
                        <span class="tx-amount-negative">
                            {format!("{:.2}", detail.amount)}
                        </span>
                        <span class="tx-date">
                            {move || if detail.is_paid { "已还款" } else { "待还款" }}
                        </span>
                        <span class="tx-note">
                            "(分期)"
                        </span>
                    </div>
                </For>
            </div>
        </div>
    }
}
