use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use chrono::Datelike;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

// ============================================================================
// Data Types
// ============================================================================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MonthlySummary {
    pub year: i32,
    pub month: i32,
    pub total_income: f64,
    pub total_expense: f64,
    pub net_amount: f64,
    pub transaction_count: i32,
}

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

// ============================================================================
// Main App Component
// ============================================================================

#[component]
pub fn App() -> impl IntoView {
    // Categories state
    let (categories, set_categories) = create_signal(Vec::<Category>::new());

    // Current view: "categories", "transactions", "installments"
    let (current_view, set_current_view) = create_signal("transactions".to_string());

    // Selected month for transaction view - use current date
    let now = chrono::Local::now();
    let current_year = now.year();
    let current_month = now.month();

    let (selected_year, set_selected_year) = create_signal(current_year);
    let (selected_month, set_selected_month) = create_signal(current_month as i32);

    // Load categories on mount
    let load_categories = {
        let set_categories = set_categories.clone();
        move || {
            let set_categories = set_categories.clone();
            spawn_local(async move {
                let result = invoke("get_categories", JsValue::NULL).await;
                if let Ok(cats) = serde_wasm_bindgen::from_value::<Vec<Category>>(result) {
                    set_categories.set(cats);
                }
            });
        }
    };

    // Load categories on mount
    create_effect(move |_| {
        load_categories();
    });

    // Navigation
    let show_categories = move |_| set_current_view.set("categories".to_string());
    let show_transactions = move |_| set_current_view.set("transactions".to_string());
    let show_installments = move |_| set_current_view.set("installments".to_string());
    let show_summary = move |_| set_current_view.set("summary".to_string());

    view! {
        <main class="container">
            <div class="header">
                <h1>"简易记账"</h1>
                <nav class="nav">
                    <button
                        class=move || format!("nav-btn {}", if current_view.get() == "transactions" { "active" } else { "" })
                        on:click=show_transactions
                    >
                        "记账"
                    </button>
                    <button
                        class=move || format!("nav-btn {}", if current_view.get() == "categories" { "active" } else { "" })
                        on:click=show_categories
                    >
                        "消费项目"
                    </button>
                    <button
                        class=move || format!("nav-btn {}", if current_view.get() == "installments" { "active" } else { "" })
                        on:click=show_installments
                    >
                        "分期管理"
                    </button>
                    <button
                        class=move || format!("nav-btn {}", if current_view.get() == "summary" { "active" } else { "" })
                        on:click=show_summary
                    >
                        "汇总"
                    </button>
                </nav>
            </div>

            <div class="content">
                <Show when=move || current_view.get() == "categories">
                    <CategoryManager categories=categories set_categories=set_categories />
                </Show>

                <Show when=move || current_view.get() == "transactions">
                    <TransactionView
                        categories=categories
                        selected_year=selected_year
                        selected_month=selected_month
                        set_selected_year=set_selected_year
                        set_selected_month=set_selected_month
                    />
                </Show>

                <Show when=move || current_view.get() == "installments">
                    <InstallmentManager categories=categories />
                </Show>

                <Show when=move || current_view.get() == "summary">
                    <crate::summary::SummaryView categories=categories />
                </Show>
            </div>
        </main>
    }
}

// ============================================================================
// Category Manager Component
// ============================================================================

