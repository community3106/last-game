use leptos::*;
use crate::nostr::whitelist::SubscriptionTier;

#[component]
pub fn GamesPage(
    user_tier: ReadSignal<SubscriptionTier>,
    open_sub_modal: WriteSignal<bool>,
    set_current_tab: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div class="games-page-container">
            <div class="page-header">
                <h1 class="page-title">"🎮 Nostr ASCII ゲームライブラリ"</h1>
                <p class="page-description">"100% Rust WASMで動作する完全なASCIIゲームコレクション。Nostrの公開鍵（npub）と暗号化（NIP-04/44）による分散型アクセス制御。"</p>
            </div>

            <!-- Games Grid -->
            <div class="games-grid">
                <!-- Game 1: World 1-1 & 1-2 Classic -->
                <div class="game-card free-tier">
                    <div class="game-badge free">"🆓 完全無料 (FREE)"</div>
                    <div class="game-ascii-preview">
                        <pre>
"  [?]  [#][?][#]   |__|\n"
"                   |  |\n"
" m>   [o_o]        |  |\n"
"███████████████████████"
                        </pre>
                    </div>
                    <h3 class="game-title">"ASCII Mario 1985 : Classic Overworld"</h3>
                    <p class="game-info">"World 1-1（地上王国）＆ World 1-2（地下洞窟）。誰でもログイン不要でプレイ可能！"</p>
                    <div class="game-actions">
                        <button class="btn-play primary" on:click=move |_| set_current_tab.set("play".to_string())>
                            "▶ 今すぐプレイ (Play Free)"
                        </button>
                    </div>
                </div>

                <!-- Game 2: World 1-3 Mushroom Treetops -->
                <div class="game-card pro-tier">
                    <div class="game-badge pro">"💎 PRO NOSTR GAMER ($4.99/月)"</div>
                    <div class="game-ascii-preview">
                        <pre>
"    [###]       [###]   \n"
"           m>           \n"
"   <(OO)>               \n"
" [#######]    [#######] "
                        </pre>
                    </div>
                    <h3 class="game-title">"ASCII Mario : Mushroom Treetops"</h3>
                    <p class="game-info">"World 1-3 アスレチック高所ステージ。Pro会員専用。落下注意のスリリングなキノコ足場！"</p>
                    <div class="game-actions">
                        {move || if user_tier.get() >= SubscriptionTier::ProGamer {
                            view! {
                                <button class="btn-play pro" on:click=move |_| set_current_tab.set("play".to_string())>
                                    "▶ アンロック済みでプレイ"
                                </button>
                            }.into_view()
                        } else {
                            view! {
                                <button class="btn-play upgrade" on:click=move |_| open_sub_modal.set(true)>
                                    "🔒 Proプランでアンロック"
                                </button>
                            }.into_view()
                        }}
                    </div>
                </div>

                <!-- Game 3: World 1-4 Bowser Castle -->
                <div class="game-card vip-tier">
                    <div class="game-badge vip">"⭐ LEGENDARY VIP ($9.99/月・NIP-04/44暗号化)"</div>
                    <div class="game-ascii-preview">
                        <pre>
"▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓\n"
" [F*]>   [[W_W]]   T ∩ \n"
" ###################   \n"
"~~~~~~~~~~~~~~~~~~~~~~~"
                        </pre>
                    </div>
                    <h3 class="game-title">"ASCII Mario : Bowser Castle Boss"</h3>
                    <p class="game-info">"World 1-4 溶岩とクッパの城。NIP-04/NIP-44で暗号化され、ホワイトリスト登録された鍵のみ解読可能！"</p>
                    <div class="game-actions">
                        {move || if user_tier.get() >= SubscriptionTier::LegendaryVip {
                            view! {
                                <button class="btn-play vip" on:click=move |_| set_current_tab.set("play".to_string())>
                                    "▶ VIP暗号解読済みでプレイ"
                                </button>
                            }.into_view()
                        } else {
                            view! {
                                <button class="btn-play upgrade-vip" on:click=move |_| open_sub_modal.set(true)>
                                    "⭐ VIPプランで暗号解読"
                                </button>
                            }.into_view()
                        }}
                    </div>
                </div>

                <!-- Game 4: Speedrun Challenge -->
                <div class="game-card speedrun">
                    <div class="game-badge gold">"⏱️ Nostr スピードラン (Kind 30078)"</div>
                    <div class="game-ascii-preview">
                        <pre>
"⚡ TIME ATTACK: 1-1\n"
"RECORD: 00:21.84 by npub1...\n"
"SIG: 5a5a... (Signed Schnorr)\n"
"NOSTR RELAY VERIFIED"
                        </pre>
                    </div>
                    <h3 class="game-title">"Nostr タイムアタック＆ランキング"</h3>
                    <p class="game-info">"クリアタイムとスコアをSchnorr署名付きNostrイベント（Kind 30078）としてリレーに送信！"</p>
                    <div class="game-actions">
                        <button class="btn-play secondary" on:click=move |_| set_current_tab.set("leaderboard".to_string())>
                            "🏆 ランキングを見る"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
