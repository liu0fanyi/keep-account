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
}

// Re-export commonly used items for convenience
pub use wasm_bindgen::prelude::JsValue;

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
