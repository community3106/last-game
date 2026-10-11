//! リレー上で使う定数 / constants used on the wire (event kinds and tags).

/// アプリ固有データ用の kind (NIP-78) / NIP-78 app-specific data.
///
/// 「会員名簿」も「有料コンテンツ」もこの kind を使い、`d` タグで区別します。
pub const KIND_APP_DATA: u16 = 30078;

/// 会員名簿の `d` タグ / `d` tag of the replaceable membership list.
pub const DTAG_MEMBERS: &str = "lastgame:v1:members";

/// 有料コンテンツの `d` タグの接頭辞 / prefix of the `d` tag of paid content events.
///
/// 完全な値は `lastgame:v1:content:<pack_id>:<npub>` になり、
/// 受け取り手ごとに別の (置き換え可能な) イベントになります。
pub const DTAG_CONTENT_PREFIX: &str = "lastgame:v1:content:";

/// すべてのイベントに付ける `t` タグ (検索用) / hashtag used to find our events.
pub const HASHTAG: &str = "lastgame";

/// プラン種別を表すタグ名 / tag name carrying the membership tier.
pub const TAG_TIER: &str = "tier";

/// 無料 / free tier.
pub const TIER_FREE: &str = "free";
/// 有料プラン / paid plan (サブスク).
pub const TIER_PRO: &str = "pro";
/// 管理者特権 / admin privileges.
pub const TIER_ADMIN: &str = "admin";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d_tags_are_unique_per_recipient() {
        let npub = "npub1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqz";
        let a = format!("{DTAG_CONTENT_PREFIX}world2:{npub}");
        let b = format!("{DTAG_CONTENT_PREFIX}world2:npub1other");
        assert_ne!(a, b);
        assert!(a.starts_with("lastgame:v1:content:"));
    }

    #[test]
    fn kinds_are_in_the_replaceable_range() {
        assert!((30_000..40_000).contains(&KIND_APP_DATA));
    }
}
