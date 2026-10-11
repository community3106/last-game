use leptos::*;
use crate::game::engine::{GameState, InputState, LEVEL_1, LEVEL_2_ENCRYPTED_HINT};
use crate::game::renderer::render_ascii;
use crate::nostr::client::NostrContext;
use crate::nostr::whitelist::is_paid_hex;

#[component]
pub fn GamePage() -> impl IntoView {
    let nostr = expect_context::<NostrContext>();
    let pubkey_hex = nostr.pubkey_hex;

    // URL ?level=2 を読む (簡易)
    let level_param = move || {
        web_sys::window()
            .and_then(|w| w.location().search().ok())
            .unwrap_or_default()
    };
    let is_level2 = move || level_param().contains("level=2") || level_param().contains("level=3");
    let has_access = move || {
        if !is_level2() { true } else {
            pubkey_hex.get().map(|h| is_paid_hex(&h)).unwrap_or(false)
        }
    };

    // ゲーム状態
    let game = create_rw_signal(GameState::new(LEVEL_1));
    let input = create_rw_signal(InputState::default());
    let tick = create_rw_signal(0u64);
    let paused = create_rw_signal(false);

    // キーボード
    let handle_key = {
        let input = input;
        let game = game;
        let paused = paused;
        move |ev: web_sys::KeyboardEvent| {
            let key = ev.key();
            // prevent scroll on space/arrows
            if matches!(key.as_str(), " " | "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight") {
                ev.prevent_default();
            }
            match key.as_str() {
                "ArrowLeft" | "a" | "A" => {
                    if ev.type_() == "keydown" { input.update(|i| i.left = true); }
                    else { input.update(|i| i.left = false); }
                }
                "ArrowRight" | "d" | "D" => {
                    if ev.type_() == "keydown" { input.update(|i| i.right = true); }
                    else { input.update(|i| i.right = false); }
                }
                " " | "ArrowUp" | "w" | "W" => {
                    if ev.type_() == "keydown" { input.update(|i| i.jump = true); }
                    else { input.update(|i| i.jump = false); }
                }
                "r" | "R" if ev.type_() == "keydown" => {
                    game.update(|g| g.reset(LEVEL_1));
                }
                "p" | "P" if ev.type_() == "keydown" => {
                    paused.update(|p| *p = !*p);
                }
                _ => {}
            }
        }
    };

    // 60FPS ループ
    {
        let game = game;
        let input = input;
        let tick = tick;
        let paused = paused;
        // set_interval 16ms
        gloo_timers::callback::Interval::new(16, move || {
            if paused.get() { return; }
            if !has_access() { return; }
            let inp = input.get();
            game.update(|g| {
                g.tick(inp);
            });
            tick.update(|t| *t += 1);
        }).forget();
    }

    // レンダリング文字列
    let frame = move || {
        let g = game.get();
        // tick を読んで再計算をトリガ
        let _ = tick.get();
        render_ascii(&g)
    };

    view! {
        // キーボードリスナを window に付与
        <div
            tabindex="0"
            on:keydown=handle_key.clone()
            on:keyup=handle_key
            class="outline-none"
            _ref={
                let f = |el: Option<web_sys::HtmlElement>| {
                    if let Some(e) = el { let _ = e.focus(); }
                };
                f
            }
        >
            <section class="max-w-6xl mx-auto px-4 md:px-6 py-6">
                <div class="flex flex-wrap items-center justify-between gap-3">
                    <h1 class="pixel-font text-sm md:text-base flex items-center gap-3">
                        <span class="w-2 h-6 bg-red-500 inline-block"></span> ASCII MARIO
                        {move || if is_level2() { view!{ <span class="text-[10px] px-2 py-1 rounded-full bg-amber-500 text-black font-bold">PREMIUM 1-2</span> }.into_view() } else { view!{ <span class="text-[10px] px-2 py-1 rounded-full bg-emerald-500 text-black font-bold">FREE 1-1</span> }.into_view() }}
                    </h1>
                    <div class="flex gap-2 text-xs">
                        <a href="/play" class=move || if !is_level2() { "px-3 py-1.5 rounded-full bg-white text-black font-bold" } else { "px-3 py-1.5 rounded-full border border-zinc-700" }>"1-1 無料"</a>
                        <a href="/play?level=2" class=move || if is_level2() { "px-3 py-1.5 rounded-full bg-amber-500 text-black font-bold" } else { "px-3 py-1.5 rounded-full border border-zinc-700" }>"1-2 有料"</a>
                    </div>
                </div>

                // 有料ゲート
                {move || if !has_access() {
                    view!{
                        <div class="mt-6 rounded-2xl border border-amber-500/30 bg-gradient-to-br from-amber-500/15 to-orange-600/15 p-8 text-center">
                            <div class="w-12 h-12 mx-auto rounded-2xl bg-amber-500 grid place-items-center text-xl">🔒</div>
                            <h2 class="mt-4 font-black">このステージは NIP-44 で暗号化されています</h2>
                            <p class="mt-2 text-sm text-zinc-300 max-w-xl mx-auto leading-relaxed">
                                有料会員 (Premium / God Mode) の公開鍵だけが復号できます。<br/>
                                まずは <a href="/pricing" class="underline decoration-amber-500">有料プラン</a> で Stripe 決済し、あなたの <code class="px-1 bg-black rounded">npub</code> を有料会員リストに登録してください。<br/>
                                <span class="text-xs text-zinc-400">ヒント: 開発モックでは「有料プランを見る → モックで有料化を試す」で即時解放できます。</span>
                            </p>
                            <div class="mt-4 flex justify-center gap-3">
                                <a href="/pricing" class="px-6 py-2.5 rounded-full bg-amber-500 text-black text-sm font-black">有料プランを見る</a>
                                <a href="/play" class="px-6 py-2.5 rounded-full border border-zinc-700 text-sm">無料 1-1 を遊ぶ</a>
                            </div>
                            <pre class="mt-6 text-left text-[11px] bg-black rounded-xl p-4 border border-zinc-800 overflow-x-auto whitespace-pre-wrap">{LEVEL_2_ENCRYPTED_HINT}</pre>
                        </div>
                    }.into_view()
                } else {
                    view!{}.into_view()
                }}

                // ゲーム画面
                <div class=move || if has_access() { "" } else { "hidden" }>
                    <div class="mt-6 grid lg:grid-cols-[1fr_300px] gap-6">
                        // 左: ゲーム
                        <div class="rounded-[20px] border border-zinc-800 bg-zinc-900/60 p-3 md:p-4 scanline">
                            <div class="flex items-center justify-between text-xs mb-2">
                                <div class="flex gap-1.5">
                                    <span class="w-3 h-3 rounded-full bg-red-500"></span>
                                    <span class="w-3 h-3 rounded-full bg-amber-400"></span>
                                    <span class="w-3 h-3 rounded-full bg-emerald-500"></span>
                                </div>
                                <span class="font-mono text-zinc-500">{move || format!("tick {} • 60 FPS • Rust", tick.get())}</span>
                                <button on:click=move |_| paused.update(|p| *p = !*p) class="px-3 py-1 rounded-full border border-zinc-700 hover:bg-zinc-800">
                                    {move || if paused.get() { "▶ Resume" } else { "⏸ Pause (P)" }}
                                </button>
                            </div>

                            // ASCII canvas
                            <pre class="game-pre bg-[#0b0e14] rounded-xl p-2 md:p-3 border border-zinc-800 overflow-x-auto whitespace-pre select-none"
                                 style="font-family:'JetBrains Mono',ui-monospace,monospace"
                            >{frame}</pre>

                            // コントローラ (モバイル)
                            <div class="mt-3 grid grid-cols-3 gap-2 md:hidden">
                                <button
                                    on:touchstart=move |_| input.update(|i| i.left = true)
                                    on:touchend=move |_| input.update(|i| i.left = false)
                                    on:mousedown=move |_| input.update(|i| i.left = true)
                                    on:mouseup=move |_| input.update(|i| i.left = false)
                                    class="py-3 rounded-xl bg-zinc-800 border border-zinc-700 active:bg-zinc-700 select-none"
                                >◀</button>
                                <button
                                    on:touchstart=move |_| input.update(|i| i.jump = true)
                                    on:touchend=move |_| input.update(|i| i.jump = false)
                                    on:mousedown=move |_| input.update(|i| i.jump = true)
                                    on:mouseup=move |_| input.update(|i| i.jump = false)
                                    class="py-3 rounded-xl bg-violet-600 border border-violet-500 active:bg-violet-500 font-bold select-none"
                                >JUMP</button>
                                <button
                                    on:touchstart=move |_| input.update(|i| i.right = true)
                                    on:touchend=move |_| input.update(|i| i.right = false)
                                    on:mousedown=move |_| input.update(|i| i.right = true)
                                    on:mouseup=move |_| input.update(|i| i.right = false)
                                    class="py-3 rounded-xl bg-zinc-800 border border-zinc-700 active:bg-zinc-700 select-none"
                                >▶</button>
                            </div>

                            <div class="mt-3 flex flex-wrap gap-2 text-[11px] text-zinc-500">
                                <span class="px-2 py-1 rounded-full bg-black border border-zinc-800">[←→] 移動</span>
                                <span class="px-2 py-1 rounded-full bg-black border border-zinc-800">[Space] ジャンプ</span>
                                <span class="px-2 py-1 rounded-full bg-black border border-zinc-800">[R] リセット</span>
                                <span class="px-2 py-1 rounded-full bg-black border border-zinc-800">[P] ポーズ</span>
                            </div>
                        </div>

                        // 右: ステータス + 説明
                        <div class="space-y-4">
                            <div class="rounded-2xl border border-zinc-800 bg-zinc-900/40 p-5">
                                <h3 class="font-bold text-sm">ステータス</h3>
                                <div class="mt-3 grid grid-cols-2 gap-2 text-xs">
                                    <div class="rounded-xl bg-black border border-zinc-800 p-3">
                                        <div class="text-zinc-500">SCORE</div>
                                        <div class="font-mono font-bold text-lg">{move || game.get().score}</div>
                                    </div>
                                    <div class="rounded-xl bg-black border border-zinc-800 p-3">
                                        <div class="text-zinc-500">COINS</div>
                                        <div class="font-mono font-bold text-lg">{move || format!("○ x{}", game.get().coins)}</div>
                                    </div>
                                    <div class="rounded-xl bg-black border border-zinc-800 p-3">
                                        <div class="text-zinc-500">LIVES</div>
                                        <div class="font-bold">{move || "♥".repeat(game.get().lives.max(0) as usize) + &"♡".repeat((3-game.get().lives).max(0) as usize)}</div>
                                    </div>
                                    <div class="rounded-xl bg-black border border-zinc-800 p-3">
                                        <div class="text-zinc-500">POS</div>
                                        <div class="font-mono text-xs">{move || { let g=game.get(); format!("({:.1},{:.1})", g.player_x, g.player_y) }}</div>
                                    </div>
                                </div>
                                {move || {
                                    let g = game.get();
                                    if g.won {
                                        view!{ <div class="mt-3 rounded-xl bg-emerald-500 text-black p-3 text-center font-black">🎉 GOAL!  Rで再挑戦</div> }.into_view()
                                    } else if g.game_over {
                                        view!{ <div class="mt-3 rounded-xl bg-red-500 text-white p-3 text-center font-black">GAME OVER  Rでリスタート</div> }.into_view()
                                    } else { view!{}.into_view() }
                                }}
                                <button on:click=move |_| game.update(|g| g.reset(LEVEL_1)) class="mt-3 w-full py-2.5 rounded-full bg-zinc-800 border border-zinc-700 text-xs font-bold hover:bg-zinc-700">↻ リセット (R)</button>
                            </div>

                            <div class="rounded-2xl border border-violet-500/20 bg-violet-500/10 p-5">
                                <h3 class="font-bold text-sm">100% Rustで動いています</h3>
                                <p class="mt-2 text-xs leading-relaxed text-zinc-300">
                                    このゲームの物理・衝突判定・レンダリングはすべて <code class="px-1 bg-black rounded">src/game/engine.rs</code> の純Rustです。
                                    Leptosがそれを60FPSで <code>&lt;pre&gt;</code> に描画し、入力はWASMが受け取ります。
                                </p>
                                <pre class="mt-3 text-[11px] bg-black rounded-xl p-3 border border-zinc-800 overflow-x-auto">{r#"// engine.rs (抜粋)
pub struct GameState {
  pub player_x: f32, pub player_y: f32,
  pub vel_x: f32, pub vel_y: f32,
  pub on_ground: bool,
}
impl GameState {
  pub fn tick(&mut self, input: InputState) {
    self.vel_y += GRAVITY; // 0.55
    // 衝突判定, コイン, 敵, ゴール...
  }
}"#}</pre>
                            </div>

                            <div class="rounded-2xl border border-zinc-800 bg-zinc-900/40 p-5">
                                <h3 class="font-bold text-sm">有料ステージの仕組み</h3>
                                <p class="mt-2 text-xs leading-relaxed text-zinc-400">
                                    1-2は本来、Nostrリレーから <code>kind:30078</code> の暗号化イベントを取得して復号してからプレイします。
                                    このデモでは <code>is_paid</code> が true ならローカルのレベル定義を使っていますが、
                                    本番は <code>nostr-sdk::nip44::decrypt</code> で得たJSONを <code>Level::from_json</code> します。
                                </p>
                                <a href="/pricing" class="mt-3 inline-flex px-4 py-2 rounded-full bg-amber-500 text-black text-xs font-black">有料プランを見る →</a>
                            </div>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    }
}
