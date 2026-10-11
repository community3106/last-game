//! ASCII Mario World Level Maps and Parsers in 100% Rust

use super::entities::{Enemy, Item, ItemType, TileType};
use serde::{Deserialize, Serialize};

pub const LEVEL_HEIGHT: usize = 15;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelData {
    pub world: u32,
    pub level: u32,
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<TileType>>,
    pub enemies: Vec<Enemy>,
    pub items: Vec<Item>,
    pub background_type: String,
    pub music_theme: String,
    pub is_vip_locked: bool,
}

impl LevelData {
    /// Loads World 1-1: Classic Overworld
    pub fn world_1_1() -> Self {
        let width = 120;
        let mut tiles = vec![vec![TileType::Air; width]; LEVEL_HEIGHT];

        // Bottom Ground
        for x in 0..width {
            if (x >= 35 && x <= 37) || (x >= 70 && x <= 72) {
                // Pits
                continue;
            }
            tiles[13][x] = TileType::Ground;
            tiles[14][x] = TileType::Ground;
        }

        // Mystery Boxes & Bricks
        tiles[9][10] = TileType::MysteryBox { item: ItemType::Coin, is_opened: false };
        tiles[9][14] = TileType::Brick;
        tiles[9][15] = TileType::MysteryBox { item: ItemType::SuperMushroom, is_opened: false };
        tiles[9][16] = TileType::Brick;
        tiles[9][17] = TileType::MysteryBox { item: ItemType::Coin, is_opened: false };
        tiles[9][18] = TileType::Brick;
        tiles[5][16] = TileType::MysteryBox { item: ItemType::FireFlower, is_opened: false };

        // Pipe 1 (x=20)
        tiles[12][20] = TileType::PipeTopLeft;
        tiles[12][21] = TileType::PipeTopRight;

        // Pipe 2 (x=28)
        tiles[11][28] = TileType::PipeTopLeft;
        tiles[11][29] = TileType::PipeTopRight;
        tiles[12][28] = TileType::PipeBodyLeft;
        tiles[12][29] = TileType::PipeBodyRight;

        // Pipe 3 (x=45)
        tiles[10][45] = TileType::PipeTopLeft;
        tiles[10][46] = TileType::PipeTopRight;
        tiles[11][45] = TileType::PipeBodyLeft;
        tiles[11][46] = TileType::PipeBodyRight;
        tiles[12][45] = TileType::PipeBodyLeft;
        tiles[12][46] = TileType::PipeBodyRight;

        // Brick Pyramid (x=80..86)
        for step in 0..4 {
            for x in (80 + step)..86 {
                tiles[12 - step][x] = TileType::Brick;
            }
        }

        // Flagpole (x=100)
        for y in 4..13 {
            tiles[y][100] = TileType::Flagpole;
        }

        // Castle (x=106..114)
        for y in 8..13 {
            for x in 106..114 {
                tiles[y][x] = TileType::CastleWall;
            }
        }
        tiles[11][109] = TileType::CastleDoor;
        tiles[12][109] = TileType::CastleDoor;

        let enemies = vec![
            Enemy::new_goomba(16.0, 12.0),
            Enemy::new_goomba(24.0, 12.0),
            Enemy::new_goomba(32.0, 12.0),
            Enemy::new_koopa(52.0, 12.0),
            Enemy::new_goomba(62.0, 12.0),
            Enemy::new_goomba(64.0, 12.0),
            Enemy::new_koopa(88.0, 12.0),
        ];

        Self {
            world: 1,
            level: 1,
            name: "World 1-1 : Mushroom Kingdom Ground".to_string(),
            width,
            height: LEVEL_HEIGHT,
            tiles,
            enemies,
            items: Vec::new(),
            background_type: "overworld".to_string(),
            music_theme: "overworld".to_string(),
            is_vip_locked: false,
        }
    }

