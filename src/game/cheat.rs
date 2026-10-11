//! VIP & Admin Cheats for ASCII Mario

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheatState {
    pub god_mode: bool,
    pub moon_jump: bool,
    pub infinite_fireballs: bool,
    pub coin_magnet: bool,
    pub infinite_time: bool,
}

impl Default for CheatState {
    fn default() -> Self {
        Self {
            god_mode: false,
            moon_jump: false,
            infinite_fireballs: false,
            coin_magnet: false,
            infinite_time: false,
        }
    }
}
