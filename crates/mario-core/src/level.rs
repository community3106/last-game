//! レベル定義 (ASCII アート) / Level definition from ASCII art.
//!
//! レベルは 1 文字 = 1 タイルの ASCII アートで書きます。
//! 行の長さが違う場合は、最も長い行に合わせて右側を空白で埋めます。
//!
//! | 文字 | 意味 | 当たり判定 |
//! |------|------|-----------|
//! | `' '` | 空 / sky | なし |
//! | `#` | 地面ブロック / ground | 固体 |
//! | `B` | レンガ / brick | 固体 (下から叩くと壊れて `X` に) |
//! | `?` | ハテナブロック (コイン) | 固体 |
//! | `M` | ハテナブロック (キノコ) | 固体 |
//! | `X` | 使用済みブロック | 固体 |
//! | `=` | 硬いブロック / 足場 | 固体 |
//! | `[` | 土管 (左) | 固体 |
//! | `]` | 土管 (右) | 固体 |
//! | `^` | トゲ / spikes | ダメージ |
//! | `o` | コイン | 取得 |
//! | `g` | クリボー / goomba | 敵 |
//! | `k` | ノコノコ / koopa | 敵 |
//! | `p` | プレイヤー開始位置 | — |
//! | `F` | ゴールの旗 / goal | — |
//! | `|` | 旗ポール / flag pole | — |

use crate::world::EnemyKind;
use std::fmt;

/// タイル 1 マスの種類 / a single tile of the level.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Tile {
    /// 空 / sky
    #[default]
    Sky,
    /// 地面 / ground block
    Ground,
    /// レンガ / brick block
    Brick,
    /// ハテナブロック / question block
    Question {
        /// 中身がキノコかどうか / whether the block holds a mushroom
        mushroom: bool,
    },
    /// 使用済みブロック / used block
    Used,
    /// 硬いブロック / hard block
    Hard,
    /// 土管 (左) / pipe (left half)
    PipeLeft,
    /// 土管 (右) / pipe (right half)
    PipeRight,
    /// トゲ / spikes
    Spike,
    /// ゴールの旗 / goal flag
    Flag,
    /// 旗ポール / flag pole
    Flagpole,
}

impl Tile {
    /// 固体かどうか (ぶつかると止まる) / does this tile block movement?
    pub const fn is_solid(self) -> bool {
        matches!(
            self,
            Tile::Ground
                | Tile::Brick
                | Tile::Question { .. }
                | Tile::Used
                | Tile::Hard
                | Tile::PipeLeft
                | Tile::PipeRight
        )
    }

    /// ダメージを受けるタイルか / does this tile hurt the player?
    pub const fn is_hazard(self) -> bool {
        matches!(self, Tile::Spike)
    }

    /// 下から叩けるタイルか / can this tile be bumped from below?
    pub const fn is_bumpable(self) -> bool {
        matches!(self, Tile::Brick | Tile::Question { .. })
    }

    /// 描画に使う文字 / glyph used for rendering.
    pub const fn glyph(self) -> char {
        match self {
            Tile::Sky => ' ',
            Tile::Ground => '#',
            Tile::Brick => 'B',
            Tile::Question { .. } => '?',
            Tile::Used => 'X',
            Tile::Hard => '=',
            Tile::PipeLeft => '[',
            Tile::PipeRight => ']',
            Tile::Spike => '^',
            Tile::Flag => 'F',
            Tile::Flagpole => '|',
        }
    }
}

/// レベル読み込み時のエラー / error returned while parsing a level.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LevelError {
    /// アートが空 / the art was empty
    Empty,
    /// プレイヤー開始位置 (`p`) が無い / no player spawn
    MissingPlayerSpawn,
    /// 使えない文字 / unknown glyph
    UnknownGlyph {
        /// 見つかった文字 / the offending character
        ch: char,
        /// 行 (0 始まり) / row, zero based
        row: usize,
        /// 列 (0 始まり) / column, zero based
        col: usize,
    },
    /// レベルが大きすぎる / level too large
    TooLarge {
        /// 幅 / width
        width: usize,
        /// 高さ / height
        height: usize,
    },
}

