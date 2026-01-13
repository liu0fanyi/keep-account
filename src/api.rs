//! Tauri API bindings and helper functions for the keep-accounts application.
//!
//! This module provides the interface to the Rust backend via Tauri's invoke system.

use wasm_bindgen::prelude::*;

// Tauri invoke binding - original version that may panic on error
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

// Tauri invoke binding with catch - returns Result
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = "invoke", catch)]
    async fn invoke_with_catch(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>) -> JsValue;
}

// Re-export commonly used items for convenience
pub use wasm_bindgen::prelude::JsValue;

/// Safe listen wrapper
pub async fn listen_safe<F>(event: &str, handler: F) -> Result<JsValue, String>
where
    F: FnMut(JsValue) + 'static,
{
    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(JsValue)>);
    // We intentionally leak the closure because the event listener needs to live as long as the app
    // or until manual unlistening (not implemented here for simplicity)
    let handler_ref = &closure; 
    let result = listen(event, handler_ref).await;
    closure.forget();
    Ok(result)
}

/// Safe invoke wrapper that converts errors to Result<JsValue, String>
pub async fn invoke_safe(cmd: &str, args: JsValue) -> Result<JsValue, String> {
    match invoke_with_catch(cmd, args).await {
        Ok(val) => Ok(val),
        Err(e) => {
            let err_str = if let Some(s) = e.as_string() {
                s
            } else {
                format!("{:?}", e)
            };
            Err(err_str)
        }
    }
}
