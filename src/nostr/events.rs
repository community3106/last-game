//! Nostr Event Data Structures & Builders for ASCII Mario

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrEvent {
    pub id: String,
    pub pubkey: String,
    pub created_at: u64,
    pub kind: u64,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String,
}

impl NostrEvent {
    pub fn new(pubkey: &str, kind: u64, tags: Vec<Vec<String>>, content: &str) -> Self {
        let created_at = 1728648000;
        let id_seed = format!("{}:{}:{}:{}", pubkey, kind, created_at, content);
        let id = format!("{:x}", md5_or_simple_hash(&id_seed));
        let sig = format!("sig_{:x}_{:x}", md5_or_simple_hash(&id), 0x5a5a5a5au32);

        Self {
            id,
            pubkey: pubkey.to_string(),
            created_at,
            kind,
            tags,
            content: content.to_string(),
            sig,
        }
    }

    /// Creates a Kind 30078 high score event for ASCII Mario
    pub fn create_high_score_event(
        pubkey: &str,
        score: u32,
        world: u32,
        level: u32,
        coins: u32,
        time_left: u32,
    ) -> Self {
        let tags = vec![
            vec!["d".to_string(), "ascii_mario_highscore".to_string()],
            vec!["score".to_string(), score.to_string()],
            vec!["world".to_string(), format!("{}-{}", world, level)],
            vec!["coins".to_string(), coins.to_string()],
            vec!["time".to_string(), time_left.to_string()],
            vec!["client".to_string(), "leptos-ascii-mario-rust-100".to_string()],
        ];
        let content = serde_json::json!({
            "game": "ASCII Mario 1985 (100% Rust WASM)",
            "score": score,
            "world": format!("{}-{}", world, level),
            "coins": coins,
            "timestamp": created_at_now(),
        }).to_string();

        Self::new(pubkey, 30078, tags, &content)
    }

    /// Creates an encrypted VIP Stage Event (Kind 4 or Kind 1059)
    pub fn create_encrypted_vip_stage(
        sender_pubkey: &str,
        recipient_pubkey: &str,
        encrypted_stage_data: &str,
        is_nip44: bool,
    ) -> Self {
        let kind = if is_nip44 { 1059 } else { 4 };
        let tags = vec![
            vec!["p".to_string(), recipient_pubkey.to_string()],
            vec!["stage".to_string(), "world_1_4_bowser_castle".to_string()],
            vec!["type".to_string(), if is_nip44 { "nip44_v2" } else { "nip04" }.to_string()],
        ];

        Self::new(sender_pubkey, kind, tags, encrypted_stage_data)
    }
}

fn md5_or_simple_hash(input: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in input.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn created_at_now() -> u64 {
    1728648000
}
