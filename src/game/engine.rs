//! 60 FPS ASCII Mario Physics & Game State Engine in 100% Rust

use super::entities::{Direction, Enemy, EnemyType, Fireball, Item, ItemType, MarioForm, Player, TileType};
use super::level::{LevelData, LEVEL_HEIGHT};
use super::cheat::CheatState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub fire: bool,
    pub crouch: bool,
    pub run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    Playing,
    Paused,
    Dead,
    LevelClear,
    GameOver,
    GameWon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEngine {
    pub player: Player,
    pub level: LevelData,
    pub fireballs: Vec<Fireball>,
    pub camera_x: f32,
    pub status: GameStatus,
    pub status_timer: u32,
    pub tick_count: u64,
    pub cheats: CheatState,
    pub sound_events: Vec<String>,
}

impl GameEngine {
    pub fn new(world: u32, level: u32) -> Self {
        let level_data = LevelData::load_level(world, level);
        let mut player = Player::default();
        player.world = world;
        player.level = level;

        Self {
            player,
            level: level_data,
            fireballs: Vec::new(),
            camera_x: 0.0,
            status: GameStatus::Playing,
            status_timer: 0,
            tick_count: 0,
            cheats: CheatState::default(),
            sound_events: Vec::new(),
        }
    }

    pub fn tick(&mut self, input: &InputState) {
        if self.status != GameStatus::Playing {
            self.status_timer += 1;
            if self.status == GameStatus::Dead && self.status_timer > 90 {
                if self.player.lives > 1 {
                    self.player.lives -= 1;
                    self.restart_level();
                } else {
                    self.status = GameStatus::GameOver;
                }
            } else if self.status == GameStatus::LevelClear && self.status_timer > 120 {
                self.next_level();
            }
            return;
        }

        self.tick_count += 1;
        self.sound_events.clear();

        // 1. Time countdown
        if self.tick_count % 60 == 0 && self.player.time_left > 0 {
            self.player.time_left -= 1;
            if self.player.time_left == 0 && !self.cheats.god_mode {
                self.kill_player();
                return;
            }
        }

        // 2. Handle Player Input & Physics
        self.update_player_movement(input);

        // 3. Update Fireballs
        self.update_fireballs();

        // 4. Update Enemies
        self.update_enemies();

        // 5. Update Items
        self.update_items();

        // 6. Camera follows player
        let target_cam = (self.player.x - 12.0).max(0.0).min((self.level.width as f32) - 40.0);
        if target_cam > self.camera_x {
            self.camera_x = target_cam;
        }

        // 7. Check if player fell into pit
        if self.player.y > 14.5 {
            if self.cheats.god_mode {
                self.player.y = 12.0;
                self.player.vy = 0.0;
            } else {
                self.kill_player();
            }
        }
    }

    fn update_player_movement(&mut self, input: &InputState) {
        let max_speed = if input.run { 0.22 } else { 0.14 };
        let accel = 0.02;
        let friction = 0.85;
        let gravity = if self.cheats.moon_jump { 0.015 } else { 0.035 };

        if input.left {
            self.player.vx -= accel;
            self.player.direction = Direction::Left;
        } else if input.right {
            self.player.vx += accel;
            self.player.direction = Direction::Right;
        } else {
            self.player.vx *= friction;
            if self.player.vx.abs() < 0.005 {
                self.player.vx = 0.0;
            }
        }

        self.player.vx = self.player.vx.clamp(-max_speed, max_speed);

        // Jumping
        if input.jump && self.player.is_on_ground && !self.player.is_jumping {
            let jump_power = if self.cheats.moon_jump { -0.58 } else { -0.48 };
            self.player.vy = jump_power;
            self.player.is_on_ground = false;
            self.player.is_jumping = true;
            self.sound_events.push("jump".to_string());
        }

        if !input.jump {
            self.player.is_jumping = false;
            if self.player.vy < -0.15 {
                self.player.vy = -0.15; // Variable jump height
            }
        }

        // Apply Gravity
        self.player.vy += gravity;
        self.player.vy = self.player.vy.min(0.5);

        // Horizontal Movement & Collision
        let new_x = self.player.x + self.player.vx;
        if !self.check_tile_collision(new_x, self.player.y, self.player.height()) {
            self.player.x = new_x.max(self.camera_x);
        } else {
            self.player.vx = 0.0;
        }

        // Vertical Movement & Collision
        let new_y = self.player.y + self.player.vy;
        if self.player.vy < 0.0 {
            // Moving UP: Head Collision
            let head_y = new_y - self.player.height();
            if self.check_tile_collision(self.player.x, head_y, 0.2) {
                self.player.vy = 0.0;
                let tile_x = self.player.x.round() as usize;
                let tile_y = head_y.round() as usize;
                self.bump_block(tile_x, tile_y);
            } else {
                self.player.y = new_y;
                self.player.is_on_ground = false;
            }
        } else {
            // Moving DOWN: Ground Collision
            if self.check_tile_collision(self.player.x, new_y, 0.2) {
                self.player.y = new_y.floor();
                self.player.vy = 0.0;
                self.player.is_on_ground = true;
            } else {
                self.player.y = new_y;
                self.player.is_on_ground = false;
            }
        }

        // Invincibility Timer
        if self.player.invincibility_timer > 0 {
            self.player.invincibility_timer -= 1;
        }

        // Fireball Shooting
        if input.fire && (self.player.form == MarioForm::Fire || self.cheats.infinite_fireballs) {
            if self.fireballs.len() < 3 || self.cheats.infinite_fireballs {
                let f_vx = match self.player.direction {
                    Direction::Right => 0.35,
                    Direction::Left => -0.35,
                };
                self.fireballs.push(Fireball {
                    x: self.player.x + if self.player.direction == Direction::Right { 1.0 } else { -0.5 },
                    y: self.player.y - 0.5,
                    vx: f_vx,
                    vy: 0.1,
                    is_active: true,
                    is_enemy_fire: false,
                });
                self.sound_events.push("fireball".to_string());
            }
        }
    }

