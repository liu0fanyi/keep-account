//! Tauri API bindings and helper functions for the keep-accounts application.
//!
//! This module provides the interface to the Rust backend via Tauri's invoke system.

use wasm_bindgen::prelude::*;

// Tauri invoke binding
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

// Re-export commonly used items for convenience
pub use wasm_bindgen::prelude::JsValue;
