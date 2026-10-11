//! 有料コンテンツ (ステージパック) の暗号化と配信 / encrypting and publishing paid content.
//!
//! 1 つのパック ([`ContentPack`]) を、会員 1 人ずつに NIP-44 で暗号化して
//! リレー用のイベント ([`nostr::Event`]) にします。
//!
//! ```text
//!  ContentPack (JSON)
//!        │  NIP-44 (会員 A の公開鍵で暗号化)
//!        ├──────────────► kind 30078, d=lastgame:v1:content:world2:<npub A>, p=<npub A>
//!        │  NIP-44 (会員 B の公開鍵で暗号化)
//!        └──────────────► kind 30078, d=lastgame:v1:content:world2:<npub B>, p=<npub B>
//! ```
//!
//! リレー上には暗号文しか流れないので、無料ユーザーが拾っても中身は読めません。

use serde::{Deserialize, Serialize};

use crate::crypto::{normalize_npub, parse_public_key, Cipher, SealedContent};
use crate::kinds::{
    DTAG_CONTENT_PREFIX, DTAG_MEMBERS, HASHTAG, KIND_APP_DATA, TAG_TIER,
};
use crate::membership::{Tier, Whitelist};
use crate::{Result, VaultError};

/// レベル 1 つ分のソース (ASCII アート) / one level inside a pack.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelSource {
    /// レベル ID (例: `2-1`) / level id
    pub id: String,
    /// 表示名 / display title
    pub title: String,
    /// ASCII アート本体 / the ASCII art itself
    pub art: String,
}

/// 有料ステージパック / a bundle of paid levels.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentPack {
    /// パック ID (例: `world2`) / pack id
    pub id: String,
    /// タイトル / title
    pub title: String,
    /// どの種別以上に配るか / minimum tier required
    #[serde(default)]
    pub tier: Tier,
    /// 作成日時 (unix 秒) / creation timestamp
    #[serde(default)]
    pub created_at: u64,
    /// 収録レベル / the levels
    #[serde(default)]
    pub levels: Vec<LevelSource>,
}

impl ContentPack {
    /// デモ用のパック (WORLD 2) / the demo pack used by the site and the CLI examples.
    ///
    /// 実運用では運営者の手元で作った非公開のパックに置き換えてください。
    pub fn demo() -> Self {
        Self {
            id: "world2".to_string(),
            title: "WORLD 2 — 封印されたステージ".to_string(),
            tier: Tier::Pro,
            created_at: 0,
            levels: vec![
                LevelSource {
                    id: "2-1".to_string(),
                    title: "2-1 灼熱の大穴".to_string(),
                    art: crate::demo_levels::WORLD_2_1.to_string(),
                },
                LevelSource {
                    id: "2-2".to_string(),
                    title: "2-2 トゲの回廊".to_string(),
                    art: crate::demo_levels::WORLD_2_2.to_string(),
                },
            ],
        }
    }

    /// JSON から読む / parse a pack from JSON.
    pub fn from_json(text: &str) -> Result<Self> {
        Ok(serde_json::from_str(text)?)
    }

    /// JSON にする / serialize a pack.
    pub fn to_json_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// レベル数 / number of levels.
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// レベル ID の一覧 / level ids (safe to show to non-members as a teaser).
    pub fn level_ids(&self) -> Vec<String> {
        self.levels.iter().map(|level| level.id.clone()).collect()
    }
}

/// 会員 1 人分の暗号化済みパック / a pack sealed for exactly one recipient.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SealedPack {
    /// 受け取り手の npub / recipient npub
    pub recipient: String,
    /// パック ID / pack id
    pub pack_id: String,
    /// `d` タグ / the `d` tag to publish under
    pub d_tag: String,
    /// リレーに載せる文字列 (`nip44:...`) / the wire payload
    pub content_text: String,
    /// 使った暗号方式 / cipher used
    pub cipher: Cipher,
}

impl SealedPack {
    /// 受け取り手の公開鍵 / the recipient as a public key.
    pub fn recipient_key(&self) -> Result<nostr::PublicKey> {
        parse_public_key(&self.recipient)
    }

    /// 復号してパックを取り出す / decrypt back into a pack.
    pub fn open(&self, keys: &nostr::Keys, sender: &nostr::PublicKey) -> Result<ContentPack> {
        let sealed = SealedContent::parse(&self.content_text)?;
        let plaintext = sealed.open(keys, sender)?;
        ContentPack::from_json(&plaintext)
    }
}

/// 会員ごとの `d` タグ / the replaceable-event address of one recipient's copy.
pub fn d_tag_for(pack_id: &str, recipient_npub: &str) -> String {
    format!("{DTAG_CONTENT_PREFIX}{pack_id}:{recipient_npub}")
}