impl fmt::Display for LevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LevelError::Empty => write!(f, "レベルが空です / level art is empty"),
            LevelError::MissingPlayerSpawn => write!(
                f,
                "プレイヤー開始位置 'p' がありません / level has no player spawn ('p')"
            ),
            LevelError::UnknownGlyph { ch, row, col } => write!(
                f,
                "不正な文字 {ch:?} (行 {row}, 列 {col}) / unknown glyph at row {row}, col {col}"
            ),
            LevelError::TooLarge { width, height } => {
                write!(f, "レベルが大きすぎます ({width}x{height}) / level too large")
            }
        }
    }
}

impl std::error::Error for LevelError {}

/// レベルで使える最大サイズ / maximum level size (memory safety net).
pub const MAX_WIDTH: usize = 4096;
/// レベルで使える最大の高さ / maximum level height.
pub const MAX_HEIGHT: usize = 256;

/// 敵の出現位置 / an enemy spawn point baked into the level art.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EnemySpawn {
    /// 敵の種類 / enemy kind
    pub kind: EnemyKind,
    /// 列 / column
    pub col: i32,
    /// 行 / row
    pub row: i32,
}

/// パース済みのレベル / a parsed level.
#[derive(Clone, Debug, PartialEq)]
pub struct Level {
    name: String,
    width: i32,
    height: i32,
    tiles: Vec<Tile>,
    coin_spots: Vec<(i32, i32)>,
    enemy_spawns: Vec<EnemySpawn>,
    player_spawn: (f32, f32),
    goal: Option<(i32, i32)>,
}

