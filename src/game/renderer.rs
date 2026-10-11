//! ASCII Retro Terminal & CRT Frame Buffer Renderer in 100% Rust

use super::engine::GameEngine;
use super::entities::{MarioForm, TileType};
use serde::{Deserialize, Serialize};

pub const SCREEN_WIDTH: usize = 48;
pub const SCREEN_HEIGHT: usize = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorTheme {
    ClassicNes,
    PhosphorGreen,
    Amber80s,
    CyberpunkNeon,
}

impl ColorTheme {
    pub fn display_name(&self) -> &'static str {
        match self {
            ColorTheme::ClassicNes => "Classic NES 8-Bit",
            ColorTheme::PhosphorGreen => "Matrix Phosphor Green",
            ColorTheme::Amber80s => "Amber CRT Terminal",
            ColorTheme::CyberpunkNeon => "Cyberpunk Synthwave Neon",
        }
    }
}

pub struct AsciiRenderer {
    pub theme: ColorTheme,
    pub crt_scanlines: bool,
    pub crt_glow: bool,
}

impl Default for AsciiRenderer {
    fn default() -> Self {
        Self {
            theme: ColorTheme::ClassicNes,
            crt_scanlines: true,
            crt_glow: true,
        }
    }
}

impl AsciiRenderer {
    pub fn render_frame_to_string(&self, engine: &GameEngine) -> String {
        let mut buffer = vec![vec![' '; SCREEN_WIDTH]; SCREEN_HEIGHT];
        let cam_x = engine.camera_x.round() as usize;

        // 1. Render Map Tiles
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let world_x = cam_x + x;
                if world_x < engine.level.width && y < engine.level.height {
                    let tile = engine.level.tiles[y][world_x];
                    buffer[y][x] = tile.ascii_char();
                }
            }
        }

        // 2. Render Items
        for item in &engine.level.items {
            if !item.is_collected {
                let ix = (item.x.round() as usize).saturating_sub(cam_x);
                let iy = item.y.round() as usize;
                if ix < SCREEN_WIDTH && iy < SCREEN_HEIGHT {
                    buffer[iy][ix] = match item.item_type {
                        super::entities::ItemType::SuperMushroom => 'M',
                        super::entities::ItemType::FireFlower => '*',
                        super::entities::ItemType::StarMan => '★',
                        super::entities::ItemType::Coin => '$',
                        super::entities::ItemType::OneUpMushroom => 'U',
                    };
                }
            }
        }

        // 3. Render Enemies
        for enemy in &engine.level.enemies {
            if enemy.is_alive {
                let ex = (enemy.x.round() as usize).saturating_sub(cam_x);
                let ey = enemy.y.round() as usize;
                if ex < SCREEN_WIDTH && ey < SCREEN_HEIGHT {
                    buffer[ey][ex] = if enemy.is_stomped { '_' } else { 'E' };
                }
            }
        }

        // 4. Render Fireballs
        for fb in &engine.fireballs {
            if fb.is_active {
                let fx = (fb.x.round() as usize).saturating_sub(cam_x);
                let fy = fb.y.round() as usize;
                if fx < SCREEN_WIDTH && fy < SCREEN_HEIGHT {
                    buffer[fy][fx] = 'o';
                }
            }
        }

        // 5. Render Player
        let px = (engine.player.x.round() as usize).saturating_sub(cam_x);
        let py = engine.player.y.round() as usize;
        if px < SCREEN_WIDTH && py < SCREEN_HEIGHT {
            buffer[py][px] = match engine.player.form {
                MarioForm::Small => 'm',
                MarioForm::Super => 'M',
                MarioForm::Fire => 'F',
                MarioForm::Star => '★',
            };
            if engine.player.form != MarioForm::Small && py > 0 {
                buffer[py - 1][px] = '^';
            }
        }

        // 6. Assemble Final Screen with Retro HUD
        let mut output = String::new();

        // Top HUD line 1
        output.push_str(&format!(
            "MARIO        COINS   WORLD   TIME   LIVES\n"
        ));
        // Top HUD line 2
        output.push_str(&format!(
            "{:06}        x{:02}     {}-{}    {:03}    x{:02}\n",
            engine.player.score,
            engine.player.coins,
            engine.player.world,
            engine.player.level,
            engine.player.time_left,
            engine.player.lives
        ));
        output.push_str(&"=".repeat(SCREEN_WIDTH));
        output.push('\n');

        // Playfield
        for row in buffer {
            let row_str: String = row.into_iter().collect();
            output.push_str(&row_str);
            output.push('\n');
        }

        output.push_str(&"=".repeat(SCREEN_WIDTH));
        output.push('\n');

        // Status Line
        if engine.cheats.god_mode {
            output.push_str("[GOD MODE ACTIVE] [NOSTR ADMIN VIP PRIVILEGE]\n");
        } else if engine.cheats.moon_jump {
            output.push_str("[VIP CHEAT: MOON JUMP ACTIVATED]\n");
        } else {
            output.push_str(&format!("STATUS: {:?} | Nostr ASCII Engine 100% Rust\n", engine.status));
        }

        output
    }
}
