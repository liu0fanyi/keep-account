use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::app::{Category, TransactionWithCategory};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}


#[derive(Clone, Debug)]
struct MonthGroup {
    year: i32,
    month: i32,
    transactions: Vec<TransactionWithCategory>,
    income: f64,
    expense: f64,
}

#[component]
pub fn SummaryView(
    categories: ReadSignal<Vec<Category>>,
) -> impl IntoView {
    let (all_transactions, set_all_transactions) = create_signal(Vec::<TransactionWithCategory>::new());
    let (grouped_by_month, set_grouped_by_month) = create_signal(Vec::<MonthGroup>::new());
    let (total_income, set_total_income) = create_signal(0.0);
    let (total_expense, set_total_expense) = create_signal(0.0);
    let (total_balance, set_total_balance) = create_signal(0.0);

    // Load all transactions
    let load_all_transactions = {
        let set_all_transactions = set_all_transactions.clone();
        let set_grouped_by_month = set_grouped_by_month.clone();
        let set_total_income = set_total_income.clone();
        let set_total_expense = set_total_expense.clone();
        let set_total_balance = set_total_balance.clone();

        move || {
            let set_all_transactions = set_all_transactions.clone();
            let set_grouped_by_month = set_grouped_by_month.clone();
            let set_total_income = set_total_income.clone();
            let set_total_expense = set_total_expense.clone();
            let set_total_balance = set_total_balance.clone();

            spawn_local(async move {
                let result = invoke("get_transactions", JsValue::NULL).await;
                web_sys::console::log_1(&format!("Summary: 收到的数据: {:?}", result).into());

                match serde_wasm_bindgen::from_value::<Vec<TransactionWithCategory>>(result) {
                    Ok(txs) => {
                        web_sys::console::log_1(&format!("Summary: 解析成功，共{}条记录", txs.len()).into());
                        set_all_transactions.set(txs.clone());

                        // Group by month
                    let mut month_map: std::collections::HashMap<(i32, i32), MonthGroup> = std::collections::HashMap::new();
                    let mut total_inc = 0.0;
                    let mut total_exp = 0.0;

                    for tx in txs {
                        // Parse date to get year and month
                        if let Some(date_part) = tx.transaction_date.split('T').next() {
                            let parts: Vec<&str> = date_part.split('-').collect();
                            if parts.len() >= 2 {
                                if let Ok(year) = parts[0].parse::<i32>() {
                                    if let Ok(month) = parts[1].parse::<i32>() {
                                        let key = (year, month);
                                        let group = month_map.entry(key).or_insert(MonthGroup {
                                            year,
                                            month,
                                            transactions: Vec::new(),
                                            income: 0.0,
                                            expense: 0.0,
                                        });

                                        group.transactions.push(tx.clone());

                                        if tx.amount >= 0.0 {
                                            group.income += tx.amount;
                                            total_inc += tx.amount;
                                        } else {
                                            group.expense += tx.amount.abs();
                                            total_exp += tx.amount.abs();
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Convert to sorted vector
                    let mut groups: Vec<MonthGroup> = month_map.into_values().collect();
                    groups.sort_by(|a, b| {
                        if a.year != b.year {
                            b.year.cmp(&a.year)
                        } else {
                            b.month.cmp(&a.month)
                        }
                    });

                    // Sort transactions within each group by date
                    for group in &mut groups {
                        group.transactions.sort_by(|a, b| b.transaction_date.cmp(&a.transaction_date));
                    }

                    set_grouped_by_month.set(groups);
                    set_total_income.set(total_inc);
                    set_total_expense.set(total_exp);
                    set_total_balance.set(total_inc - total_exp);

                    web_sys::console::log_1(&format!("Summary: 总收入={:.2}, 总支出={:.2}, 结余={:.2}", total_inc, total_exp, total_inc - total_exp).into());
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Summary: 解析失败: {:?}", e).into());
                }
                }
            });
        }
    };

    // Load on mount
    create_effect(move |_| {
        load_all_transactions();
    });

    view! {
        <div class="summary-view" style="padding: 16px;">
            <div class="section-header" style="margin: 0 0 16px 0;">
                <h2 style="margin: 0; font-size: 20px;">"账目汇总"</h2>
            </div>

            // Total summary
            <div class="monthly-summary">
                <div class="summary-item">
                    <span>"总收入"</span>
                    <span class="amount-positive">{move || format!("{:.2}", total_income.get())}</span>
                </div>
                <div class="summary-item">
                    <span>"总支出"</span>
                    <span class="amount-negative">{move || format!("{:.2}", total_expense.get())}</span>
                </div>
                <div class="summary-item">
                    <span>"总结余"</span>
                    <span class=move || {
                        if total_balance.get() >= 0.0 { "amount-positive" } else { "amount-negative" }
                    }>
                        {move || format!("{:.2}", total_balance.get())}
                    </span>
                </div>
            </div>

            // Monthly breakdown
            <div class="monthly-breakdown">
                <For
                    each=move || grouped_by_month.get()
                    key=|group| format!("{}-{}", group.year, group.month)
                    let(group)
                >
                    <div class="month-group">
                        <div class="month-header">
                            <h3>{format!("{}年{}月", group.year, group.month)}</h3>
                            <div class="month-totals">
                                <span class="month-income">
                                    {format!("收入: {:.2}", group.income)}
                                </span>
                                <span class="month-expense">
                                    {format!("支出: {:.2}", group.expense)}
                                </span>
                            </div>
                        </div>

                        <div class="transaction-list">
                            <For
                                each=move || group.transactions.clone()
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
                                    <span class="tx-date">{tx.transaction_date.split('T').next().unwrap_or(&tx.transaction_date).to_string()}</span>
                                </div>
                            </For>
                        </div>
                    </div>
                </For>
            </div>
        </div>
    }
}
