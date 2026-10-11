//! 会員名簿 (公開鍵ホワイトリスト) / the paid-member whitelist.
//!
//! 決済 (Stripe) が完了したユーザーの **npub** を登録するだけの、単純で監査しやすいリストです。
//! クレジットカード番号やメールアドレスは一切保存しません。

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::crypto::normalize_npub;
use crate::{Result, VaultError};

/// 会員の種別 / membership tier.
///
/// `Free < Pro < Admin` の順に強い権限を持ちます。
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// 無料ユーザー / free user (暗号化された有料コンテンツは読めない)
    #[default]
    Free,
    /// 有料プラン (サブスク) / paid subscription
    Pro,
    /// 管理者特権 / admin privileges
    Admin,
}

impl Tier {
    /// 文字列表現 / wire name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Tier::Free => crate::kinds::TIER_FREE,
            Tier::Pro => crate::kinds::TIER_PRO,
            Tier::Admin => crate::kinds::TIER_ADMIN,
        }
    }

    /// 日本語ラベル / Japanese label for the UI.
    pub const fn label(self) -> &'static str {
        match self {
            Tier::Free => "無料",
            Tier::Pro => "有料プラン (サブスク)",
            Tier::Admin => "管理者特権",
        }
    }

    /// 文字列から (不明な値は [`Tier::Free`]) / parse, unknown values fall back to free.
    pub fn parse(text: &str) -> Self {
        match text.trim().to_ascii_lowercase().as_str() {
            crate::kinds::TIER_ADMIN | "administrator" | "owner" => Tier::Admin,
            crate::kinds::TIER_PRO | "paid" | "member" | "premium" => Tier::Pro,
            _ => Tier::Free,
        }
    }

    /// 有料の権利を持っているか / does this tier unlock paid content?
    pub const fn is_paid(self) -> bool {
        matches!(self, Tier::Pro | Tier::Admin)
    }

    /// 管理者か / is this tier an admin?
    pub const fn is_admin(self) -> bool {
        matches!(self, Tier::Admin)
    }

    /// バッジ表示用の記号 / small glyph for badges.
    pub const fn glyph(self) -> char {
        match self {
            Tier::Free => '.',
            Tier::Pro => '*',
            Tier::Admin => '#',
        }
    }
}

/// 会員 1 人分のレコード / a single whitelist entry.
///
/// **個人情報は入れません。** npub (公開鍵)・プラン・有効期限だけです。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    /// 会員の npub / the member's npub (bech32)
    pub npub: String,
    /// 種別 / tier
    #[serde(default)]
    pub tier: Tier,
    /// 決済プラン名 (Stripe の price id など) / plan identifier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    /// 登録日時 (unix 秒) / when the member was added
    #[serde(default)]
    pub since: u64,
    /// 有効期限 (unix 秒)。`None` なら無期限 / expiry, `None` means forever
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<u64>,
    /// メモ (任意) / free-form note
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Member {
    /// npub を正規化して会員を作る / build a member, normalising the npub.
    pub fn new(npub: &str, tier: Tier, since: u64) -> Result<Self> {
        Ok(Self {
            npub: normalize_npub(npub)?,
            tier,
            plan: None,
            since,
            until: None,
            note: None,
        })
    }

    /// プラン名を付ける / attach a plan id.
    pub fn with_plan(mut self, plan: impl Into<String>) -> Self {
        self.plan = Some(plan.into());
        self
    }

    /// 有効期限を付ける / attach an expiry.
    pub fn with_until(mut self, until: u64) -> Self {
        self.until = Some(until);
        self
    }

    /// メモを付ける / attach a note.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// いま有効か / is this membership valid at `now`?
    pub fn is_active(&self, now: u64) -> bool {
        if !self.tier.is_paid() {
            return false;
        }
        match self.until {
            Some(until) => until > now,
            None => true,
        }
    }

    /// 表示用に短くした npub / shortened npub for tables and logs.
    pub fn short_npub(&self) -> String {
        let npub = &self.npub;
        if npub.len() <= 18 {
            return npub.clone();
        }
        format!("{}…{}", &npub[..11], &npub[npub.len() - 5..])
    }
}

/// 会員名簿 / the whitelist that gates paid content.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Whitelist {
    /// フォーマットのバージョン / format version
    #[serde(default)]
    pub version: u32,
    /// 最終更新 (unix 秒) / last update
    #[serde(default)]
    pub updated_at: u64,
    /// 会員 / the members
    #[serde(default)]
    pub members: Vec<Member>,
}

