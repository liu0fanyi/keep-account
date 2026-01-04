use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

// Import types from app module
use crate::app::{Category, TransactionWithCategory};

#[derive(Clone, Copy, PartialEq)]
enum MobileView {
    List,            // 交易列表
    Form,            // 记账表单
    Categories,      // 消费项目管理
    CategoryForm,    // 新增消费类型表单
    Installments,    // 分期管理
    InstallmentForm, // 新增分期表单
    Summary,         // 月度汇总
}

/// 移动端记账组件
#[component]
pub fn MobileTransactionView(
    categories: ReadSignal<Vec<Category>>,
    selected_year: ReadSignal<i32>,
    selected_month: ReadSignal<i32>,
) -> impl IntoView {
    // 当前视图：列表或表单
    let current_view = RwSignal::new(MobileView::List);
    
    // 交易列表
    let transactions = RwSignal::new(Vec::<TransactionWithCategory>::new());
    
    // 加载交易列表
    let load_transactions = move || {
        let year = selected_year.get_untracked();
        let month = selected_month.get_untracked();
        
        spawn_local(async move {
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
    
    // 初始加载
    create_effect(move |_| {
        let _year = selected_year.get();
        let _month = selected_month.get();
        load_transactions();
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
                            <div style="flex: 1; overflow-y: auto;">
                                <div style="padding: 16px;">
                                    <h2 style="margin: 0 0 16px 0; font-size: 20px;">"消费类型"</h2>
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
                        <div style="display: flex; flex-direction: column; height: 100vh; position: relative;">
                            <div style="flex: 1; overflow-y: auto;">
                                <div style="padding: 16px;">
                                    <h2 style="margin: 0 0 16px 0; font-size: 20px;">"分期管理"</h2>
                                    <div style="padding: 40px 20px; text-align: center; color: #7f8c8d;">
                                        "暂无分期记录"
                                    </div>
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
                            <div style="flex: 1; overflow-y: auto; padding: 20px; text-align: center;">
                                <h2>"月度汇总"</h2>
                                <p style="color: #7f8c8d; margin-top: 20px;">"请在桌面版查看汇总"</p>
                            </div>
                            <MobileBottomNav current_view=current_view />
                        </div>
                    </Show>
                    <Show when=move || view_type == MobileView::CategoryForm fallback=|| ()>
                        <div style="height: 100vh;">
                            <MobileCategoryForm
                                on_success=move || current_view.set(MobileView::Categories)
                                on_cancel=move || current_view.set(MobileView::Categories)
                            />
                        </div>
                    </Show>
                    <Show when=move || view_type == MobileView::InstallmentForm fallback=|| ()>
                        <div style="height: 100vh;">
                            <MobileInstallmentForm
                                on_success=move || current_view.set(MobileView::Installments)
                                on_cancel=move || current_view.set(MobileView::Installments)
                            />
                        </div>
                    </Show>
                }
            }}
        </div>
    }
}

/// 底部导航栏
#[component]
fn MobileBottomNav(
    current_view: RwSignal<MobileView>,
) -> impl IntoView {
    view! {
        <div class="mobile-bottom-nav">
            <button
                class=move || if current_view.get() == MobileView::List { "mobile-nav-item active" } else { "mobile-nav-item" }
                on:click=move |_| current_view.set(MobileView::List)
            >
                <div class="mobile-nav-icon">"📝"</div>
                <div class="mobile-nav-label">"记账"</div>
            </button>
            
            <button
                class=move || if current_view.get() == MobileView::Categories { "mobile-nav-item active" } else { "mobile-nav-item" }
                on:click=move |_| current_view.set(MobileView::Categories)
            >
                <div class="mobile-nav-icon">"📂"</div>
                <div class="mobile-nav-label">"项目"</div>
            </button>
            
            <button
                class=move || if current_view.get() == MobileView::Installments { "mobile-nav-item active" } else { "mobile-nav-item" }
                on:click=move |_| current_view.set(MobileView::Installments)
            >
                <div class="mobile-nav-icon">"💳"</div>
                <div class="mobile-nav-label">"分期"</div>
            </button>
            
            <button
                class=move || if current_view.get() == MobileView::Summary { "mobile-nav-item active" } else { "mobile-nav-item" }
                on:click=move |_| current_view.set(MobileView::Summary)
            >
                <div class="mobile-nav-icon">"📊"</div>
                <div class="mobile-nav-label">"汇总"</div>
            </button>
        </div>
    }
}

/// 移动端交易列表
#[component]
fn MobileTransactionList(
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

/// 移动端记账表单
#[component]
fn MobileTransactionForm(
    categories: ReadSignal<Vec<Category>>,
    selected_year: ReadSignal<i32>,
    selected_month: ReadSignal<i32>,
    on_success: impl Fn() + 'static + Copy,
    on_cancel: impl Fn() + 'static + Copy,
) -> impl IntoView {
    // 选中的分类
    let selected_category_id = RwSignal::new(0i64);
    
    // 金额输入（作为字符串保存，便于处理小数点输入）
    let amount_display = RwSignal::new(String::from("0"));
    
    // 是否为支出（true=支出，false=收入）
    let is_expense = RwSignal::new(true);
    
    // 备注
    let note = RwSignal::new(String::new());
    
    // 错误信息
    let error_message = RwSignal::new(String::new());
    
    // 成功信息
    let success_message = RwSignal::new(String::new());

    // 处理数字键盘输入
    let handle_number_input = move |num: &str| {
        let current = amount_display.get();
        
        match num {
            "." => {
                // 只允许一个小数点
                if !current.contains('.') {
                    let new_val = if current == "0" {
                        "0.".to_string()
                    } else {
                        format!("{}{}", current, num)
                    };
                    amount_display.set(new_val);
                }
            }
            "⌫" => {
                // 退格键
                if current.len() > 1 {
                    let new_val = current[..current.len() - 1].to_string();
                    amount_display.set(new_val);
                } else {
                    amount_display.set("0".to_string());
                }
            }
            _ => {
                // 数字键
                let new_val = if current == "0" {
                    num.to_string()
                } else {
                    format!("{}{}", current, num)
                };
                amount_display.set(new_val);
            }
        }
    };

    // 提交记账
    let submit_transaction = move |_| {
        error_message.set(String::new());
        success_message.set(String::new());
        
        let cat_id = selected_category_id.get();
        let amount_str = amount_display.get();
        let note_val = note.get();
        
        // 验证：必须选择分类
        if cat_id == 0 {
            error_message.set("请选择消费类型".to_string());
            return;
        }
        
        // 验证：金额必须有效
        let amount: f64 = match amount_str.parse::<f64>() {
            Ok(a) if a != 0.0 => {
                // 如果是支出，金额为负数
                if is_expense.get() { -a } else { a }
            },
            _ => {
                error_message.set("请输入有效金额".to_string());
                return;
            }
        };
        
        // 获取当前日期
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let note_value = if note_val.is_empty() { None } else { Some(note_val.clone()) };
        
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "categoryId": cat_id,
                "amount": amount,
                "transactionDate": today,
                "note": note_value,
            })).unwrap();
            
            let result = invoke("create_transaction", args).await;
            
            // 检查是否有错误
            if let Some(error) = result.as_string() {
                if error.contains("Error") || error.contains("error") {
                    error_message.set(format!("保存失败: {}", error));
                    return;
                }
            }
            
            // 成功：显示消息并在短暂延迟后切换视图
            success_message.set("记账成功！".to_string());
            
            // 延迟后调用成功回调
            set_timeout(
                move || {
                    on_success();
                },
                std::time::Duration::from_millis(800),
            );
        });
    };

    view! {
        <div class="mobile-form-view">
            // 顶部：标题和取消按钮
            <div class="mobile-form-header">
                <button 
                    class="mobile-form-cancel"
                    on:click=move |_| on_cancel()
                >
                    "←"
                </button>
                <h2>"新建记账"</h2>
                <div class="mobile-form-spacer"></div>
            </div>
            
            // 顶部：消息提示
            {move || {
                let error = error_message.get();
                let success = success_message.get();
                
                if !error.is_empty() {
                    Some(view! {
                        <div class="mobile-message mobile-error">
                            {error}
                        </div>
                    })
                } else if !success.is_empty() {
                    Some(view! {
                        <div class="mobile-message mobile-success">
                            {success}
                        </div>
                    })
                } else {
                    None
                }
            }}
            
            // 分类图标选择区（顶部）
            <div class="mobile-section mobile-icon-section" style="padding: 8px; flex: 1; overflow-y: auto;">
                <div class="mobile-section-title" style="font-size: 14px; margin-bottom: 6px;">"选择类型"</div>
                <div class="mobile-icon-grid" style="display: grid; grid-template-columns: repeat(5, 1fr); gap: 6px;">
                    <For
                        each=move || categories.get()
                        key=|cat| cat.id
                        let:category
                    >
                        <button
                            class=move || {
                                if selected_category_id.get() == category.id {
                                    "mobile-icon-btn mobile-icon-selected"
                                } else {
                                    "mobile-icon-btn"
                                }
                            }
                            on:click=move |_| selected_category_id.set(category.id)
                            style="padding: 6px; border: 1px solid #ddd; border-radius: 8px; background: white; font-size: 11px; display: flex; flex-direction: column; align-items: center; gap: 2px; min-height: 0;"
                        >
                            <div class="mobile-icon-emoji" style="font-size: 24px;">
                                {category.icon.clone().unwrap_or_else(|| "📦".to_string())}
                            </div>
                            <div class="mobile-icon-label" style="font-size: 10px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; width: 100%;">
                                {category.name.clone()}
                            </div>
                        </button>
                    </For>
                </div>
            </div>
            
            // 金额显示区和备注在同一行，添加+/-切换
            <div style="display: flex; gap: 8px; padding: 10px; background: #f8f9fa; border-top: 1px solid #e0e0e0;">
                // +/- 切换按钮
                <button
                    style=move || format!(
                        "min-width: 50px; padding: 8px; border-radius: 8px; font-size: 20px; font-weight: bold; border: 2px solid; {}",
                        if is_expense.get() {
                            "background: #fff5f5; color: #e53e3e; border-color: #fc8181;"
                        } else {
                            "background: #f0fff4; color: #38a169; border-color: #68d391;"
                        }
                    )
                    on:click=move |_| is_expense.set(!is_expense.get())
                >
                    {move || if is_expense.get() { "-" } else { "+" }}
                </button>
                
                // 金额显示
                <div style="flex: 1; display: flex; align-items: center; background: white; padding: 8px 12px; border-radius: 8px; border: 1px solid #ddd;">
                    <span style="font-size: 18px; font-weight: bold;">
                        "¥ " {move || amount_display.get()}
                    </span>
                </div>
                
                // 备注输入
                <input
                    type="text"
                    placeholder="备注"
                    value=note
                    on:input=move |ev| note.set(event_target_value(&ev))
                    style="flex: 1; padding: 8px 12px; border-radius: 8px; border: 1px solid #ddd; font-size: 14px;"
                />
            </div>
            
            // 确认按钮
            <div style="padding: 8px;">
                <button 
                    style="width: 100%; padding: 12px; background: #3b82f6; color: white; border: none; border-radius: 8px; font-size: 16px; font-weight: bold;"
                    on:click=submit_transaction
                >
                    "✓ 确认记账"
                </button>
            </div>
            
            // 数字键盘区（底部）- 缩小尺寸
            <div style="padding: 8px; background: #f8f9fa; border-top: 1px solid #e0e0e0;">
                <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px;">
                    {["7", "8", "9"].iter().map(|num| {
                        let num_str = num.to_string();
                        view! {
                            <button 
                                on:click=move |_| handle_number_input(&num_str)
                                style="padding: 12px; background: white; border: 1px solid #ddd; border-radius: 8px; font-size: 18px; font-weight: 500;"
                            >
                                {num_str.clone()}
                            </button>
                        }
                    }).collect_view()}
                    
                    {["4", "5", "6"].iter().map(|num| {
                        let num_str = num.to_string();
                        view! {
                            <button 
                                on:click=move |_| handle_number_input(&num_str)
                                style="padding: 12px; background: white; border: 1px solid #ddd; border-radius: 8px; font-size: 18px; font-weight: 500;"
                            >
                                {num_str.clone()}
                            </button>
                        }
                    }).collect_view()}
                    
                    {["1", "2", "3"].iter().map(|num| {
                        let num_str = num.to_string();
                        view! {
                            <button 
                                on:click=move |_| handle_number_input(&num_str)
                                style="padding: 12px; background: white; border: 1px solid #ddd; border-radius: 8px; font-size: 18px; font-weight: 500;"
                            >
                                {num_str.clone()}
                            </button>
                        }
                    }).collect_view()}
                    
                    <button 
                        on:click=move |_| handle_number_input(".")
                        style="padding: 12px; background: white; border: 1px solid #ddd; border-radius: 8px; font-size: 18px; font-weight: 500;"
                    >
                        "."
                    </button>
                    <button 
                        on:click=move |_| handle_number_input("0")
                        style="padding: 12px; background: white; border: 1px solid #ddd; border-radius: 8px; font-size: 18px; font-weight: 500;"
                    >
                        "0"
                    </button>
                    <button 
                        on:click=move |_| handle_number_input("⌫")
                        style="padding: 12px; background: #fff5f5; color: #e53e3e; border: 1px solid #fc8181; border-radius: 8px; font-size: 18px;"
                    >
                        "⌫"
                    </button>
                </div>
            </div>
        </div>
    }
}

