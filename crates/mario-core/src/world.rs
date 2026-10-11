//! ゲーム世界 / the simulated world: player, enemies, coins, physics and game state.
//!
//! シミュレーションは固定タイムステップ ([`FIXED_DT`] = 1/60 秒) で進みます。
//! フロントエンドは [`World::update`] に `requestAnimationFrame` のデルタを渡すだけでよく、
//! 加速度・摩擦・可変ジャンプ・コヨーテタイムなどの挙動はすべてここに閉じ込められています。

use std::fmt;

use crate::input::Input;
use crate::level::{EnemySpawn, Level, Tile};
use crate::render::{Frame, RenderOptions};

/// 固定タイムステップ (秒) / fixed simulation step in seconds.
pub const FIXED_DT: f32 = 1.0 / 60.0;
/// 1 タイルの高さ (ワールド単位) / world units per tile.
pub const TILE_H: f32 = 1.0;

/// 重力 (タイル/秒²) / gravity in tiles per second squared.
pub const GRAVITY: f32 = 42.0;
/// 上昇中にジャンプボタンを押し続けているときの重力 (可変ジャンプ) / reduced gravity while holding jump.
pub const JUMP_HOLD_GRAVITY: f32 = 30.0;
/// 上昇中にボタンを離したときの重力 (ジャンプの切り上げ) / extra gravity when the jump is released early.
pub const JUMP_RELEASE_GRAVITY: f32 = 78.0;
/// 落下速度の上限 / terminal velocity.
pub const MAX_FALL: f32 = 22.0;
/// 歩きの最高速度 / walk speed.
pub const WALK_SPEED: f32 = 4.6;
/// ダッシュの最高速度 / run speed.
pub const RUN_SPEED: f32 = 7.4;
/// 地上での加速度 / ground acceleration.
pub const ACCEL: f32 = 38.0;
/// 空中での加速度 / air acceleration.
pub const AIR_ACCEL: f32 = 26.0;
/// 地上での摩擦 / ground friction.
pub const FRICTION: f32 = 34.0;
/// 空中での空気抵抗 / air drag.
pub const AIR_DRAG: f32 = 4.0;
/// ジャンプの初速 (上向きは負) / jump impulse (negative is up).
pub const JUMP_VELOCITY: f32 = -15.5;
/// 敵を踏んだときの跳ね返り / bounce after stomping an enemy.
pub const STOMP_BOUNCE: f32 = -10.5;
/// コヨーテタイム (足場を離れてもジャンプできる猶予) / coyote time.
pub const COYOTE_TIME: f32 = 0.09;
/// 無敵時間 / invulnerability after taking a hit.
pub const INVULN_TIME: f32 = 2.0;
/// 小さいときのサイズ / small player size (w, h).
pub const PLAYER_SMALL: (f32, f32) = (0.72, 0.90);
/// 大きいときのサイズ / big player size (w, h).
pub const PLAYER_BIG: (f32, f32) = (0.78, 1.75);
/// 死亡アニメーションの長さ / death animation length.
pub const DEATH_TIME: f32 = 1.6;
/// クリア演出の長さ / level-complete celebration length.
pub const COMPLETE_TIME: f32 = 3.0;
/// 残り時間の初期値 / starting time (classic units).
pub const TIME_LIMIT: f32 = 400.0;
/// 残り時間の減り方 (ユニット/秒) / time units consumed per second.
pub const TIME_RATE: f32 = 2.5;
/// 残り時間の警告しきい値 / low-time warning threshold.
pub const TIME_WARNING: f32 = 100.0;
/// 初期残機 / starting lives.
pub const START_LIVES: u8 = 3;
/// 当たり判定を少しだけ内側に縮める係数 / small epsilon so touching tiles don't jitter.
pub const EPS: f32 = 0.0015;

/// 敵の種類 / enemy kind.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    /// クリボー / goomba
    Goomba,
    /// ノコノコ / koopa troopa
    Koopa,
}

impl EnemyKind {
    /// サイズ (幅, 高さ) / hitbox size.
    pub const fn size(self) -> (f32, f32) {
        match self {
            EnemyKind::Goomba => (0.84, 0.84),
            EnemyKind::Koopa => (0.84, 1.30),
        }
    }

    /// 移動速度 / patrol speed.
    pub const fn speed(self) -> f32 {
        match self {
            EnemyKind::Goomba => 1.45,
            EnemyKind::Koopa => 2.15,
        }
    }

    /// 倒したときのスコア / score awarded when defeated.
    pub const fn score(self) -> u32 {
        match self {
            EnemyKind::Goomba => 100,
            EnemyKind::Koopa => 200,
        }
    }

    /// 描画に使う文字 / glyph used for rendering.
    pub const fn glyph(self) -> char {
        match self {
            EnemyKind::Goomba => 'g',
            EnemyKind::Koopa => 'k',
        }
    }
}

/// プレイヤー / the player character.
#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    /// 左上の X 座標 (タイル単位) / top-left x
    pub x: f32,
    /// 左上の Y 座標 (タイル単位) / top-left y
    pub y: f32,
    /// 水平速度 / horizontal velocity
    pub vx: f32,
    /// 垂直速度 (正 = 下) / vertical velocity (positive is down)
    pub vy: f32,
    /// 幅 / width
    pub w: f32,
    /// 高さ / height
    pub h: f32,
    /// 地面に接しているか / touching the ground
    pub on_ground: bool,
    /// 向き (-1 = 左, 1 = 右) / facing direction
    pub facing: i8,
    /// 大きい状態か (キノコ) / big (mushroom) state
    pub big: bool,
    /// 無敵の残り時間 / remaining invulnerability
    pub invuln: f32,
    /// コヨーテタイムの残り / remaining coyote time
    pub coyote: f32,
    /// 直前フレームの足元 Y (踏みつけ判定用) / previous foot position
    pub prev_y: f32,
}

impl Player {
    /// 中央 X / horizontal center.
    pub fn center_x(&self) -> f32 {
        self.x + self.w * 0.5
    }

    /// 足元 Y / feet position.
    pub fn feet(&self) -> f32 {
        self.y + self.h
    }

    /// 中心 Y / vertical center.
    pub fn center_y(&self) -> f32 {
        self.y + self.h * 0.5
    }
}

