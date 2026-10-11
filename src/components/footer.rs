use leptos::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-zinc-800 bg-black/40">
            <div class="max-w-6xl mx-auto px-6 py-10 grid md:grid-cols-4 gap-8 text-sm">
                <div>
                    <div class="flex items-center gap-2">
                        <div class="w-7 h-7 rounded-lg bg-gradient-to-br from-red-500 to-violet-600 grid place-items-center font-black text-white text-xs">LG</div>
                        <span class="pixel-font text-[10px]">LAST-GAME</span>
                    </div>
                    <p class="mt-3 text-xs text-zinc-500 leading-relaxed">
                        100% Rustで書かれたASCIIマリオ。<br/>
                        Leptos × Nostr × Stripe × Cloudflare Pages.
                    </p>
                </div>
                <div>
                    <h4 class="font-bold text-xs tracking-widest text-zinc-300">NAVIGATION</h4>
                    <ul class="mt-3 space-y-1.5 text-zinc-400 text-xs">
                        <li><a href="/games" class="hover:text-white">Games</a></li>
                        <li><a href="/play" class="hover:text-white">Play ASCII Mario</a></li>
                        <li><a href="/pricing" class="hover:text-white">Pricing / 有料プラン</a></li>
                        <li><a href="/about" class="hover:text-white">About us</a></li>
                    </ul>
                </div>
                <div>
                    <h4 class="font-bold text-xs tracking-widest text-zinc-300">NOSTR</h4>
                    <ul class="mt-3 space-y-1.5 text-zinc-400 text-xs font-mono">
                        <li>relays: relay.damus.io</li>
                        <li>nos.lol / relay.nostr.band</li>
                        <li>kind: 30078 (paid list)</li>
                        <li>NIP-04 / NIP-44</li>
                    </ul>
                </div>
                <div>
                    <h4 class="font-bold text-xs tracking-widest text-zinc-300">DEPLOY</h4>
                    <ul class="mt-3 space-y-1.5 text-zinc-400 text-xs">
                        <li>Cloudflare Pages</li>
                        <li>GitHub Actions</li>
                        <li class="font-mono text-[11px]">CLOUDFLARE_API_TOKEN</li>
                        <li class="font-mono text-[11px]">CLOUDFLARE_ACCOUNT_ID</li>
                    </ul>
                </div>
            </div>
            <div class="max-w-6xl mx-auto px-6 py-4 border-t border-zinc-800 flex flex-col md:flex-row justify-between gap-2 text-[11px] text-zinc-500">
                <span>© 2026 last-game. Built with Rust & Leptos. No JS framework.</span>
                <span class="font-mono">commit: arena • branch: arena/4b9050fc-last-game</span>
            </div>
        </footer>
    }
}
