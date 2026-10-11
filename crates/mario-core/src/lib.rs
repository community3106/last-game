//! # mario-core — 依存ゼロの ASCII プラットフォーマーエンジン
//!
//! 「LAST GAME」のゲーム部分は **100% Rust** です。このクレートは
//! 外部クレートに一切依存せず、ブラウザ (wasm32) でもネイティブでも同じように動きます。
//!
//! * [`level`] — ASCII アートからレベルをパースする
//! * [`world`] — 物理演算・敵・アイテム・ゲーム進行
//! * [`render`] — タイル世界を ASCII のフレーム (HTML / テキスト) に変換する
//! * [`input`] — 入力 (左右・ジャンプ・ダッシュ)
//! * [`bot`] — デモプレイ用の自動運転 AI
//!
//! ```no_run
//! use mario_core::{Input, Level, RenderOptions, World};
//!
//! let level = Level::from_ascii("1-1", mario_core::levels::WORLD_1_1).unwrap();
//! let mut world = World::new(level);
//! world.step(Input::new().right(true).jump(true));
//! let frame = world.render(RenderOptions::default());
//! println!("{}", frame.to_text());
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::dbg_macro, clippy::todo)]

pub mod bot;
pub mod input;
pub mod level;
pub mod levels;
pub mod render;
pub mod world;

pub use bot::Bot;
pub use input::Input;
pub use level::{Level, LevelError, Tile};
pub use render::{Cell, Frame, RenderOptions, Style};
pub use world::{
    Coin, Enemy, EnemyKind, Event, GameState, Player, Popup, World, FIXED_DT, TILE_H,
};

/// このゲームのタイトル / name of the game.
pub const GAME_TITLE: &str = "LAST GAME — ASCII MARIO";

/// アスキーアートのロゴ / title-screen logo.
pub const LOGO: &str = r#"
    __    ___   _____ ______      ____    __    ____  __  __ ____  ____
   / /   /   | / ___//_  __/     / __/   / /   / __ \/ / / // __ \/ __/
  / /   / /| | \__ \  / /       / _/    / /__ / /_/ / /_/ // /_/ /\ \
 / /___/ ___ |___/ / / /       / ___/  /____/ \__, /\____// ____//_/
/_____/_/  |_/____/ /_/       /_/            /____/      /_/

         ASCII MARIO  —  100% Rust  —  powered by Nostr
"#;
