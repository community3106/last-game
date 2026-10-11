//! ASCII Mario Game Entities & Items in 100% Rust

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarioForm {
    Small,      // Standard 'm' (1 tile high)
    Super,      // Grown 'M' (2 tiles high)
    Fire,       // Fire Mario 'F' (shoots fireballs)
    Star,       // Invincible Mario '★'
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub form: MarioForm,
    pub direction: Direction,
    pub is_on_ground: bool,
    pub is_jumping: bool,
    pub is_crouching: bool,
    pub is_dead: bool,
    pub death_timer: u32,
    pub invincibility_timer: u32,
    pub score: u32,
    pub coins: u32,
    pub lives: i32,
    pub world: u32,
    pub level: u32,
    pub time_left: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            x: 3.0,
            y: 12.0,
            vx: 0.0,
            vy: 0.0,
            form: MarioForm::Small,
            direction: Direction::Right,
            is_on_ground: true,
            is_jumping: false,
            is_crouching: false,
            is_dead: false,
            death_timer: 0,
            invincibility_timer: 0,
            score: 0,
            coins: 0,
            lives: 3,
            world: 1,
            level: 1,
            time_left: 400,
        }
    }
}

impl Player {
    pub fn height(&self) -> f32 {
        match self.form {
            MarioForm::Small => 1.0,
            MarioForm::Super | MarioForm::Fire | MarioForm::Star => 1.8,
        }
    }

    pub fn width(&self) -> f32 {
        0.8
    }

    pub fn ascii_sprite(&self) -> &'static str {
        match (self.form, self.direction, self.is_on_ground) {
            (MarioForm::Small, Direction::Right, true) => " m>",
            (MarioForm::Small, Direction::Left, true) => "<m ",
            (MarioForm::Small, Direction::Right, false) => "^m>",
            (MarioForm::Small, Direction::Left, false) => "<m^",
            (MarioForm::Super, Direction::Right, _) => "[M]>",
            (MarioForm::Super, Direction::Left, _) => "<[M]",
            (MarioForm::Fire, Direction::Right, _) => "[F*]>",
            (MarioForm::Fire, Direction::Left, _) => "<[F*]",
            (MarioForm::Star, _, _) => "★M★",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyType {
    Goomba,
    Koopa,
    KoopaShell { is_moving: bool },
    PiranhaPlant,
    Bowser { hp: i32, attack_timer: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub enemy_type: EnemyType,
    pub is_alive: bool,
    pub is_stomped: bool,
    pub stomp_timer: u32,
}

impl Enemy {
    pub fn new_goomba(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            vx: -0.05,
            vy: 0.0,
            enemy_type: EnemyType::Goomba,
            is_alive: true,
            is_stomped: false,
            stomp_timer: 0,
        }
    }

    pub fn new_koopa(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            vx: -0.06,
            vy: 0.0,
            enemy_type: EnemyType::Koopa,
            is_alive: true,
            is_stomped: false,
            stomp_timer: 0,
        }
    }

    pub fn new_bowser(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            vx: -0.02,
            vy: 0.0,
            enemy_type: EnemyType::Bowser { hp: 5, attack_timer: 60 },
            is_alive: true,
            is_stomped: false,
            stomp_timer: 0,
        }
    }

    pub fn ascii_sprite(&self) -> &'static str {
        if self.is_stomped {
            return "__";
        }
        match self.enemy_type {
            EnemyType::Goomba => "[o_o]",
            EnemyType::Koopa => "<(OO)>",
            EnemyType::KoopaShell { .. } => "[OO]",
            EnemyType::PiranhaPlant => "{V_V}",
            EnemyType::Bowser { .. } => "[[W_W]]",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    SuperMushroom,
    FireFlower,
    StarMan,
    Coin,
    OneUpMushroom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub item_type: ItemType,
    pub is_collected: bool,
}

impl Item {
    pub fn ascii_sprite(&self) -> &'static str {
        match self.item_type {
            ItemType::SuperMushroom => "(M)",
            ItemType::FireFlower => "(*)",
            ItemType::StarMan => "(★)",
            ItemType::Coin => "($)",
            ItemType::OneUpMushroom => "(1U)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fireball {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub is_active: bool,
    pub is_enemy_fire: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Air,
    Ground,
    Brick,
    MysteryBox { item: ItemType, is_opened: bool },
    EmptyBox,
    PipeTopLeft,
    PipeTopRight,
    PipeBodyLeft,
    PipeBodyRight,
    Lava,
    Flagpole,
    CastleWall,
    CastleDoor,
    AxeSwitch, // Drops Bowser Bridge
}

impl TileType {
    pub fn is_solid(&self) -> bool {
        match self {
            TileType::Air | TileType::Lava | TileType::Flagpole | TileType::CastleDoor | TileType::AxeSwitch => false,
            _ => true,
        }
    }

    pub fn ascii_char(&self) -> char {
        match self {
            TileType::Air => ' ',
            TileType::Ground => '█',
            TileType::Brick => '#',
            TileType::MysteryBox { is_opened: false, .. } => '?',
            TileType::MysteryBox { is_opened: true, .. } | TileType::EmptyBox => '.',
            TileType::PipeTopLeft => '┌',
            TileType::PipeTopRight => '┐',
            TileType::PipeBodyLeft => '│',
            TileType::PipeBodyRight => '│',
            TileType::Lava => '~',
            TileType::Flagpole => '|',
            TileType::CastleWall => '▓',
            TileType::CastleDoor => '∩',
            TileType::AxeSwitch => 'T',
        }
    }
}