    fn check_tile_collision(&self, x: f32, y: f32, height: f32) -> bool {
        let ix = x.round() as i32;
        let iy_bottom = y.round() as i32;
        let iy_top = (y - height).round() as i32;

        for iy in iy_top..=iy_bottom {
            if ix >= 0 && ix < self.level.width as i32 && iy >= 0 && iy < LEVEL_HEIGHT as i32 {
                let tile = self.level.tiles[iy as usize][ix as usize];
                if tile.is_solid() {
                    return true;
                }
            }
        }
        false
    }

    fn bump_block(&mut self, tile_x: usize, tile_y: usize) {
        if tile_x >= self.level.width || tile_y >= LEVEL_HEIGHT {
            return;
        }
        match self.level.tiles[tile_y][tile_x] {
            TileType::MysteryBox { item, is_opened: false } => {
                self.level.tiles[tile_y][tile_x] = TileType::MysteryBox { item, is_opened: true };
                self.sound_events.push("powerup_appear".to_string());
                match item {
                    ItemType::Coin => {
                        self.player.coins += 1;
                        self.player.score += 200;
                        self.sound_events.push("coin".to_string());
                    }
                    _ => {
                        self.level.items.push(Item {
                            x: tile_x as f32,
                            y: (tile_y - 1) as f32,
                            vx: 0.05,
                            vy: -0.15,
                            item_type: item,
                            is_collected: false,
                        });
                    }
                }
            }
            TileType::Brick => {
                if self.player.form != MarioForm::Small || self.cheats.god_mode {
                    self.level.tiles[tile_y][tile_x] = TileType::Air;
                    self.player.score += 50;
                    self.sound_events.push("break_block".to_string());
                } else {
                    self.sound_events.push("bump".to_string());
                }
            }
            TileType::AxeSwitch => {
                // Bowser Castle Bridge Drop!
                self.sound_events.push("bowser_fall".to_string());
                for x in 70..84 {
                    self.level.tiles[11][x] = TileType::Air;
                }
                self.level.tiles[tile_y][tile_x] = TileType::Air;
                self.player.score += 5000;
                self.status = GameStatus::GameWon;
                self.sound_events.push("world_clear".to_string());
            }
            _ => {}
        }
    }

    fn update_fireballs(&mut self) {
        for fb in self.fireballs.iter_mut().filter(|f| f.is_active) {
            fb.x += fb.vx;
            fb.y += fb.vy;
            fb.vy += 0.02; // Bouncing gravity

            if fb.y > 12.0 {
                fb.y = 12.0;
                fb.vy = -0.15; // Bounce off floor
            }

            if fb.x < self.camera_x || fb.x > self.camera_x + 45.0 {
                fb.is_active = false;
            }
        }
        self.fireballs.retain(|f| f.is_active);
    }