impl Whitelist {
    /// 空の名簿 / an empty whitelist.
    pub fn new() -> Self {
        Self {
            version: 1,
            updated_at: 0,
            members: Vec::new(),
        }
    }

    /// JSON から読む / parse from JSON.
    pub fn from_json(text: &str) -> Result<Self> {
        let whitelist: Self = serde_json::from_str(text)?;
        Ok(whitelist)
    }

    /// JSON にする / serialize (pretty printed).
    pub fn to_json_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// 会員数 / number of entries.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// 空か / is the list empty?
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// 追加または更新する。変更があったら `true` / upsert, returns whether something changed.
    pub fn upsert(&mut self, member: Member) -> bool {
        let npub = match normalize_npub(&member.npub) {
            Ok(npub) => npub,
            Err(_) => return false,
        };
        match self.members.iter_mut().find(|entry| entry.npub == npub) {
            Some(existing) => {
                if *existing == member {
                    return false;
                }
                *existing = member;
                true
            }
            None => {
                self.members.push(member);
                self.members.sort_by(|a, b| a.npub.cmp(&b.npub));
                true
            }
        }
    }

    /// 名簿から外す / remove a member.
    pub fn remove(&mut self, npub: &str) -> bool {
        let before = self.members.len();
        self.members.retain(|entry| !key_matches(&entry.npub, npub));
        before != self.members.len()
    }

    /// 会員を探す / look up a member (accepts npub or hex).
    pub fn get(&self, npub: &str, now: u64) -> Option<&Member> {
        self.members
            .iter()
            .find(|entry| key_matches(&entry.npub, npub) && entry.is_active(now))
    }

    /// 期限切れを除いた種別 / the tier of a key, ignoring expired entries.
    pub fn tier_of(&self, npub: &str, now: u64) -> Tier {
        self.get(npub, now).map(|entry| entry.tier).unwrap_or(Tier::Free)
    }

    /// 有料会員か / is this key a paying member right now?
    pub fn is_member(&self, npub: &str, now: u64) -> bool {
        self.get(npub, now).is_some()
    }

    /// 管理者特権を持っているか / does this key have admin privileges?
    pub fn is_admin(&self, npub: &str, now: u64) -> bool {
        self.get(npub, now).map(|entry| entry.tier) == Some(Tier::Admin)
    }

    /// 有効な会員一覧 / active members only.
    pub fn active(&self, now: u64) -> Vec<&Member> {
        self.members.iter().filter(|entry| entry.is_active(now)).collect()
    }

    /// 期限切れの古いレコードを消す / drop entries that expired long ago.
    pub fn revoke_expired(&mut self, now: u64) -> usize {
        let before = self.members.len();
        self.members.retain(|entry| match entry.until {
            Some(until) => until > now,
            None => true,
        });
        before - self.members.len()
    }

    /// `tier` 以上の有料コンテンツを配るべき相手 (重複なし) / recipients that may open `tier` content.
    pub fn recipients(&self, tier: Tier, now: u64) -> Vec<String> {
        let mut set = BTreeSet::new();
        for member in self.members.iter().filter(|entry| entry.is_active(now)) {
            if member.tier >= tier {
                set.insert(member.npub.clone());
            }
        }
        set.into_iter().collect()
    }

    /// テストやデモ用のビルダー / convenience builder.
    pub fn with_member(mut self, npub: &str, tier: Tier) -> Self {
        if let Ok(member) = Member::new(npub, tier, 0) {
            self.upsert(member);
        }
        self
    }

    /// 更新時刻を打つ / stamp `updated_at`.
    pub fn touch(&mut self, now: u64) {
        self.updated_at = now;
    }

    /// 名簿が壊れていないか確認する (npub と重複) / sanity check used by the CLI and tests.
    pub fn validate(&self) -> Result<()> {
        let mut seen = BTreeSet::new();
        for member in &self.members {
            let npub = normalize_npub(&member.npub)
                .map_err(|_| VaultError::InvalidPublicKey(member.npub.clone()))?;
            if !seen.insert(npub.clone()) {
                return Err(VaultError::Data(format!("npub が重複しています: {npub}")));
            }
        }
        Ok(())
    }
}