/// 複数の受け取り手にパックを配る (暗号化のみ) / seal a pack for a list of recipients.
pub fn seal_for_members(
    pack: &ContentPack,
    sender: &nostr::Keys,
    recipients: &[String],
    cipher: Cipher,
) -> Result<Vec<SealedPack>> {
    if recipients.is_empty() {
        return Err(VaultError::Missing(
            "配布先が空です (先に会員名簿へ登録してください)".to_string(),
        ));
    }
    let plaintext = pack.to_json_pretty()?;
    let mut sealed = Vec::with_capacity(recipients.len());
    for recipient in recipients {
        let key = parse_public_key(recipient)?;
        let npub = normalize_npub(recipient)?;
        let content = SealedContent::seal(&plaintext, cipher, sender, &key)?;
        sealed.push(SealedPack {
            d_tag: d_tag_for(&pack.id, &npub),
            recipient: npub,
            pack_id: pack.id.clone(),
            content_text: content.to_text(),
            cipher,
        });
    }
    Ok(sealed)
}

/// 名簿の有効な会員全員に配る / seal a pack for every active member that qualifies.
pub fn seal_for_whitelist(
    pack: &ContentPack,
    sender: &nostr::Keys,
    whitelist: &Whitelist,
    now: u64,
    cipher: Cipher,
) -> Result<Vec<SealedPack>> {
    let recipients = whitelist.recipients(pack.tier, now);
    seal_for_members(pack, sender, &recipients, cipher)
}

/// パック 1 つ分の署名済みイベントを作る / build the signed event for one sealed pack.
pub fn build_paid_event(
    pack: &ContentPack,
    recipient: &nostr::PublicKey,
    sealed_text: &str,
    sender: &nostr::Keys,
) -> Result<nostr::Event> {
    use nostr::nips::nip19::ToBech32;
    use nostr::{EventBuilder, Kind, Tag, TagKind, Timestamp};

    let npub = recipient
        .to_bech32()
        .map_err(|err| VaultError::InvalidPublicKey(err.to_string()))?;
    let alt = format!("有料コンテンツ: {} ({})", pack.title, npub);
    EventBuilder::new(Kind::Custom(KIND_APP_DATA), sealed_text)
        .tag(Tag::identifier(d_tag_for(&pack.id, &npub)))
        .tag(Tag::public_key(*recipient))
        .tag(Tag::hashtag(HASHTAG))
        .tag(Tag::custom(
            TagKind::Custom(TAG_TIER.into()),
            [pack.tier.as_str()],
        ))
        .tag(Tag::title(pack.title.clone()))
        .tag(Tag::alt(alt))
        .custom_created_at(Timestamp::now())
        .sign_with_keys(sender)
        .map_err(|err| VaultError::Data(err.to_string()))
}

/// 会員名簿そのものを載せる署名イベント / the signed replaceable event carrying the whitelist.
pub fn build_membership_event(whitelist: &Whitelist, keys: &nostr::Keys) -> Result<nostr::Event> {
    use nostr::{EventBuilder, Kind, Tag, Timestamp};

    let content = whitelist.to_json_pretty()?;
    EventBuilder::new(Kind::Custom(KIND_APP_DATA), content)
        .tag(Tag::identifier(DTAG_MEMBERS))
        .tag(Tag::hashtag(HASHTAG))
        .tag(Tag::client("last-game"))
        .tag(Tag::alt("LAST GAME 会員名簿 (公開鍵ホワイトリスト)"))
        .custom_created_at(Timestamp::now())
        .sign_with_keys(keys)
        .map_err(|err| VaultError::Data(err.to_string()))
}

/// リレーから拾った有料イベントの見え方 / what the site can see about a paid event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaidEventView {
    /// イベント ID (hex) / event id
    pub event_id: String,
    /// 送信者 (運営者) の npub / the author's npub
    pub author: String,
    /// パック ID / pack id
    pub pack_id: String,
    /// タイトル / title
    pub title: String,
    /// 必要な種別 / required tier
    pub tier: Tier,
    /// 受け取り手の npub 一覧 / recipients
    pub recipients: Vec<String>,
    /// 暗号文 (`nip44:...`) / the sealed payload
    pub content_text: String,
    /// 作成日時 / created at
    pub created_at: u64,
}

impl PaidEventView {
    /// 自分宛てか / is this copy addressed to `npub`?
    pub fn is_for(&self, npub: &str) -> bool {
        let normalized = normalize_npub(npub).unwrap_or_else(|_| npub.to_string());
        self.recipients.iter().any(|entry| *entry == normalized)
    }