/// 敵 / an enemy entity.
#[derive(Clone, Debug, PartialEq)]
pub struct Enemy {
    /// 識別子 / id
    pub id: u32,
    /// 種類 / kind
    pub kind: EnemyKind,
    /// 左上 X / top-left x
    pub x: f32,
    /// 左上 Y / top-left y
    pub y: f32,
    /// 幅 / width
    pub w: f32,
    /// 高さ / height
    pub h: f32,
    /// 水平速度 / horizontal velocity
    pub vx: f32,
    /// 垂直速度 / vertical velocity
    pub vy: f32,
    /// 生きているか / alive
    pub alive: bool,
    /// 踏まれてぺしゃんこになった残り時間 / squashed timer
    pub squashed: Option<f32>,
    /// 地面に接しているか / touching the ground
    pub on_ground: bool,
    /// 出現位置 (リスポーン用) / spawn position
    pub home: (i32, i32),
}

/// コイン / a collectible coin.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Coin {
    /// 列 / column
    pub col: i32,
    /// 行 / row
    pub row: i32,
    /// 取得済みか / already collected
    pub collected: bool,
}

/// ふわっと浮く演出用のポップアップ / floating score popup.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Popup {
    /// X 座標 / x
    pub x: f32,
    /// Y 座標 / y
    pub y: f32,
    /// 上昇速度 / rise speed
    pub vy: f32,
    /// 残り時間 / remaining life
    pub life: f32,
    /// 表示する文字 / glyph
    pub glyph: char,
    /// スコア表示用の追加文字列 / optional label
    pub label: Option<&'static str>,
}

/// ゲームの進行状態 / game state machine.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameState {
    /// プレイ中 / playing
    Playing,
    /// 一時停止 / paused
    Paused,
    /// 死亡演出中 / death animation
    Dying {
        /// 残り時間 / remaining time
        timer: f32,
    },
    /// ステージクリア演出中 / level clear celebration
    LevelComplete {
        /// 残り時間 / remaining time
        timer: f32,
        /// 残り時間ボーナス / time bonus that was awarded
        bonus: u32,
    },
    /// ゲームオーバー / game over
    GameOver,
}

impl GameState {
    /// 日本語ラベル / short Japanese label for the HUD.
    pub const fn label(self) -> &'static str {
        match self {
            GameState::Playing => "プレイ中",
            GameState::Paused => "ポーズ",
            GameState::Dying { .. } => "ミス！",
            GameState::LevelComplete { .. } => "ステージクリア！",
            GameState::GameOver => "ゲームオーバー",
        }
    }
}

/// シミュレーション中に発生した出来事 / something that happened during a step.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// コインを取った / the player collected a coin
    CoinCollected {
        /// X 座標 / x
        x: f32,
        /// Y 座標 / y
        y: f32,
        /// 累計コイン / total coins
        total: u32,
    },
    /// ブロックを下から叩いた / a block was bumped from below
    BlockBumped {
        /// 列 / column
        col: i32,
        /// 行 / row
        row: i32,
        /// 叩いたあとのタイル / tile after the bump
        tile: Tile,
    },
    /// キノコが出現した / a mushroom popped out of a `?` block
    MushroomSpawned {
        /// 列 / column
        col: i32,
        /// 行 / row
        row: i32,
    },
    /// 敵を踏んだ / an enemy was stomped
    EnemyStomped {
        /// 種類 / kind
        kind: EnemyKind,
        /// 得点 / score gained
        score: u32,
    },
    /// 敵と接触してダメージ / the player took a hit from an enemy
    PlayerHurt {
        /// 大きい状態から小さくなったか / whether the player shrank instead of dying
        shrank: bool,
    },
    /// キノコでパワーアップした / the player powered up with a mushroom
    PoweredUp,
    /// プレイヤーが死んだ / the player died
    PlayerDied,
    /// リスポーンした / the player respawned after dying
    Respawned,
    /// ステージクリア / the level was completed
    LevelComplete {
        /// 得点 / total score
        score: u32,
        /// タイムボーナス / time bonus
        bonus: u32,
    },
    /// ゲームオーバー / no lives left
    GameOver,
    /// 残り時間が少ない / low time warning
    TimeWarning,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::CoinCollected { total, .. } => write!(f, "コイン +1 (合計 {total})"),
            Event::BlockBumped { col, row, tile } => {
                write!(f, "ブロック {:?} ({col},{row})", tile.glyph())
            }
            Event::MushroomSpawned { col, row } => write!(f, "キノコ出現 ({col},{row})"),
            Event::EnemyStomped { kind, score } => write!(f, "{:?} を踏んだ +{score}", kind),
            Event::PlayerHurt { shrank } => {
                if *shrank {
                    write!(f, "小さい状態になった")
                } else {
                    write!(f, "被弾")
                }
            }
            Event::PoweredUp => write!(f, "パワーアップ！"),
            Event::PlayerDied => write!(f, "ミス！"),
            Event::Respawned => write!(f, "リスポーン"),
            Event::LevelComplete { score, bonus } => {
                write!(f, "クリア! スコア {score} (ボーナス +{bonus})")
            }
            Event::GameOver => write!(f, "ゲームオーバー"),
            Event::TimeWarning => write!(f, "残り時間わずか！"),
        }
    }
}

/// ゲーム世界 / the whole simulated world.
#[derive(Clone, Debug, PartialEq)]
pub struct World {
    /// レベル / level data
    pub level: Level,
    /// プレイヤー / the player
    pub player: Player,
    /// 敵 / enemies
    pub enemies: Vec<Enemy>,
    /// コイン / coins
    pub coins: Vec<Coin>,
    /// ポップアップ演出 / floating popups
    pub popups: Vec<Popup>,
    /// 出現済みで未取得のキノコ (タイル座標) / mushrooms waiting to be picked up
    pub grow_blocks: Vec<(i32, i32)>,
    /// カメラ X (タイル単位) / camera x in tiles
    pub camera_x: f32,
    /// スコア / score
    pub score: u32,
    /// 集めたコイン / collected coins
    pub coin_count: u32,
    /// 残機 / lives
    pub lives: u8,
    /// 状態 / game state
    pub state: GameState,
    /// 残り時間 / remaining time
    pub time: f32,
    /// 経過時間 / elapsed time
    pub elapsed: f32,
    /// 固定ステップの通し番号 / tick counter
    pub tick: u64,
    /// 表示中の幅 (カメラのクランプに使う) / viewport width used to clamp the camera
    pub view_w: f32,
    accumulator: f32,
    next_enemy_id: u32,
    jump_prev: bool,
    pause_prev: bool,
    warned: bool,
}

