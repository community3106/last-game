//! 描画 / rendering: turn the world into an ASCII frame.
//!
//! フレームは [`Cell`] の配列 (文字 + スタイル) です。
//! * [`Frame::to_text`] — 端末・ログ・テスト用のプレーンテキスト
//! * [`Frame::to_html`] — ブラウザ用 (同じスタイルの連続をまとめて `<span>` にし、DOM を軽くする)
//!
//! ブラウザでは 1 フレームごとにこの HTML を `<pre>` の中に入れ替えるだけで描画できます。

use crate::world::{GameState, World};

/// 文字の色 / visual style of a cell (mapped to a CSS class).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Style {
    /// 空 / sky (no class, plain space)
    Sky,
    /// 地面 / ground
    Ground,
    /// レンガ / brick
    Brick,
    /// ハテナブロック / question block
    Question,
    /// 使用済みブロック / used block
    Used,
    /// 硬いブロック / hard block
    Hard,
    /// 土管 / pipe
    Pipe,
    /// トゲ / spikes
    Spike,
    /// コイン / coin
    Coin,
    /// 敵 / enemy
    Enemy,
    /// プレイヤー / player
    Player,
    /// 旗 / flag
    Flag,
    /// ポップアップ / popup
    Popup,
    /// 目立たない飾り / dim decoration
    Dim,
}

impl Style {
    /// CSS クラス名 / CSS class name used by [`Frame::to_html`].
    pub const fn class(self) -> Option<&'static str> {
        match self {
            Style::Sky => None,
            Style::Ground => Some("t-ground"),
            Style::Brick => Some("t-brick"),
            Style::Question => Some("t-question"),
            Style::Used => Some("t-used"),
            Style::Hard => Some("t-hard"),
            Style::Pipe => Some("t-pipe"),
            Style::Spike => Some("t-spike"),
            Style::Coin => Some("t-coin"),
            Style::Enemy => Some("t-enemy"),
            Style::Player => Some("t-player"),
            Style::Flag => Some("t-flag"),
            Style::Popup => Some("t-popup"),
            Style::Dim => Some("t-dim"),
        }
    }
}

/// 1 文字 / a single rendered character.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    /// 表示する文字 / the character to draw
    pub ch: char,
    /// 色 / style
    pub style: Style,
}

impl Cell {
    /// 空のセル / a blank sky cell.
    pub const EMPTY: Cell = Cell {
        ch: ' ',
        style: Style::Sky,
    };

    /// セルを作る / build a cell.
    pub const fn new(ch: char, style: Style) -> Self {
        Self { ch, style }
    }
}

/// 描画オプション / how many characters to render.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RenderOptions {
    /// 横幅 (文字数) / viewport width in characters
    pub cols: u32,
    /// 高さ (文字数) / viewport height in characters
    pub rows: u32,
    /// プレイヤーを点滅させるか (無敵時間) / blink the player while invulnerable
    pub blink: bool,
}

impl RenderOptions {
    /// サイズを指定して作る / new options with the given size.
    pub const fn new(cols: u32, rows: u32) -> Self {
        Self {
            cols,
            rows,
            blink: true,
        }
    }

    /// 点滅を無効にする / disable the invulnerability blink (deterministic tests).
    pub const fn without_blink(mut self) -> Self {
        self.blink = false;
        self
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self::new(48, 20)
    }
}

/// 描画済みの 1 フレーム / a rendered frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// 横幅 / width
    pub cols: u32,
    /// 高さ / height
    pub rows: u32,
    /// 左上から右下へ並んだセル / cells in row-major order
    pub cells: Vec<Cell>,
}

impl Frame {
    /// 空のフレーム / an empty frame.
    pub fn new(cols: u32, rows: u32) -> Self {
        Self {
            cols,
            rows,
            cells: vec![Cell::EMPTY; (cols * rows) as usize],
        }
    }

    /// セルを書き換える / overwrite a cell.
    pub fn put(&mut self, col: u32, row: u32, ch: char, style: Style) {
        if col >= self.cols || row >= self.rows {
            return;
        }
        let index = (row * self.cols + col) as usize;
        self.cells[index] = Cell::new(ch, style);
    }

