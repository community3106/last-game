//! 100% Rust ASCII Mario Engine — DOM非依存
//! Leptos からは `GameState::tick(InputState)` を 60FPS で呼ぶだけ。

use crate::game::level::Level;
use serde::{Deserialize, Serialize};

// ---- 定数 ----
pub const GRAVITY: f32 = 0.55;
pub const JUMP_VEL: f32 = -10.5;
pub const MOVE_SPEED: f32 = 2.2;
pub const FRICTION: f32 = 0.85;
pub const MAX_FALL: f32 = 12.0;
pub const TILE_SIZE: f32 = 1.0; // 論理タイル = 1.0

// 画面サイズ (文字数)
pub const VIEW_W: usize = 40;
pub const VIEW_H: usize = 18;

// ---- 入力 ----
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
}

// ---- エンティティ ----
#[derive(Debug, Clone)]
pub struct Coin {
    pub x: i32,
    pub y: i32,
    pub taken: bool,
}
#[derive(Debug, Clone)]
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub dir: f32, // -1 or 1
    pub alive: bool,
}

// ---- ゲーム状態 ----
#[derive(Debug, Clone)]
pub struct GameState {
    pub player_x: f32,
    pub player_y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub on_ground: bool,
    pub score: u32,
    pub coins: u32,
    pub lives: i32,
    pub won: bool,
    pub game_over: bool,
    pub coins_entities: Vec<Coin>,
    pub enemies: Vec<Enemy>,
    pub level: Level,
    pub cam_x: f32,
}

impl GameState {
    pub fn new(level: Level) -> Self {
        let mut s = Self {
            player_x: level.player_spawn.0,
            player_y: level.player_spawn.1,
            vel_x: 0.0,
            vel_y: 0.0,
            on_ground: false,
            score: 0,
            coins: 0,
            lives: 3,
            won: false,
            game_over: false,
            coins_entities: vec![],
            enemies: vec![],
            level,
            cam_x: 0.0,
        };
        s.spawn_entities();
        s
    }

    pub fn reset(&mut self, level: Level) {
        *self = Self::new(level);
    }

    fn spawn_entities(&mut self) {
        self.coins_entities.clear();
        self.enemies.clear();
        for y in 0..self.level.height {
            for x in 0..self.level.width {
                match self.level.tile_at(x as i32, y as i32) {
                    4 => self.coins_entities.push(Coin { x: x as i32, y: y as i32, taken: false }),
                    5 => self.enemies.push(Enemy { x: x as f32 + 0.5, y: y as f32, dir: -1.0, alive: true }),
                    _ => {}
                }
            }
        }
    }

    pub fn tick(&mut self, input: InputState) {
        if self.won || self.game_over {
            return;
        }

        // 横移動
        if input.left && !input.right {
            self.vel_x = -MOVE_SPEED;
        } else if input.right && !input.left {
            self.vel_x = MOVE_SPEED;
        } else {
            self.vel_x *= FRICTION;
            if self.vel_x.abs() < 0.05 { self.vel_x = 0.0; }
        }

        // ジャンプ (地上のみ)
        if input.jump && self.on_ground {
            self.vel_y = JUMP_VEL;
            self.on_ground = false;
        }

        // 重力
        self.vel_y += GRAVITY;
        if self.vel_y > MAX_FALL { self.vel_y = MAX_FALL; }

        // 移動 + 衝突 (簡易AABB: 1x1のプレイヤーをタイルと衝突)
        self.move_and_collide();

        // カメラ追従
        self.cam_x = (self.player_x - VIEW_W as f32 / 2.5).clamp(0.0, (self.level.width as f32 - VIEW_W as f32).max(0.0));

        // コイン取得
        for c in &mut self.coins_entities {
            if !c.taken && (c.x as f32 - self.player_x).abs() < 0.7 && (c.y as f32 - self.player_y).abs() < 0.7 {
                c.taken = true;
                self.coins += 1;
                self.score += 100;
            }
        }

        // 敵
        for e in &mut self.enemies {
            if !e.alive { continue; }
            e.x += e.dir * 0.6;
            // 壁で反転
            let tx = (e.x + e.dir * 0.6) as i32;
            let ty = e.y as i32;
            if self.level.is_solid(tx, ty) || self.level.is_solid(tx, ty+1) {
                e.dir *= -1.0;
            }
            // 踏みつけ判定
            let dx = (e.x - self.player_x).abs();
            let dy = e.y - self.player_y;
            if dx < 0.8 && dy.abs() < 0.9 {
                if self.vel_y > 0.0 && dy > 0.1 {
                    // 踏んだ
                    e.alive = false;
                    self.vel_y = JUMP_VEL * 0.55;
                    self.score += 200;
                } else {
                    // ダメージ
                    self.lives -= 1;
                    if self.lives <= 0 {
                        self.game_over = true;
                    } else {
                        // リスポーン
                        self.player_x = self.level.player_spawn.0;
                        self.player_y = self.level.player_spawn.1;
                        self.vel_x = 0.0;
                        self.vel_y = 0.0;
                    }
                }
            }
        }

        // ゴール判定 (tile 9)
        let px = self.player_x as i32;
        let py = self.player_y as i32;
        if self.level.tile_at(px, py) == 9 || self.level.tile_at(px, py+1) == 9 {
            self.won = true;
            self.score += 1000;
        }

        // 落下死
        if self.player_y > self.level.height as f32 + 5.0 {
            self.lives -= 1;
            if self.lives <= 0 {
                self.game_over = true;
            } else {
                self.player_x = self.level.player_spawn.0;
                self.player_y = self.level.player_spawn.1;
                self.vel_x = 0.0;
                self.vel_y = 0.0;
            }
        }
    }

