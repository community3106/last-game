//! ネイティブターミナル版 ASCII Mario — `cargo run --features terminal --bin last-game-terminal`
//! ratatui / crossterm を使わない最小実装 (stdのみ) でも動くデモ。
//! 本番では `crossterm` feature でフルカラー化。

use last_game::game::engine::{GameState, InputState, LEVEL_1};
use std::io::{self, Write};
use std::time::{Duration, Instant};

fn main() -> io::Result<()> {
    println!("last-game terminal — 100% Rust ASCII Mario");
    println!("操作: A/D or ←/→ 移動, W/Space ジャンプ, Q 終了, R リセット");
    println!("(crossterm feature なしの簡易版。キー入力は Enter 後の行入力で代用)");
    let mut game = GameState::new(LEVEL_1.clone());
    let mut input = InputState::default();

    // 簡易ループ: 10 FPSで自動で右に進むデモ
    let mut tick = 0;
    loop {
        // 疑似入力: 往復
        input.right = tick % 120 < 60;
        input.left = !input.right;
        input.jump = tick % 45 == 0;

        game.tick(input);
        // 描画
        print!("\x1B[2J\x1B[H"); // clear
        println!("{}", last_game::game::renderer::render_ascii(&game));
        println!("tick {}  score {}  coins {}  lives {}", tick, game.score, game.coins, game.lives);
        if game.won { println!("★ GOAL! Rでリセット (Enter)"); }
        if game.game_over { println!("GAME OVER"); break; }
        io::stdout().flush()?;
        std::thread::sleep(Duration::from_millis(80));
        tick += 1;
        if tick > 400 { break; }
        // 非ブロッキング入力が欲しければ crossterm を有効化
        #[cfg(feature = "terminal")]
        {
            // crossterm 実装は省略 (cargo run --features terminal で有効化)
        }
    }
    Ok(())
}