impl Level {
    /// ASCII アートからレベルを作る / build a level from ASCII art.
    pub fn from_ascii(name: impl Into<String>, art: &str) -> Result<Self, LevelError> {
        // タブは 4 スペースに展開し、先頭・末尾の空行は無視する
        // / expand tabs, drop leading/trailing blank lines
        let mut rows: Vec<String> = art
            .replace('\t', "    ")
            .split('\n')
            .map(|row| row.trim_end_matches('\r').to_string())
            .collect();

        while rows.first().is_some_and(|row| row.trim().is_empty()) {
            rows.remove(0);
        }
        while rows.last().is_some_and(|row| row.trim().is_empty()) {
            rows.pop();
        }
        if rows.is_empty() {
            return Err(LevelError::Empty);
        }

        let width = rows.iter().map(|row| row.chars().count()).max().unwrap_or(0);
        let height = rows.len();
        if width == 0 || width > MAX_WIDTH || height > MAX_HEIGHT {
            return Err(LevelError::TooLarge { width, height });
        }

        let mut tiles = vec![Tile::Sky; width * height];
        let mut coin_spots = Vec::new();
        let mut enemy_spawns = Vec::new();
        let mut player_spawn = None;
        let mut goal = None;

        for (row, line) in rows.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let (x, y) = (col as i32, row as i32);
                let index = row * width + col;
                let tile = match ch {
                    ' ' | '.' => Tile::Sky,
                    '#' => Tile::Ground,
                    'B' => Tile::Brick,
                    '?' => Tile::Question { mushroom: false },
                    'M' => Tile::Question { mushroom: true },
                    'X' => Tile::Used,
                    '=' => Tile::Hard,
                    '[' => Tile::PipeLeft,
                    ']' => Tile::PipeRight,
                    '^' => Tile::Spike,
                    'F' => Tile::Flag,
                    '|' => Tile::Flagpole,
                    'o' => {
                        coin_spots.push((x, y));
                        Tile::Sky
                    }
                    'g' => {
                        enemy_spawns.push(EnemySpawn {
                            kind: EnemyKind::Goomba,
                            col: x,
                            row: y,
                        });
                        Tile::Sky
                    }
                    'k' => {
                        enemy_spawns.push(EnemySpawn {
                            kind: EnemyKind::Koopa,
                            col: x,
                            row: y,
                        });
                        Tile::Sky
                    }
                    'p' => {
                        // スポーン位置は「そのマスの左上」に合わせる
                        // / spawn aligned to the top-left of the tile
                        player_spawn = Some((x as f32, y as f32));
                        Tile::Sky
                    }
                    other => {
                        return Err(LevelError::UnknownGlyph {
                            ch: other,
                            row,
                            col,
                        })
                    }
                };
                tiles[index] = tile;
            }
        }

        let Some(player_spawn) = player_spawn else {
            return Err(LevelError::MissingPlayerSpawn);
        };

        let mut level = Self {
            name: name.into(),
            width: width as i32,
            height: height as i32,
            tiles,
            coin_spots,
            enemy_spawns,
            player_spawn,
            goal,
        };
        level.goal = find_goal(&level);
        Ok(level)
    }

    /// レベル名 / level name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 幅 (タイル数) / width in tiles
    pub const fn width(&self) -> i32 {
        self.width
    }

    /// 高さ (タイル数) / height in tiles
    pub const fn height(&self) -> i32 {
        self.height
    }

    /// タイルを取得する (範囲外は [`Tile::Sky`]、ただし左右の外は壁扱い) / tile lookup.
    pub fn tile(&self, col: i32, row: i32) -> Tile {
        if col < 0 || row < 0 || col >= self.width || row >= self.height {
            return Tile::Sky;
        }
        let index = (row * self.width + col) as usize;
        self.tiles[index]
    }

    /// タイルを書き換える / mutate a tile (block bumps, broken bricks).
    pub fn set_tile(&mut self, col: i32, row: i32, tile: Tile) {
        if col < 0 || row < 0 || col >= self.width || row >= self.height {
            return;
        }
        let index = (row * self.width + col) as usize;
        self.tiles[index] = tile;
    }

    /// 固体判定 (左右の外側は壁、下は奈落) / solid lookup, sides act as walls, the bottom is a pit.
    pub fn solid_at(&self, col: i32, row: i32) -> bool {
        if col < 0 || col >= self.width {
            return true; // 左右の外は壁 / walls outside the level bounds
        }
        if row < 0 {
            return false; // 上は開いている / open sky above
        }
        if row >= self.height {
            return false; // 下は奈落 / pit below
        }
        self.tile(col, row).is_solid()
    }

    /// ダメージタイル判定 / hazard lookup
    pub fn hazard_at(&self, col: i32, row: i32) -> bool {
        self.tile(col, row).is_hazard()
    }

    /// コインの位置 / coin positions
    pub fn coin_spots(&self) -> &[(i32, i32)] {
        &self.coin_spots
    }

    /// 敵の出現位置 / enemy spawns
    pub fn enemy_spawns(&self) -> &[EnemySpawn] {
        &self.enemy_spawns
    }

    /// プレイヤー開始位置 (タイル座標) / player spawn in tile coordinates
    pub const fn player_spawn(&self) -> (f32, f32) {
        self.player_spawn
    }

    /// ゴール位置 / goal position, if the level has one
    pub const fn goal(&self) -> Option<(i32, i32)> {
        self.goal
    }

    /// タイルに含まれる固体の数 / number of solid tiles (useful for tests & stats)
    pub fn solid_count(&self) -> usize {
        self.tiles.iter().filter(|tile| tile.is_solid()).count()
    }

    /// アートに戻す (テスト・デバッグ用) / render the tiles back to ASCII art.
    pub fn to_ascii(&self) -> String {
        let mut out = String::with_capacity(((self.width + 1) * self.height) as usize);
        for row in 0..self.height {
            for col in 0..self.width {
                if self.coin_spots.contains(&(col, row)) {
                    out.push('o');
                } else {
                    out.push(self.tile(col, row).glyph());
                }
            }
            out.push('\n');
        }
        out
    }
}