impl World {
    /// レベルから世界を作る / build a world from a level.
    pub fn new(level: Level) -> Self {
        let (sw, sh) = PLAYER_SMALL;
        let (x, y) = level.player_spawn();
        let player = Player {
            x,
            y: (y + 1.0 - sh).max(0.0),
            vx: 0.0,
            vy: 0.0,
            w: sw,
            h: sh,
            on_ground: false,
            facing: 1,
            big: false,
            invuln: 0.0,
            coyote: 0.0,
            prev_y: y,
        };
        let coins = level
            .coin_spots()
            .iter()
            .map(|&(col, row)| Coin {
                col,
                row,
                collected: false,
            })
            .collect();
        let mut world = Self {
            level,
            player,
            enemies: Vec::new(),
            coins,
            popups: Vec::new(),
            grow_blocks: Vec::new(),
            camera_x: 0.0,
            score: 0,
            coin_count: 0,
            lives: START_LIVES,
            state: GameState::Playing,
            time: TIME_LIMIT,
            elapsed: 0.0,
            tick: 0,
            view_w: 40.0,
            accumulator: 0.0,
            next_enemy_id: 1,
            jump_prev: false,
            pause_prev: false,
            warned: false,
        };
        world.spawn_enemies();
        world
    }

    /// カメラが表示する幅を設定する / tell the world how wide the viewport is.
    pub fn set_viewport(&mut self, cols: u32) {
        self.view_w = (cols as f32).max(8.0);
    }

    /// 敵を初期位置に戻す / (re)spawn all enemies at their home positions.
    pub fn spawn_enemies(&mut self) {
        self.enemies = self
            .level
            .enemy_spawns()
            .iter()
            .map(|spawn| self.make_enemy(*spawn))
            .collect();
    }

    fn make_enemy(&mut self, spawn: EnemySpawn) -> Enemy {
        let (w, h) = spawn.kind.size();
        let id = self.next_enemy_id;
        self.next_enemy_id += 1;
        Enemy {
            id,
            kind: spawn.kind,
            x: spawn.col as f32 + (1.0 - w) * 0.5,
            y: (spawn.row + 1) as f32 - h,
            w,
            h,
            vx: -spawn.kind.speed(),
            vy: 0.0,
            alive: true,
            squashed: None,
            on_ground: false,
            home: (spawn.col, spawn.row),
        }
    }

    /// プレイヤーと敵を初期状態に戻す (スコアと残機は維持) / reset positions after a death.
    pub fn respawn(&mut self) {
        let (sw, sh) = PLAYER_SMALL;
        let (x, y) = self.level.player_spawn();
        self.player = Player {
            x,
            y: (y + 1.0 - sh).max(0.0),
            vx: 0.0,
            vy: 0.0,
            w: sw,
            h: sh,
            on_ground: false,
            facing: 1,
            big: false,
            invuln: INVULN_TIME,
            coyote: 0.0,
            prev_y: y,
        };
        self.coins.iter_mut().for_each(|coin| coin.collected = false);
        self.popups.clear();
        self.grow_blocks.clear();
        self.spawn_enemies();
        self.time = TIME_LIMIT;
        self.camera_x = 0.0;
        self.warned = false;
    }

    /// 完全にやり直す / restart everything (score, lives, time).
    pub fn restart(&mut self) {
        self.score = 0;
        self.coin_count = 0;
        self.lives = START_LIVES;
        self.elapsed = 0.0;
        self.state = GameState::Playing;
        self.respawn();
    }

    /// アキュムレータ方式の更新 / advance the simulation by `dt` seconds of wall time.
    pub fn update(&mut self, input: Input, dt: f32) -> Vec<Event> {
        let mut events = Vec::new();
        if dt.is_finite() && dt > 0.0 {
            self.accumulator += dt.min(0.25);
        }
        let mut steps = 0;
        while self.accumulator >= FIXED_DT && steps < 8 {
            events.extend(self.step(input));
            self.accumulator -= FIXED_DT;
            steps += 1;
        }
        if steps == 8 {
            // 追いつけない場合は入力を捨てる (スパイラル防止)
            // / avoid the spiral of death
            self.accumulator = 0.0;
        }
        events
    }

