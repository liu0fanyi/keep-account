//! Mobile transaction list component.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::types::TransactionWithCategory;
use crate::api::invoke;
/// 移动端交易列表
#[component]
pub fn MobileTransactionList(
    transactions: RwSignal<Vec<TransactionWithCategory>>,
    selected_year: ReadSignal<i32>,
    selected_month: ReadSignal<i32>,
) -> impl IntoView {
    let delete_transaction = move |tx_id: i64| {
        let year = selected_year.get_untracked();
        let month = selected_month.get_untracked();
        
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "id": tx_id
            })).unwrap();
            
            let _result = invoke("delete_transaction", args).await;
            
            // Reload transactions after delete
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "year": year,
                "month": month,
            })).unwrap();
            
            let result = invoke("get_transactions_by_month", args).await;
            if let Ok(txs) = serde_wasm_bindgen::from_value::<Vec<TransactionWithCategory>>(result) {
                transactions.set(txs);
            }
        });
    };
    
    view! {
        <div class="mobile-list-view">
            <div class="mobile-list-header">
                <h2>{move || format!("{}年{:02}月", selected_year.get(), selected_month.get())}</h2>
            </div>
            
            <div class="mobile-list-content">
                <Show
                    when=move || !transactions.get().is_empty()
                    fallback=|| view! {
                        <div class="mobile-empty-state">
                            <div class="mobile-empty-icon">"📝"</div>
                            <div class="mobile-empty-text">"暂无记账记录"</div>
                            <div class="mobile-empty-hint">"点击右下角 + 按钮开始记账"</div>
                        </div>
                    }
                >
                    <For
                        each=move || transactions.get()
                        key=|tx| tx.id
                        let:tx
                    >
                        <div class="mobile-transaction-item">
                            <div class="mobile-tx-icon">
                                {tx.category_icon.clone().unwrap_or_else(|| "📦".to_string())}
                            </div>
                            <div class="mobile-tx-info">
                                <div class="mobile-tx-category">{tx.category_name.clone()}</div>
                                {tx.note.clone().map(|n| view! {
                                    <div class="mobile-tx-note">{n}</div>
                                })}
                                <div class="mobile-tx-date">{tx.transaction_date.clone()}</div>
                            </div>
                            <div class=move || {
                                if tx.amount >= 0.0 {
                                    "mobile-tx-amount positive"
                                } else {
                                    "mobile-tx-amount negative"
                                }
                            }>
                                {format!("{:+.2}", tx.amount)}
                            </div>
                            <button
                                class="mobile-tx-delete"
                                on:click=move |_| delete_transaction(tx.id)
                            >
                                "×"
                            </button>
                        </div>
                    </For>
                </Show>
            </div>
        </div>
    }
}
