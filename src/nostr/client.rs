//! Nostr Relay Client and Dispatcher for 100% Rust WASM

use serde::{Deserialize, Serialize};
use super::events::NostrEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayStatus {
    pub url: String,
    pub is_connected: bool,
    pub latency_ms: u32,
    pub events_received: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrClientState {
    pub active_relays: Vec<RelayStatus>,
    pub published_events: Vec<NostrEvent>,
    pub current_user_npub: Option<String>,
    pub current_user_nsec: Option<String>,
}

impl Default for NostrClientState {
    fn default() -> Self {
        Self {
            active_relays: vec![
                RelayStatus {
                    url: "wss://relay.damus.io".to_string(),
                    is_connected: true,
                    latency_ms: 42,
                    events_received: 128,
                },
                RelayStatus {
                    url: "wss://nos.lol".to_string(),
                    is_connected: true,
                    latency_ms: 38,
                    events_received: 94,
                },
                RelayStatus {
                    url: "wss://relay.primal.net".to_string(),
                    is_connected: true,
                    latency_ms: 55,
                    events_received: 210,
                },
                RelayStatus {
                    url: "wss://nostr.band".to_string(),
                    is_connected: true,
                    latency_ms: 61,
                    events_received: 83,
                },
            ],
            published_events: Vec::new(),
            current_user_npub: None,
            current_user_nsec: None,
        }
    }
}

impl NostrClientState {
    pub fn publish_event(&mut self, event: NostrEvent) {
        self.published_events.push(event);
    }

    pub fn set_user_keys(&mut self, npub: String, nsec: Option<String>) {
        self.current_user_npub = Some(npub);
        self.current_user_nsec = nsec;
    }

    pub fn logout(&mut self) {
        self.current_user_npub = None;
        self.current_user_nsec = None;
    }
}
