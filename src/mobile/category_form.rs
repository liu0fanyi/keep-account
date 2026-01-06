//! Mobile category form component.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::shared::{COMMON_ICONS, DEFAULT_ICON, create_category};

/// 移动端新增消费类型表单
#[component]
pub fn MobileCategoryForm(
    on_success: impl Fn() + 'static + Copy,
    on_cancel: impl Fn() + 'static + Copy,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let icon = RwSignal::new(DEFAULT_ICON.to_string());
    let error_message = RwSignal::new(String::new());
    
    // 提交逻辑
    let do_submit = move || {
        error_message.set(String::new());
        
        let name_val = name.get();
        if name_val.is_empty() {
            error_message.set("请输入类型名称".to_string());
            return;
        }
        
        let icon_val = icon.get();
        
        spawn_local(async move {
            let _ = create_category(&name_val, &icon_val).await;
            on_success();
        });
    };
    
    let submit = move |_| do_submit();
    
    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            ev.prevent_default();
            do_submit();
        }
    };
    
    view! {
        <div style="display: flex; flex-direction: column; height: 100vh; background: #f8f9fa;">
            // 顶部header
            <div style="display: flex; align-items: center; padding: 12px 16px; background: white; border-bottom: 1px solid #e0e0e0;">
                <button on:click=move |_| on_cancel()
                    style="padding: 8px; background: none; border: none; font-size: 24px; cursor: pointer;">
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
                } else { None }
            }}
            
            // 表单内容
            <div style="flex: 1; overflow-y: auto; padding: 16px;">
                <div style="margin-bottom: 20px;">
                    <label style="display: block; margin-bottom: 8px; font-weight: 500;">"类型名称"</label>
                    <input type="text" placeholder="例如：早餐、交通"
                        prop:value=move || name.get()
                        on:input=move |ev| name.set(event_target_value(&ev))
                        on:keydown=handle_keydown
                        style="width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 16px;" />
                </div>
                
                <div style="margin-bottom: 20px;">
                    <label style="display: block; margin-bottom: 8px; font-weight: 500;">"选择图标"</label>
                    <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px;">
                        {COMMON_ICONS.iter().take(16).map(|&ic| {
                            view! {
                                <button on:click=move |_| icon.set(ic.to_string())
                                    style=move || format!(
                                        "padding: 16px; font-size: 32px; border-radius: 8px; border: 2px solid {}; background: white;",
                                        if icon.get() == ic { "#3b82f6" } else { "#ddd" }
                                    )>
                                    {ic}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </div>
            
            // 底部按钮
            <div style="padding: 16px; background: white; border-top: 1px solid #e0e0e0;">
                <button on:click=submit
                    style="width: 100%; padding: 14px; background: #3b82f6; color: white; border: none; border-radius: 8px; font-size: 16px; font-weight: bold;">
                    "保存"
                </button>
            </div>
        </div>
    }
}
