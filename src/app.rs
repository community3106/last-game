use leptos::*;
use leptos_router::*;

use crate::components::{
    about::AboutPage, admin::AdminPage, game_view::GamePage, hero::HeroSection,
    navbar::Navbar, pricing::PricingPage, footer::Footer,
};
use crate::nostr::client::NostrContext;

#[component]
pub fn App() -> impl IntoView {
    // Global Nostr context (provide_context)
    let nostr_ctx = NostrContext::new();
    provide_context(nostr_ctx);

    view! {
        <Router>
            <div class="min-h-screen flex flex-col bg-[#080a0f] text-zinc-100 selection:bg-violet-500/30">
                <Navbar />
                <main class="flex-1">
                    <Routes>
                        <Route path="" view=HomePage />
                        <Route path="/games" view=GamesPage />
                        <Route path="/play" view=GamePage />
                        <Route path="/about" view=AboutPage />
                        <Route path="/pricing" view=PricingPage />
                        <Route path="/admin" view=AdminPage />
                        <Route path="/*any" view=NotFound />
                    </Routes>
                </main>
                <Footer />
            </div>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <HeroSection />
        // --- Feature highlights ---
        <section class="max-w-6xl mx-auto px-6 py-12 grid md:grid-cols-3 gap-6">
            <FeatureCard
                icon="🎮"
                title="100% Rust ASCII Mario"
                desc="物理演算・衝突判定・レベル生成まで純Rust。WASMで60FPS動作。"
            />
            <FeatureCard
                icon="🔐"
                title="Nostr + NIP-44 暗号化"
                desc="有料レベルは公開鍵ごとに暗号化してリレー配信。無料ユーザーには復号不可。"
            />
            <FeatureCard
                icon="💳"
                title="Stripe サブスク"
                desc="決済後に npub が自動で有料会員リストに登録。Cloudflare Pagesで完結。"
            />
        </section>

        // --- Games teaser ---
        <section id="games" class="max-w-6xl mx-auto px-6 py-10">
            <h2 class="pixel-font text-lg md:text-xl mb-6 flex items-center gap-3">
                <span class="w-2 h-6 bg-red-500 inline-block"></span> GAMES
            </h2>
            <div class="grid md:grid-cols-2 gap-6">
                <GameTeaser
                    title="ASCII MARIO — STAGE 1-1"
                    badge="FREE"
                    badge_color="bg-emerald-500"
                    desc="だれでも遊べる最初のステージ。コインを集めてゴールを目指せ。"
                    href="/play"
                />
                <GameTeaser
                    title="ASCII MARIO — STAGE 1-2 (PREMIUM)"
                    badge="PREMIUM"
                    badge_color="bg-amber-500"
                    desc="NIP-44で暗号化された有料ステージ。星の洞窟と隠しブロック。"
                    href="/play?level=2"
                    locked=true
                />
            </div>
        </section>

        // --- About teaser ---
        <section class="max-w-6xl mx-auto px-6 py-8">
            <div class="rounded-2xl border border-zinc-800 bg-zinc-900/40 p-6 flex flex-col md:flex-row gap-6 items-center">
                <div class="flex-1">
                    <h3 class="font-bold text-lg">なぜ 100% Rust なのか？</h3>
                    <p class="text-zinc-400 mt-2 text-sm leading-relaxed">
                        フロントもバックも同じ言語で型を共有。Nostrの暗号化もゲーム物理も同じクレートで完結する。
                        Cloudflare Pages は静的配信、ロジックはWASMとリレーに分散。
                    </p>
                </div>
                <a href="/about" class="px-5 py-2.5 rounded-full bg-white text-black text-sm font-bold hover:bg-zinc-200 transition">About us →</a>
            </div>
        </section>
    }
}

#[component]
fn FeatureCard(icon: &'static str, title: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-2xl border border-zinc-800 bg-zinc-900/50 p-6 hover:border-zinc-700 transition">
            <div class="text-2xl">{icon}</div>
            <h3 class="mt-3 font-bold">{title}</h3>
            <p class="mt-2 text-sm text-zinc-400 leading-relaxed">{desc}</p>
        </div>
    }
}

#[component]
fn GameTeaser(
    title: &'static str,
    badge: &'static str,
    badge_color: &'static str,
    desc: &'static str,
    href: &'static str,
    #[prop(optional)] locked: bool,
) -> impl IntoView {
    view! {
        <a href=href class="group rounded-2xl border border-zinc-800 bg-zinc-900/60 p-6 hover:border-violet-600/40 hover:bg-zinc-900 transition block relative overflow-hidden">
            <div class="absolute inset-0 opacity-0 group-hover:opacity-100 transition bg-gradient-to-br from-violet-600/10 to-transparent pointer-events-none"></div>
            <div class="flex items-center gap-2 mb-3">
                <span class=format!("text-[10px] px-2 py-1 rounded-full font-bold text-black {}", badge_color)>{badge}</span>
                {if locked { view!{ <span class="text-xs text-zinc-500">🔒 NIP-44 暗号化</span> }.into_view() } else { view!{}.into_view() }}
            </div>
            <h3 class="pixel-font text-xs leading-5">{title}</h3>
            <p class="mt-3 text-sm text-zinc-400">{desc}</p>
            <span class="mt-4 inline-flex items-center gap-1 text-sm font-bold text-violet-400 group-hover:gap-2 transition-all">Play →</span>
        </a>
    }
}

#[component]
fn GamesPage() -> impl IntoView {
    view! {
        <section class="max-w-6xl mx-auto px-6 py-12">
            <h1 class="pixel-font text-xl mb-2">GAMES</h1>
            <p class="text-zinc-400 text-sm mb-8">Leptos CSRで動く100% Rustゲーム集。追加タイトルはNostrで配信されます。</p>
            <div class="grid md:grid-cols-3 gap-6">
                <GameTeaser title="ASCII MARIO 1-1" badge="FREE" badge_color="bg-emerald-500" desc="チュートリアル。操作を覚えよう。" href="/play" />
                <GameTeaser title="ASCII MARIO 1-2" badge="PREMIUM" badge_color="bg-amber-500" desc="暗号化ステージ。決済で解放。" href="/play?level=2" locked=true />
                <GameTeaser title="ASCII MARIO 1-3" badge="GOD MODE" badge_color="bg-red-500" desc="管理者のみテストプレイ可。近日公開。" href="/play?level=3" locked=true />
            </div>
        </section>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section class="max-w-6xl mx-auto px-6 py-24 text-center">
            <p class="pixel-font text-6xl opacity-20">404</p>
            <p class="mt-4 text-zinc-400">ページが見つかりません</p>
            <a href="/" class="mt-6 inline-block px-6 py-2.5 rounded-full bg-white text-black font-bold text-sm">ホームへ戻る</a>
        </section>
    }
}
