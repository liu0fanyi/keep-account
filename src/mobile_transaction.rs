use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

// Import Category from app module to avoid duplication
use crate::app::Category;

/// 移动端记账组件
#[component]
pub fn MobileTransactionView(
    categories: ReadSignal<Vec<Category>>,
    selected_year: ReadSignal<i32>,
    selected_month: ReadSignal<i32>,
) -> impl IntoView {
    // 选中的分类
    let selected_category_id = RwSignal::new(0i64);
    
    // 金额输入（作为字符串保存，便于处理小数点输入）
   let amount_display = RwSignal::new(String::from("0"));
    
    // 备注
    let note = RwSignal::new(String::new());
    
    // 是否显示备注输入框
    let show_note_input = RwSignal::new(false);
    
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
        let amount: f64 = match amount_str.parse() {
            Ok(a) if a != 0.0 => a,
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
            
            // 成功：重置表单
            success_message.set("记账成功！".to_string());
            amount_display.set("0".to_string());
            note.set(String::new());
            selected_category_id.set(0);
            show_note_input.set(false);
            
            // 2秒后清除成功消息
            set_timeout(
                move || {
                    success_message.set(String::new());
                },
                std::time::Duration::from_secs(2),
            );
        });
    };

    view! {
        <div class="mobile-transaction-view">
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
            <div class="mobile-section mobile-icon-section">
                <div class="mobile-section-title">"选择消费类型"</div>
                <div class="mobile-icon-grid">
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
                        >
                            <div class="mobile-icon-emoji">
                                {category.icon.clone().unwrap_or_else(|| "📦".to_string())}
                            </div>
                            <div class="mobile-icon-label">
                                {category.name.clone()}
                            </div>
                        </button>
                    </For>
                </div>
            </div>
            
            // 金额显示区
            <div class="mobile-section mobile-display-section">
                <div class="mobile-amount-label">"金额"</div>
                <div class="mobile-amount-display">
                    "¥ " {move || amount_display.get()}
                </div>
            </div>
            
            // 备注输入区
            <div class="mobile-section mobile-note-section">
                <Show
                    when=move || !show_note_input.get()
                    fallback=move || view! {
                        <input
                            type="text"
                            class="mobile-note-input"
                            placeholder="输入备注（可选）"
                            value=note
                            on:input=move |ev| note.set(event_target_value(&ev))
                            on:blur=move |_| {
                                // 如果为空，收起输入框
                                if note.get().is_empty() {
                                    show_note_input.set(false);
                                }
                            }
                        />
                    }
                >
                    <button 
                        class="mobile-note-btn"
                        on:click=move |_| show_note_input.set(true)
                    >
                        {move || {
                            let n = note.get();
                            if n.is_empty() {
                                "📝 点击添加备注".to_string()
                            } else {
                                format!("📝 {}", n)
                            }
                        }}
                    </button>
                </Show>
            </div>
            
            // 确认按钮
            <div class="mobile-section">
                <button 
                    class="mobile-confirm-btn"
                    on:click=submit_transaction
                >
                    "✓ 确认记账"
                </button>
            </div>
            
            // 数字键盘区（底部）
            <div class="mobile-keypad-section">
                <div class="mobile-keypad">
                    {["7", "8", "9"].iter().map(|num| {
                        let num_str = num.to_string();
                        view! {
                            <button 
                                class="mobile-keypad-btn"
                                on:click=move |_| handle_number_input(&num_str)
                            >
                                {num_str.clone()}
                            </button>
                        }
                    }).collect_view()}
                    
                    {["4", "5", "6"].iter().map(|num| {
                        let num_str = num.to_string();
                        view! {
                            <button 
                                class="mobile-keypad-btn"
                                on:click=move |_| handle_number_input(&num_str)
                            >
                                {num_str.clone()}
                            </button>
                        }
                    }).collect_view()}
                    
                    {["1", "2", "3"].iter().map(|num| {
                        let num_str = num.to_string();
                        view! {
                            <button 
                                class="mobile-keypad-btn"
                                on:click=move |_| handle_number_input(&num_str)
                            >
                                {num_str.clone()}
                            </button>
                        }
                    }).collect_view()}
                    
                    <button 
                        class="mobile-keypad-btn"
                        on:click=move |_| handle_number_input(".")
                    >
                        "."
                    </button>
                    <button 
                        class="mobile-keypad-btn mobile-keypad-zero"
                        on:click=move |_| handle_number_input("0")
                    >
                        "0"
                    </button>
                    <button 
                        class="mobile-keypad-btn mobile-keypad-backspace"
                        on:click=move |_| handle_number_input("⌫")
                    >
                        "⌫"
                    </button>
                </div>
            </div>
        </div>
    }
}
