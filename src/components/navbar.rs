use leptos::*;
use leptos_router::*;
use crate::nostr::client::NostrContext;
use crate::nostr::whitelist::{is_admin_hex, is_paid_hex};

#[component]
pub fn Navbar() -> impl IntoView {
    let nostr = expect_context::<NostrContext>();
    let pubkey_hex = nostr.pubkey_hex;
    let npub = nostr.npub;
    let connected = nostr.connected;

    let is_admin = move || pubkey_hex.get().map(|h| is_admin_hex(&h)).unwrap_or(false);
    let is_paid = move || pubkey_hex.get().map(|h| is_paid_hex(&h)).unwrap_or(false);

    // 簡易ログイン: nsec / npub / hex をペースト
    let show_login = create_rw_signal(false);
    let input_key = create_rw_signal(String::new());
    let login_error = create_rw_signal::<Option<String>>(None);

    let on_login = move |_| {
        let key = input_key.get().trim().to_string();
        if key.is_empty() {
            login_error.set(Some("鍵を入力してください".into()));
            return;
        }
        match nostr.try_login(&key) {
            Ok(_) => {
                login_error.set(None);
                show_login.set(false);
            }
            Err(e) => login_error.set(Some(e)),
        }
    };

    view! {
        <header class="sticky top-0 z-40 backdrop-blur-xl bg-[#080a0f]/80 border-b border-zinc-800">
            <div class="max-w-6xl mx-auto px-4 md:px-6 h-[64px] flex items-center justify-between gap-4">
                // 左: ロゴ + ナビ
                <div class="flex items-center gap-6">
                    <a href="/" class="flex items-center gap-2.5">
                        <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-red-500 to-violet-600 grid place-items-center font-black text-white text-sm">LG</div>
                        <span class="pixel-font text-[11px] hidden sm:inline tracking-widest">LAST-GAME</span>
                        <span class="text-[10px] px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400 hidden md:inline">100% Rust</span>
                    </a>
                    <nav class="hidden md:flex items-center gap-1 text-sm">
                        <a href="/games" class="px-3 py-1.5 rounded-full hover:bg-zinc-800 transition">Games</a>
                        <a href="/pricing" class="px-3 py-1.5 rounded-full hover:bg-zinc-800 transition">Pricing</a>
                        <a href="/about" class="px-3 py-1.5 rounded-full hover:bg-zinc-800 transition">About us</a>
                        <a href="/play" class="px-3 py-1.5 rounded-full bg-white text-black font-bold hover:bg-zinc-200 transition">▶ Play</a>
                    </nav>
                </div>

                // 右: Nostr 状態 + CTA
                <div class="flex items-center gap-2">
                    // 接続ドット
                    <span class="hidden sm:inline-flex items-center gap-1.5 text-xs text-zinc-400">
                        <span class=move || if connected.get() { "w-2 h-2 rounded-full bg-emerald-500 animate-pulse" } else { "w-2 h-2 rounded-full bg-zinc-600" }></span>
                        {move || if connected.get() { "Nostr connected" } else { "offline" }}
                    </span>

                    // バッジ
                    {move || if is_admin() {
                        view!{ <span class="hidden sm:inline text-[10px] px-2 py-1 rounded-full bg-amber-500 text-black font-bold">👑 管理者</span> }.into_view()
                    } else if is_paid() {
                        view!{ <span class="hidden sm:inline text-[10px] px-2 py-1 rounded-full bg-violet-600 text-white font-bold">★ Premium</span> }.into_view()
                    } else {
                        view!{}.into_view()
                    }}

                    // npub 表示 or ログインボタン
                    {move || {
                        if let Some(n) = npub.get() {
                            let short = if n.len() > 16 { format!("{}…{}", &n[0..8], &n[n.len()-4..]) } else { n.clone() };
                            view!{
                                <div class="flex items-center gap-2">
                                    <span class="text-xs px-2.5 py-1.5 rounded-full bg-zinc-800 border border-zinc-700 font-mono hidden sm:inline">{short}</span>
                                    <button on:click=move |_| nostr.logout() class="text-xs px-3 py-1.5 rounded-full border border-zinc-700 hover:bg-zinc-800">Logout</button>
                                    <a href="/admin" class=move || if is_admin() { "text-xs px-3 py-1.5 rounded-full bg-amber-500 text-black font-bold" } else { "hidden" }>管理者</a>
                                </div>
                            }.into_view()
                        } else {
                            view!{
                                <button on:click=move |_| show_login.set(!show_login.get()) class="text-xs px-4 py-1.5 rounded-full bg-violet-600 hover:bg-violet-500 text-white font-bold">
                                    Nostrでログイン
                                </button>
                            }.into_view()
                        }
                    }}
                </div>
            </div>

            // モバイルナビ
            <div class="md:hidden border-t border-zinc-800 flex gap-1 px-2 py-2 overflow-x-auto text-xs">
                <a href="/games" class="px-3 py-1.5 rounded-full bg-zinc-900 border border-zinc-800 whitespace-nowrap">Games</a>
                <a href="/play" class="px-3 py-1.5 rounded-full bg-white text-black font-bold whitespace-nowrap">▶ Play</a>
                <a href="/pricing" class="px-3 py-1.5 rounded-full bg-zinc-900 border border-zinc-800 whitespace-nowrap">Pricing</a>
                <a href="/about" class="px-3 py-1.5 rounded-full bg-zinc-900 border border-zinc-800 whitespace-nowrap">About</a>
                <a href="/admin" class="px-3 py-1.5 rounded-full bg-zinc-900 border border-zinc-800 whitespace-nowrap">Admin</a>
            </div>

            // ログインモーダル
            <Show when=move || show_login.get() fallback=|| ()>
                <div class="absolute top-[64px] right-4 md:right-6 w-[min(420px,95vw)] rounded-2xl border border-zinc-700 bg-zinc-900 p-5 shadow-2xl">
                    <h3 class="font-bold text-sm">Nostrでログイン</h3>
                    <p class="text-xs text-zinc-400 mt-1">nsec / npub / hex のいずれかを貼り付け。鍵はローカルにのみ保存され、リレーには送信されません。</p>
                    <input
                        prop:value=move || input_key.get()
                        on:input=move |ev| input_key.set(event_target_value(&ev))
                        placeholder="nsec1..., npub1..., または hex"
                        class="mt-3 w-full px-3 py-2.5 rounded-xl bg-black border border-zinc-700 text-sm font-mono placeholder:text-zinc-600 focus:outline-none focus:border-violet-600"
                    />
                    {move || login_error.get().map(|e| view!{ <p class="mt-2 text-xs text-red-400">{e}</p> })}
                    <div class="mt-3 flex gap-2">
                        <button on:click=on_login class="flex-1 py-2.5 rounded-xl bg-violet-600 hover:bg-violet-500 text-white text-sm font-bold">ログイン</button>
                        <button on:click=move |_| show_login.set(false) class="px-4 py-2.5 rounded-xl border border-zinc-700 text-sm">閉じる</button>
                    </div>
                    <p class="mt-3 text-[11px] text-zinc-500">
                        拡張機能をお持ちなら <code class="px-1 py-0.5 bg-zinc-800 rounded">window.nostr (NIP-07)</code> も自動検出します。
                        管理者ホワイトリストはビルド時 <code>ADMIN_PUBKEYS</code> で指定。
                    </p>
                    <div class="mt-3 flex gap-2">
                        <button on:click=move |_| {
                            // デモ鍵でログイン
                            let _ = nostr.try_login("d95f3b8b3c3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3");
                            show_login.set(false);
                        } class="text-[11px] px-2.5 py-1 rounded-full border border-zinc-700 hover:bg-zinc-800">デモ鍵で試す</button>
                        <a href="/pricing" class="text-[11px] px-2.5 py-1 rounded-full bg-amber-500 text-black font-bold">有料プランを見る</a>
                    </div>
                </div>
            </Show>
        </header>
    }
}
