//! Stripe 連携 / turning Stripe payments into whitelist entries.
//!
//! ここには **Stripe SDK を使わない** 実装が入っています。必要なのは 2 つだけです:
//!
//! 1. [`verify_webhook`] — Webhook が本当に Stripe から来たことを HMAC-SHA256 で検証する
//!    (Stripe の署名方式: `Stripe-Signature: t=<unix>,v1=<hex hmac of "t.payload">`)
//! 2. [`member_from_event`] — `checkout.session.completed` などの JSON から
//!    npub とプランを取り出して [`Member`] にする
//!
//! npub は Stripe の `client_reference_id` (推奨) か `metadata.npub` に入れておきます。
//! 決済ページで「あなたの npub」を入力してもらう形でも、ログイン連携で自動で埋めても構いません。

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::crypto::normalize_npub;
use crate::membership::{Member, Tier};
use crate::{Result, VaultError};

/// 署名ヘッダの許容時間 (秒)。これより古い/新しいタイムスタンプは拒否する (リプレイ対策)
/// / maximum accepted clock skew for the signature timestamp.
pub const DEFAULT_TOLERANCE_SECONDS: i64 = 300;

/// パース済みの `Stripe-Signature` ヘッダ / a parsed `Stripe-Signature` header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StripeSignature {
    /// `t=` の値 / the timestamp
    pub timestamp: i64,
    /// `v1=` の値 (複数可) / the candidate signatures
    pub signatures: Vec<String>,
}

/// `t=1699999999,v1=abcdef...` をパースする / parse the header value.
pub fn parse_signature_header(header: &str) -> Result<StripeSignature> {
    let mut timestamp = None;
    let mut signatures = Vec::new();
    for part in header.split(',') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("t=") {
            timestamp = value.parse::<i64>().ok();
        } else if let Some(value) = part.strip_prefix("v1=") {
            signatures.push(value.to_string());
        }
    }
    match timestamp {
        Some(timestamp) if !signatures.is_empty() => Ok(StripeSignature {
            timestamp,
            signatures,
        }),
        _ => Err(VaultError::StripeSignature(
            "Stripe-Signature ヘッダの形式が不正です".to_string(),
        )),
    }
}

/// 期待される署名 (hex) を計算する / compute the expected `v1` signature.
pub fn expected_signature(secret: &str, timestamp: i64, payload: &[u8]) -> Result<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|err| VaultError::StripeSignature(err.to_string()))?;
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(payload);
    Ok(hex::encode(mac.finalize().into_bytes()))
}

/// Webhook を検証する / verify a webhook call.
///
/// * `secret` — Stripe の署名シークレット (`whsec_...`)
/// * `payload` — 生のリクエストボディ (JSON を再シリアライズしてはいけない)
/// * `header` — `Stripe-Signature` ヘッダ
/// * `now` — 現在の unix 秒 (テスト可能にするため引数で受ける)
pub fn verify_webhook(
    secret: &str,
    payload: &[u8],
    header: &str,
    now: i64,
    tolerance: i64,
) -> Result<StripeSignature> {
    let parsed = parse_signature_header(header)?;
    let skew = (now - parsed.timestamp).abs();
    if skew > tolerance {
        return Err(VaultError::StripeSignature(format!(
            "タイムスタンプが古すぎます ({skew} 秒ずれています)"
        )));
    }
    let expected = expected_signature(secret, parsed.timestamp, payload)?;
    let matched = parsed
        .signatures
        .iter()
        .any(|candidate| constant_time_eq(candidate.as_bytes(), expected.as_bytes()));
    if !matched {
        return Err(VaultError::StripeSignature(
            "署名が一致しません".to_string(),
        ));
    }
    Ok(parsed)
}

