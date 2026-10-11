//! 暗号化 / per-member encryption with NIP-04 or NIP-44.
//!
//! 有料コンテンツは「会員の公開鍵」と「運営者の秘密鍵」で暗号化します。
//! 復号できるのは、会員の秘密鍵 (と運営者の公開鍵) を持っている本人だけです。
//!
//! * **NIP-44 v2** (既定) — ChaCha20 + HMAC-SHA256 + HKDF。監査済みで推奨。
//! * **NIP-04** (互換用) — AES-256-CBC。古いクライアントとの互換のために残していますが、
//!   リスクがあるため新規の配信では NIP-44 を使ってください。

use nostr::nips::nip19::ToBech32;
use nostr::{Keys, PublicKey, SecretKey};

use crate::{Result, VaultError};

/// 暗号方式 / which NIP to use.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Cipher {
    /// NIP-44 v2 (推奨) / NIP-44 v2, recommended
    #[default]
    Nip44,
    /// NIP-04 (互換用・非推奨) / NIP-04 (legacy)
    Nip04,
}

impl Cipher {
    /// 文字列表現 / wire name (`nip44` / `nip04`).
    pub const fn as_str(self) -> &'static str {
        match self {
            Cipher::Nip44 => "nip44",
            Cipher::Nip04 => "nip04",
        }
    }

    /// 文字列から / parse from the wire name.
    pub fn parse(text: &str) -> Result<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "nip44" | "nip44v2" | "v2" => Ok(Cipher::Nip44),
            "nip04" | "v1" => Ok(Cipher::Nip04),
            other => Err(VaultError::Data(format!(
                "未知の暗号方式です: {other} (nip44 / nip04)"
            ))),
        }
    }

    /// 暗号化する / encrypt a plaintext for `recipient`.
    pub fn encrypt(
        self,
        secret_key: &SecretKey,
        recipient: &PublicKey,
        plaintext: &str,
    ) -> Result<String> {
        match self {
            Cipher::Nip44 => nostr::nips::nip44::encrypt(
                secret_key,
                recipient,
                plaintext,
                nostr::nips::nip44::Version::V2,
            ),
            Cipher::Nip04 => nostr::nips::nip04::encrypt(secret_key, recipient, plaintext),
        }
        .map_err(|err| VaultError::Crypto(err.to_string()))
    }

    /// 復号する / decrypt a ciphertext that `sender` produced.
    pub fn decrypt(
        self,
        secret_key: &SecretKey,
        sender: &PublicKey,
        ciphertext: &str,
    ) -> Result<String> {
        match self {
            Cipher::Nip44 => {
                nostr::nips::nip44::decrypt(secret_key, sender, ciphertext)
            }
            Cipher::Nip04 => nostr::nips::nip04::decrypt(secret_key, sender, ciphertext),
        }
        .map_err(|err| VaultError::Crypto(err.to_string()))
    }
}

/// 暗号化された中身 (方式つき) / a sealed payload that carries its own cipher name.
///
/// リレー上では `nip44:<base64>` のような 1 本の文字列として運びます。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SealedContent {
    /// 使った暗号方式 / cipher used
    pub cipher: Cipher,
    /// 暗号文 / the ciphertext
    pub payload: String,
}

impl SealedContent {
    /// 文字列 (`nip44:xxxx`) から作る / parse the wire format.
    pub fn parse(text: &str) -> Result<Self> {
        let (prefix, payload) = text.split_once(':').ok_or_else(|| {
            VaultError::Data("暗号文の形式が不正です (例: nip44:xxxx)".to_string())
        })?;
        if payload.is_empty() {
            return Err(VaultError::Data("暗号文が空です".to_string()));
        }
        Ok(Self {
            cipher: Cipher::parse(prefix)?,
            payload: payload.to_string(),
        })
    }

    /// 文字列にする / wire format.
    pub fn to_text(&self) -> String {
        format!("{}:{}", self.cipher.as_str(), self.payload)
    }

