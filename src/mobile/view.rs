//! Mobile Transaction View - main container for mobile views.

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::types::{Category, TransactionWithCategory, InstallmentWithCategory};
use crate::api::JsValue;

pub use super::nav::{MobileView, MobileBottomNav};
pub use super::list::MobileTransactionList;
pub use super::form::MobileTransactionForm;
pub use super::category_form::MobileCategoryForm;
pub use super::installment_form::MobileInstallmentForm;
pub use super::sync_settings::SyncSettingsForm;
/// 移动端记账组件
#[component]
pub fn MobileTransactionView(
    categories: ReadSignal<Vec<Category>>,
    set_categories: WriteSignal<Vec<Category>>,
    selected_year: ReadSignal<i32>,
    selected_month: ReadSignal<i32>,
    set_selected_year: WriteSignal<i32>,
    set_selected_month: WriteSignal<i32>,
) -> impl IntoView {
    // 当前视图：列表或表单
    let current_view = RwSignal::new(MobileView::List);
    
    // 交易列表
    let transactions = RwSignal::new(Vec::<TransactionWithCategory>::new());
    
    // 分期列表
    let installments = RwSignal::new(Vec::<InstallmentWithCategory>::new());
    
    // 加载分类列表
    let load_categories = move || {
        spawn_local(async move {
            if let Ok(result) = crate::api::invoke_safe("get_categories", JsValue::NULL).await {
                if let Ok(cats) = serde_wasm_bindgen::from_value::<Vec<Category>>(result) {
                    set_categories.set(cats);
                }
            }
        });
    };
    
    // 加载交易列表
    let load_transactions = move || {
        let year = selected_year.get_untracked();
        let month = selected_month.get_untracked();
        
        spawn_local(async move {

            let load_fn = move || {
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                    "year": year,
                    "month": month,
                })).unwrap();
                spawn_local(async move {
                    if let Ok(result) = crate::api::invoke_safe("get_transactions_by_month", args).await {
                        if let Ok(txs) = serde_wasm_bindgen::from_value::<Vec<TransactionWithCategory>>(result) {
                            transactions.set(txs);
                        }
                    }
                });
            };
            
            // Initial load
            load_fn();
            
            // Listen
             let _ = crate::api::listen_safe("db-initialized", move |_| {
                load_fn();
            }).await;
        });
    };
    
    // 加载分期列表
    let load_installments = move || {
        spawn_local(async move {
            let load_fn = move || {
                spawn_local(async move {
                    if let Ok(result) = crate::api::invoke_safe("get_installments", JsValue::NULL).await {
                        if let Ok(items) = serde_wasm_bindgen::from_value::<Vec<InstallmentWithCategory>>(result) {
                            installments.set(items);
                        }
                    }
                });
            };
            
            load_fn();
            
            let _ = crate::api::listen_safe("db-initialized", move |_| {
                load_fn();
            }).await;
        });
    };
    
    // 初始加载
    create_effect(move |_| {
        let _year = selected_year.get();
        let _month = selected_month.get();
        load_transactions();
    });
    
    // Android 返回键处理：使用浏览器历史 API
    // 当进入表单视图时推入历史状态，返回键触发 popstate 事件时导航回上一视图
    create_effect(move |prev_view: Option<MobileView>| {
        let view = current_view.get();
        
        // 如果从非表单视图切换到表单视图，推入历史状态
        if let Some(prev) = prev_view {
            let is_entering_form = matches!(view, MobileView::Form | MobileView::CategoryForm | MobileView::InstallmentForm)
                && !matches!(prev, MobileView::Form | MobileView::CategoryForm | MobileView::InstallmentForm);
            
            if is_entering_form {
                if let Some(window) = web_sys::window() {
                    if let Ok(history) = window.history() {
                        let _ = history.push_state_with_url(&JsValue::NULL, "", None);
                    }
                }
            }
        }
        
        view
    });
    
    // 监听 popstate 事件（返回键触发）
    create_effect(move |_| {
        use wasm_bindgen::closure::Closure;
        
        let current_view = current_view.clone();
        
        let closure = Closure::wrap(Box::new(move |_: web_sys::PopStateEvent| {
            let view = current_view.get_untracked();
            
            // 根据当前视图决定返回到哪里
            match view {
                MobileView::Form => current_view.set(MobileView::List),
                MobileView::CategoryForm => current_view.set(MobileView::Categories),
                MobileView::InstallmentForm => current_view.set(MobileView::Installments),
                _ => {
                    // 主视图时允许默认行为（退出应用）
                    // 但需要补回历史状态以保持一致性
                }
            }
        }) as Box<dyn FnMut(_)>);
        
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref());
        }
        
        // 防止闭包被释放
        closure.forget();
    });

    view! {
        <div class="mobile-transaction-view">
            {move || {
                let view_type = current_view.get();
                view! {
                    <Show when=move || view_type == MobileView::Form fallback=|| ()>
                        <div style="height: 100vh;">
                            <MobileTransactionForm
                                categories=categories
                                selected_year=selected_year
                                selected_month=selected_month
                                on_success=move || {
                                    current_view.set(MobileView::List);
                                    load_transactions();
                                }
                                on_cancel=move || current_view.set(MobileView::List)
                            />
                        </div>
                    </Show>
                    <Show when=move || view_type == MobileView::List fallback=|| ()>
                        <div style="display: flex; flex-direction: column; height: 100vh; position: relative;">
                            <div style="flex: 1; overflow: hidden;">
                                <MobileTransactionList
                                    transactions=transactions
                                    selected_year=selected_year
                                    selected_month=selected_month
                                    set_selected_year=set_selected_year
                                    set_selected_month=set_selected_month
                                />
                            </div>
                            <MobileBottomNav current_view=current_view />
                            <button
                                class="mobile-fab"
                                on:click=move |_| current_view.set(MobileView::Form)
                                style="position: fixed; bottom: 80px; right: 20px; width: 56px; height: 56px; border-radius: 28px; background: #3b82f6; color: white; border: none; font-size: 28px; box-shadow: 0 4px 12px rgba(0,0,0,0.3); z-index: 100;"
                            >
                                "+"
                            </button>
                        </div>
                    </Show>
                    <Show when=move || view_type == MobileView::Categories fallback=|| ()>
                        <div style="display: flex; flex-direction: column; height: 100vh; position: relative;">
                            <h2 style="margin: 0; font-size: 18px; padding: 12px 16px; background: white; border-bottom: 1px solid #e0e0e0; flex-shrink: 0;">"消费类型"</h2>
                            <div style="flex: 1; overflow-y: auto;">
                                <div style="padding: 16px;">
                                    <For
                                        each=move || categories.get()
                                        key=|cat| cat.id
                                        let:category
                                    >
                                        <div style="padding: 12px; margin-bottom: 8px; background: white; border-radius: 8px; border: 1px solid #e0e0e0; display: flex; align-items: center; gap: 12px;">
                                            <div style="font-size: 32px;">
                                                {category.icon.clone().unwrap_or_else(|| "📦".to_string())}
                                            </div>
                                            <div style="flex: 1;">
                                                <div style="font-weight: 500; font-size: 16px;">{category.name.clone()}</div>
                                            </div>
                                            <button
                                                on:click={
                                                    let cat_id = category.id;
                                                    move |_| {
                                                        spawn_local(async move {
                                                            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                                                                "id": cat_id
                                                            })).unwrap();
                                                            let _ = crate::api::invoke_safe("delete_category", args).await;
                                                            // Reload categories
                                                            if let Ok(result) = crate::api::invoke_safe("get_categories", JsValue::NULL).await {
                                                                if let Ok(cats) = serde_wasm_bindgen::from_value::<Vec<Category>>(result) {
                                                                    set_categories.set(cats);
                                                                }
                                                            }
                                                        });
                                                    }
                                                }
                                                style="width: 32px; height: 32px; border-radius: 50%; background: #fee; color: #e74c3c; border: none; font-size: 18px; cursor: pointer; display: flex; align-items: center; justify-content: center;"
                                            >
                                                "×"
                                            </button>
                                        </div>
                                    </For>
                                </div>
                            </div>
                            <MobileBottomNav current_view=current_view />
                            <button
                                on:click=move |_| current_view.set(MobileView::CategoryForm)
                                style="position: fixed; bottom: 80px; right: 20px; width: 56px; height: 56px; border-radius: 28px; background: #3b82f6; color: white; border: none; font-size: 28px; box-shadow: 0 4px 12px rgba(0,0,0,0.3); z-index: 100;"
                            >
                                "+"
                            </button>
                        </div>
                    </Show>

                    <Show when=move || view_type == MobileView::Installments fallback=|| ()>
                        {load_installments();}
                        <div style="display: flex; flex-direction: column; height: 100vh; position: relative;">
                            <h2 style="margin: 0; font-size: 18px; padding: 12px 16px; background: white; border-bottom: 1px solid #e0e0e0; flex-shrink: 0;">"分期管理"</h2>
                            <div style="flex: 1; overflow-y: auto;">
                                <div style="padding: 16px;">
                                    {move || {
                                        let items = installments.get();
                                        if items.is_empty() {
                                            view! {
                                                <div style="padding: 40px 20px; text-align: center; color: #7f8c8d;">
                                                    "暂无分期记录"
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div>
                                                    <For
                                                        each=move || installments.get()
                                                        key=|item| item.id
                                                        let:item
                                                    >
                                                        <div style="padding: 12px; margin-bottom: 8px; background: white; border-radius: 8px; border: 1px solid #e0e0e0;">
                                                            <div style="display: flex; align-items: center; gap: 12px;">
                                                                <div style="font-size: 32px;">
                                                                    {item.category_icon.clone().unwrap_or_else(|| "📦".to_string())}
                                                                </div>
                                                                <div style="flex: 1;">
                                                                    <div style="font-weight: 500; font-size: 16px;">
                                                                        {item.category_name.clone()}
                                                                    </div>
                                                                    <div style="font-size: 12px; color: #666;">
                                                                        {format!("{}期 · 开始于 {}", item.installment_count, item.start_date)}
                                                                    </div>
                                                                    {item.note.clone().map(|n| view! {
                                                                        <div style="font-size: 12px; color: #888;">{n}</div>
                                                                    })}
                                                                </div>
                                                                <div style="text-align: right;">
                                                                    <div style="font-weight: bold; color: #e53e3e;">
                                                                        {format!("¥{:.2}", item.total_amount)}
                                                                    </div>
                                                                    <div style="font-size: 12px; color: #666;">
                                                                        {format!("每期 ¥{:.2}", item.total_amount / item.installment_count as f64)}
                                                                    </div>
                                                                </div>
                                                                <button
                                                                    on:click={
                                                                        let item_id = item.id;
                                                                        move |_| {
                                                                            spawn_local(async move {
                                                                                let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                                                                                    "id": item_id
                                                                                })).unwrap();
                                                                                let _ = crate::api::invoke_safe("delete_installment", args).await;
                                                                                // Reload installments
                                                                                if let Ok(result) = crate::api::invoke_safe("get_installments", JsValue::NULL).await {
                                                                                    if let Ok(items) = serde_wasm_bindgen::from_value::<Vec<InstallmentWithCategory>>(result) {
                                                                                        installments.set(items);
                                                                                    }
                                                                                }
                                                                            });
                                                                        }
                                                                    }
                                                                    style="width: 32px; height: 32px; border-radius: 50%; background: #fee; color: #e74c3c; border: none; font-size: 18px; cursor: pointer; display: flex; align-items: center; justify-content: center; margin-left: 8px;"
                                                                >
                                                                    "×"
                                                                </button>
                                                            </div>
                                                        </div>
                                                    </For>
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                            </div>
                            <MobileBottomNav current_view=current_view />
                            <button
                                on:click=move |_| current_view.set(MobileView::InstallmentForm)
                                style="position: fixed; bottom: 80px; right: 20px; width: 56px; height: 56px; border-radius: 28px; background: #3b82f6; color: white; border: none; font-size: 28px; box-shadow: 0 4px 12px rgba(0,0,0,0.3); z-index: 100;"
                            >
                                "+"
                            </button>
                        </div>
                    </Show>
                    <Show when=move || view_type == MobileView::Summary fallback=|| ()>
                        <div style="display: flex; flex-direction: column; height: 100vh;">
                            <div style="flex: 1; overflow-y: auto;">
                                <crate::summary::SummaryView categories=categories />
                            </div>
                            <MobileBottomNav current_view=current_view />
                        </div>
                    </Show>

                    <Show when=move || view_type == MobileView::CategoryForm fallback=|| ()>
                        <div style="height: 100vh;">
                            <MobileCategoryForm
                                on_success=move || {
                                    load_categories();
                                    current_view.set(MobileView::Categories);
                                }
                                on_cancel=move || current_view.set(MobileView::Categories)
                            />
                        </div>
                    </Show>

                    <Show when=move || view_type == MobileView::InstallmentForm fallback=|| ()>
                        <div style="height: 100vh;">
                            <MobileInstallmentForm
                                categories=categories
                                on_success=move || current_view.set(MobileView::Installments)
                                on_cancel=move || current_view.set(MobileView::Installments)
                            />
                        </div>
                    </Show>

                    <Show when=move || view_type == MobileView::Settings fallback=|| ()>
                        <div style="display: flex; flex-direction: column; height: 100vh;">
                            <div style="flex: 1; overflow: hidden;">
                                <SyncSettingsForm
                                    on_back=move || current_view.set(MobileView::List)
                                />
                            </div>
                            <MobileBottomNav current_view=current_view />
                        </div>
                    </Show>
                }
            }}
        </div>
    }
}