#[component]
fn CategoryManager(
    categories: ReadSignal<Vec<Category>>,
    set_categories: WriteSignal<Vec<Category>>,
) -> impl IntoView {
    let (show_add_form, set_show_add_form) = create_signal(false);
    let (new_category_name, set_new_category_name) = create_signal(String::new());
    let (new_category_icon, set_new_category_icon) = create_signal(String::new());

    // Available icons (emoji for simplicity)
    let available_icons = vec![
        "🍔", "🍕", "🍜", "🍰", "☕", "🍺", "🥤",
        "🛒", "🏠", "🚗", "✈️", "🎮", "📱", "💻",
        "👕", "👟", "💄", "💊", "🏥", "📚", "✏️",
        "🎬", "🎵", "🎨", "💡", "🔧", "📦", "💰",
    ];

    let add_category = move |_| {
        let name = new_category_name.get();
        let icon = new_category_icon.get();
        let set_categories = set_categories.clone();
        let set_show_add_form = set_show_add_form.clone();
        let set_new_category_name = set_new_category_name.clone();
        let set_new_category_icon = set_new_category_icon.clone();

        if name.is_empty() {
            return;
        }

        // Use default icon if none selected
        let icon_val = if icon.is_empty() {
            Some("📦".to_string())
        } else {
            Some(icon)
        };

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "name": name,
                "icon": icon_val,
            })).unwrap();

            let result = invoke("create_category", args).await;
            if let Ok(_new_cat) = serde_wasm_bindgen::from_value::<Category>(result) {
                // Reload categories from server to ensure consistency
                let reload_result = invoke("get_categories", JsValue::NULL).await;
                if let Ok(cats) = serde_wasm_bindgen::from_value::<Vec<Category>>(reload_result) {
                    set_categories.set(cats);
                }
                set_new_category_name.set(String::new());
                set_new_category_icon.set(String::new());
                set_show_add_form.set(false);
            }
        });
    };

    let delete_category = move |cat_id: i64| {
        let set_categories = set_categories.clone();
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "id": cat_id })).unwrap();

            let _result = invoke("delete_category", args).await;
            // Reload categories from server to ensure consistency
            let reload_result = invoke("get_categories", JsValue::NULL).await;
            if let Ok(cats) = serde_wasm_bindgen::from_value::<Vec<Category>>(reload_result) {
                set_categories.set(cats);
            }
        });
    };

    view! {
        <div class="category-manager">
            <div class="section-header">
                <h2>"消费项目"</h2>
                <button class="btn-primary" on:click=move |_| set_show_add_form.set(!show_add_form.get())>
                    {move || if show_add_form.get() { "取消" } else { "+ 新建项目" }}
                </button>
            </div>

            {move || if show_add_form.get() {
                Some(view! {
                    <div class="add-form">
                        <input
                            type="text"
                            placeholder="项目名称"
                            value=new_category_name
                            on:input=move |ev| set_new_category_name.set(event_target_value(&ev))
                        />
                        <div class="icon-selector">
                            <span>"选择图标:"</span>
                            {available_icons.iter().map(|icon| {
                                let icon = icon.to_string();
                                let is_selected = new_category_icon.get() == icon;
                                let display_icon = icon.clone();
                                view! {
                                    <button
                                        class=move || format!("icon-btn {}", if is_selected { "selected" } else { "" })
                                        on:click=move |_| set_new_category_icon.set(icon.clone())
                                    >
                                        {display_icon}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <button
                            class="btn-primary"
                            on:click=add_category
                        >
                            "保存"
                        </button>
                    </div>
                })
            } else {
                None
            }}

            <div class="category-list">
                <For
                    each=move || categories.get()
                    key=|category| category.id
                    let(category)
                >
                    <div class="category-item">
                        <span class="category-icon">
                            {category.icon.clone().unwrap_or_else(|| "📦".to_string())}
                        </span>
                        <span class="category-name">{category.name}</span>
                        <button class="btn-danger" on:click=move |_| delete_category(category.id)>
                            "删除"
                        </button>
                    </div>
                </For>
            </div>
        </div>
    }
}

// ============================================================================
// Transaction View Component
// ============================================================================

#[component]
fn TransactionView(
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
                    let(tx)
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
                    let(detail)
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

// ============================================================================
// Installment Manager Component
// ============================================================================

#[component]
fn InstallmentManager(
    categories: ReadSignal<Vec<Category>>,
) -> impl IntoView {
    let (installments, set_installments) = create_signal(Vec::<InstallmentWithCategory>::new());
    let (due_installments, set_due_installments) = create_signal(Vec::<InstallmentDetail>::new());
    let (selected_installment, set_selected_installment) = create_signal(None::<i64>);
    let (installment_details, set_installment_details) = create_signal(Vec::<InstallmentDetail>::new());

    // Form state
    let (show_add_form, set_show_add_form) = create_signal(false);
    let (selected_category_id, set_selected_category_id) = create_signal(0i64);
    let (total_amount, set_total_amount) = create_signal(String::new());
    let (installment_count, set_installment_count) = create_signal(3i32);
    let (start_date, set_start_date) = create_signal({
        let now = chrono::Local::now();
        now.format("%Y-%m-%d").to_string()
    });
    let (note, set_note) = create_signal(String::new());

    // Load all installments on mount
    let load_installments = {
        let set_installments = set_installments.clone();
        let set_due_installments = set_due_installments.clone();
        move || {
            let set_installments = set_installments.clone();
            let set_due_installments = set_due_installments.clone();
            spawn_local(async move {
                // Load all installments
                let result = invoke("get_installments", JsValue::NULL).await;
                if let Ok(insts) = serde_wasm_bindgen::from_value::<Vec<InstallmentWithCategory>>(result) {
                    set_installments.set(insts);
                }

                // Load due installments for current month
                let now = chrono::Local::now();
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                    "year": now.year(),
                    "month": now.month(),
                })).unwrap();

                let due_result = invoke("get_due_installments_by_month", args).await;
                if let Ok(due_insts) = serde_wasm_bindgen::from_value::<Vec<InstallmentDetail>>(due_result) {
                    set_due_installments.set(due_insts);
                }
            });
        }
    };

    // Load installments (delayed to avoid startup issues)
    load_installments();

    // Load installment details
    let load_installment_details = move |installment_id: i64| {
        let set_installment_details = set_installment_details.clone();
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "installmentId": installment_id })).unwrap();
            let result = invoke("get_installment_details", args).await;
            if let Ok(details) = serde_wasm_bindgen::from_value::<Vec<InstallmentDetail>>(result) {
                set_installment_details.set(details);
            }
        });
    };

    // Add installment
    let add_installment = move |_| {
        let cat_id = selected_category_id.get();
        let amt_str = total_amount.get();
        let count = installment_count.get();
        let dt = start_date.get();
        let nt = note.get();
        let set_total_amount = set_total_amount.clone();
        let set_note = set_note.clone();
        let set_show_add_form = set_show_add_form.clone();

        if cat_id == 0 || amt_str.is_empty() {
            return;
        }

        let amt: f64 = match amt_str.parse() {
            Ok(a) => a,
            Err(_) => {
                eprintln!("Invalid amount");
                return;
            }
        };

        let note_val = if nt.is_empty() { None } else { Some(nt) };

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "categoryId": cat_id,
                "totalAmount": amt,
                "installmentCount": count,
                "startDate": dt,
                "note": note_val,
            })).unwrap();

            let _result = invoke("create_installment", args).await;

            // Reload installments
            let result = invoke("get_installments", JsValue::NULL).await;
            if let Ok(insts) = serde_wasm_bindgen::from_value::<Vec<InstallmentWithCategory>>(result) {
                set_installments.set(insts);
            }

            // Reset form
            set_total_amount.set(String::new());
            set_note.set(String::new());
            set_show_add_form.set(false);
        });
    };

    // Mark installment as paid
    let mark_paid = move |detail_id: i64| {
        let set_installment_details = set_installment_details.clone();
        spawn_local(async move {
            let now = chrono::Local::now();
            let paid_date = now.format("%Y-%m-%d").to_string();

            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "detailId": detail_id,
                "paidDate": paid_date,
            })).unwrap();

            let _result = invoke("mark_installment_paid", args).await;

            // Reload details
            let installment_id = selected_installment.get().unwrap_or(0);
            if installment_id > 0 {
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "installmentId": installment_id })).unwrap();
                let result = invoke("get_installment_details", args).await;
                if let Ok(details) = serde_wasm_bindgen::from_value::<Vec<InstallmentDetail>>(result) {
                    set_installment_details.set(details);
                }
            }

            // Reload due installments
            let now = chrono::Local::now();
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "year": now.year(),
                "month": now.month(),
            })).unwrap();

            let due_result = invoke("get_due_installments_by_month", args).await;
            if let Ok(due_insts) = serde_wasm_bindgen::from_value::<Vec<InstallmentDetail>>(due_result) {
                set_due_installments.set(due_insts);
            }
        });
    };

    // Delete installment
    let delete_installment = move |inst_id: i64| {
        let set_installments = set_installments.clone();
        let set_installment_details = set_installment_details.clone();
        let set_selected_installment = set_selected_installment.clone();
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "id": inst_id })).unwrap();
            let _result = invoke("delete_installment", args).await;

            let result = invoke("get_installments", JsValue::NULL).await;
            if let Ok(insts) = serde_wasm_bindgen::from_value::<Vec<InstallmentWithCategory>>(result) {
                set_installments.set(insts);
            }

            set_installment_details.set(Vec::new());
            set_selected_installment.set(None);
        });
    };

    // Show installment details
    let show_details = move |inst_id: i64| {
        set_selected_installment.set(Some(inst_id));
        load_installment_details(inst_id);
    };

    view! {
        <div class="installment-manager">
            <div class="section-header">
                <h2>"分期管理"</h2>
                <button class="btn-primary" on:click=move |_| {
                    let is_showing = show_add_form.get();
                    if !is_showing {
                        // Reset to today's date when opening the form
                        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                        set_start_date.set(today);
                    }
                    set_show_add_form.set(!is_showing);
                }>
                    {move || if show_add_form.get() { "取消" } else { "+ 新建分期" }}
                </button>
            </div>

            {move || if show_add_form.get() {
                Some(view! {
                    <div class="add-form">
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
                            placeholder="总金额"
                            value=total_amount
                            on:input=move |ev| set_total_amount.set(event_target_value(&ev))
                        />
                        <div class="input-group">
                            <label>"分期期数:"</label>
                            <select
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    set_installment_count.set(val.parse().unwrap_or(3));
                                }
                            >
                                <option value="3">"3期"</option>
                                <option value="6">"6期"</option>
                                <option value="12">"12期"</option>
                                <option value="24">"24期"</option>
                            </select>
                        </div>
                        <input
                            type="date"
                            value=start_date
                            on:input=move |ev| set_start_date.set(event_target_value(&ev))
                        />
                        <input
                            type="text"
                            placeholder="备注（可选）"
                            value=note
                            on:input=move |ev| set_note.set(event_target_value(&ev))
                        />
                        <button
                            class="btn-primary"
                            on:click=add_installment
                        >
                            "保存"
                        </button>
                    </div>
                })
            } else {
                None
            }}

            // Installment list
            <div class="installment-list">
                <For
                    each=move || installments.get()
                    key=|inst| inst.id
                    let(installment)
                >
                    {
                        let inst_id = installment.id;
                        let monthly_amount = installment.total_amount / installment.installment_count as f64;
                        view! {
                            <div class="installment-item">
                                <span class="installment-icon">
                                    {installment.category_icon.clone().unwrap_or_else(|| "📦".to_string())}
                                </span>
                                <div class="installment-info">
                                    <span class="installment-category">{installment.category_name}</span>
                                    <span class="installment-dates">
                                        {format!("{}起 · {}期", installment.start_date, installment.installment_count)}
                                    </span>
                                </div>
                                <div class="installment-amount">
                                    <span class="total-amount">{format!("{:.2}", installment.total_amount)}</span>
                                    <span class="monthly-amount">{format!("每期 {:.2}", monthly_amount)}</span>
                                </div>
                                <button
                                    class="btn-delete"
                                    on:click=move |_| delete_installment(inst_id)
                                >
                                    "删除"
                                </button>
                            </div>
                        }
                    }
                </For>
            </div>
        </div>
    }
}
