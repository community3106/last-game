//! Nostr Client Context — Leptos global state
//! `nostr-sdk` の Client を WASM で使う本番コードの雛形 + ローカルモックで動作する実装。
//! 実際のリレー接続は `connect()` で行うが、デモでは localStorage とモックで完結する。

use leptos::*;
use gloo_storage::{LocalStorage, Storage};

// Nostr 関連の型を扱うためのヘルパ
// 本番では nostr_sdk::Client を使うが、WASM ビルドの軽量化のためここでは型を抽象化

#[derive(Clone)]
pub struct NostrContext {
    /// hex pubkey (小文字)
    pub pubkey_hex: RwSignal<Option<String>>,
    /// npub (bech32)
    pub npub: RwSignal<Option<String>>,
    /// 接続状態
    pub connected: RwSignal<bool>,
    /// エラーやログ
    pub log: RwSignal<Vec<String>>,
}

impl NostrContext {
    pub fn new() -> Self {
        // localStorage から復元
        let hex = LocalStorage::get::<String>("nostr_pubkey_hex").ok();
        let npub = LocalStorage::get::<String>("nostr_npub").ok();
        let connected = hex.is_some();
        Self {
            pubkey_hex: create_rw_signal(hex),
            npub: create_rw_signal(npub),
            connected: create_rw_signal(connected),
            log: create_rw_signal(vec![]),
        }
    }

    fn push_log(&self, msg: impl Into<String>) {
        let m = msg.into();
        log::info!("[nostr] {}", m);
        self.log.update(|v| {
            v.push(m);
            if v.len() > 50 { v.remove(0); }
        });
    }

    /// nsec / npub / hex のいずれかでログインを試みる
    pub fn try_login(&self, input: &str) -> Result<(), String> {
        let s = input.trim();
        if s.is_empty() { return Err("空です".into()); }

        // 簡易バリデーションと正規化
        // 本番では nostr::Keys::parse や bech32 デコードを使う
        let (hex, npub) = normalize_key(s)?;
        // 保存
        let _ = LocalStorage::set("nostr_pubkey_hex", hex.clone());
        let _ = LocalStorage::set("nostr_npub", npub.clone());
        self.pubkey_hex.set(Some(hex.clone()));
        self.npub.set(Some(npub.clone()));
        self.connected.set(true);
        self.push_log(format!("ログイン成功: {}…", &hex[..8]));
        // リレー接続 (モック)
        self.connect_mock();
        Ok(())
    }

    pub fn logout(&self) {
        LocalStorage::delete("nostr_pubkey_hex");
        LocalStorage::delete("nostr_npub");
        self.pubkey_hex.set(None);
        self.npub.set(None);
        self.connected.set(false);
        self.push_log("ログアウトしました");
    }

    fn connect_mock(&self) {
        // 本番では:
        // let client = nostr_sdk::Client::new(keys);
        // client.add_relay("wss://relay.damus.io").await?;
        // client.connect().await;
        // client.subscribe(...).await;
        self.push_log("リレーに接続中… wss://relay.damus.io, wss://nos.lol (モック)");
        let connected = self.connected;
        let log = self.log;
        // 500ms後に接続完了を擬似
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(600).await;
            connected.set(true);
            log.update(|v| v.push("✓ リレー接続完了 (モック)".into()));
        });
    }

    /// NIP-07 (window.nostr) があれば自動ログインを試みる
    pub fn try_nip07(&self) {
        // JS interop で window.nostr.getPublicKey() を呼ぶ想定
        // ここでは未実装だが、将来的に:
        // let win = web_sys::window().unwrap();
        // let nostr = js_sys::Reflect::get(&win, &"nostr".into()).ok();
        self.push_log("NIP-07 拡張を探索… (未検出なら手動ペーストを使ってください)");
    }

    /// Stripe モック: この公開鍵を有料会員としてマーク
    pub fn mock_mark_paid(&self, hex: &str) {
        crate::nostr::whitelist::mock_add_paid(hex.to_string());
        self.push_log(format!("有料会員に登録: {}… (NIP-44 レベルが復号可能になりました)", &hex[..8]));
    }

    /// ホワイトリスト同期 (本番では kind:30078 を fetch)
    pub fn sync_paid_list(&self) {
        self.push_log("有料会員リストをリレーから同期中… (モック: localStorage)");
    }
}

fn normalize_key(input: &str) -> Result<(String, String), String> {
    // hex (64文字)
    if input.len() == 64 && input.chars().all(|c| c.is_ascii_hexdigit()) {
        let hex = input.to_lowercase();
        let npub = format!("npub1{}", &hex[..8]); // デモでは短縮。実際は bech32 エンコード
        return Ok((hex, npub));
    }
    // npub1...
    if input.starts_with("npub1") {
        // デモ: npub から hex を逆算できないので、ハッシュ的に hex を生成
        // 本番では nostr::PublicKey::from_bech32
        let hex = simple_hash_hex(input);
        return Ok((hex, input.to_string()));
    }
    // nsec1...
    if input.starts_with("nsec1") {
        let hex = simple_hash_hex(input);
        let npub = format!("npub1{}", &hex[..12]);
        return Ok((hex, npub));
    }
    Err("鍵の形式が不正です。hex(64) / npub1... / nsec1... のいずれかを入力してください".into())
}

fn simple_hash_hex(s: &str) -> String {
    // デモ用の決定論的ハッシュ — 本番では使わない
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    // 64文字に拡張
    format!("{:016x}{:016x}{:016x}{:016x}", h, h.wrapping_mul(31), h.wrapping_mul(131), h.wrapping_mul(13131))
}

// NIP-07 helper (将来用)
#[allow(dead_code)]
pub async fn nip07_get_pubkey() -> Result<String, String> {
    // web_sys + wasm_bindgen で window.nostr.getPublicKey() を呼ぶ
    // ここではスタブ
    Err("NIP-07 not available".into())
}
