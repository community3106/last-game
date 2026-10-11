//! 公開鍵ホワイトリスト + 有料会員リスト
//! - ADMIN: ビルド時 env `ADMIN_PUBKEYS` + 実行時 localStorage + Nostr kind:30030 等で同期する想定
//! - PAID: Nostr kind:30078 (parameterized replaceable) から同期。ここでは localStorage モックで再現。

use leptos::*;
use once_cell::sync::Lazy;
use std::collections::HashSet;

// ビルド時に埋め込む管理者リスト (カンマ区切り hex)。なければデモ鍵を入れる。
static ADMIN_DEFAULT: Lazy<HashSet<String>> = Lazy::new(|| {
    // 本来は option_env!("ADMIN_PUBKEYS") で埋め込む
    // デモ用に固定のダミーを1件
    let mut s = HashSet::new();
    // この hex はデモ用。実運用では `wrangler pages secret put ADMIN_PUBKEYS` で注入
    s.insert("ede8d8c3b2e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3".to_string());
    s.insert("f".repeat(64));
    s
});

/// Leptos global signals (localStorage と同期)
pub static ADMIN_LIST: Lazy<RwSignal<Vec<String>>> = Lazy::new(|| {
    let stored = load_list("admin_list");
    let mut init = ADMIN_DEFAULT.iter().cloned().collect::<Vec<_>>();
    for k in stored { if !init.contains(&k) { init.push(k); } }
    create_rw_signal(init)
});

pub static PAID_LIST: Lazy<RwSignal<Vec<String>>> = Lazy::new(|| {
    let stored = load_list("paid_list");
    create_rw_signal(stored)
});

fn load_list(key: &str) -> Vec<String> {
    if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            if let Ok(Some(v)) = storage.get_item(key) {
                if let Ok(list) = serde_json::from_str::<Vec<String>>(&v) {
                    return list;
                }
            }
        }
    }
    vec![]
}
fn save_list(key: &str, list: &[String]) {
    if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.set_item(key, &serde_json::to_string(list).unwrap_or_default());
        }
    }
}

// ---- 判定 ----
pub fn is_admin_hex(hex: &str) -> bool {
    let h = hex.to_lowercase();
    ADMIN_LIST.get().iter().any(|x| x.to_lowercase() == h) || ADMIN_DEFAULT.contains(&h)
}
pub fn is_paid_hex(hex: &str) -> bool {
    let h = hex.to_lowercase();
    PAID_LIST.get().iter().any(|x| x.to_lowercase() == h)
}

// ---- モック操作 (開発・デモ用) ----
pub fn mock_add_admin(hex: &str) {
    let h = hex.to_lowercase();
    ADMIN_LIST.update(|l| if !l.iter().any(|x| x.to_lowercase()==h) { l.push(h.clone()); });
    save_list("admin_list", &ADMIN_LIST.get());
}
pub fn mock_add_paid(hex: String) {
    let h = hex.to_lowercase();
    PAID_LIST.update(|l| if !l.iter().any(|x| x.to_lowercase()==h) { l.push(h.clone()); });
    save_list("paid_list", &PAID_LIST.get());
}
pub fn mock_clear_paid() {
    PAID_LIST.set(vec![]);
    save_list("paid_list", &[]);
}
pub fn mock_set_paid(list: Vec<String>) {
    PAID_LIST.set(list.clone());
    save_list("paid_list", &list);
}
