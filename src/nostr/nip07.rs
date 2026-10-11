//! NIP-07 Browser Extension Interface (window.nostr)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nip07Provider {
    pub is_available: bool,
    pub extension_name: String,
}

impl Default for Nip07Provider {
    fn default() -> Self {
        Self {
            is_available: true,
            extension_name: "Alby / nos2x / Nostr Extension".to_string(),
        }
    }
}

impl Nip07Provider {
    /// Detects if window.nostr is present in WASM environment
    pub fn detect() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(nostr_val) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("nostr")) {
                    return !nostr_val.is_undefined() && !nostr_val.is_null();
                }
            }
        }
        false
    }
}
