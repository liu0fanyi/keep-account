//! Mobile installment form component.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::invoke;
/// 移动端新增分期表单
#[component]
pub fn MobileInstallmentForm(
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
