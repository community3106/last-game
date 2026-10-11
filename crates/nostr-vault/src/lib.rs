//! # nostr-vault — 有料コンテンツを Nostr で配るための小さなパッケージ
//!
//! 「LAST GAME」の課金まわりはこのクレートにまとまっています。仕組みはとても単純です:
//!
//! 1. Stripe などの決済を通ったら、その人の **公開鍵 (npub)** を会員名簿
//!    ([`Whitelist`]) に登録する。
//! 2. 有料コンテンツ ([`ContentPack`]) は、名簿に載っている人ごとに
//!    **NIP-44 (または NIP-04) で個別に暗号化** してリレーに投げる ([`seal_for_members`])。
//! 3. リレー上の暗号文は誰でも読めるが、**その当人の秘密鍵が無いと復号できない**。
//!    つまり無料ユーザーには中身が読めない。
//! 4. サイト側は `p` タグ (受け取り手) が自分の npub のイベントだけを拾って復号し、
//!    そのままゲームのステージとして読み込む。
//!
//! 秘密鍵はブラウザ (`localStorage`) に置くことも、NIP-07 拡張に任せることもできます。
//!
//! ## 特徴
//!
//! * 依存は `nostr` + `serde` のみ。**wasm32 でもネイティブでも同じコードが動く**
//! * 暗号化は「その人だけ」が対象。共通鍵を配る方式と違い、1 人が漏らしても他へ波及しない
//! * 名簿 ([`Whitelist`]) は平文の JSON として署名イベントに載せるので、
//!   「誰が会員か」は誰でも検証できる (金額や個人情報は載せない)
//!
//! ## 例: 会員名簿に載っている人だけにステージを配る
//!
//! ```no_run
//! use nostr::{nips::nip19::ToBech32, Keys};
//! use nostr_vault::{content::{ContentPack, seal_for_whitelist, build_paid_event}, crypto::Cipher, membership::{Member, Whitelist}, Tier};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let admin = Keys::generate();                       // 運営者の鍵 (秘密鍵)
//! let member_key = Keys::generate();                  // 会員から預かった npub (ここではデモ)
//! let npub = member_key.public_key().to_bech32()?;
//!
//! let mut whitelist = Whitelist::new();
//! whitelist.upsert(Member::new(&npub, Tier::Pro, 0)?.with_plan("price_monthly"));
//!
//! let pack = ContentPack::demo();
//! let sealed = seal_for_whitelist(&pack, &admin, &whitelist, 0, Cipher::Nip44)?;
//! for copy in &sealed {
//!     let event = build_paid_event(&pack, &copy.recipient_key()?, &copy.content_text, &admin)?;
//!     println!("{} => {}", copy.recipient, event.id);   // これをリレーに送る
//! }
//! # Ok(())
//! # }
//! ```
//!
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod content;
pub mod crypto;
pub mod demo_levels;
pub mod kinds;
pub mod membership;
#[cfg(feature = "stripe")]
pub mod stripe;

pub use content::{
    build_membership_event, build_paid_event, parse_membership_event, parse_paid_event,
    seal_for_members, seal_for_whitelist, ContentPack, LevelSource, PaidEventView, SealedPack,
};
pub use crypto::{Cipher, SealedContent};
pub use kinds::{
    DTAG_CONTENT_PREFIX, DTAG_MEMBERS, HASHTAG, KIND_APP_DATA, TAG_TIER, TIER_ADMIN, TIER_FREE,
    TIER_PRO,
};
pub use membership::{Member, Tier, Whitelist};
pub use thiserror::Error;

/// このパッケージのエラー / errors returned by this crate.
#[derive(Debug, Error)]
pub enum VaultError {
    /// 暗号化・復号に失敗した / encryption or decryption failed
    #[error("暗号エラー: {0}")]
    Crypto(String),
    /// JSON / 署名まわりのエラー / serialization error
    #[error("データ形式エラー: {0}")]
    Data(String),
    /// npub として読めない / the given string is not a valid public key
    #[error("公開鍵として解釈できません: {0}")]
    InvalidPublicKey(String),
    /// 会員ではない (有料コンテンツを開こうとした) / the key is not on the whitelist
    #[error("この公開鍵は有料会員ではありません: {0}")]
    NotAMember(String),
    /// 自分のためのイベントではない / the event is not addressed to this key
    #[error("このイベントは自分宛てではありません")]
    NotAddressedToYou,
    /// イベントの種類が想定と違う / unexpected event kind
    #[error("イベント種別が想定と違います (kind {got}, 期待 {expected})")]
    WrongKind {
        /// 実際の kind / actual kind
        got: u16,
        /// 期待した kind / expected kind
        expected: u16,
    },
    /// 署名が検証できない / the event signature is invalid
    #[error("署名検証に失敗しました")]
    InvalidSignature,
    /// Stripe の署名が不正 / invalid Stripe signature
    #[error("Stripe 署名検証に失敗しました: {0}")]
    StripeSignature(String),
    /// 必須の情報が欠けている / required data is missing
    #[error("必要な情報がありません: {0}")]
    Missing(String),
    /// JSON の読み書きに失敗した / JSON error
    #[error("JSON エラー: {0}")]
    Json(#[from] serde_json::Error),
}

/// このクレートの結果型 / result alias.
pub type Result<T, E = VaultError> = std::result::Result<T, E>;
