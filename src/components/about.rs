use leptos::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <section class="max-w-6xl mx-auto px-6 py-10">
            <span class="text-[11px] px-3 py-1 rounded-full border border-zinc-700 bg-zinc-900">About us</span>
            <h1 class="pixel-font text-xl md:text-2xl mt-4">私たちは、最後のゲームを作っています。</h1>
            <p class="mt-4 text-sm leading-relaxed text-zinc-400 max-w-2xl">
                last-game は「中央集権に頼らないゲーム配信」を目指す実験です。フロントは100% Rust (Leptos)、
                バックは Nostr リレー、課金は Stripe、配信は Cloudflare Pages。すべての境界を Rust の型でつなぎます。
            </p>

            <div class="mt-8 grid md:grid-cols-3 gap-6">
                <Member name="Satoshi" role="Rust / Leptos" desc="WASMで60FPSを刻むゲームエンジン担当。物理と衝突判定に生きる。" />
                <Member name="Nakadai" role="Nostr / Crypto" desc="NIP-04/44 の暗号化とホワイトリスト設計。リレーは友達。" />
                <Member name="Clara" role="Stripe / Infra" desc="Cloudflare Pages と GitHub Actions をつなぐ。CLOUDFLARE_API_TOKEN の守護者。" />
            </div>

            <div class="mt-8 rounded-2xl border border-zinc-800 bg-zinc-900/40 p-6 grid md:grid-cols-2 gap-6">
                <div>
                    <h3 class="font-bold">技術スタック</h3>
                    <ul class="mt-3 space-y-1.5 text-sm text-zinc-400">
                        <li>• <b class="text-white">Leptos 0.6 CSR</b> — 100% Rust フロント (WASM)</li>
                        <li>• <b class="text-white">nostr-sdk 0.35</b> — NIP-04 / NIP-44 暗号化</li>
                        <li>• <b class="text-white">Stripe Checkout</b> — サブスク・管理者特権と連動</li>
                        <li>• <b class="text-white">Cloudflare Pages + Wrangler</b></li>
                        <li>• <b class="text-white">GitHub Actions</b> — trunk build → pages-action</li>
                        <li>• <b class="text-white">Trunk</b> — index.html → dist</li>
                    </ul>
                </div>
                <div>
                    <h3 class="font-bold">思想</h3>
                    <p class="mt-3 text-sm leading-relaxed text-zinc-400">
                        ゲームの進行も、課金状態も、リレー上のイベントで表現する。サーバーは「有料会員リスト」という1つの真実を
                        Nostr に書き込むだけ。あとは各クライアントが自分の鍵で復号する。
                        これが <code class="px-1 bg-zinc-800 rounded">公開鍵ホワイトリスト + 暗号化</code> の本質です。
                    </p>
                    <div class="mt-4 flex gap-2">
                        <a href="/pricing" class="px-4 py-2 rounded-full bg-violet-600 text-white text-xs font-bold">有料プランを見る</a>
                        <a href="https://github.com/community3106/last-game" target="_blank" class="px-4 py-2 rounded-full border border-zinc-700 text-xs">GitHub →</a>
                    </div>
                </div>
            </div>

            <div class="mt-8 rounded-2xl border border-zinc-800 bg-black p-6">
                <h3 class="font-bold text-sm">Architecture</h3>
                <pre class="mt-3 text-[11px] leading-relaxed overflow-x-auto text-zinc-300">{r#"Browser (Leptos WASM) ──WSS──▶ Nostr Relays ──► Encrypted Levels (kind:30078, p=paid_pubkey)
        │                    ▲
        │ Stripe Checkout    │ nip44::decrypt
        └────► Cloudflare Pages Functions ──► verify → publish paid list (kind:30078)
                         │
                         └─► GitHub Actions (trunk build → cloudflare/pages-action)
                             Secrets: CLOUDFLARE_API_TOKEN, CLOUDFLARE_ACCOUNT_ID"#}</pre>
            </div>
        </section>
    }
}

#[component]
fn Member(name: &'static str, role: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-2xl border border-zinc-800 bg-zinc-900/40 p-6">
            <div class="w-10 h-10 rounded-full bg-gradient-to-br from-zinc-700 to-zinc-900 border border-zinc-700 grid place-items-center font-bold">{&name[0..1]}</div>
            <h3 class="mt-3 font-bold">{name} <span class="text-xs font-normal text-violet-400">{role}</span></h3>
            <p class="mt-2 text-sm text-zinc-400 leading-relaxed">{desc}</p>
        </div>
    }
}