/// 旗 (`F`) の位置を探す / locate the goal flag.
fn find_goal(level: &Level) -> Option<(i32, i32)> {
    for row in 0..level.height {
        for col in 0..level.width {
            if level.tile(col, row) == Tile::Flag {
                return Some((col, row));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY: &str = r#"
##########
#        #
#  p   o #
#    g   #
##########
"#;

    #[test]
    fn parses_ascii_art() {
        let level = Level::from_ascii("tiny", TINY).expect("level should parse");
        assert_eq!(level.name(), "tiny");
        assert_eq!(level.width(), 10);
        assert_eq!(level.height(), 5);
        assert_eq!(level.player_spawn(), (3.0, 2.0));
        assert_eq!(level.coin_spots(), &[(7, 2)]);
        assert_eq!(level.enemy_spawns().len(), 1);
        assert_eq!(level.enemy_spawns()[0].kind, EnemyKind::Goomba);
        assert!(level.solid_at(0, 0));
        assert!(!level.solid_at(3, 2));
    }

    #[test]
    fn ragged_rows_are_padded_with_sky() {
        let art = "###\n#p\n###\n";
        let level = Level::from_ascii("ragged", art).unwrap();
        assert_eq!(level.width(), 3);
        assert_eq!(level.tile(2, 1), Tile::Sky);
    }

    #[test]
    fn outside_bounds_are_walls_but_pit_is_open() {
        let level = Level::from_ascii("tiny", TINY).unwrap();
        assert!(level.solid_at(-1, 2));
        assert!(level.solid_at(level.width(), 2));
        assert!(!level.solid_at(3, level.height() + 5));
        assert!(!level.solid_at(3, -1));
    }

    #[test]
    fn rejects_unknown_glyph() {
        let err = Level::from_ascii("bad", "###\n#Z#\n#p#\n").unwrap_err();
        assert_eq!(
            err,
            LevelError::UnknownGlyph {
                ch: 'Z',
                row: 1,
                col: 1
            }
        );
    }

    #[test]
    fn rejects_level_without_player_spawn() {
        let err = Level::from_ascii("noplayer", "###\n#o#\n###\n").unwrap_err();
        assert_eq!(err, LevelError::MissingPlayerSpawn);
    }

    #[test]
    fn rejects_empty_art() {
        assert_eq!(Level::from_ascii("empty", "\n\n   \n").unwrap_err(), LevelError::Empty);
    }

    #[test]
    fn bumpable_and_solid_flags() {
        assert!(Tile::Question { mushroom: false }.is_solid());
        assert!(Tile::Question { mushroom: true }.is_bumpable());
        assert!(Tile::Brick.is_bumpable());
        assert!(!Tile::Sky.is_solid());
        assert!(Tile::Spike.is_hazard());
        assert!(!Tile::Spike.is_solid());
        assert_eq!(Tile::PipeLeft.glyph(), '[');
    }

    #[test]
    fn round_trips_to_ascii() {
        let level = Level::from_ascii("tiny", TINY).unwrap();
        let art = level.to_ascii();
        assert!(art.contains("p"), "spawn tile should be blank, player is an entity");
        assert_eq!(art.lines().count(), 5);
        let reparsed = Level::from_ascii("tiny2", &art).unwrap();
        assert_eq!(reparsed.tile(7, 2), Tile::Sky);
        assert_eq!(reparsed.coin_spots(), level.coin_spots());
    }

    #[test]
    fn finds_goal_flag() {
        let art = "#######\n#p   F#\n#######\n";
        let level = Level::from_ascii("goal", art).unwrap();
        assert_eq!(level.goal(), Some((5, 1)));
    }

    #[test]
    fn world_1_1_and_1_2_are_valid() {
        for (name, art) in [
            ("1-1", crate::levels::WORLD_1_1),
            ("1-2", crate::levels::WORLD_1_2),
        ] {
            let level = Level::from_ascii(name, art)
                .unwrap_or_else(|err| panic!("level {name} failed to parse: {err}"));
            assert!(level.width() > 40, "{name} should be a real level");
            assert!(level.goal().is_some(), "{name} needs a goal flag 'F'");
            assert!(!level.enemy_spawns().is_empty(), "{name} needs enemies");
        }
    }
}
