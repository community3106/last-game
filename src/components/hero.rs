use leptos::*;

#[component]
pub fn HeroSection() -> impl IntoView {
    view! {
        <section class="relative overflow-hidden">
            // 背景グリッド + グラデ
            <div class="absolute inset-0 bg-gradient-to-b from-violet-900/20 via-transparent to-transparent pointer-events-none"></div>
            <div class="absolute inset-0 grid-paper opacity-40 pointer-events-none"></div>

            <div class="relative max-w-6xl mx-auto px-6 pt-10 md:pt-16 pb-10">
                <div class="flex flex-col lg:flex-row gap-8 items-center">
                    // 左: コピー
                    <div class="flex-1">
                        <div class="inline-flex items-center gap-2 text-[11px] px-3 py-1 rounded-full border border-violet-500/30 bg-violet-500/10 text-violet-300">
                            <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
                            100% Rust • Leptos CSR • nostr-sdk • Cloudflare Pages
                        </div>
                        <h1 class="pixel-font text-[18px] md:text-[30px] leading-[1.35] mt-4">
                            <span class="text-white">ASCIIで駆ける、</span><br/>
                            <span class="text-transparent bg-clip-text bg-gradient-to-r from-red-500 via-amber-400 to-violet-500">最後のゲーム。</span>
                        </h1>
                        <p class="mt-4 text-sm md:text-[15px] leading-relaxed text-zinc-300 max-w-xl">
                            ターミナルに広がるドット絵の王国。ジャンプ、コイン、クリボー。
                            フロントは <b>Leptos</b>、バックは <b>Nostr</b>。有料ステージは <b>NIP-44</b> で暗号化され、
                            あなたの公開鍵だけが扉を開く。
                        </p>

                        <div class="mt-6 flex flex-wrap gap-3">
                            <a href="/play" class="px-6 py-3 rounded-full bg-white text-black font-black text-sm hover:bg-zinc-100 transition glow flex items-center gap-2">
                                ▶ 今すぐ遊ぶ <span class="text-xs font-normal text-zinc-600">無料 1-1</span>
                            </a>
                            <a href="/pricing" class="px-6 py-3 rounded-full border border-zinc-700 bg-zinc-900 hover:bg-zinc-800 text-sm font-bold flex items-center gap-2">
                                有料プラン・サブスクを見る <span class="text-amber-400">★</span>
                            </a>
                        </div>

                        <div class="mt-6 flex flex-wrap gap-2 text-[11px]">
                            <span class="px-2.5 py-1 rounded-full bg-zinc-900 border border-zinc-800">管理者特権: 公開鍵ホワイトリスト</span>
                            <span class="px-2.5 py-1 rounded-full bg-zinc-900 border border-zinc-800">NIP-04 / NIP-44 対応</span>
                            <span class="px-2.5 py-1 rounded-full bg-zinc-900 border border-zinc-800">Stripe 連携</span>
                        </div>

                        // 広告バナー (有料プラン)
                        <div class="mt-6 rounded-2xl border border-amber-500/30 bg-gradient-to-r from-amber-500/15 to-orange-600/15 p-4 flex gap-4 items-center">
                            <div class="w-10 h-10 rounded-xl bg-amber-500 grid place-items-center text-black font-black">★</div>
                            <div class="flex-1">
                                <p class="text-sm font-bold">有料会員で全ステージ解放 + 限定スキン</p>
                                <p class="text-xs text-zinc-400">月額 ¥980〜。Stripe決済後にあなたのnpubが自動登録。暗号化レベルを即時復号。</p>
                            </div>
                            <a href="/pricing" class="hidden sm:inline-flex px-4 py-2 rounded-full bg-amber-500 text-black text-xs font-black hover:bg-amber-400">詳細</a>
                        </div>
                    </div>

                    // 右: ASCII プレビュー
                    <div class="flex-1 w-full max-w-[520px]">
                        <div class="rounded-[20px] border border-zinc-800 bg-zinc-900/70 backdrop-blur p-3 shadow-2xl scanline">
                            <div class="flex items-center justify-between px-1 pb-2">
                                <div class="flex gap-1.5">
                                    <span class="w-3 h-3 rounded-full bg-red-500"></span>
                                    <span class="w-3 h-3 rounded-full bg-amber-400"></span>
                                    <span class="w-3 h-3 rounded-full bg-emerald-500"></span>
                                </div>
                                <span class="text-[10px] px-2 py-1 rounded-full bg-black border border-zinc-700 font-mono">last-game — wasm32</span>
                                <span class="text-[10px] text-zinc-500 hidden sm:inline">60 FPS • Rust</span>
                            </div>
                            // 疑似ASCII画面
                            <pre class="game-pre bg-[#0b0e14] rounded-xl p-3 md:p-4 overflow-x-auto border border-zinc-800 leading-none whitespace-pre text-[11px] md:text-[12px]">{r#"┌──────────────────────────────────────┐
│ SCORE 00420  COINS ○x12  1-1  ♥♥♥    │
│                                      │
│        ○     ○           ■■■          │
│      ■■■■  ■■■■    G    ■   ■  ○    │
│                         ───          │
│   M         ■■■         G    ■■■     │
│  ──────────────────────────────────  │
│  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  │
└──────────────────────────────────────┘
  [←→] 移動  [Space] ジャンプ  [R] リセット"#}</pre>
                            <div class="mt-3 grid grid-cols-3 gap-2 text-[11px]">
                                <div class="rounded-xl bg-black border border-zinc-800 p-2.5 text-center">
                                    <div class="text-zinc-500">Players</div><div class="font-bold">1,284</div>
                                </div>
                                <div class="rounded-xl bg-black border border-zinc-800 p-2.5 text-center">
                                    <div class="text-zinc-500">Relays</div><div class="font-bold">3 connected</div>
                                </div>
                                <div class="rounded-xl bg-black border border-zinc-800 p-2.5 text-center">
                                    <div class="text-zinc-500">Encrypted</div><div class="font-bold">NIP-44 ✓</div>
                                </div>
                            </div>
                        </div>
                        <p class="mt-2 text-center text-[11px] text-zinc-500">↑ 実際のゲームは <a href="/play" class="underline decoration-violet-500">/play</a> でWASMが描画します</p>
                    </div>
                </div>
            </div>
        </section>
    }
}
