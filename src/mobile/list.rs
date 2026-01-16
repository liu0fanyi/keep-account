//! Mobile transaction list component.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::types::{TransactionWithCategory, InstallmentDetail};
use crate::shared::{delete_transaction, fetch_transactions, DEFAULT_ICON};
use crate::api::JsValue;
use crate::mobile::LiquidContainer;

/// 移动端交易列表
#[component]
pub fn MobileTransactionList(
    transactions: RwSignal<Vec<TransactionWithCategory>>,
    selected_year: ReadSignal<i32>,
    selected_month: ReadSignal<i32>,
    set_selected_year: WriteSignal<i32>,
    set_selected_month: WriteSignal<i32>,
) -> impl IntoView {
    // Current month's total expense for liquid container
    let current_month_expense = RwSignal::new(0.0);
    // 当月到期分期
    let due_installments = RwSignal::new(Vec::<InstallmentDetail>::new());
    
    // 加载当月到期分期
    let load_due_installments = move || {
        let year = selected_year.get_untracked();
        let month = selected_month.get_untracked();
        
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "year": year,
                "month": month,
            })).unwrap();
            
            if let Ok(result) = crate::api::invoke_safe("get_due_installments_by_month", args).await {
                if let Ok(items) = serde_wasm_bindgen::from_value::<Vec<InstallmentDetail>>(result) {
                    due_installments.set(items);
                }
            }
        });
    };
    
    // 初始加载
    create_effect(move |_| {
        let _year = selected_year.get();
        let _month = selected_month.get();
        load_due_installments();
    });
    
    // Calculate current month's total expense (transactions + installments)
    create_effect(move |_| {
        let txs = transactions.get();
        let installments = due_installments.get();
        
        // Sum of transaction expenses (negative amounts)
        let tx_expense: f64 = txs.iter()
            .filter(|tx| tx.amount < 0.0)
            .map(|tx| tx.amount.abs())
            .sum();
        
        // Sum of installment amounts
        let installment_expense: f64 = installments.iter()
            .map(|i| i.amount)
            .sum();
        
        current_month_expense.set(tx_expense + installment_expense);
    });
    
    let on_delete = move |tx_id: i64| {
        let year = selected_year.get_untracked();
        let month = selected_month.get_untracked();
        
        spawn_local(async move {
            let _ = delete_transaction(tx_id).await;
            if let Ok(txs) = fetch_transactions(year, month).await {
                transactions.set(txs);
            }
        });
    };
    
    // 上一个月
    let prev_month = move |_| {
        let year = selected_year.get_untracked();
        let month = selected_month.get_untracked();
        if month == 1 {
            set_selected_year.set(year - 1);
            set_selected_month.set(12);
        } else {
            set_selected_month.set(month - 1);
        }
    };
    
    // 下一个月
    let next_month = move |_| {
        let year = selected_year.get_untracked();
        let month = selected_month.get_untracked();
        if month == 12 {
            set_selected_year.set(year + 1);
            set_selected_month.set(1);
        } else {
            set_selected_month.set(month + 1);
        }
    };
    
    view! {
        <div style="height: 100vh; overflow-y: auto; -webkit-overflow-scrolling: touch; scroll-snap-type: y proximity;">
            // Liquid Container (占据第一屏)
            <div style="height: 100vh; scroll-snap-align: start;">
                <LiquidContainer current_expense=current_month_expense.read_only() />
            </div>
            
            // Transaction list section (第二屏开始)
            <div class="mobile-list-view" style="min-height: 100vh; background: #f5f5f5; scroll-snap-align: start;">
                <div class="mobile-list-header" style="display: flex; align-items: center; justify-content: space-between; padding: 8px 16px; background: white; box-shadow: 0 2px 4px rgba(0,0,0,0.1); position: sticky; top: 0; z-index: 10;">
                    <button 
                        on:click=prev_month
                        style="background: none; border: none; font-size: 20px; cursor: pointer; padding: 8px;"
                    >
                        "◀"
                    </button>
                    <h2 style="margin: 0; flex: 1; text-align: center;">
                        {move || format!("{}年{:02}月", selected_year.get(), selected_month.get())}
                    </h2>
                    <button 
                        on:click=next_month
                        style="background: none; border: none; font-size: 20px; cursor: pointer; padding: 8px;"
                    >
                        "▶"
                    </button>
                </div>
                
                // 当月分期到期提醒
                {move || {
                    let items = due_installments.get();
                    let total: f64 = items.iter().map(|i| i.amount).sum();
                    if items.is_empty() {
                        None
                    } else {
                        Some(view! {
                            <div style="margin: 8px 16px; padding: 12px; background: #fff3cd; border-radius: 8px; border-left: 4px solid #ffc107;">
                                <div style="font-size: 14px; font-weight: 500; color: #856404;">
                                    {format!("本月分期: {}笔 共 ¥{:.2}", items.len(), total)}
                                </div>
                            </div>
                        })
                    }
                }}
                
                <div class="mobile-list-content" style="padding-bottom: 100px;">
                    <Show when=move || !transactions.get().is_empty()
                        fallback=|| view! {
                            <div class="mobile-empty-state">
                                <div class="mobile-empty-icon">"📝"</div>
                                <div class="mobile-empty-text">"暂无记账记录"</div>
                                <div class="mobile-empty-hint">"点击右下角 + 按钮开始记账"</div>
                            </div>
                        }>
                        <For each=move || transactions.get() key=|tx| tx.id let:tx>
                            <div class="mobile-transaction-item">
                                <div class="mobile-tx-icon">
                                    {tx.category_icon.clone().unwrap_or_else(|| DEFAULT_ICON.to_string())}
                                </div>
                                <div class="mobile-tx-info">
                                    <div class="mobile-tx-category">{tx.category_name.clone()}</div>
                                    {tx.note.clone().map(|n| view! { <div class="mobile-tx-note">{n}</div> })}
                                    <div class="mobile-tx-date">{tx.transaction_date.clone()}</div>
                                </div>
                                <div class=move || {
                                    if tx.amount >= 0.0 { "mobile-tx-amount positive" } else { "mobile-tx-amount negative" }
                                }>
                                    {format!("{:+.2}", tx.amount)}
                                </div>
                                <button class="mobile-tx-delete" on:click=move |_| on_delete(tx.id)>"×"</button>
                            </div>
                        </For>
                    </Show>
                </div>
            </div>
        </div>
    }
}