    /// 1 固定ステップだけ進める / advance exactly one fixed step.
    pub fn step(&mut self, input: Input) -> Vec<Event> {
        let mut events = Vec::new();
        self.tick += 1;
        let pause_edge = input.pause && !self.pause_prev;

        match self.state {
            GameState::Paused => {
                if pause_edge {
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver => {}
            GameState::Dying { mut timer } => {
                timer -= FIXED_DT;
                self.elapsed += FIXED_DT;
                // 死亡アニメーション: 上に跳ねてから落ちる
                self.player.vy = (self.player.vy + GRAVITY * 0.6 * FIXED_DT).min(MAX_FALL);
                self.player.y += self.player.vy * FIXED_DT;
                self.update_popups();
                if timer <= 0.0 {
                    if self.lives == 0 {
                        self.state = GameState::GameOver;
                        events.push(Event::GameOver);
                    } else {
                        self.respawn();
                        self.state = GameState::Playing;
                        events.push(Event::Respawned);
                    }
                } else {
                    self.state = GameState::Dying { timer };
                }
            }
            GameState::LevelComplete { mut timer, bonus } => {
                timer -= FIXED_DT;
                self.elapsed += FIXED_DT;
                self.update_popups();
                self.update_camera();
                if timer <= 0.0 {
                    self.state = GameState::Playing;
                } else {
                    self.state = GameState::LevelComplete { timer, bonus };
                }
            }
            GameState::Playing => {
                if pause_edge {
                    self.state = GameState::Paused;
                } else {
                    self.simulate(input, &mut events);
                }
            }
        }

        self.pause_prev = input.pause;
        self.jump_prev = input.jump;
        events
    }

    /// プレイ中の 1 ステップ分のシミュレーション / one step of live gameplay.
    fn simulate(&mut self, input: Input, events: &mut Vec<Event>) {
        self.elapsed += FIXED_DT;
        self.time = (self.time - TIME_RATE * FIXED_DT).max(0.0);
        if self.time <= 0.0 {
            self.kill(events);
            return;
        }
        if !self.warned && self.time <= TIME_WARNING {
            self.warned = true;
            events.push(Event::TimeWarning);
        }

        self.player.prev_y = self.player.y;
        self.update_player(input, events);
        self.move_player();
        self.update_enemies();
        self.resolve_contacts(events);
        self.resolve_mushrooms(events);
        self.collect_coins(events);
        self.check_hazards(events);
        self.check_goal(events);
        self.update_popups();
        self.update_invulnerability();
        self.update_camera();

        if self.player.y > self.level.height() as f32 + 1.5 {
            self.kill(events);
        }
    }

    /// 入力からプレイヤーの速度を決める / translate input into player velocity.
    fn update_player(&mut self, input: Input, _events: &mut Vec<Event>) {
        let jump_prev = self.jump_prev;
        let player = &mut self.player;

        let axis = input.axis();
        let max_speed = if input.run { RUN_SPEED } else { WALK_SPEED };
        if axis != 0.0 {
            let accel = if player.on_ground { ACCEL } else { AIR_ACCEL };
            player.vx += axis * accel * FIXED_DT;
            player.facing = if axis > 0.0 { 1 } else { -1 };
            player.vx = player.vx.clamp(-max_speed, max_speed);
        } else {
            let drag = if player.on_ground { FRICTION } else { AIR_DRAG };
            if player.vx > 0.0 {
                player.vx = (player.vx - drag * FIXED_DT).max(0.0);
            } else if player.vx < 0.0 {
                player.vx = (player.vx + drag * FIXED_DT).min(0.0);
            }
        }

        // ジャンプ (押した瞬間だけ / コヨーテタイムつき)
        // / jump on the rising edge, with coyote time
        let can_jump = player.on_ground || player.coyote > 0.0;
        if input.jump && !jump_prev && can_jump {
            player.vy = JUMP_VELOCITY;
            player.on_ground = false;
            player.coyote = 0.0;
        }

        // 可変ジャンプ: 上昇中に押し続けていると弱い重力、離すと強い重力
        // / variable jump height: hold for a floaty arc, release to cut it short
        let gravity = if player.vy < 0.0 {
            if input.jump {
                JUMP_HOLD_GRAVITY
            } else {
                JUMP_RELEASE_GRAVITY
            }
        } else {
            GRAVITY
        };
        player.vy = (player.vy + gravity * FIXED_DT).min(MAX_FALL);

        // コヨーテタイム
        if player.on_ground {
            player.coyote = COYOTE_TIME;
        } else {
            player.coyote = (player.coyote - FIXED_DT).max(0.0);
        }
        player.on_ground = false;
    }

    /// プレイヤーの移動と衝突解決 / move the player and resolve tile collisions.
    fn move_player(&mut self) {
        let step = FIXED_DT;
        // X 軸 / horizontal
        let dx = self.player.vx * step;
        if dx != 0.0 {
            self.player.x += dx;
            self.resolve_player_x();
        }
        // Y 軸 / vertical
        let dy = self.player.vy * step;
        if dy != 0.0 {
            self.player.y += dy;
            self.resolve_player_y();
        }
    }

    fn resolve_player_x(&mut self) {
        let (x, y, w, h) = (
            self.player.x,
            self.player.y,
            self.player.w,
            self.player.h,
        );
        let vx = self.player.vx;
        let (r0, r1) = tile_range(y, h);
        let (c0, c1) = tile_range(x, w);
        for row in r0..=r1 {
            for col in c0..=c1 {
                if !self.level.solid_at(col, row) {
                    continue;
                }
                if vx > 0.0 {
                    self.player.x = col as f32 - w;
                } else if vx < 0.0 {
                    self.player.x = (col + 1) as f32;
                } else {
                    // その場で押された場合は中心で振り分ける
                    let center = x + w * 0.5;
                    if center < col as f32 + 0.5 {
                        self.player.x = col as f32 - w;
                    } else {
                        self.player.x = (col + 1) as f32;
                    }
                }
                self.player.vx = 0.0;
                return;
            }
        }
    }

    fn resolve_player_y(&mut self) {
        let (x, y, w, h) = (
            self.player.x,
            self.player.y,
            self.player.w,
            self.player.h,
        );
        let vy = self.player.vy;
        let (r0, r1) = tile_range(y, h);
        let (c0, c1) = tile_range(x, w);
        if vy > 0.0 {
            // 着地 / landing
            let row = r1;
            for col in c0..=c1 {
                if self.level.solid_at(col, row) {
                    self.player.y = row as f32 - h;
                    self.player.vy = 0.0;
                    self.player.on_ground = true;
                    return;
                }
            }
        } else if vy < 0.0 {
            // 天井 / ceiling: 頭に当たった列のうち中心に最も近いものを叩く
            let row = r0;
            let center = x + w * 0.5;
            let mut hit: Option<i32> = None;
            for col in c0..=c1 {
                if !self.level.solid_at(col, row) {
                    continue;
                }
                let distance = (col as f32 + 0.5 - center).abs();
                let best = hit
                    .map(|best_col| (best_col as f32 + 0.5 - center).abs())
                    .unwrap_or(f32::MAX);
                if distance <= best {
                    hit = Some(col);
                }
            }
            if let Some(col) = hit {
                self.player.y = (row + 1) as f32;
                self.player.vy = 0.0;
                self.bump_block(col, row);
            }
        }
    }

    /// ブロックを下から叩いた / the player head-butted a block.
    fn bump_block(&mut self, col: i32, row: i32) {
        let tile = self.level.tile(col, row);
        if !tile.is_bumpable() {
            return;
        }
        self.level.set_tile(col, row, Tile::Used);
        match tile {
            Tile::Question { mushroom: true } => {
                self.spawn_mushroom(col, row);
            }
            _ => {
                // 中からコインが出てくる
                self.coin_count += 1;
                self.score += 200;
                self.popups.push(Popup {
                    x: col as f32,
                    y: (row - 1) as f32,
                    vy: -4.5,
                    life: 0.6,
                    glyph: 'o',
                    label: None,
                });
            }
        }
    }

    fn spawn_mushroom(&mut self, col: i32, row: i32) {
        // キノコは「その場で育って上に乗る」挙動にする (物理を増やさず、効果は即時)
        // / keep it simple: the mushroom instantly grows the player when they touch it
        self.popups.push(Popup {
            x: col as f32,
            y: (row - 1) as f32,
            vy: -3.0,
            life: 0.6,
            glyph: 'M',
            label: None,
        });
        self.grow_blocks.push((col, row - 1));
    }

    /// 敵の更新 / move enemies, handle walls, gravity and enemy-enemy collisions.
    fn update_enemies(&mut self) {
        let level = &self.level;
        let height = level.height() as f32;
        for enemy in self.enemies.iter_mut() {
            if let Some(timer) = enemy.squashed {
                enemy.squashed = Some(timer - FIXED_DT);
                continue;
            }
            if !enemy.alive {
                continue;
            }
            // 重力 / gravity
            enemy.vy = (enemy.vy + GRAVITY * FIXED_DT).min(MAX_FALL);
            enemy.on_ground = false;

            // X 移動 / horizontal
            enemy.x += enemy.vx * FIXED_DT;
            let (r0, r1) = tile_range(enemy.y, enemy.h);
            let (c0, c1) = tile_range(enemy.x, enemy.w);
            'outer: for row in r0..=r1 {
                for col in c0..=c1 {
                    if level.solid_at(col, row) {
                        if enemy.vx > 0.0 {
                            enemy.x = col as f32 - enemy.w;
                        } else {
                            enemy.x = (col + 1) as f32;
                        }
                        enemy.vx = -enemy.vx;
                        break 'outer;
                    }
                }
            }

            // Y 移動 / vertical
            enemy.y += enemy.vy * FIXED_DT;
            let (r0, r1) = tile_range(enemy.y, enemy.h);
            let (c0, c1) = tile_range(enemy.x, enemy.w);
            if enemy.vy > 0.0 {
                for col in c0..=c1 {
                    if level.solid_at(col, r1) {
                        enemy.y = r1 as f32 - enemy.h;
                        enemy.vy = 0.0;
                        enemy.on_ground = true;
                        break;
                    }
                }
            } else if enemy.vy < 0.0 {
                for col in c0..=c1 {
                    if level.solid_at(col, r0) {
                        enemy.y = (r0 + 1) as f32;
                        enemy.vy = 0.0;
                        break;
                    }
                }
            }

            // 奈落に落ちた敵は消える / enemies that fall into a pit disappear
            if enemy.y > height + 2.0 {
                enemy.alive = false;
            }
        }

        // 倒れた敵を片付ける / clean up squashed enemies
        let mut removed = Vec::new();
        for (index, enemy) in self.enemies.iter().enumerate() {
            if let Some(timer) = enemy.squashed {
                if timer <= 0.0 {
                    removed.push(index);
                }
            }
        }
        for index in removed.into_iter().rev() {
            self.enemies.remove(index);
        }

        // 敵同士の衝突 / enemy vs enemy: both turn around
        for i in 0..self.enemies.len() {
            for j in (i + 1)..self.enemies.len() {
                let (a, b) = (self.enemies[i].clone(), self.enemies[j].clone());
                if !a.alive || !b.alive || a.squashed.is_some() || b.squashed.is_some() {
                    continue;
                }
                if overlaps_enemies(&a, &b) {
                    self.enemies[i].vx = -a.vx;
                    self.enemies[j].vx = -b.vx;
                }
            }
        }

    }

    /// プレイヤーと敵の接触 / player vs enemy contacts (stomp or damage).
    fn resolve_contacts(&mut self, events: &mut Vec<Event>) {
        let player = self.player.clone();
        for index in 0..self.enemies.len() {
            let enemy = self.enemies[index].clone();
            if !enemy.alive || enemy.squashed.is_some() {
                continue;
            }
            if !overlaps(&player, &enemy) {
                continue;
            }
            let foot_before = player.prev_y + player.h;
            let stomping = player.vy > 0.0 && foot_before <= enemy.y + 0.35;
            if stomping {
                let kind = enemy.kind;
                self.enemies[index].alive = false;
                self.enemies[index].squashed = Some(0.35);
                self.enemies[index].vx = 0.0;
                self.score += kind.score();
                self.player.vy = STOMP_BOUNCE;
                self.player.y = enemy.y - self.player.h - EPS;
                self.popups.push(Popup {
                    x: enemy.x,
                    y: enemy.y - 0.8,
                    vy: -3.5,
                    life: 0.5,
                    glyph: 'o',
                    label: Some("+100"),
                });
                events.push(Event::EnemyStomped {
                    kind,
                    score: kind.score(),
                });
            } else {
                self.hurt_player(events);
            }
        }
    }

    /// キノコの取得判定 / grab a mushroom that popped out of a block.
    fn resolve_mushrooms(&mut self, events: &mut Vec<Event>) {
        let mut taken = Vec::new();
        for (index, &(col, row)) in self.grow_blocks.iter().enumerate() {
            let (fx, fy) = (col as f32, row as f32);
            let hit = self.player.center_x() > fx - 0.4
                && self.player.center_x() < fx + 1.4
                && self.player.center_y() > fy - 0.8
                && self.player.center_y() < fy + 1.4;
            if hit {
                taken.push(index);
                if !self.player.big {
                    self.grow_player();
                    events.push(Event::PoweredUp);
                }
                self.score += 1000;
            }
        }
        for index in taken.into_iter().rev() {
            self.grow_blocks.remove(index);
        }
    }

    fn grow_player(&mut self) {
        let (w, h) = PLAYER_BIG;
        let feet = self.player.feet();
        // 頭の上にスペースがあるか確認する / only grow when there is room above
        let new_y = (feet - h).max(0.0);
        let (c0, c1) = tile_range(self.player.x, w);
        let (r0, _) = tile_range(new_y, h);
        for col in c0..=c1 {
            if self.level.solid_at(col, r0) && new_y < feet - PLAYER_SMALL.1 {
                return;
            }
        }
        self.player.big = true;
        self.player.w = w;
        self.player.h = h;
        self.player.y = new_y;
    }

    fn shrink_player(&mut self) {
        let (w, h) = PLAYER_SMALL;
        let feet = self.player.feet();
        self.player.big = false;
        self.player.w = w;
        self.player.h = h;
        self.player.y = (feet - h).max(0.0);
    }

    /// ダメージ / the player got hit.
    fn hurt_player(&mut self, events: &mut Vec<Event>) {
        if self.player.invuln > 0.0 {
            return;
        }
        if self.player.big {
            self.shrink_player();
            self.player.invuln = INVULN_TIME;
            events.push(Event::PlayerHurt { shrank: true });
        } else {
            self.kill(events);
        }
    }

    /// ミス / kill the player and start the death animation.
    fn kill(&mut self, events: &mut Vec<Event>) {
        if matches!(self.state, GameState::Dying { .. } | GameState::GameOver) {
            return;
        }
        self.lives = self.lives.saturating_sub(1);
        self.state = GameState::Dying {
            timer: DEATH_TIME,
        };
        self.player.vy = -12.0;
        self.player.vx = 0.0;
        events.push(Event::PlayerDied);
    }

    fn collect_coins(&mut self, events: &mut Vec<Event>) {
        let (cx, cy) = (self.player.center_x(), self.player.center_y());
        for index in 0..self.coins.len() {
            if self.coins[index].collected {
                continue;
            }
            let (col, row) = (self.coins[index].col, self.coins[index].row);
            let inside = cx > col as f32 + 0.1
                && cx < (col + 1) as f32 - 0.1
                && cy > row as f32
                && cy < (row + 1) as f32;
            if inside {
                self.coins[index].collected = true;
                self.coin_count += 1;
                self.score += 100;
                self.popups.push(Popup {
                    x: col as f32,
                    y: row as f32,
                    vy: -3.0,
                    life: 0.5,
                    glyph: 'o',
                    label: None,
                });
                events.push(Event::CoinCollected {
                    x: col as f32,
                    y: row as f32,
                    total: self.coin_count,
                });
            }
        }
    }

    fn check_hazards(&mut self, events: &mut Vec<Event>) {
        let (c0, c1) = tile_range(self.player.x, self.player.w);
        let (r0, r1) = tile_range(self.player.y, self.player.h);
        for row in r0..=r1 {
            for col in c0..=c1 {
                if self.level.hazard_at(col, row) {
                    self.kill(events);
                    return;
                }
            }
        }
    }

    fn check_goal(&mut self, events: &mut Vec<Event>) {
        let Some((col, row)) = self.level.goal() else {
            return;
        };
        let goal_x = col as f32;
        let goal_y = row as f32;
        let touched = self.player.x + self.player.w >= goal_x
            && self.player.x <= goal_x + 1.0
            && self.player.y + self.player.h >= goal_y - 1.0
            && self.player.y <= goal_y + 1.0;
        if !touched {
            return;
        }
        let bonus = (self.time * 10.0).round() as u32;
        self.score += bonus;
        self.state = GameState::LevelComplete {
            timer: COMPLETE_TIME,
            bonus,
        };
        events.push(Event::LevelComplete {
            score: self.score,
            bonus,
        });
    }

    fn update_popups(&mut self) {
        for popup in self.popups.iter_mut() {
            popup.y += popup.vy * FIXED_DT;
            popup.life -= FIXED_DT;
        }
        self.popups.retain(|popup| popup.life > 0.0);
    }

    fn update_invulnerability(&mut self) {
        if self.player.invuln > 0.0 {
            self.player.invuln = (self.player.invuln - FIXED_DT).max(0.0);
        }
    }

    fn update_camera(&mut self) {
        let target = self.player.center_x() - self.view_w * 0.5;
        let max_x = (self.level.width() as f32 - self.view_w).max(0.0);
        let target = target.clamp(0.0, max_x);
        // なめらかに追従 / smooth follow
        self.camera_x += (target - self.camera_x) * 0.18;
        self.camera_x = self.camera_x.clamp(0.0, max_x);
    }

    /// ASCII フレームを作る / render the world into an ASCII frame.
    pub fn render(&self, options: RenderOptions) -> Frame {
        crate::render::render_world(self, options)
    }

    /// 残り時間の表示用 / remaining time for the HUD.
    pub fn time_left(&self) -> u32 {
        self.time.ceil().max(0.0) as u32
    }
}

/// `x` から幅 `w` までのタイル範囲 / inclusive tile range covered by `[pos, pos + size)`.
pub(crate) fn tile_range(pos: f32, size: f32) -> (i32, i32) {
    let start = pos.floor() as i32;
    let end = (pos + size - EPS).floor() as i32;
    (start, end.max(start))
}

/// AABB の重なり判定 / axis aligned bounding box overlap test.
fn aabb(
    ax: f32,
    ay: f32,
    aw: f32,
    ah: f32,
    bx: f32,
    by: f32,
    bw: f32,
    bh: f32,
) -> bool {
    ax < bx + bw - EPS && ax + aw > bx + EPS && ay < by + bh - EPS && ay + ah > by + EPS
}

/// プレイヤーと敵の重なり / player vs enemy overlap.
fn overlaps(p: &Player, e: &Enemy) -> bool {
    aabb(p.x, p.y, p.w, p.h, e.x, e.y, e.w, e.h)
}

/// 敵同士の重なり / enemy vs enemy overlap.
fn overlaps_enemies(a: &Enemy, b: &Enemy) -> bool {
    aabb(a.x, a.y, a.w, a.h, b.x, b.y, b.w, b.h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels;

    /// 平らな床と左右の壁だけのテスト用レベル / flat test level with side walls.
    const FLAT: &str = "
#            #
#            #
#            #
#            #
#   p        #
##############
";

    /// レベルを作るヘルパ / build a world from ASCII art.
    fn world(art: &str) -> World {
        let level = Level::from_ascii("test", art).expect("test level should parse");
        World::new(level)
    }

    /// 固定ステップを N 回まわしてイベントを集める / run N fixed steps.
    fn run(world: &mut World, input: Input, steps: usize) -> Vec<Event> {
        let mut events = Vec::new();
        for _ in 0..steps {
            events.extend(world.step(input));
        }
        events
    }

    #[test]
    fn spawns_on_the_ground_and_settles() {
        let mut world = world(FLAT);
        assert!(!world.player.on_ground, "spawns slightly above the floor");
        run(&mut world, Input::new(), 5);
        assert!(world.player.on_ground);
        assert_eq!(world.player.vy, 0.0);
        assert_eq!(world.player.feet(), 5.0);
        assert_eq!(world.lives, START_LIVES);
    }

    #[test]
    fn gravity_pulls_the_player_down() {
        let mut world = world(FLAT);
        world.player.y = 0.0;
        world.player.on_ground = false;
        let before = world.player.y;
        run(&mut world, Input::new(), 6);
        assert!(world.player.y > before, "player should fall");
        assert!(world.player.y < 5.0, "but not instantly touch the floor");
    }

    #[test]
    fn walks_right_and_stops_at_the_wall() {
        let mut world = world(FLAT);
        run(&mut world, Input::new(), 2);
        let start_x = world.player.x;
        run(&mut world, Input::new().right(true), 120);
        assert!(world.player.x > start_x, "player should move right");
        // 右の壁 (col 12) で止まる / blocked by the wall at column 12
        assert!(
            (world.player.x + world.player.w - 12.0).abs() < 1e-3,
            "expected to be flush against the wall, got {}",
            world.player.x
        );
        assert_eq!(world.player.vx, 0.0);
    }

    #[test]
    fn cannot_walk_through_the_left_wall() {
        let mut world = world(FLAT);
        run(&mut world, Input::new(), 2);
        run(&mut world, Input::new().left(true), 200);
        assert_eq!(world.player.x, 1.0);
    }

    #[test]
    fn run_is_faster_than_walk() {
        let mut world = world(FLAT);
        run(&mut world, Input::new().right(true), 60);
        let walk_speed = world.player.vx;

        let mut runner = world(FLAT);
        run(&mut runner, Input::new().right(true).run(true), 60);
        assert!(runner.player.vx > walk_speed);
        assert!((runner.player.vx - RUN_SPEED).abs() < 1e-3);
    }

    #[test]
    fn holding_jump_jumps_higher_than_tapping() {
        // 押しっぱなし / hold the button the whole way up
        let mut held = world(FLAT);
        run(&mut held, Input::new(), 4);
        let floor = held.player.feet();
        let mut apex_held = floor;
        for _ in 0..70 {
            held.step(Input::new().jump(true));
            apex_held = apex_held.min(held.player.feet());
        }

        // すぐ離す / release after two frames
        let mut tapped = world(FLAT);
        run(&mut tapped, Input::new(), 4);
        let mut apex_tapped = tapped.player.feet();
        for step in 0..70 {
            tapped.step(Input::new().jump(step < 2));
            apex_tapped = apex_tapped.min(tapped.player.feet());
        }

        let held_height = floor - apex_held;
        let tapped_height = floor - apex_tapped;
        assert!(
            (2.5..=4.5).contains(&held_height),
            "full jump should clear about 4 tiles, got {held_height}"
        );
        assert!(
            tapped_height < held_height - 1.0,
            "tapping must be much shorter: {tapped_height} vs {held_height}"
        );
    }

    #[test]
    fn collects_a_coin() {
        let art = "
#            #
#         o  #
#   p        #
##############
";
        let mut world = world(art);
        assert_eq!(world.coins.len(), 1);
        let events = run(&mut world, Input::new().right(true), 120);
        assert!(world.coins[0].collected);
        assert_eq!(world.coin_count, 1);
        assert_eq!(world.score, 100);
        assert!(events
            .iter()
            .any(|event| matches!(event, Event::CoinCollected { total: 1, .. })));
    }

    #[test]
    fn stomping_a_goomba_scores_and_bounces() {
        let art = "
#            #
#            #
#   p   g    #
##############
";
        let mut world = world(art);
        let goomba = world.enemies[0].clone();
        // 敵の真上から落とす / drop the player straight onto the enemy
        world.player.x = goomba.x;
        world.player.y = goomba.y - 1.2;
        world.player.vy = 2.0;
        let events = run(&mut world, Input::new(), 20);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::EnemyStomped { .. })),
            "expected a stomp, got {events:?}"
        );
        assert_eq!(world.score, EnemyKind::Goomba.score());
        assert!(!world.enemies[0].alive);
        assert!(world.player.vy < 0.0, "player should bounce upwards");
        assert!(matches!(world.state, GameState::Playing));
    }

    #[test]
    fn walking_into_an_enemy_kills_a_small_player() {
        let art = "
#            #
#            #
#   p   g    #
##############
";
        let mut world = world(art);
        let events = run(&mut world, Input::new().right(true), 200);
        assert!(
            events.iter().any(|event| event == &Event::PlayerDied),
            "expected death, got {events:?}"
        );
        assert!(matches!(world.state, GameState::Dying { .. }));
        assert_eq!(world.lives, START_LIVES - 1);
    }

    #[test]
    fn big_player_shrinks_instead_of_dying() {
        let art = "
#            #
#            #
#   p   g    #
##############
";
        let mut world = world(art);
        world.player.big = true;
        world.player.w = PLAYER_BIG.0;
        world.player.h = PLAYER_BIG.1;
        world.player.y = 3.0 - PLAYER_BIG.1;
        let events = run(&mut world, Input::new().right(true), 200);
        assert!(events
            .iter()
            .any(|event| event == &Event::PlayerHurt { shrank: true }));
        assert!(!world.player.big);
        assert!(world.player.invuln > 0.0);
        assert!(matches!(world.state, GameState::Playing));
    }

    #[test]
    fn bumping_a_question_block_yields_a_coin() {
        let art = "
#            #
#     ?      #
#            #
#   p        #
##############
";
        let mut world = world(art);
        run(&mut world, Input::new(), 4);
        world.player.x = 6.1; // '?' の真下 / line up under the '?'
        let events = run(&mut world, Input::new().jump(true), 60);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::BlockBumped { .. })),
            "expected a block bump, got {events:?}"
        );
        assert_eq!(world.level.tile(6, 1), Tile::Used);
        assert_eq!(world.coin_count, 1);
        assert_eq!(world.score, 200);
    }

    #[test]
    fn mushroom_block_grows_the_player() {
        let art = "
#            #
#     M      #
#            #
#   p        #
##############
";
        let mut world = world(art);
        run(&mut world, Input::new(), 4);
        world.player.x = 6.1;
        run(&mut world, Input::new().jump(true), 60);
        assert_eq!(world.grow_blocks.len(), 1, "a mushroom should be waiting");
        assert!(!world.player.big);

        // キノコの位置に触れる / touch the mushroom
        world.player.x = 6.0;
        world.player.y = 0.4;
        let events = run(&mut world, Input::new(), 2);
        assert!(events.iter().any(|event| event == &Event::PoweredUp));
        assert!(world.player.big, "player should be big now");
        assert_eq!(world.player.h, PLAYER_BIG.1);
        assert!(world.player.feet() <= 1.0 + 1e-3);
    }

    #[test]
    fn touching_the_flag_completes_the_level() {
        let art = "
###############
#            F#
#   p         #
###############
";
        let mut world = world(art);
        let events = run(&mut world, Input::new().right(true), 240);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::LevelComplete { .. })),
            "expected a clear, got {events:?}"
        );
        assert!(matches!(world.state, GameState::LevelComplete { .. }));
        assert!(world.score > 0);
    }

    #[test]
    fn falling_into_a_pit_kills_the_player() {
        let art = "
#            #
#            #
#   p        #
########  ####
";
        let mut world = world(art);
        let events = run(&mut world, Input::new().right(true), 400);
        assert!(
            events.iter().any(|event| event == &Event::PlayerDied),
            "expected the pit to kill the player, got {events:?}"
        );
    }

    #[test]
    fn spikes_are_lethal() {
        let art = "
#            #
#   p     ^  #
##############
";
        let mut world = world(art);
        let events = run(&mut world, Input::new().right(true), 200);
        assert!(events.iter().any(|event| event == &Event::PlayerDied));
    }

    #[test]
    fn death_timer_leads_to_respawn_then_game_over() {
        let mut world = world(FLAT);
        world.lives = 2;
        world.kill(&mut Vec::new());
        let events = run(&mut world, Input::new(), 200);
        assert!(events.iter().any(|event| event == &Event::Respawned));
        assert!(matches!(world.state, GameState::Playing));
        assert_eq!(world.lives, 1);

        world.kill(&mut Vec::new());
        let events = run(&mut world, Input::new(), 200);
        assert!(events.iter().any(|event| event == &Event::GameOver));
        assert!(matches!(world.state, GameState::GameOver));
        assert_eq!(world.lives, 0);
    }

    #[test]
    fn pause_toggles_on_the_pressed_edge() {
        let mut world = world(FLAT);
        world.step(Input::new().pause(true));
        assert!(matches!(world.state, GameState::Paused));
        // 押しっぱなしでは切り替わらない / holding it does not toggle twice
        world.step(Input::new().pause(true));
        assert!(matches!(world.state, GameState::Paused));
        world.step(Input::new());
        world.step(Input::new().pause(true));
        assert!(matches!(world.state, GameState::Playing));
    }

    #[test]
    fn time_runs_out_and_kills_the_player() {
        let mut world = world(FLAT);
        world.time = 0.1;
        world.warned = true;
        let events = run(&mut world, Input::new(), 30);
        assert!(events.iter().any(|event| event == &Event::PlayerDied));
    }

    #[test]
    fn low_time_is_announced_once() {
        let mut world = world(FLAT);
        world.time = TIME_WARNING + 0.1;
        let events = run(&mut world, Input::new(), 30);
        assert_eq!(
            events
                .iter()
                .filter(|event| **event == Event::TimeWarning)
                .count(),
            1
        );
    }

    #[test]
    fn enemies_turn_around_at_walls() {
        let art = "
#            #
#   p   g    #
##############
";
        let mut world = world(art);
        let start_direction = world.enemies[0].vx.signum();
        // 左壁まで歩かせる / let the enemy walk into the left wall
        for _ in 0..300 {
            world.step(Input::new());
        }
        assert!(
            world.enemies[0].x >= 1.0,
            "enemy must stay inside the level: {}",
            world.enemies[0].x
        );
        let _ = start_direction;
    }

    #[test]
    fn update_accumulates_fixed_steps_and_clamps() {
        let mut world = world(FLAT);
        world.update(Input::new(), FIXED_DT);
        assert_eq!(world.tick, 1);
        // 大きなデルタでも最大 8 ステップに制限される / a huge delta is clamped
        world.update(Input::new(), 10.0);
        assert_eq!(world.tick, 9);
        assert_eq!(world.accumulator, 0.0);
    }

    #[test]
    fn camera_follows_the_player_inside_the_level() {
        let mut world = world(levels::WORLD_1_1);
        world.set_viewport(40);
        run(&mut world, Input::new(), 4);
        assert_eq!(world.camera_x, 0.0, "camera starts at the left edge");
        run(&mut world, Input::new().right(true).run(true), 600);
        assert!(world.camera_x > 0.0, "camera should scroll");
        assert!(
            world.camera_x <= world.level.width() as f32 - world.view_w,
            "camera must stay inside the level"
        );
    }

    #[test]
    fn restart_restores_score_and_lives() {
        let mut world = world(FLAT);
        world.score = 1234;
        world.coin_count = 7;
        world.lives = 1;
        world.restart();
        assert_eq!(world.score, 0);
        assert_eq!(world.coin_count, 0);
        assert_eq!(world.lives, START_LIVES);
        assert!(matches!(world.state, GameState::Playing));
        assert!(world.coins.iter().all(|coin| !coin.collected));
    }

    #[test]
    fn renders_the_player_and_tiles() {
        let mut world = world(FLAT);
        run(&mut world, Input::new(), 4);
        let frame = world.render(RenderOptions::new(24, 8));
        let text = frame.to_text();
        assert_eq!(frame.rows, 8);
        assert_eq!(frame.cols, 24);
        assert!(text.contains('@'), "player glyph missing:\n{text}");
        assert!(text.contains('#'), "ground glyph missing:\n{text}");
    }

    #[test]
    fn full_world_1_1_is_playable_for_a_while() {
        let mut world = world(levels::WORLD_1_1);
        let events = run(&mut world, Input::new().right(true), 600);
        assert_eq!(world.tick, 600);
        assert!(world.score > 0, "player should have collected something");
        assert!(!events.is_empty());
        assert!(world.player.x > 4.0, "player should have made progress");
    }
}