    /// セルを取得する / read a cell.
    pub fn get(&self, col: u32, row: u32) -> Cell {
        if col >= self.cols || row >= self.rows {
            return Cell::EMPTY;
        }
        self.cells[(row * self.cols + col) as usize]
    }

    /// 1 行を文字列にする (スタイル無視) / one row as plain text.
    pub fn row_text(&self, row: u32) -> String {
        (0..self.cols)
            .map(|col| self.get(col, row).ch)
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    /// プレーンテキストにする / render as plain text.
    pub fn to_text(&self) -> String {
        (0..self.rows)
            .map(|row| {
                (0..self.cols)
                    .map(|col| self.get(col, row).ch)
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// HTML にする (スタイルの連続は 1 つの `<span>` にまとめる) / render as HTML.
    pub fn to_html(&self) -> String {
        let mut out = String::with_capacity((self.cols * self.rows) as usize * 2);
        for row in 0..self.rows {
            let mut col = 0;
            while col < self.cols {
                let cell = self.get(col, row);
                let mut run = String::new();
                let mut end = col;
                while end < self.cols && self.get(end, row).style == cell.style {
                    let ch = self.get(end, row).ch;
                    match ch {
                        '<' => run.push_str("&lt;"),
                        '>' => run.push_str("&gt;"),
                        '&' => run.push_str("&amp;"),
                        other => run.push(other),
                    }
                    end += 1;
                }
                match cell.style.class() {
                    Some(class) => {
                        out.push_str("<span class=\"");
                        out.push_str(class);
                        out.push_str("\">");
                        out.push_str(&run);
                        out.push_str("</span>");
                    }
                    None => out.push_str(&run),
                }
                col = end;
            }
            out.push('\n');
        }
        out
    }

    /// `<span>` の数 (テストと性能確認用) / number of style runs.
    pub fn run_count(&self) -> usize {
        let mut count = 0;
        for row in 0..self.rows {
            let mut col = 0;
            while col < self.cols {
                count += 1;
                let style = self.get(col, row).style;
                while col < self.cols && self.get(col, row).style == style {
                    col += 1;
                }
            }
        }
        count
    }
}

/// ワールドを描画する / render the world into a frame.
///
/// * カメラは [`World::camera_x`] に従って横スクロールします。
/// * レベルは画面の下端に合わせて配置されます (上は空)。
pub fn render_world(world: &World, options: RenderOptions) -> Frame {
    let cols = options.cols.max(4) as i32;
    let rows = options.rows.max(4) as i32;
    let mut frame = Frame::new(cols as u32, rows as u32);

    let cam = world.camera_x.floor();
    // 端数のカメラ位置は 1 文字ずらして表現する (ASCII にピクセル補間は無い)
    // / ASCII has no sub-character camera, so snap to the nearest tile
    let shift = if world.camera_x - cam > 0.5 { 1 } else { 0 };
    let base_col = cam as i32 + shift;
    // レベルの最下段を画面の最下段に合わせる / anchor the level to the bottom of the view
    let base_row = world.level.height() - rows;

    for screen_row in 0..rows {
        let world_row = base_row + screen_row;
        for screen_col in 0..cols {
            let cell = sample(world, base_col + screen_col, world_row);
            frame.put(screen_col as u32, screen_row as u32, cell.ch, cell.style);
        }
    }

    // コインのポップアップなど / floating popups
    for popup in &world.popups {
        let col = popup.x.floor() as i32;
        let row = popup.y.floor() as i32;
        put_world(&mut frame, col - base_col, row - base_row, popup.glyph, Style::Popup);
    }

    // 敵 / enemies (1 体 1 文字)
    for enemy in &world.enemies {
        let (ch, style) = match enemy.squashed {
            Some(_) => ('_', Style::Dim),
            None => (enemy.kind.glyph(), Style::Enemy),
        };
        let col = (enemy.x + enemy.w * 0.5).floor() as i32;
        let row = (enemy.y + enemy.h - 0.5).floor() as i32;
        put_world(&mut frame, col - base_col, row - base_row, ch, style);
    }

    // プレイヤー / the player (blinks while invulnerable)
    let blink = options.blink
        && world.player.invuln > 0.0
        && (world.tick / 4) % 2 == 0
        && !matches!(world.state, GameState::GameOver);
    if !blink {
        let player = &world.player;
        let col = (player.x + player.w * 0.5).floor() as i32;
        let row = (player.feet() - 0.5).floor() as i32;
        put_world(&mut frame, col - base_col, row - base_row, '@', Style::Player);
    }

    frame
}

/// 画面座標に書き込む (範囲外は無視) / plot a glyph using screen coordinates.
fn put_world(frame: &mut Frame, col: i32, row: i32, ch: char, style: Style) {
    if col < 0 || row < 0 {
        return;
    }
    frame.put(col as u32, row as u32, ch, style);
}

/// 1 マスをサンプリングする / pick what should be visible at one tile.
fn sample(world: &World, col: i32, row: i32) -> Cell {
    // コイン / coins
    for coin in &world.coins {
        if !coin.collected && coin.col == col && coin.row == row {
            return Cell::new('o', Style::Coin);
        }
    }
    let tile = world.level.tile(col, row);
    let style = match tile {
        crate::level::Tile::Sky => return Cell::EMPTY,
        crate::level::Tile::Ground => Style::Ground,
        crate::level::Tile::Brick => Style::Brick,
        crate::level::Tile::Question { .. } => Style::Question,
        crate::level::Tile::Used => Style::Used,
        crate::level::Tile::Hard => Style::Hard,
        crate::level::Tile::PipeLeft | crate::level::Tile::PipeRight => Style::Pipe,
        crate::level::Tile::Spike => Style::Spike,
        crate::level::Tile::Flag | crate::level::Tile::Flagpole => Style::Flag,
    };
    Cell::new(tile.glyph(), style)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Input;
    use crate::level::Level;

    const FLAT: &str = "
#            #
#   p        #
##############
";

    fn world() -> World {
        World::new(Level::from_ascii("render", FLAT).unwrap())
    }

    #[test]
    fn frame_has_the_requested_size() {
        let frame = world().render(RenderOptions::new(24, 8));
        assert_eq!(frame.cols, 24);
        assert_eq!(frame.rows, 8);
        assert_eq!(frame.cells.len(), 24 * 8);
    }

    #[test]
    fn draws_tiles_and_the_player() {
        let mut world = world();
        for _ in 0..4 {
            world.step(Input::new());
        }
        let frame = world.render(RenderOptions::new(20, 8).without_blink());
        let text = frame.to_text();
        assert!(text.contains('@'), "player missing:\n{text}");
        assert!(text.contains('#'), "ground missing:\n{text}");
        // プレイヤーは 3 文字目あたり / the player stands near the spawn column
        let player_row = text
            .lines()
            .find(|line| line.contains('@'))
            .expect("a row with the player");
        let col = player_row.chars().position(|ch| ch == '@').unwrap() as i32;
        assert!((3..=4).contains(&col), "player at column {col}");
    }

    #[test]
    fn camera_scrolls_the_view() {
        let mut world = world();
        world.set_viewport(20);
        for _ in 0..4 {
            world.step(Input::new());
        }
        let before = world.render(RenderOptions::new(20, 8));
        world.camera_x = 4.0;
        let after = world.render(RenderOptions::new(20, 8));
        assert_ne!(before.to_text(), after.to_text());
    }

    #[test]
    fn html_output_is_escaped_and_grouped() {
        let mut world = world();
        for _ in 0..4 {
            world.step(Input::new());
        }
        let frame = world.render(RenderOptions::new(20, 8));
        let html = frame.to_html();
        assert!(html.contains("class=\"t-ground\""));
        assert!(html.contains("class=\"t-player\""));
        assert!(frame.run_count() < (frame.cols * frame.rows) as usize);
        assert!(!html.contains("<script"));
    }

    #[test]
    fn text_output_has_one_line_per_row() {
        let frame = world().render(RenderOptions::new(16, 6));
        assert_eq!(frame.to_text().lines().count(), 6);
    }
}
