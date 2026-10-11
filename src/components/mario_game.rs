use leptos::*;
use crate::game::engine::{GameEngine, GameStatus, InputState};
use crate::game::renderer::{AsciiRenderer, ColorTheme};
use crate::nostr::whitelist::SubscriptionTier;

#[component]
pub fn MarioGame(
    user_tier: ReadSignal<SubscriptionTier>,
    open_sub_modal: WriteSignal<bool>,
) -> impl IntoView {
    let (selected_world, set_selected_world) = create_signal((1u32, 1u32));
    let (theme, set_theme) = create_signal(ColorTheme::ClassicNes);
    let (scanlines_enabled, set_scanlines_enabled) = create_signal(true);
    let (sound_enabled, set_sound_enabled) = create_signal(true);

    // Cheats
    let (god_mode, set_god_mode) = create_signal(false);
    let (moon_jump, set_moon_jump) = create_signal(false);
    let (infinite_fireballs, set_infinite_fireballs) = create_signal(false);

    let (ascii_display, set_ascii_display) = create_signal(String::new());
    let (game_status, set_game_status) = create_signal(GameStatus::Playing);
    let (score_display, set_score_display) = create_signal(0u32);
    let (coins_display, set_coins_display) = create_signal(0u32);
    let (lives_display, set_lives_display) = create_signal(3i32);

    let handle_stage_select = move |world: u32, level: u32| {
        let tier = user_tier.get();
        if !tier.can_play_world(world, level) {
            open_sub_modal.set(true);
            return;
        }
        set_selected_world.set((world, level));
    };

    view! {
        <div class="mario-arcade-wrapper">
            <div class="arcade-cabinet-top">
                <div class="cabinet-marquee">
                    <span class="marquee-star">"★"</span>
                    <span class="marquee-text">"SUPER MARIO BROS. 1985 (100% RUST WASM)"</span>
                    <span class="marquee-star">"★"</span>
                </div>
            </div>

            <!-- Stage Selector Toolbar -->
            <div class="stage-bar">
                <div class="stage-buttons">
                    <button
                        class=move || if selected_world.get() == (1, 1) { "stage-btn active" } else { "stage-btn" }
                        on:click=move |_| handle_stage_select(1, 1)
                    >
                        "WORLD 1-1 [無料]"
                    </button>
                    <button
                        class=move || if selected_world.get() == (1, 2) { "stage-btn active" } else { "stage-btn" }
                        on:click=move |_| handle_stage_select(1, 2)
                    >
                        "WORLD 1-2 地下 [無料]"
                    </button>
                    <button
                        class=move || if selected_world.get() == (1, 3) { "stage-btn active" } else { "stage-btn pro-locked" }
                        on:click=move |_| handle_stage_select(1, 3)
                    >
                        {move || if user_tier.get() >= SubscriptionTier::ProGamer {
                            "WORLD 1-3 キノコ木 [PRO]"
                        } else {
                            "🔒 WORLD 1-3 [PRO限定]"
                        }}
                    </button>
                    <button
                        class=move || if selected_world.get() == (1, 4) { "stage-btn active" } else { "stage-btn vip-locked" }
                        on:click=move |_| handle_stage_select(1, 4)
                    >
                        {move || if user_tier.get() >= SubscriptionTier::LegendaryVip {
                            "WORLD 1-4 クッパ城 [VIP暗号化]"
                        } else {
                            "🔒 WORLD 1-4 クッパ城 [VIP限定/暗号化]"
                        }}
                    </button>
                </div>

                <div class="theme-controls">
                    <select
                        class="retro-select"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            match val.as_str() {
                                "matrix" => set_theme.set(ColorTheme::PhosphorGreen),
                                "amber" => set_theme.set(ColorTheme::Amber80s),
                                "cyberpunk" => set_theme.set(ColorTheme::CyberpunkNeon),
                                _ => set_theme.set(ColorTheme::ClassicNes),
                            }
                        }
                    >
                        <option value="classic">"🎨 テーマ: Classic NES 8-Bit"</option>
                        <option value="matrix">"🎨 テーマ: Matrix Phosphor Green"</option>
                        <option value="amber">"🎨 テーマ: 1980s Amber CRT"</option>
                        <option value="cyberpunk">"🎨 テーマ: Cyberpunk Synthwave"</option>
                    </select>

                    <button
                        class=move || if scanlines_enabled.get() { "toggle-btn active" } else { "toggle-btn" }
                        on:click=move |_| set_scanlines_enabled.update(|s| *s = !*s)
                    >
                        "📺 CRT 走査線"
                    </button>

                    <button
                        class=move || if sound_enabled.get() { "toggle-btn active" } else { "toggle-btn" }
                        on:click=move |_| set_sound_enabled.update(|s| *s = !*s)
                    >
                        "🔊 8-Bit 音声"
                    </button>
                </div>
            </div>

            <!-- Game CRT Terminal Display Screen -->
            <div class=move || {
                let theme_class = match theme.get() {
                    ColorTheme::ClassicNes => "theme-nes",
                    ColorTheme::PhosphorGreen => "theme-matrix",
                    ColorTheme::Amber80s => "theme-amber",
                    ColorTheme::CyberpunkNeon => "theme-cyberpunk",
                };
                let scanline_class = if scanlines_enabled.get() { "with-scanlines" } else { "" };
                format!("crt-screen {} {}", theme_class, scanline_class)
            }>
                <div class="crt-overlay"></div>
                <div class="crt-glow"></div>
                
                <!-- Interactive Playable Game View -->
                <div id="mario-game-container" class="mario-game-container">
                    <canvas id="mario-canvas" width="640" height="380" class="mario-canvas"></canvas>
                    <pre id="mario-ascii-terminal" class="mario-ascii-terminal"></pre>
                </div>
            </div>

            <!-- Virtual Arcade Controls & Keyboard Guide -->
            <div class="arcade-controls-panel">
                <div class="controls-guide">
                    <div class="guide-item"><span class="key-cap">"←"</span> <span class="key-cap">"→"</span> / <span class="key-cap">"A"</span> <span class="key-cap">"D"</span> : 移動 (Move)</div>
                    <div class="guide-item"><span class="key-cap">"Z"</span> / <span class="key-cap">"Space"</span> / <span class="key-cap">"↑"</span> : ジャンプ (Jump)</div>
                    <div class="guide-item"><span class="key-cap">"X"</span> / <span class="key-cap">"Shift"</span> : ダッシュ / ファイアボール (Run / Fire)</div>
                    <div class="guide-item"><span class="key-cap">"R"</span> : リトライ (Restart)</div>
                </div>

                <!-- VIP & Admin Cheats Bar -->
                <div class="cheats-bar">
                    <span class="cheat-label">"⚡ VIP/管理者チート:"</span>
                    <button
                        class=move || if god_mode.get() { "cheat-btn active" } else { "cheat-btn" }
                        on:click=move |_| {
                            if user_tier.get() >= SubscriptionTier::LegendaryVip {
                                set_god_mode.update(|g| *g = !*g);
                            } else {
                                open_sub_modal.set(true);
                            }
                        }
                    >
                        {move || if god_mode.get() { "👑 無敵ゴッドモード [ON]" } else { "👑 無敵ゴッドモード [OFF]" }}
                    </button>
                    <button
                        class=move || if moon_jump.get() { "cheat-btn active" } else { "cheat-btn" }
                        on:click=move |_| {
                            if user_tier.get() >= SubscriptionTier::ProGamer {
                                set_moon_jump.update(|m| *m = !*m);
                            } else {
                                open_sub_modal.set(true);
                            }
                        }
                    >
                        {move || if moon_jump.get() { "🌙 ムーンジャンプ [ON]" } else { "🌙 ムーンジャンプ [OFF]" }}
                    </button>
                    <button
                        class=move || if infinite_fireballs.get() { "cheat-btn active" } else { "cheat-btn" }
                        on:click=move |_| {
                            if user_tier.get() >= SubscriptionTier::LegendaryVip {
                                set_infinite_fireballs.update(|f| *f = !*f);
                            } else {
                                open_sub_modal.set(true);
                            }
                        }
                    >
                        {move || if infinite_fireballs.get() { "🔥 無限ファイア [ON]" } else { "🔥 無限ファイア [OFF]" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