/// Webhook を検証して、名簿に反映すべき内容を返す / verify and translate a webhook.
pub fn member_from_event(payload: &str, now: u64) -> Result<StripeOutcome> {
    let value: serde_json::Value = serde_json::from_str(payload)?;
    let event_type = value
        .get("type")
        .and_then(|value| value.as_str())
        .ok_or_else(|| VaultError::Missing("type フィールドがありません".to_string()))?;
    let object = value
        .get("data")
        .and_then(|data| data.get("object"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    match event_type {
        "checkout.session.completed" => {
            let npub = extract_npub(&object)
                .ok_or_else(|| VaultError::Missing("npub が見つかりません (client_reference_id / metadata.npub)".to_string()))?;
            let tier = object
                .get("metadata")
                .and_then(|meta| meta.get("tier"))
                .and_then(|value| value.as_str())
                .map(Tier::parse)
                .unwrap_or(Tier::Pro);
            let plan = object
                .get("metadata")
                .and_then(|meta| meta.get("plan"))
                .and_then(|value| value.as_str())
                .map(str::to_string);
            let email = object
                .get("customer_details")
                .and_then(|details| details.get("email"))
                .and_then(|value| value.as_str())
                .map(str::to_string);
            // メールアドレスは保存しない (プライバシー)。プラン名だけ残す
            // / never store PII: keep only the plan identifier
            let _ = email;
            let mut member = Member::new(&npub, tier, now)?;
            if let Some(plan) = plan {
                member = member.with_plan(plan);
            }
            Ok(StripeOutcome::Grant(member))
        }
        "customer.subscription.created" | "customer.subscription.updated" => {
            let status = object
                .get("status")
                .and_then(|value| value.as_str())
                .unwrap_or("active");
            let npub = extract_npub(&object).ok_or_else(|| {
                VaultError::Missing("npub が見つかりません (metadata.npub)".to_string())
            })?;
            if matches!(status, "canceled" | "unpaid" | "incomplete_expired") {
                return Ok(StripeOutcome::Revoke {
                    npub: normalize_npub(&npub)?,
                });
            }
            let tier = object
                .get("metadata")
                .and_then(|meta| meta.get("tier"))
                .and_then(|value| value.as_str())
                .map(Tier::parse)
                .unwrap_or(Tier::Pro);
            let mut member = Member::new(&npub, tier, now)?;
            if let Some(until) = object
                .get("current_period_end")
                .and_then(|value| value.as_u64())
            {
                member = member.with_until(until);
            }
            if let Some(plan) = object
                .get("items")
                .and_then(|items| items.get("data"))
                .and_then(|data| data.get(0))
                .and_then(|item| item.get("price"))
                .and_then(|price| price.get("id"))
                .and_then(|value| value.as_str())
            {
                member = member.with_plan(plan);
            }
            Ok(StripeOutcome::Grant(member))
        }
        "customer.subscription.deleted" => {
            let npub = extract_npub(&object).ok_or_else(|| {
                VaultError::Missing("npub が見つかりません (metadata.npub)".to_string())
            })?;
            Ok(StripeOutcome::Revoke {
                npub: normalize_npub(&npub)?,
            })
        }
        other => Ok(StripeOutcome::Ignore {
            reason: format!("対応していないイベントです: {other}"),
        }),
    }
}

/// Webhook 1 件の結果 / what a single webhook means for the whitelist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StripeOutcome {
    /// 会員を追加・更新する / add or update a member
    Grant(Member),
    /// 会員権を剥奪する / revoke a membership
    Revoke {
        /// 対象の npub / the affected npub
        npub: String,
    },
    /// 何もしない / nothing to do
    Ignore {
        /// 理由 (ログ用) / human readable reason
        reason: String,
    },
}

/// `client_reference_id` → `metadata.npub` → `metadata.pubkey` の順に探す
/// / find the npub in the usual places.
fn extract_npub(object: &serde_json::Value) -> Option<String> {
    let direct = object
        .get("client_reference_id")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    direct.or_else(|| {
        object
            .get("metadata")
            .and_then(|meta| {
                meta.get("npub")
                    .or_else(|| meta.get("pubkey"))
                    .or_else(|| meta.get("nostr"))
            })
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

/// タイミング攻撃を避ける比較 / constant time comparison.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::nips::nip19::ToBech32;
    use nostr::Keys;

    const SECRET: &str = "whsec_test_secret_do_not_use";

    fn signed_header(secret: &str, timestamp: i64, payload: &str) -> String {
        let signature = expected_signature(secret, timestamp, payload.as_bytes()).unwrap();
        format!("t={timestamp},v1={signature}")
    }

    #[test]
    fn valid_signature_is_accepted() {
        let payload = r#"{"type":"checkout.session.completed"}"#;
        let header = signed_header(SECRET, 1_700_000_000, payload);
        let parsed = verify_webhook(
            SECRET,
            payload.as_bytes(),
            &header,
            1_700_000_010,
            DEFAULT_TOLERANCE_SECONDS,
        )
        .unwrap();
        assert_eq!(parsed.timestamp, 1_700_000_000);
        assert_eq!(parsed.signatures.len(), 1);
    }

    #[test]
    fn wrong_secret_payload_or_stale_timestamp_is_rejected() {
        let payload = r#"{"type":"checkout.session.completed"}"#;
        let header = signed_header(SECRET, 1_700_000_000, payload);
        // 違うシークレット / wrong secret
        assert!(verify_webhook(
            "whsec_other",
            payload.as_bytes(),
            &header,
            1_700_000_000,
            DEFAULT_TOLERANCE_SECONDS
        )
        .is_err());
        // 改ざんされたペイロード / tampered payload
        assert!(verify_webhook(
            SECRET,
            r#"{"type":"checkout.session.completed","x":1}"#.as_bytes(),
            &header,
            1_700_000_000,
            DEFAULT_TOLERANCE_SECONDS
        )
        .is_err());
        // リプレイ / replay
        assert!(verify_webhook(
            SECRET,
            payload.as_bytes(),
            &header,
            1_700_010_000,
            DEFAULT_TOLERANCE_SECONDS
        )
        .is_err());
        // 壊れたヘッダ / malformed header
        assert!(verify_webhook(SECRET, payload.as_bytes(), "v1=abc", 0, 300).is_err());
    }

    #[test]
    fn multiple_signatures_are_supported() {
        let payload = "{}";
        let good = expected_signature(SECRET, 100, payload.as_bytes()).unwrap();
        let header = format!("t=100,v1=deadbeef,v1={good}");
        assert!(verify_webhook(SECRET, payload.as_bytes(), &header, 100, 300).is_ok());
    }

    #[test]
    fn checkout_grants_a_membership() {
        let member_key = Keys::generate();
        let npub = member_key.public_key().to_bech32().unwrap();
        let payload = format!(
            r#"{{
              "id": "evt_1",
              "type": "checkout.session.completed",
              "data": {{ "object": {{
                 "id": "cs_1",
                 "client_reference_id": "{npub}",
                 "metadata": {{ "tier": "admin", "plan": "price_admin_monthly" }},
                 "customer_details": {{ "email": "someone@example.com" }}
              }} }}
            }}"#
        );
        let outcome = member_from_event(&payload, 1_700_000_000).unwrap();
        match outcome {
            StripeOutcome::Grant(member) => {
                assert_eq!(member.npub, npub);
                assert_eq!(member.tier, Tier::Admin);
                assert_eq!(member.plan.as_deref(), Some("price_admin_monthly"));
                assert_eq!(member.since, 1_700_000_000);
                // メールアドレスは保存されない / no PII is stored
                assert!(!format!("{member:?}").contains("example.com"));
            }
            other => panic!("expected a grant, got {other:?}"),
        }
    }

    #[test]
    fn subscription_update_grants_and_deletion_revokes() {
        let npub = Keys::generate().public_key().to_bech32().unwrap();
        let update = format!(
            r#"{{
              "type": "customer.subscription.updated",
              "data": {{ "object": {{
                 "id": "sub_1",
                 "status": "active",
                 "current_period_end": 1800000000,
                 "metadata": {{ "npub": "{npub}", "tier": "pro" }},
                 "items": {{ "data": [ {{ "price": {{ "id": "price_pro_monthly" }} }} ] }}
              }} }}
            }}"#
        );
        match member_from_event(&update, 1_700_000_000).unwrap() {
            StripeOutcome::Grant(member) => {
                assert_eq!(member.tier, Tier::Pro);
                assert_eq!(member.until, Some(1_800_000_000));
                assert_eq!(member.plan.as_deref(), Some("price_pro_monthly"));
            }
            other => panic!("expected a grant, got {other:?}"),
        }

        let deleted = format!(
            r#"{{ "type": "customer.subscription.deleted",
                 "data": {{ "object": {{ "metadata": {{ "npub": "{npub}" }} }} }} }}"#
        );
        match member_from_event(&deleted, 1_700_000_000).unwrap() {
            StripeOutcome::Revoke { npub: revoked } => assert_eq!(revoked, npub),
            other => panic!("expected a revoke, got {other:?}"),
        }

        let canceled = format!(
            r#"{{ "type": "customer.subscription.updated",
                 "data": {{ "object": {{ "status": "canceled", "metadata": {{ "npub": "{npub}" }} }} }} }}"#
        );
        assert!(matches!(
            member_from_event(&canceled, 0).unwrap(),
            StripeOutcome::Revoke { .. }
        ));
    }

    #[test]
    fn unrelated_events_are_ignored() {
        let payload = r#"{"type":"invoice.paid","data":{"object":{}}}"#;
        match member_from_event(payload, 0).unwrap() {
            StripeOutcome::Ignore { reason } => assert!(reason.contains("invoice.paid")),
            other => panic!("expected ignore, got {other:?}"),
        }
    }

    #[test]
    fn missing_npub_is_an_error() {
        let payload = r#"{"type":"checkout.session.completed","data":{"object":{}}}"#;
        assert!(member_from_event(payload, 0).is_err());
    }
}