    fn move_and_collide(&mut self) {
        // X
        let next_x = self.player_x + self.vel_x * 0.18;
        let y = self.player_y as i32;
        let nx = next_x as i32;
        // 足元と頭の2点で衝突
        if !self.level.is_solid(nx, y) && !self.level.is_solid(nx, y+1) && !self.level.is_solid(nx, (self.player_y+0.9) as i32) {
            self.player_x = next_x;
        } else {
            self.vel_x = 0.0;
        }
        // Y
        let next_y = self.player_y + self.vel_y * 0.12;
        let x = self.player_x as i32;
        let ny = next_y as i32;
        let head_solid = self.level.is_solid(x, ny);
        let feet_solid = self.level.is_solid(x, (next_y+1.0) as i32);
        let next_feet_solid = self.level.is_solid(x, (next_y+1.1) as i32);

        if self.vel_y < 0.0 {
            // 上昇中: 頭がぶつかったら停止
            if head_solid {
                self.vel_y = 0.0;
                // ブロックを叩いたらコイン化 (タイル2)
                if self.level.tile_at(x, ny) == 2 {
                    // 簡易: ブロックを空にしてコイン加算
                    // 注: Level は clone なので直接書き換え可能
                    if ny >= 0 && (ny as usize) < self.level.height && x >=0 {
                        self.level.tiles[ny as usize][x as usize] = 0;
                        self.coins += 1;
                        self.score += 50;
                    }
                }
            } else {
                self.player_y = next_y;
                self.on_ground = false;
            }
        } else {
            // 下降中
            if feet_solid || next_feet_solid {
                // 着地
                self.player_y = (ny as f32).floor();
                // 地面の上に立たせる
                // feet の1つ上
                // 正確にはタイルの上面に合わせる
                // 簡易: yをタイル境界にスナップ
                let ground_y = (next_y+1.0).floor() - 1.0;
                if self.player_y > ground_y { self.player_y = ground_y; }
                self.vel_y = 0.0;
                self.on_ground = true;
            } else {
                self.player_y = next_y;
                self.on_ground = false;
            }
        }

        // 画面外クランプ
        self.player_x = self.player_x.clamp(0.5, self.level.width as f32 - 0.5);
    }
}

// ---- レベル定義 (静的) ----
pub const LEVEL_1_STR: &str = r#"{
  "name": "1-1",
  "width": 80,
  "height": 16,
  "player_spawn": [2.0, 10.0],
  "bg": "sky",
  "tiles": []
}"#;

// 実体はコードで生成 (JSONだと長すぎるため)
pub fn make_level_1() -> Level {
    let w = 80;
    let h = 16;
    let mut tiles = vec![vec![0u8; w]; h];
    // 地面
    for x in 0..w {
        tiles[h-2][x] = 1;
        tiles[h-1][x] = 1;
    }
    // 穴を2つ
    tiles[h-2][18] = 0; tiles[h-1][18] = 0;
    tiles[h-2][19] = 0; tiles[h-1][19] = 0;
    tiles[h-2][45] = 0; tiles[h-1][45] = 0;
    tiles[h-2][46] = 0; tiles[h-1][46] = 0;
    // ブロック
    for x in 6..10 { tiles[10][x] = 2; }
    tiles[10][7] = 4; // コイン
    tiles[9][8] = 4;
    for x in 14..17 { tiles[8][x] = 2; }
    tiles[8][15] = 4;
    for x in 22..26 { tiles[10][x] = 2; }
    for x in 30..34 { tiles[7][x] = 2; }
    tiles[7][31] = 4;
    // 土管
    tiles[12][26] = 3; tiles[13][26] = 3; tiles[12][27] = 3; tiles[13][27] = 3;
    tiles[11][50] = 3; tiles[12][50] = 3; tiles[13][50] = 3; tiles[11][51] = 3; tiles[12][51] = 3; tiles[13][51] = 3;
    // 敵
    tiles[13][12] = 5;
    tiles[13][32] = 5;
    tiles[13][55] = 5;
    // コイン散らし
    tiles[12][9] = 4; tiles[12][20] = 4; tiles[9][23] = 4; tiles[6][31] = 4;
    // ゴール
    tiles[10][74] = 9; tiles[11][74] = 9; tiles[12][74] = 9; tiles[13][74] = 9;
    tiles[9][74] = 2; // 旗の上
    Level { name: "1-1".into(), width: w, height: h, tiles, player_spawn: (2.0, 11.0), bg: "sky".into() }
}

pub fn make_level_2() -> Level {
    let mut l = make_level_1();
    l.name = "1-2 Premium - 星の洞窟".into();
    // 追加の洞窟ギミック
    for x in 10..15 { l.tiles[5][x] = 2; }
    for y in 6..12 { l.tiles[y][12] = 1; }
    l.tiles[6][20] = 4; l.tiles[6][21] = 4; l.tiles[5][20] = 2;
    l.tiles[13][40] = 5; l.tiles[13][41] = 5;
    l.player_spawn = (2.0, 11.0);
    l
}

pub static LEVEL_1: once_cell::sync::Lazy<Level> = once_cell::sync::Lazy::new(make_level_1);
pub static LEVEL_2: once_cell::sync::Lazy<Level> = once_cell::sync::Lazy::new(make_level_2);

/// 有料レベルが暗号化されていることを示すヒント (NIP-44の暗号文モック)
pub const LEVEL_2_ENCRYPTED_HINT: &str = r#"{
  "kind": 30078,
  "d": "level-2",
  "p": "<your-pubkey-hex>",
  "content": "nip44:AgEf...7X9z==  ← 復号しないと読めません",
  "hint": "このレベルは NIP-44 であなたの公開鍵向けに暗号化されています。/pricing で有料会員になり、nip44::decrypt で復号してください。"
}"#;
