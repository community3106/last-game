//! Whitelist Management for Paid Subscriptions & Admin Privileges
//! Maps Stripe customer payments to Nostr npub public keys.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Free,       // Default: Access to World 1-1, 1-2
    ProGamer,   // $4.99/mo: Access to World 1-3, Custom Color Themes, Nostr Highscores
    LegendaryVip,// $9.99/mo: Access to World 1-4 Bowser Castle, Secret Stage, VIP Cheats
    AdminGod,   // Owner: Full God Mode, Level Editor, Instant Whitelist control
}

impl SubscriptionTier {
    pub fn display_name(&self) -> &'static str {
        match self {
            SubscriptionTier::Free => "無料プラン (Free Player)",
            SubscriptionTier::ProGamer => "Pro Nostr Gamer ($4.99/月)",
            SubscriptionTier::LegendaryVip => "Legendary 1-Up VIP ($9.99/月)",
            SubscriptionTier::AdminGod => "管理者特権 (Admin God Mode)",
        }
    }

    pub fn badge_color(&self) -> &'static str {
        match self {
            SubscriptionTier::Free => "#888888",
            SubscriptionTier::ProGamer => "#00d2ff",
            SubscriptionTier::LegendaryVip => "#ffaa00",
            SubscriptionTier::AdminGod => "#ff0055",
        }
    }

    pub fn can_play_world(&self, world: u32, level: u32) -> bool {
        match (world, level) {
            (1, 1) | (1, 2) => true, // Everyone can play 1-1 and 1-2
            (1, 3) => *self >= SubscriptionTier::ProGamer,
            (1, 4) => *self >= SubscriptionTier::LegendaryVip,
            _ => *self >= SubscriptionTier::LegendaryVip,
        }
    }

    pub fn has_god_mode(&self) -> bool {
        *self == SubscriptionTier::AdminGod
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub npub: String,
    pub pubkey_hex: String,
    pub tier: SubscriptionTier,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub activated_at: u64,
    pub expires_at: Option<u64>,
    pub is_active: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistManager {
    pub admin_pubkey: String,
    pub entries: Vec<WhitelistEntry>,
}

impl Default for WhitelistManager {
    fn default() -> Self {
        let mut manager = Self {
            admin_pubkey: "3bf0c63fcb93463407af97b5e0918838bce40f29c164a521551232443e3aa116".to_string(),
            entries: Vec::new(),
        };

        // Seed with Default Admin
        manager.add_or_update(
            "npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64qv6spqqq28cfm",
            "3bf0c63fcb93463407af97b5e0918838bce40f29c164a521551232443e3aa116",
            SubscriptionTier::AdminGod,
            Some("cus_admin_root".to_string()),
            Some("sub_admin_lifetime".to_string()),
            "Primary Server Administrator".to_string(),
        );

        // Seed with Demo VIP User
        manager.add_or_update(
            "npub10pensourcerocks777nostrgamer999legendaryvipclubmember001",
            "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            SubscriptionTier::LegendaryVip,
            Some("cus_stripe_vip_001".to_string()),
            Some("sub_stripe_vip_annual".to_string()),
            "Stripe Verified Legendary VIP Subscriber".to_string(),
        );

        // Seed with Pro Gamer
        manager.add_or_update(
            "npub1speedrunner2026rustnostrgamerpro420marioarcadechampion",
            "4b4238e8893b4f7a1262cb21f39f02ba5c413f02123456789abcdef012345678",
            SubscriptionTier::ProGamer,
            Some("cus_stripe_pro_042".to_string()),
            Some("sub_stripe_pro_monthly".to_string()),
            "Stripe Pro Gamer Subscriber".to_string(),
        );

        manager
    }
}

impl WhitelistManager {
    pub fn add_or_update(
        &mut self,
        npub: &str,
        pubkey_hex: &str,
        tier: SubscriptionTier,
        stripe_customer_id: Option<String>,
        stripe_subscription_id: Option<String>,
        notes: String,
    ) {
        if let Some(existing) = self.entries.iter_mut().find(|e| e.npub == npub || e.pubkey_hex == pubkey_hex) {
            existing.tier = tier;
            existing.stripe_customer_id = stripe_customer_id;
            existing.stripe_subscription_id = stripe_subscription_id;
            existing.is_active = true;
            existing.notes = notes;
        } else {
            self.entries.push(WhitelistEntry {
                npub: npub.to_string(),
                pubkey_hex: pubkey_hex.to_string(),
                tier,
                stripe_customer_id,
                stripe_subscription_id,
                activated_at: 1728000000,
                expires_at: None,
                is_active: true,
                notes,
            });
        }
    }

    pub fn get_user_tier(&self, npub_or_hex: &str) -> SubscriptionTier {
        if npub_or_hex.is_empty() {
            return SubscriptionTier::Free;
        }
        for entry in &self.entries {
            if entry.is_active && (entry.npub == npub_or_hex || entry.pubkey_hex == npub_or_hex) {
                return entry.tier;
            }
        }
        SubscriptionTier::Free
    }

    pub fn is_whitelisted_for_vip(&self, npub_or_hex: &str) -> bool {
        let tier = self.get_user_tier(npub_or_hex);
        tier >= SubscriptionTier::LegendaryVip
    }

    pub fn revoke(&mut self, npub_or_hex: &str) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.npub == npub_or_hex || e.pubkey_hex == npub_or_hex) {
            entry.is_active = false;
            true
        } else {
            false
        }
    }
}
