//! Mobile installment form component.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::invoke;
use crate::types::Category;

/// 移动端新增分期表单 - 与记账表单对齐的UI
#[component]
pub fn MobileInstallmentForm(
    categories: ReadSignal<Vec<Category>>,
    on_success: impl Fn() + 'static + Copy,
    on_cancel: impl Fn() + 'static + Copy,
) -> impl IntoView {
    // 选中的分类
    let selected_category_id = RwSignal::new(0i64);
    
    // 金额输入（作为字符串保存，便于处理小数点输入）
    let amount_display = RwSignal::new(String::from("0"));
    
    // 分期期数
    let periods = RwSignal::new(12i32);
    
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

    // 提交分期
    let submit = move |_| {
        error_message.set(String::new());
        success_message.set(String::new());
        
        let cat_id = selected_category_id.get();
        if cat_id == 0 {
            error_message.set("请选择消费类型".to_string());
            return;
        }
        
        let amount_str = amount_display.get();
        let amount_val: f64 = match amount_str.parse() {
            Ok(a) if a > 0.0 => a,
            _ => {
                error_message.set("请输入有效的总金额".to_string());
                return;
            }
        };
        
        let periods_val = periods.get();
        let note_val = note.get();
        
        spawn_local(async move {
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "categoryId": cat_id,
                "totalAmount": amount_val,
                "installmentCount": periods_val,
                "startDate": today,
                "note": if note_val.is_empty() { None::<String> } else { Some(note_val) },
            })).unwrap();
            
            let _result = invoke("create_installment", args).await;
            on_success();
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
                <h2>"新建分期"</h2>
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
            
            // 分类图标选择区（从已有分类选择）
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
            
            // 金额显示区和备注
            <div style="display: flex; gap: 8px; padding: 10px; background: #f8f9fa; border-top: 1px solid #e0e0e0;">
                // 金额显示
                <div style="flex: 1; display: flex; align-items: center; background: white; padding: 8px 12px; border-radius: 8px; border: 1px solid #ddd;">
                    <span style="font-size: 18px; font-weight: bold;">
                        "¥ " {move || amount_display.get()}
                    </span>
                </div>
                
                // 备注输入
                <input
                    type="text"
                    placeholder="备注（可选）"
                    value=note
                    on:input=move |ev| note.set(event_target_value(&ev))
                    style="flex: 1; padding: 8px 12px; border-radius: 8px; border: 1px solid #ddd; font-size: 14px;"
                />
            </div>
            
            // 分期期数选择
            <div style="padding: 8px 10px; background: #f8f9fa;">
                <div style="font-size: 12px; color: #666; margin-bottom: 6px;">"分期期数"</div>
                <div style="display: flex; gap: 6px;">
                    {[3, 6, 12, 24, 36].iter().map(|&p| {
                        view! {
                            <button
                                on:click=move |_| periods.set(p)
                                style=move || format!(
                                    "flex: 1; padding: 10px 0; border-radius: 8px; font-size: 14px; font-weight: 500; border: 2px solid; {}",
                                    if periods.get() == p {
                                        "background: #3b82f6; color: white; border-color: #3b82f6;"
                                    } else {
                                        "background: white; color: #333; border-color: #ddd;"
                                    }
                                )
                            >
                                {format!("{}期", p)}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>
            
            // 每期金额预览
            <div style="padding: 8px 10px; background: #e3f2fd; margin: 0 10px; border-radius: 8px;">
                <div style="color: #1976d2; font-size: 12px;">"每期还款"</div>
                <div style="font-size: 20px; font-weight: bold; color: #1565c0;">
                    {move || {
                        let amount: f64 = amount_display.get().parse().unwrap_or(0.0);
                        let p = periods.get();
                        format!("¥ {:.2}", amount / p as f64)
                    }}
                </div>
            </div>
            
            // 确认按钮
            <div style="padding: 8px 10px;">
                <button 
                    style="width: 100%; padding: 12px; background: #3b82f6; color: white; border: none; border-radius: 8px; font-size: 16px; font-weight: bold;"
                    on:click=submit
                >
                    "✓ 创建分期"
                </button>
            </div>
            
            // 数字键盘区（底部）
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