    /// 復号してステージパックにする / decrypt into a playable pack.
    pub fn open(&self, keys: &nostr::Keys) -> Result<ContentPack> {
        let sealed = SealedContent::parse(&self.content_text)?;
        let sender = parse_public_key(&self.author)?;
        let plaintext = sealed.open(keys, &sender)?;
        ContentPack::from_json(&plaintext)
    }
}

/// リレーから拾ったイベントを解釈する / parse a paid-content event (verifies the signature).
pub fn parse_paid_event(event: &nostr::Event) -> Result<PaidEventView> {
    use nostr::nips::nip19::ToBech32;

    let kind = event.kind.as_u16();
    if kind != KIND_APP_DATA {
        return Err(VaultError::WrongKind {
            got: kind,
            expected: KIND_APP_DATA,
        });
    }
    event.verify().map_err(|_| VaultError::InvalidSignature)?;

    let mut d_tag = None;
    let mut recipients = Vec::new();
    let mut title = None;
    let mut tier = Tier::Pro;
    for tag in event.tags.iter() {
        let slice = tag.as_slice();
        match slice {
            [name, value, ..] if name == "d" => d_tag = Some(value.clone()),
            [name, value, ..] if name == "p" => {
                if let Ok(key) = parse_public_key(value) {
                    if let Ok(npub) = key.to_bech32() {
                        recipients.push(npub);
                    }
                }
            }
            [name, value, ..] if name == "title" => title = Some(value.clone()),
            [name, value, ..] if name == TAG_TIER => tier = Tier::parse(value),
            _ => {}
        }
    }

    let d_tag = d_tag.ok_or_else(|| VaultError::Missing("d タグがありません".to_string()))?;
    let pack_id = d_tag
        .strip_prefix(DTAG_CONTENT_PREFIX)
        .and_then(|rest| rest.split(':').next())
        .unwrap_or_default()
        .to_string();

    let author = event
        .pubkey
        .to_bech32()
        .map_err(|err| VaultError::InvalidPublicKey(err.to_string()))?;

    Ok(PaidEventView {
        event_id: event.id.to_hex(),
        author,
        pack_id,
        title: title.unwrap_or_else(|| "有料コンテンツ".to_string()),
        tier,
        recipients,
        content_text: event.content.clone(),
        created_at: event.created_at.as_u64(),
    })
}

