use last_game::nostr::whitelist::{SubscriptionTier, WhitelistManager};
use last_game::stripe::webhook::{handle_stripe_checkout_completed, StripeCheckoutSession, StripeMetadata};
use last_game::nostr::crypto::KeyPair;

#[test]
fn test_whitelist_tier_permissions() {
    let free_tier = SubscriptionTier::Free;
    let pro_tier = SubscriptionTier::ProGamer;
    let vip_tier = SubscriptionTier::LegendaryVip;

    assert!(free_tier.can_play_world(1, 1));
    assert!(free_tier.can_play_world(1, 2));
    assert!(!free_tier.can_play_world(1, 3));
    assert!(!free_tier.can_play_world(1, 4));

    assert!(pro_tier.can_play_world(1, 3));
    assert!(!pro_tier.can_play_world(1, 4));

    assert!(vip_tier.can_play_world(1, 4));
}

#[test]
fn test_stripe_webhook_whitelist_flow() {
    let mut whitelist = WhitelistManager::default();
    let admin = KeyPair::default_admin();

    let new_user_npub = "npub1testpaiduser2026stripepaymentverified";
    let session = StripeCheckoutSession {
        id: "cs_test_abc123".to_string(),
        customer_id: "cus_test_user_777".to_string(),
        customer_email: "gamer@nostr.org".to_string(),
        payment_status: "paid".to_string(),
        amount_total: 999,
        currency: "usd".to_string(),
        client_reference_id: None,
        metadata: StripeMetadata {
            nostr_npub: new_user_npub.to_string(),
            nostr_pubkey_hex: None,
            plan_tier: "vip".to_string(),
        },
    };

    let result = handle_stripe_checkout_completed(
        session,
        &mut whitelist,
        &admin.secret_key_hex,
        &admin.public_key_hex,
    ).expect("Stripe webhook processing should succeed");

    assert!(result.success);
    assert_eq!(result.granted_tier, SubscriptionTier::LegendaryVip);
    assert!(whitelist.is_whitelisted_for_vip(new_user_npub));
}
