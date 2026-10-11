//! NIP-04 / NIP-44 暗号化ラッパ
//! 本番は `nostr-sdk` の `nip04` / `nip44` を直接使う。
//! ここでは WASM ビルドでも動くように、デモ用の擬似実装と、本番用の `#[cfg]` 分岐を併記。

use serde::{Deserialize, Serialize};

// ---- デモ用の可逆擬似暗号 (本番では nostr::nips::nip44 へ置換) ----
fn xor_cipher(input: &str, key: &str) -> String {
    let kb = key.as_bytes();
    let out: Vec<u8> = input.bytes().enumerate().map(|(i,b)| b ^ kb[i % kb.len()]).collect();
    // base64風に hex で返す (デモなので)
    hex::encode(out)
}
fn xor_decipher(hex_str: &str, key: &str) -> Result<String, String> {
    let bytes = hex::decode(hex_str).map_err(|e| e.to_string())?;
    let kb = key.as_bytes();
    let out: Vec<u8> = bytes.into_iter().enumerate().map(|(i,b)| b ^ kb[i % kb.len()]).collect();
    String::from_utf8(out).map_err(|e| e.to_string())
}

const DEMO_KEY: &str = "last-game-nip44-demo-key-32bytes!!";

/// NIP-44 で暗号化 (デモ: xor + hex)。本番では下の `real_encrypt_nip44` を使う。
pub fn encrypt_nip44_demo(plaintext: &str, _recipient_pubkey: &str) -> Result<String, String> {
    // 本来: nip44::encrypt(&sender_secret, &recipient_pubkey, plaintext, None)
    // デモでは recipient によらず DEMO_KEY で xor
    Ok(format!("nip44:{}", xor_cipher(plaintext, DEMO_KEY)))
}
pub fn decrypt_nip44_demo(ciphertext: &str) -> Result<String, String> {
    let payload = ciphertext.strip_prefix("nip44:").unwrap_or(ciphertext);
    xor_decipher(payload, DEMO_KEY)
}

/// NIP-04 フォールバック (デモ)
pub fn encrypt_nip04_demo(plaintext: &str, _pubkey: &str) -> Result<String, String> {
    Ok(format!("nip04:{}", xor_cipher(plaintext, &format!("{}-nip04", DEMO_KEY))))
}
pub fn decrypt_nip04_demo(ciphertext: &str) -> Result<String, String> {
    let payload = ciphertext.strip_prefix("nip04:").unwrap_or(ciphertext);
    xor_decipher(payload, &format!("{}-nip04", DEMO_KEY))
}

// ---- 本番用の nostr-sdk 連携 (有効化するときは Cargo.toml の features を見る) ----
// WASM でも動くが、ここでは cfg で分岐できるように例示。

#[cfg(all(feature = "nostr-real", not(target_arch = "wasm32")))]
pub mod real {
    use nostr::nips::{nip04, nip44};
    use nostr::{Keys, PublicKey};

    pub fn encrypt_nip44(keys: &Keys, pubkey: &PublicKey, plaintext: &str) -> Result<String, String> {
        nip44::encrypt(keys.secret_key(), pubkey, plaintext, None).map_err(|e| e.to_string())
    }
    pub fn decrypt_nip44(keys: &Keys, pubkey: &PublicKey, ciphertext: &str) -> Result<String, String> {
        // nostr 0.35 では nip44::decrypt(secret, pubkey, content)
        nip44::decrypt(keys.secret_key(), pubkey, ciphertext).map_err(|e| e.to_string())
    }
    pub fn encrypt_nip04(keys: &Keys, pubkey: &PublicKey, plaintext: &str) -> Result<String, String> {
        nip04::encrypt(keys.secret_key(), pubkey, plaintext).map_err(|e| e.to_string())
    }
    pub fn decrypt_nip04(keys: &Keys, pubkey: &PublicKey, ciphertext: &str) -> Result<String, String> {
        nip04::decrypt(keys.secret_key(), pubkey, ciphertext).map_err(|e| e.to_string())
    }
}

// ---- イベント構造 (有料コンテンツ配信用) ----
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedLevelEvent {
    /// Nostr kind (例: 30078)
    pub kind: u16,
    /// `d` タグ (例: "level-2")
    pub d_tag: String,
    /// 宛先公開鍵 (pタグ)
    pub p_tag: String,
    /// 暗号文 (NIP-44)
    pub content: String,
    /// 作成時刻
    pub created_at: u64,
}

impl EncryptedLevelEvent {
    pub fn mock_encrypted(level_json: &str, recipient: &str) -> Result<Self, String> {
        let ct = encrypt_nip44_demo(level_json, recipient)?;
        Ok(Self {
            kind: 30078,
            d_tag: "level-2".into(),
            p_tag: recipient.into(),
            content: ct,
            created_at: js_sys::Date::now() as u64 / 1000,
        })
    }
    pub fn decrypt(&self) -> Result<String, String> {
        // まず NIP-44 を試し、失敗したら NIP-04
        decrypt_nip44_demo(&self.content).or_else(|_| decrypt_nip04_demo(&self.content))
    }
}