/// 会員名簿イベントを解釈する / parse the membership event (verifies the signature).
pub fn parse_membership_event(event: &nostr::Event) -> Result<Whitelist> {
    let kind = event.kind.as_u16();
    if kind != KIND_APP_DATA {
        return Err(VaultError::WrongKind {
            got: kind,
            expected: KIND_APP_DATA,
        });
    }
    event.verify().map_err(|_| VaultError::InvalidSignature)?;
    let is_members = event
        .tags
        .iter()
        .any(|tag| tag.as_slice().first().map(String::as_str) == Some("d")
            && tag.as_slice().get(1).map(String::as_str) == Some(DTAG_MEMBERS));
    if !is_members {
        return Err(VaultError::Missing(format!(
            "d タグが {DTAG_MEMBERS} ではありません"
        )));
    }
    Whitelist::from_json(&event.content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::membership::Member;
    use nostr::nips::nip19::ToBech32;
    use nostr::util::JsonUtil;
    use nostr::{Event, Keys};

    fn npub_of(keys: &Keys) -> String {
        keys.public_key().to_bech32().unwrap()
    }

    #[test]
    fn demo_pack_contains_playable_levels() {
        let pack = ContentPack::demo();
        assert_eq!(pack.level_count(), 2);
        assert_eq!(pack.level_ids(), vec!["2-1", "2-2"]);
        for level in &pack.levels {
            let parsed = mario_core::Level::from_ascii(&level.id, &level.art)
                .unwrap_or_else(|err| panic!("level {} failed to parse: {err}", level.id));
            assert!(parsed.goal().is_some(), "{} needs a goal", level.id);
            assert!(parsed.width() > 60, "{} should be a full level", level.id);
            assert!(!parsed.enemy_spawns().is_empty());
        }
    }

    #[test]
    fn pack_json_round_trip() {
        let pack = ContentPack::demo();
        let json = pack.to_json_pretty().unwrap();
        assert_eq!(ContentPack::from_json(&json).unwrap(), pack);
        assert!(json.contains("\"tier\": \"pro\""));
    }

    #[test]
    fn every_member_can_open_their_copy_and_strangers_cannot() {
        let admin = Keys::generate();
        let alice = Keys::generate();
        let bob = Keys::generate();
        let stranger = Keys::generate();

        let pack = ContentPack::demo();
        let sealed = seal_for_members(
            &pack,
            &admin,
            &[npub_of(&alice), npub_of(&bob)],
            Cipher::Nip44,
        )
        .unwrap();
        assert_eq!(sealed.len(), 2);

        // Alice は自分の控えを開ける / Alice opens her own copy
        let alice_copy = sealed.iter().find(|s| s.recipient == npub_of(&alice)).unwrap();
        assert_eq!(
            alice_copy.open(&alice, &admin.public_key()).unwrap(),
            pack
        );
        // Bob は Alice 宛の控えを開けない / Bob cannot open Alice's copy
        assert!(alice_copy.open(&bob, &admin.public_key()).is_err());
        // 無料ユーザーは開けない / a free user cannot open anything
        assert!(alice_copy.open(&stranger, &admin.public_key()).is_err());
        // 他人になりすました復号もダメ / pretending to be the admin does not help
        assert!(alice_copy.open(&alice, &stranger.public_key()).is_err());
    }

    #[test]
    fn whitelist_gating_delivers_only_to_paying_members() {
        let admin = Keys::generate();
        let pro = Keys::generate();
        let free = Keys::generate();
        let mut whitelist = Whitelist::new();
        whitelist.upsert(Member::new(&npub_of(&pro), Tier::Pro, 0).unwrap());

        let sealed = seal_for_whitelist(&ContentPack::demo(), &admin, &whitelist, 100, Cipher::Nip44)
            .unwrap();
        assert_eq!(sealed.len(), 1);
        assert_eq!(sealed[0].recipient, npub_of(&pro));
        assert!(!sealed.iter().any(|s| s.recipient == npub_of(&free)));

        // 会員ゼロならエラー / no members means nothing to publish
        assert!(seal_for_whitelist(
            &ContentPack::demo(),
            &admin,
            &Whitelist::new(),
            100,
            Cipher::Nip44
        )
        .is_err());
    }

    #[test]
    fn paid_event_round_trip_through_json() {
        let admin = Keys::generate();
        let member = Keys::generate();
        let pack = ContentPack::demo();
        let sealed = seal_for_members(&pack, &admin, &[npub_of(&member)], Cipher::Nip44).unwrap();
        let event = build_paid_event(
            &pack,
            &member.public_key(),
            &sealed[0].content_text,
            &admin,
        )
        .unwrap();

        // リレーを経由したのと同じ状態にする / simulate a relay round trip
        let json = event.try_as_json().unwrap();
        let relayed = Event::from_json(json).unwrap();
        let view = parse_paid_event(&relayed).unwrap();

        assert_eq!(view.pack_id, "world2");
        assert_eq!(view.tier, Tier::Pro);
        assert!(view.is_for(&npub_of(&member)));
        assert!(!view.is_for(&npub_of(&Keys::generate())));
        assert_eq!(view.open(&member).unwrap(), pack);
        // 無料ユーザーには読めない / free users get an error, not the content
        assert!(view.open(&Keys::generate()).is_err());
        // タイトルは誰でも見られる (宣伝用のティザー) / the title is public teaser data
        assert!(view.title.contains("WORLD 2"));
    }

    #[test]
    fn tampered_events_are_rejected() {
        let admin = Keys::generate();
        let member = Keys::generate();
        let pack = ContentPack::demo();
        let event = build_paid_event(
            &pack,
            &member.public_key(),
            "nip44:aaaa",
            &admin,
        )
        .unwrap();
        let mut json: serde_json::Value =
            serde_json::from_str(&event.try_as_json().unwrap()).unwrap();
        json["content"] = serde_json::Value::String("nip44:bbbb".to_string());
        let tampered = Event::from_json(json.to_string()).unwrap();
        assert!(matches!(
            parse_paid_event(&tampered),
            Err(VaultError::InvalidSignature)
        ));
    }

    #[test]
    fn membership_event_round_trip() {
        let admin = Keys::generate();
        let member = Keys::generate();
        let mut whitelist = Whitelist::new();
        whitelist.upsert(Member::new(&npub_of(&member), Tier::Admin, 1_700_000_000).unwrap());
        whitelist.touch(1_700_000_100);

        let event = build_membership_event(&whitelist, &admin).unwrap();
        let relayed = Event::from_json(event.try_as_json().unwrap()).unwrap();
        let parsed = parse_membership_event(&relayed).unwrap();
        assert_eq!(parsed, whitelist);
        assert!(parsed.is_admin(&npub_of(&member), 1_700_000_200));
    }

    #[test]
    fn d_tags_are_per_recipient_and_replaceable() {
        let a = d_tag_for("world2", "npub1aaa");
        let b = d_tag_for("world2", "npub1bbb");
        assert_ne!(a, b);
        assert_eq!(a, "lastgame:v1:content:world2:npub1aaa");
    }
}
