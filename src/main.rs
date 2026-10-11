pub mod app;
pub mod components;
pub mod game;
pub mod nostr;
pub mod stripe;

use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    
    mount_to_body(|| {
        view! {
            <app::App />
        }
    })
}