/// 移动端新增消费类型表单
#[component]
fn MobileCategoryForm(
    on_success: impl Fn() + 'static + Copy,
    on_cancel: impl Fn() + 'static + Copy,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let icon = RwSignal::new(String::from("📦"));
    let error_message = RwSignal::new(String::new());
    
    // 常用图标
    let common_icons = vec![
        "🍔", "🍕", "🍜", "☕", "🚗", "🚌", "🏠", "💡", 
        "📱", "👔", "🎮", "📚", "💊", "🎬", "✈️", "🛒"
    ];
    
    let submit = move |_| {
        error_message.set(String::new());
        
        let name_val = name.get();
        if name_val.is_empty() {
            error_message.set("请输入类型名称".to_string());
            return;
        }
        
        let icon_val = icon.get();
        
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "name": name_val,
                "icon": icon_val,
            })).unwrap();
            
            let _result = invoke("create_category", args).await;
            on_success();
        });
    };
    
    view! {
        <div style="display: flex; flex-direction: column; height: 100vh; background: #f8f9fa;">
            // 顶部header
            <div style="display: flex; align-items: center; padding: 12px 16px; background: white; border-bottom: 1px solid #e0e0e0;">
                <button 
                    on:click=move |_| on_cancel()
                    style="padding: 8px; background: none; border: none; font-size: 24px; cursor: pointer;"
                >
                    "←"
                </button>
                <h2 style="flex: 1; margin: 0; font-size: 18px; text-align: center;">"新增消费类型"</h2>
                <div style="width: 40px;"></div>
            </div>
            
            // 错误提示
            {move || {
                let error = error_message.get();
                if !error.is_empty() {
                    Some(view! {
                        <div style="padding: 12px; background: #fee; color: #c00; margin: 8px; border-radius: 8px;">
                            {error}
                        </div>
                    })
                } else {
                    None
                }
            }}
            
            // 表单内容
            <div style="flex: 1; overflow-y: auto; padding: 16px;">
                // 名称输入
                <div style="margin-bottom: 20px;">
                    <label style="display: block; margin-bottom: 8px; font-weight: 500;">"类型名称"</label>
                    <input
                        type="text"
                        placeholder="例如：早餐、交通"
                        value=name
                        on:input=move |ev| name.set(event_target_value(&ev))
                        style="width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 16px;"
                    />
                </div>
                
                // 图标选择
                <div style="margin-bottom: 20px;">
                    <label style="display: block; margin-bottom: 8px; font-weight: 500;">"选择图标"</label>
                    <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px;">
                        <For
                            each=move || common_icons.clone()
                            key=|ic| ic.to_string()
                            let:ic
                        >
                            <button
                                on:click=move |_| icon.set(ic.to_string())
                                style=move || format!(
                                    "padding: 16px; font-size: 32px; border-radius: 8px; border: 2px solid {}; background: white;",
                                    if icon.get() == ic { "#3b82f6" } else { "#ddd" }
                                )
                            >
                                {ic}
                            </button>
                        </For>
                    </div>
                </div>
            </div>
            
            // 底部按钮
            <div style="padding: 16px; background: white; border-top: 1px solid #e0e0e0;">
                <button 
                    on:click=submit
                    style="width: 100%; padding: 14px; background: #3b82f6; color: white; border: none; border-radius: 8px; font-size: 16px; font-weight: bold;"
                >
                    "保存"
                </button>
            </div>
        </div>
    }
}

