//! Stripe Webhook Handler for Nostr Public Key Whitelist Automation
//! When a user pays via Stripe, their Nostr npub is extracted from metadata,
//! granted VIP status, and given encrypted access keys via NIP-04/NIP-44.

use serde::{Deserialize, Serialize};
use crate::nostr::whitelist::{SubscriptionTier, WhitelistManager};
use crate::nostr::crypto::{nip04_encrypt, nip44_encrypt};
use crate::nostr::events::NostrEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeCheckoutSession {
    pub id: String,
    pub customer_id: String,
    pub customer_email: String,
    pub payment_status: String,
    pub amount_total: u64,
    pub currency: String,
    pub client_reference_id: Option<String>,
    pub metadata: StripeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeMetadata {
    pub nostr_npub: String,
    pub nostr_pubkey_hex: Option<String>,
    pub plan_tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookProcessResult {
    pub success: bool,
    pub npub_whitelisted: String,
    pub granted_tier: SubscriptionTier,
    pub encrypted_event_nip04: NostrEvent,
    pub encrypted_event_nip44: NostrEvent,
    pub message: String,
}

pub fn handle_stripe_checkout_completed(
    session: StripeCheckoutSession,
    whitelist: &mut WhitelistManager,
    admin_privkey_hex: &str,
    admin_pubkey_hex: &str,
) -> Result<WebhookProcessResult, String> {
    if session.payment_status != "paid" {
        return Err("Payment status is not paid".to_string());
    }

    let npub = &session.metadata.nostr_npub;
    if npub.is_empty() {
        return Err("Missing nostr_npub in Stripe checkout metadata".to_string());
    }

    let tier = match session.metadata.plan_tier.as_str() {
        "pro" => SubscriptionTier::ProGamer,
        "vip" | "legendary" => SubscriptionTier::LegendaryVip,
        "admin" => SubscriptionTier::AdminGod,
        _ => SubscriptionTier::ProGamer,
    };

    let user_pubkey = session.metadata.nostr_pubkey_hex.clone().unwrap_or_else(|| {
        if npub.len() > 10 {
            format!("{:0<64}", &npub[npub.len() - 10..])
        } else {
            "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798".to_string()
        }
    });

    // 1. Add to Whitelist
    whitelist.add_or_update(
        npub,
        &user_pubkey,
        tier,
        Some(session.customer_id.clone()),
        Some(session.id.clone()),
        format!("Stripe payment: {} {} (Session: {})", session.amount_total, session.currency, session.id),
    );

    // 2. Encrypt VIP Stage Access Data for User's Public Key
    let secret_vip_payload = serde_json::json!({
        "status": "VIP_ACCESS_GRANTED",
        "tier": tier.display_name(),
        "unlocked_worlds": ["1-3", "1-4", "SECRET-WARP-ZONE"],
        "vip_cheat_codes": {
            "moon_jump": "ENABLED",
            "infinite_fireballs": "ENABLED",
            "god_mode": if tier == SubscriptionTier::AdminGod { "ENABLED" } else { "DISABLED" },
        },
        "bowser_castle_map_key": "CIPHER_KEY_WORLD_1_4_BOWSER_FIRE_DRAGON_1985",
        "timestamp": 1728648000,
    }).to_string();

    let nip04_cipher = nip04_encrypt(&secret_vip_payload, admin_privkey_hex, &user_pubkey)
        .map_err(|e| format!("NIP-04 encryption error: {}", e))?;

    let nip44_cipher = nip44_encrypt(&secret_vip_payload, admin_privkey_hex, &user_pubkey)
        .map_err(|e| format!("NIP-44 encryption error: {}", e))?;

    let event_04 = NostrEvent::create_encrypted_vip_stage(admin_pubkey_hex, &user_pubkey, &nip04_cipher, false);
    let event_44 = NostrEvent::create_encrypted_vip_stage(admin_pubkey_hex, &user_pubkey, &nip44_cipher, true);

    Ok(WebhookProcessResult {
        success: true,
        npub_whitelisted: npub.clone(),
        granted_tier: tier,
        encrypted_event_nip04: event_04,
        encrypted_event_nip44: event_44,
        message: format!("Successfully whitelisted {} with tier {}", npub, tier.display_name()),
    })
}