/// npub / hex の表記ゆれを吸収した比較 / compare two public keys written differently.
fn key_matches(stored: &str, query: &str) -> bool {
    if stored == query {
        return true;
    }
    normalize_npub(query).map(|npub| npub == stored).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::prelude::ToBech32;
    use nostr::Keys;

    fn npub_of(keys: &Keys) -> String {
        keys.public_key().to_bech32().unwrap()
    }

    #[test]
    fn tiers_are_ordered() {
        assert!(Tier::Free < Tier::Pro);
        assert!(Tier::Pro < Tier::Admin);
        assert!(Tier::Pro.is_paid());
        assert!(!Tier::Free.is_paid());
        assert!(Tier::Admin.is_admin());
        assert_eq!(Tier::parse("ADMIN"), Tier::Admin);
        assert_eq!(Tier::parse("???"), Tier::Free);
    }

    #[test]
    fn upsert_and_remove() {
        let keys = Keys::generate();
        let npub = npub_of(&keys);
        let mut whitelist = Whitelist::new();
        assert!(whitelist.is_empty());

        let member = Member::new(&npub, Tier::Pro, 100).unwrap();
        assert!(whitelist.upsert(member.clone()));
        assert_eq!(whitelist.len(), 1);
        // 同じ内容の再登録は変更なし / re-adding the same member changes nothing
        assert!(!whitelist.upsert(member.clone()));
        assert!(whitelist.is_member(&npub, 200));
        assert!(!whitelist.is_admin(&npub, 200));

        // hex でも引ける / lookups work with hex too
        assert!(whitelist.is_member(&keys.public_key().to_hex(), 200));

        assert!(whitelist.remove(&npub));
        assert!(!whitelist.is_member(&npub, 200));
        assert!(!whitelist.remove(&npub));
    }

    #[test]
    fn expiry_is_respected() {
        let npub = npub_of(&Keys::generate());
        let mut whitelist = Whitelist::new();
        whitelist.upsert(
            Member::new(&npub, Tier::Pro, 0)
                .unwrap()
                .with_until(1_000)
                .with_plan("price_monthly"),
        );
        assert!(whitelist.is_member(&npub, 999));
        assert!(!whitelist.is_member(&npub, 1_000));
        assert_eq!(whitelist.tier_of(&npub, 5_000), Tier::Free);
        // 失効したレコードは掃除できる / cleanup
        whitelist.upsert(Member::new(&npub_of(&Keys::generate()), Tier::Pro, 0).unwrap());
        assert_eq!(whitelist.revoke_expired(2_000), 1);
    }

    #[test]
    fn admins_are_greater_than_pro() {
        let admin = npub_of(&Keys::generate());
        let pro = npub_of(&Keys::generate());
        let free = npub_of(&Keys::generate());
        let mut whitelist = Whitelist::new();
        whitelist.upsert(Member::new(&admin, Tier::Admin, 0).unwrap());
        whitelist.upsert(Member::new(&pro, Tier::Pro, 0).unwrap());

        assert!(whitelist.is_admin(&admin, 0));
        assert!(!whitelist.is_admin(&pro, 0));
        assert!(!whitelist.is_member(&free, 0));

        // Pro 向けコンテンツの宛先は admin + pro / recipients of Pro content
        let recipients = whitelist.recipients(Tier::Pro, 0);
        assert_eq!(recipients.len(), 2);
        assert!(recipients.contains(&admin));
        assert!(recipients.contains(&pro));

        // Admin 向けの宛先は admin だけ / only admins get admin content
        assert_eq!(whitelist.recipients(Tier::Admin, 0), vec![admin]);
    }

    #[test]
    fn json_round_trip() {
        let npub = npub_of(&Keys::generate());
        let mut whitelist = Whitelist::new();
        whitelist.upsert(Member::new(&npub, Tier::Admin, 1_700_000_000).unwrap());
        whitelist.touch(1_700_000_100);
        let json = whitelist.to_json_pretty().unwrap();
        let parsed = Whitelist::from_json(&json).unwrap();
        assert_eq!(parsed, whitelist);
        assert!(parsed.validate().is_ok());
        assert!(json.contains("\"tier\": \"admin\""));
        // 秘密鍵は絶対に含まれない / never contains key material
        assert!(!json.contains("nsec"));
    }

    #[test]
    fn invalid_keys_are_rejected() {
        assert!(Member::new("not-a-key", Tier::Pro, 0).is_err());
        let mut whitelist = Whitelist::new();
        assert!(!whitelist.upsert(Member {
            npub: "bob".to_string(),
            tier: Tier::Pro,
            plan: None,
            since: 0,
            until: None,
            note: None,
        }));
        assert!(Whitelist::from_json("{\"members\":[{\"npub\":\"x\"}]}")
            .unwrap()
            .validate()
            .is_err());
    }

    #[test]
    fn short_npub_is_readable() {
        let member = Member::new(&npub_of(&Keys::generate()), Tier::Pro, 0).unwrap();
        let short = member.short_npub();
        assert!(short.contains('…'));
        assert!(short.len() < member.npub.len());
    }
}