    fn update_enemies(&mut self) {
        let player_x = self.player.x;
        let player_y = self.player.y;
        let player_h = self.player.height();

        for enemy in self.level.enemies.iter_mut().filter(|e| e.is_alive) {
            if enemy.is_stomped {
                enemy.stomp_timer += 1;
                if enemy.stomp_timer > 30 {
                    enemy.is_alive = false;
                }
                continue;
            }

            // Move
            enemy.x += enemy.vx;
            enemy.y += enemy.vy;

            // Bowser Special AI
            if let EnemyType::Bowser { ref mut hp, ref mut attack_timer } = enemy.enemy_type {
                *attack_timer = attack_timer.saturating_sub(1);
                if *attack_timer == 0 {
                    *attack_timer = 90;
                    // Bowser jumps and spits fire
                    enemy.vy = -0.2;
                }
                enemy.vy += 0.015;
                if enemy.y > 10.0 {
                    enemy.y = 10.0;
                    enemy.vy = 0.0;
                }
            }

            // Check Collision with Fireballs
            for fb in self.fireballs.iter_mut().filter(|f| f.is_active) {
                if (fb.x - enemy.x).abs() < 1.0 && (fb.y - enemy.y).abs() < 1.2 {
                    fb.is_active = false;
                    enemy.is_alive = false;
                    self.player.score += 200;
                    self.sound_events.push("stomp".to_string());
                }
            }

            // Check Collision with Mario
            let dx = (player_x - enemy.x).abs();
            let dy = (player_y - enemy.y).abs();

            if dx < 0.9 && dy < 1.2 {
                if self.player.vy > 0.0 && player_y < enemy.y + 0.3 {
                    // Stomp enemy!
                    enemy.is_stomped = true;
                    self.player.vy = -0.3; // Bounce up
                    self.player.score += 100;
                    self.sound_events.push("stomp".to_string());
                } else if self.player.invincibility_timer == 0 && !self.cheats.god_mode {
                    // Mario hit by enemy
                    match self.player.form {
                        MarioForm::Small => {
                            self.kill_player();
                        }
                        MarioForm::Super | MarioForm::Fire => {
                            self.player.form = MarioForm::Small;
                            self.player.invincibility_timer = 120;
                            self.sound_events.push("pipe".to_string());
                        }
                        MarioForm::Star => {
                            enemy.is_alive = false;
                            self.player.score += 200;
                        }
                    }
                }
            }
        }

        // Flagpole Check (World 1-1 completion)
        if self.player.x >= 100.0 && self.player.world == 1 && self.player.level == 1 {
            self.status = GameStatus::LevelClear;
            self.sound_events.push("flagpole".to_string());
        }
    }

    fn update_items(&mut self) {
        let player_x = self.player.x;
        let player_y = self.player.y;

        for item in self.level.items.iter_mut().filter(|i| !i.is_collected) {
            item.x += item.vx;
            item.y += item.vy;
            item.vy += 0.01;
            if item.y > 12.0 {
                item.y = 12.0;
                item.vy = 0.0;
            }

            if (player_x - item.x).abs() < 1.0 && (player_y - item.y).abs() < 1.2 {
                item.is_collected = true;
                self.sound_events.push("powerup".to_string());
                match item.item_type {
                    ItemType::SuperMushroom => {
                        if self.player.form == MarioForm::Small {
                            self.player.form = MarioForm::Super;
                        }
                        self.player.score += 1000;
                    }
                    ItemType::FireFlower => {
                        self.player.form = MarioForm::Fire;
                        self.player.score += 1000;
                    }
                    ItemType::StarMan => {
                        self.player.form = MarioForm::Star;
                        self.player.invincibility_timer = 300;
                        self.player.score += 1000;
                    }
                    ItemType::Coin => {
                        self.player.coins += 1;
                        self.player.score += 200;
                    }
                    ItemType::OneUpMushroom => {
                        self.player.lives += 1;
                        self.sound_events.push("1up".to_string());
                    }
                }
            }
        }
    }

    pub fn kill_player(&mut self) {
        self.player.is_dead = true;
        self.status = GameStatus::Dead;
        self.status_timer = 0;
        self.sound_events.push("gameover".to_string());
    }

    pub fn restart_level(&mut self) {
        let current_world = self.player.world;
        let current_level = self.player.level;
        let current_lives = self.player.lives;
        let current_score = self.player.score;
        let current_coins = self.player.coins;

        *self = Self::new(current_world, current_level);
        self.player.lives = current_lives;
        self.player.score = current_score;
        self.player.coins = current_coins;
    }

    pub fn next_level(&mut self) {
        let mut next_w = self.player.world;
        let mut next_l = self.player.level + 1;
        if next_l > 4 {
            next_w += 1;
            next_l = 1;
        }
        let current_lives = self.player.lives;
        let current_score = self.player.score;
        let current_coins = self.player.coins;

        *self = Self::new(next_w, next_l);
        self.player.lives = current_lives;
        self.player.score = current_score;
        self.player.coins = current_coins;
    }
}