    /// 暗号化して作る / seal a plaintext for a recipient.
    pub fn seal(
        plaintext: &str,
        cipher: Cipher,
        sender: &Keys,
        recipient: &PublicKey,
    ) -> Result<Self> {
        Ok(Self {
            cipher,
            payload: cipher.encrypt(sender.secret_key(), recipient, plaintext)?,
        })
    }

    /// 復号する / open the payload with the recipient's keys.
    pub fn open(&self, keys: &Keys, sender: &PublicKey) -> Result<String> {
        self.cipher
            .decrypt(keys.secret_key(), sender, &self.payload)
    }
}

/// npub / hex のどちらでも受け取って [`PublicKey`] にする / accept npub or hex.
pub fn parse_public_key(input: &str) -> Result<PublicKey> {
    let trimmed = input.trim();
    PublicKey::parse(trimmed).map_err(|_| VaultError::InvalidPublicKey(trimmed.to_string()))
}

/// npub (bech32) に正規化する / normalize any public-key input to an npub.
pub fn normalize_npub(input: &str) -> Result<String> {
    let key = parse_public_key(input)?;
    key.to_bech32()
        .map_err(|err| VaultError::InvalidPublicKey(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nip44_round_trip() {
        let admin = Keys::generate();
        let member = Keys::generate();
        let sealed = SealedContent::seal(
            "WORLD 2 — secret levels",
            Cipher::Nip44,
            &admin,
            &member.public_key(),
        )
        .unwrap();
        assert!(sealed.to_text().starts_with("nip44:"));

        let parsed = SealedContent::parse(&sealed.to_text()).unwrap();
        let opened = parsed.open(&member, &admin.public_key()).unwrap();
        assert_eq!(opened, "WORLD 2 — secret levels");
    }

    #[test]
    fn nip04_round_trip() {
        let admin = Keys::generate();
        let member = Keys::generate();
        let sealed = SealedContent::seal(
            "レガシーなクライアントでも読める",
            Cipher::Nip04,
            &admin,
            &member.public_key(),
        )
        .unwrap();
        assert!(sealed.to_text().starts_with("nip04:"));
        let opened = SealedContent::parse(&sealed.to_text())
            .unwrap()
            .open(&member, &admin.public_key())
            .unwrap();
        assert_eq!(opened, "レガシーなクライアントでも読める");
    }

    #[test]
    fn strangers_cannot_open_the_payload() {
        let admin = Keys::generate();
        let member = Keys::generate();
        let stranger = Keys::generate();
        let sealed = SealedContent::seal("paid", Cipher::Nip44, &admin, &member.public_key())
            .unwrap();
        let parsed = SealedContent::parse(&sealed.to_text()).unwrap();
        assert!(parsed.open(&stranger, &admin.public_key()).is_err());
        // 運営者以外になりすました復号も失敗する / impersonating the sender fails too
        assert!(parsed.open(&member, &stranger.public_key()).is_err());
    }

    #[test]
    fn cipher_names_round_trip() {
        assert_eq!(Cipher::parse("nip44").unwrap(), Cipher::Nip44);
        assert_eq!(Cipher::parse("NIP04").unwrap(), Cipher::Nip04);
        assert!(Cipher::parse("rot13").is_err());
        assert_eq!(Cipher::default(), Cipher::Nip44);
    }

    #[test]
    fn malformed_payloads_are_rejected() {
        assert!(SealedContent::parse("just-base64").is_err());
        assert!(SealedContent::parse("nip44:").is_err());
        assert!(SealedContent::parse("rot13:aaaa").is_err());
    }

    #[test]
    fn npub_normalisation() {
        let keys = Keys::generate();
        let npub = keys.public_key().to_bech32().unwrap();
        let hex = keys.public_key().to_hex();
        assert_eq!(normalize_npub(&npub).unwrap(), npub);
        assert_eq!(normalize_npub(&hex).unwrap(), npub);
        assert!(normalize_npub("bob").is_err());
    }
}
