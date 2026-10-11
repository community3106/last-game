//! `cargo run -p mario-core --example ascii_frame` でフレームを端末に表示する
//! / print a rendered frame to the terminal (CI ではログに残ります).
//!
//! 引数でステージを選べます: `cargo run -p mario-core --example ascii_frame -- 1-2`

use mario_core::{Bot, Input, Level, RenderOptions, World};

fn main() {
    let name = std::env::args().nth(1).unwrap_or_else(|| "1-1".to_string());
    let art = match name.as_str() {
        "1-2" => mario_core::levels::WORLD_1_2,
        _ => mario_core::levels::WORLD_1_1,
    };
    let level = Level::from_ascii(&name, art).expect("level should parse");
    let mut world = World::new(level);
    let mut bot = Bot::new();

    println!("{}", mario_core::LOGO);
    for _ in 0..240 {
        let input = bot.input(&world);
        world.update(input, mario_core::FIXED_DT);
    }
    world.set_viewport(72);

    let frame = world.render(RenderOptions::new(72, 18));
    println!("=== {} (ボットが 240 フレーム操作) ===", name);
    println!("{}", frame.to_text());
    println!(
        "score={} coins={} lives={} time={} x={:.1} state={}",
        world.score,
        world.coin_count,
        world.lives,
        world.time_left(),
        world.player.x,
        world.state.label()
    );

    // 手動入力の例 / a manual input example
    let mut demo = World::new(Level::from_ascii("demo", mario_core::levels::WORLD_1_1).unwrap());
    for _ in 0..30 {
        demo.step(Input::new().right(true).run(true));
    }
    println!(
        "\n30 フレーム右に走った結果: x={:.2} vx={:.2}",
        demo.player.x, demo.player.vx
    );
}