    /// Loads World 1-2: Underground Cavern
    pub fn world_1_2() -> Self {
        let width = 110;
        let mut tiles = vec![vec![TileType::Air; width]; LEVEL_HEIGHT];

        // Ceiling & Floor
        for x in 0..width {
            tiles[0][x] = TileType::Brick;
            tiles[1][x] = TileType::Brick;
            if x < 40 || (x > 44 && x < 85) || x > 90 {
                tiles[13][x] = TileType::Ground;
                tiles[14][x] = TileType::Ground;
            }
        }

        // Overhead platforms with coins
        for x in 15..28 {
            tiles[8][x] = TileType::Brick;
        }
        tiles[5][20] = TileType::MysteryBox { item: ItemType::StarMan, is_opened: false };

        let enemies = vec![
            Enemy::new_goomba(12.0, 12.0),
            Enemy::new_koopa(22.0, 7.0),
            Enemy::new_goomba(30.0, 12.0),
            Enemy::new_koopa(50.0, 12.0),
            Enemy::new_goomba(65.0, 12.0),
            Enemy::new_goomba(72.0, 12.0),
        ];

        Self {
            world: 1,
            level: 2,
            name: "World 1-2 : Underground Cavern".to_string(),
            width,
            height: LEVEL_HEIGHT,
            tiles,
            enemies,
            items: Vec::new(),
            background_type: "underground".to_string(),
            music_theme: "underground".to_string(),
            is_vip_locked: false,
        }
    }

    /// Loads World 1-3: Mushroom Treetops (Pro Plan)
    pub fn world_1_3() -> Self {
        let width = 110;
        let mut tiles = vec![vec![TileType::Air; width]; LEVEL_HEIGHT];

        // Scattered Mushroom platforms
        for x in 0..10 {
            tiles[13][x] = TileType::Ground;
            tiles[14][x] = TileType::Ground;
        }

        let platforms = [(14, 22, 10), (26, 32, 7), (36, 46, 9), (52, 60, 6), (66, 76, 8), (82, 92, 10), (96, 110, 13)];
        for (x1, x2, y) in platforms {
            for x in x1..x2 {
                tiles[y][x] = TileType::Brick;
            }
        }

        let enemies = vec![
            Enemy::new_koopa(18.0, 9.0),
            Enemy::new_koopa(40.0, 8.0),
            Enemy::new_goomba(70.0, 7.0),
        ];

        Self {
            world: 1,
            level: 3,
            name: "World 1-3 : Mushroom Treetops (Pro Nostr Tier)".to_string(),
            width,
            height: LEVEL_HEIGHT,
            tiles,
            enemies,
            items: Vec::new(),
            background_type: "athletic".to_string(),
            music_theme: "athletic".to_string(),
            is_vip_locked: true,
        }
    }

    /// Loads World 1-4: Bowser's Castle & Lava Bridge (Legendary VIP Tier)
    pub fn world_1_4() -> Self {
        let width = 100;
        let mut tiles = vec![vec![TileType::Air; width]; LEVEL_HEIGHT];

        // Stone Ceiling & Castle Walls
        for x in 0..width {
            tiles[0][x] = TileType::CastleWall;
            tiles[1][x] = TileType::CastleWall;
        }

        // Castle Floor with Lava Pits
        for x in 0..width {
            if (x >= 20 && x <= 26) || (x >= 45 && x <= 52) || (x >= 68 && x <= 85) {
                tiles[13][x] = TileType::Lava;
                tiles[14][x] = TileType::Lava;
            } else {
                tiles[13][x] = TileType::CastleWall;
                tiles[14][x] = TileType::CastleWall;
            }
        }

        // Bridge over Bowser Lava Pit (x=70..84)
        for x in 70..84 {
            tiles[11][x] = TileType::Brick;
        }
        // Axe switch behind Bowser
        tiles[10][85] = TileType::AxeSwitch;

        // Peach chamber (x=88..98)
        for y in 6..13 {
            tiles[y][98] = TileType::CastleWall;
        }

        let enemies = vec![
            Enemy::new_goomba(12.0, 12.0),
            Enemy::new_koopa(35.0, 12.0),
            Enemy::new_bowser(78.0, 10.0), // Bowser Boss on bridge!
        ];

        Self {
            world: 1,
            level: 4,
            name: "World 1-4 : Bowser's Castle (Legendary VIP Tier - NIP-04/44 Encrypted)".to_string(),
            width,
            height: LEVEL_HEIGHT,
            tiles,
            enemies,
            items: Vec::new(),
            background_type: "castle".to_string(),
            music_theme: "castle".to_string(),
            is_vip_locked: true,
        }
    }

    pub fn load_level(world: u32, level: u32) -> Self {
        match (world, level) {
            (1, 1) => Self::world_1_1(),
            (1, 2) => Self::world_1_2(),
            (1, 3) => Self::world_1_3(),
            (1, 4) => Self::world_1_4(),
            _ => Self::world_1_1(),
        }
    }
}