/// 移动端新增分期表单
#[component]
fn MobileInstallmentForm(
    on_success: impl Fn() + 'static + Copy,
    on_cancel: impl Fn() + 'static + Copy,
) -> impl IntoView {
    let item_name = RwSignal::new(String::new());
    let total_amount = RwSignal::new(String::new());
    let periods = RwSignal::new(String::from("12"));
    let error_message = RwSignal::new(String::new());
    
    let submit = move |_| {
        error_message.set(String::new());
        
        let name_val = item_name.get();
        if name_val.is_empty() {
            error_message.set("请输入分期项目名称".to_string());
            return;
        }
        
        let amount_val: f64 = match total_amount.get().parse() {
            Ok(a) if a > 0.0 => a,
            _ => {
                error_message.set("请输入有效的总金额".to_string());
                return;
            }
        };
        
        let periods_val: i32 = match periods.get().parse() {
            Ok(p) if p > 0 => p,
            _ => {
                error_message.set("请输入有效的期数".to_string());
                return;
            }
        };
        
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "itemName": name_val,
                "totalAmount": amount_val,
                "periods": periods_val,
            })).unwrap();
            
            let _result = invoke("create_installment", args).await;
            on_success();
        });
    };
    
    view! {
        <div style="display: flex; flex-direction: column; height: 100vh; background: #f8f9fa;">
            // 顶部header
            <div style="display: flex; align-items: center; padding: 12px 16px; background: white; border-bottom: 1px solid #e0e0e0;">
                <button 
                    on:click=move |_| on_cancel()
                    style="padding: 8px; background: none; border: none; font-size: 24px; cursor: pointer;"
                >
                    "←"
                </button>
                <h2 style="flex: 1; margin: 0; font-size: 18px; text-align: center;">"新增分期"</h2>
                <div style="width: 40px;"></div>
            </div>
            
            // 错误提示
            {move || {
                let error = error_message.get();
                if !error.is_empty() {
                    Some(view! {
                        <div style="padding: 12px; background: #fee; color: #c00; margin: 8px; border-radius: 8px;">
                            {error}
                        </div>
                    })
                } else {
                    None
                }
            }}
            
            // 表单内容
            <div style="flex: 1; overflow-y: auto; padding: 16px;">
                // 项目名称
                <div style="margin-bottom: 20px;">
                    <label style="display: block; margin-bottom: 8px; font-weight: 500;">"分期项目"</label>
                    <input
                        type="text"
                        placeholder="例如：手机、电脑"
                        value=item_name
                        on:input=move |ev| item_name.set(event_target_value(&ev))
                        style="width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 16px;"
                    />
                </div>
                
                // 总金额
                <div style="margin-bottom: 20px;">
                    <label style="display: block; margin-bottom: 8px; font-weight: 500;">"总金额"</label>
                    <input
                        type="number"
                        placeholder="0.00"
                        value=total_amount
                        on:input=move |ev| total_amount.set(event_target_value(&ev))
                        style="width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 16px;"
                    />
                </div>
                
                // 分期期数
                <div style="margin-bottom: 20px;">
                    <label style="display: block; margin-bottom: 8px; font-weight: 500;">"分期期数"</label>
                    <select
                        prop:value=periods
                        on:change=move |ev| periods.set(event_target_value(&ev))
                        style="width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 16px;"
                    >
                        <option value="3">"3期"</option>
                        <option value="6">"6期"</option>
                        <option value="12" selected>"12期"</option>
                        <option value="24">"24期"</option>
                        <option value="36">"36期"</option>
                    </select>
                </div>
                
                // 每期金额预览
                <div style="padding: 16px; background: #e3f2fd; border-radius: 8px;">
                    <div style="color: #1976d2; font-size: 14px; margin-bottom: 4px;">"每期还款"</div>
                    <div style="font-size: 24px; font-weight: bold; color: #1565c0;">
                        {move || {
                            let amount: f64 = total_amount.get().parse().unwrap_or(0.0);
                            let p: i32 = periods.get().parse().unwrap_or(1);
                            format!("¥ {:.2}", amount / p as f64)
                        }}
                    </div>
                </div>
            </div>
            
            // 底部按钮
            <div style="padding: 16px; background: white; border-top: 1px solid #e0e0e0;">
                <button 
                    on:click=submit
                    style="width: 100%; padding: 14px; background: #3b82f6; color: white; border: none; border-radius: 8px; font-size: 16px; font-weight: bold;"
                >
                    "创建分期"
                </button>
            </div>
        </div>
    }
}
