//! Mobile transaction form component.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::types::Category;
use crate::shared::{create_transaction, validate_category_id, DEFAULT_ICON};
/// 移动端记账表单
#[component]
pub fn MobileTransactionForm(
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
        
        // 验证分类
        if let Err(e) = validate_category_id(cat_id) {
            error_message.set(e.to_string());
            return;
        }
        
        // 验证和处理金额
        let amount: f64 = match amount_str.parse::<f64>() {
            Ok(a) if a != 0.0 => if is_expense.get() { -a } else { a },
            _ => {
                error_message.set("请输入有效金额".to_string());
                return;
            }
        };
        
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let note_value = if note_val.is_empty() { None } else { Some(note_val) };
        
        spawn_local(async move {
            if let Err(e) = create_transaction(cat_id, amount, &today, note_value).await {
                error_message.set(format!("保存失败: {}", e));
                return;
            }
            
            success_message.set("记账成功！".to_string());
            set_timeout(move || on_success(), std::time::Duration::from_millis(800));
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
