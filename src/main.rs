//! last-game entry — 100% Rust CSR via Leptos
//! `Trunk` が `index.html` の `<link data-trunk rel="rust">` からこのクレートを `wasm32-unknown-unknown` でビルドします。

use last_game::App;

fn main() {
    // WASM パニックを console に
    console_error_panic_hook::set_once();
    // log -> console
    _ = console_log::init_with_level(log::Level::Debug);
    log::info!("last-game booting (100% Rust / Leptos CSR)");

    leptos::mount_to_body(App);
}
