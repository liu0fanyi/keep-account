//! Liquid Container Component - Visual indicator for monthly spending progress
//!
//! Displays an animated liquid-filled container showing how close expenses are to baseline.

use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn LiquidContainer(
    /// Current month's total expenses
    current_expense: ReadSignal<f64>,
) -> impl IntoView {
    // Baseline setting (monthly minimum consumption target)
    let (baseline, set_baseline) = create_signal(None::<f64>);
    let (input_value, set_input_value) = create_signal(String::new());
    let (show_input, set_show_input) = create_signal(false);

    // Load baseline on mount
    create_effect(move |_| {
        spawn_local(async move {
            web_sys::console::log_1(&"Fetching baseline...".into());
            match crate::api::invoke_safe("get_baseline", wasm_bindgen::JsValue::NULL).await {
                Ok(result) => {
                    web_sys::console::log_1(&format!("get_baseline result: {:?}", result).into());
                    match serde_wasm_bindgen::from_value::<Option<f64>>(result) {
                        Ok(value) => {
                            web_sys::console::log_1(&format!("Parsed baseline value: {:?}", value).into());
                            set_baseline.set(value);
                            set_show_input.set(value.is_none());
                        }
                        Err(e) => {
                            web_sys::console::error_1(&format!("Failed to parse baseline: {:?}", e).into());
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to fetch baseline: {:?}", e).into());
                }
            }
        });
    });
    
    // Reload baseline when database initialization completes (after cloud sync)
    create_effect(move |_| {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;
        
        let load_baseline_fn = move || {
            spawn_local(async move {
                web_sys::console::log_1(&"[db-initialized event] Fetching baseline...".into());
                match crate::api::invoke_safe("get_baseline", wasm_bindgen::JsValue::NULL).await {
                    Ok(result) => {
                        web_sys::console::log_1(&format!("[db-initialized event] get_baseline result: {:?}", result).into());
                        match serde_wasm_bindgen::from_value::<Option<f64>>(result) {
                            Ok(value) => {
                                web_sys::console::log_1(&format!("[db-initialized event] Parsed baseline value: {:?}", value).into());
                                set_baseline.set(value);
                                set_show_input.set(value.is_none());
                            }
                            Err(e) => {
                                web_sys::console::error_1(&format!("[db-initialized event] Failed to parse baseline: {:?}", e).into());
                            }
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("[db-initialized event] Failed to fetch baseline: {:?}", e).into());
                    }
                }
            });
        };
        
        let callback = Closure::wrap(Box::new(move |_event: web_sys::CustomEvent| {
            web_sys::console::log_1(&"db-initialized event received!".into());
            load_baseline_fn();
        }) as Box<dyn FnMut(web_sys::CustomEvent)>);
        
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "db-initialized",
                callback.as_ref().unchecked_ref()
            );
        }
        
        // Keep callback alive
        callback.forget();
    });


    // Save baseline
    let save_baseline = move || {
        let value = input_value.get_untracked();
        web_sys::console::log_1(&format!("Saving baseline: {}", value).into());
        if let Ok(baseline_val) = value.parse::<f64>() {
            spawn_local(async move {
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                    "baseline": baseline_val
                })).unwrap();
                
                web_sys::console::log_1(&format!("Calling set_baseline with value: {}", baseline_val).into());
                match crate::api::invoke_safe("set_baseline", args).await {
                    Ok(_) => {
                        web_sys::console::log_1(&"set_baseline succeeded".into());
                        set_baseline.set(Some(baseline_val));
                        set_show_input.set(false);
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("set_baseline failed: {:?}", e).into());
                    }
                }
            });
        }
    };

    view! {
        <div class="liquid-container-wrapper" style="width: 100%; height: 100vh; display: flex; flex-direction: column; align-items: center; justify-content: center; background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%); padding: 20px; box-sizing: border-box;">
            <Show
                when=move || show_input.get()
                fallback=move || {
                    let expense = current_expense.get();
                    let base = baseline.get().unwrap_or(1.0);
                    let percentage = ((expense / base) * 100.0).min(150.0); // Cap at 150% for overflow
                    let is_overflow = percentage > 100.0;
                    
                    // Color interpolation from blue to red based on percentage
                    let (r, g, b) = if percentage <= 100.0 {
                        // Blue (#6B9BD1) to Red (#D16B6B)
                        let t = percentage / 100.0;
                        let r = (107.0 + (209.0 - 107.0) * t) as u8;
                        let g = (155.0 + (107.0 - 155.0) * t) as u8;
                        let b = (209.0 + (107.0 - 209.0) * t) as u8;
                        (r, g, b)
                    } else {
                        (209, 107, 107) // Red for overflow
                    };
                    
                    let fill_height = if is_overflow {
                        100.0 // Fill completely
                    } else {
                        percentage
                    };
                    
                    view! {
                        <div style="width: 100%; max-width: 400px; text-align: center;">
                            <h3 style="margin: 0 0 20px 0; color: #2c3e50; font-size: 24px; font-weight: 600;">
                                "本月消费进度"
                            </h3>
                            
                            <div style="position: relative; width: 250px; height: 400px; margin: 0 auto;">
                                // SVG Container
                                <svg width="250" height="400" style="position: absolute; top: 0; left: 0;">
                                    <defs>
                                        // Gradient for liquid
                                        <linearGradient id="liquidGradient" x1="0%" y1="100%" x2="0%" y2="0%">
                                            <stop offset="0%" style=format!("stop-color:rgb({},{},{}); stop-opacity:0.8", r, g, b) />
                                            <stop offset="100%" style=format!("stop-color:rgb({},{},{}); stop-opacity:0.6", r, g, b) />
                                        </linearGradient>
                                        
                                        // Clip path for container shape
                                        <clipPath id="containerClip">
                                            <rect x="50" y="50" width="150" height="300" rx="10" />
                                        </clipPath>
                                    </defs>
                                    
                                    // Container outline
                                    <rect x="50" y="50" width="150" height="300" rx="10" 
                                        fill="none" stroke="#34495e" stroke-width="3" />
                                    
                                    // Baseline marker
                                    <line x1="45" y1="350" x2="205" y2="350" 
                                        stroke="#e74c3c" stroke-width="2" stroke-dasharray="5,5" />
                                    <text x="210" y="355" fill="#e74c3c" font-size="12" font-weight="bold">"底线"</text>
                                    
                                    // Liquid fill with animation
                                    <rect 
                                        x="50" 
                                        y=format!("{}", 350.0 - (fill_height * 3.0))
                                        width="150" 
                                        height=format!("{}", fill_height * 3.0)
                                        fill="url(#liquidGradient)"
                                        clip-path="url(#containerClip)"
                                        class="liquid-fill"
                                    />
                                    
                                    // Overflow animation (if applicable)
                                    {if is_overflow {
                                        let overflow_amount = percentage - 100.0;
                                        Some(view! {
                                            <g class="overflow-drops">
                                                <circle cx="125" cy="45" r="5" fill=format!("rgb({},{},{})", r, g, b) opacity="0.7" class="drop drop-1" />
                                                <circle cx="110" cy="40" r="4" fill=format!("rgb({},{},{})", r, g, b) opacity="0.6" class="drop drop-2" />
                                                <circle cx="140" cy="42" r="4" fill=format!("rgb({},{},{})", r, g, b) opacity="0.6" class="drop drop-3" />
                                            </g>
                                        })
                                    } else {
                                        None
                                    }}
                                </svg>
                            </div>
                            
                            <div style="margin-top: 30px; padding: 20px; background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);">
                                <div style="display: flex; justify-content: space-between; margin-bottom: 10px;">
                                    <span style="color: #7f8c8d; font-size: 14px;">"当前消费:"</span>
                                    <span style="color: #2c3e50; font-weight: 600; font-size: 16px;">
                                        {format!("¥{:.2}", expense)}
                                    </span>
                                </div>
                                <div style="display: flex; justify-content: space-between; margin-bottom: 10px;">
                                    <span style="color: #7f8c8d; font-size: 14px;">"底线消费:"</span>
                                    <span style="color: #e74c3c; font-weight: 600; font-size: 16px;">
                                        {format!("¥{:.2}", base)}
                                    </span>
                                </div>
                                <div style="display: flex; justify-content: space-between;">
                                    <span style="color: #7f8c8d; font-size: 14px;">"进度:"</span>
                                    <span style=move || {
                                        if is_overflow {
                                            "color: #e74c3c; font-weight: 700; font-size: 18px;"
                                        } else if percentage > 80.0 {
                                            "color: #f39c12; font-weight: 600; font-size: 16px;"
                                        } else {
                                            "color: #27ae60; font-weight: 600; font-size: 16px;"
                                        }
                                    }>
                                        {format!("{:.1}%", percentage)}
                                        {if is_overflow { " (已超出!)" } else { "" }}
                                    </span>
                                </div>
                                
                                <button 
                                    on:click=move |_| set_show_input.set(true)
                                    style="margin-top: 15px; padding: 8px 16px; background: #3498db; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 14px; width: 100%;"
                                >
                                    "修改底线"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                }
            >
                <div style="width: 100%; max-width: 400px; text-align: center;">
                    <h3 style="margin: 0 0 20px 0; color: #2c3e50; font-size: 24px; font-weight: 600;">
                        "设置月度底线消费"
                    </h3>
                    
                    <div style="background: white; padding: 30px; border-radius: 12px; box-shadow: 0 4px 12px rgba(0,0,0,0.1);">
                        <p style="color: #7f8c8d; margin-bottom: 20px; font-size: 14px; line-height: 1.6;">
                            "设置每月最低消费目标，液体容器会根据您的消费进度动态调整高度和颜色。"
                        </p>
                        
                        <input
                            type="number"
                            placeholder="例如: 5000"
                            value=move || input_value.get()
                            on:input=move |ev| {
                                set_input_value.set(event_target_value(&ev));
                            }
                            style="width: 100%; padding: 12px; margin-bottom: 15px; border: 2px solid #bdc3c7; border-radius: 8px; font-size: 16px; box-sizing: border-box;"
                        />
                        
                        <button
                            on:click=move |_| save_baseline()
                            style="width: 100%; padding: 12px; background: #27ae60; color: white; border: none; border-radius: 8px; cursor: pointer; font-size: 16px; font-weight: 600;"
                        >
                            "保存底线"
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}
