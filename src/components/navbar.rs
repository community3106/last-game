use leptos::*;
use crate::nostr::whitelist::SubscriptionTier;

#[component]
pub fn Navbar(
    current_tab: ReadSignal<String>,
    set_current_tab: WriteSignal<String>,
    user_tier: ReadSignal<SubscriptionTier>,
    user_npub: ReadSignal<Option<String>>,
    open_login_modal: WriteSignal<bool>,
    open_sub_modal: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <header class="retro-header">
            <div class="header-container">
                <div class="brand" on:click=move |_| set_current_tab.set("games".to_string())>
                    <span class="mario-icon">"🍄"</span>
                    <div class="brand-text">
                        <span class="brand-title">"RUST NOSTR MARIO ARCADE"</span>
                        <span class="brand-subtitle">"100% Rust WASM • Nostr NIP-04/44 Crypto VIP"</span>
                    </div>
                </div>

                <nav class="retro-nav">
                    <button
                        class=move || if current_tab.get() == "games" { "nav-btn active" } else { "nav-btn" }
                        on:click=move |_| set_current_tab.set("games".to_string())
                    >
                        "🎮 ゲーム (Games)"
                    </button>
                    <button
                        class=move || if current_tab.get() == "subscription" { "nav-btn active sub-highlight" } else { "nav-btn sub-highlight" }
                        on:click=move |_| {
                            set_current_tab.set("subscription".to_string());
                            open_sub_modal.set(true);
                        }
                    >
                        "⭐ 有料プラン・サブスク"
                    </button>
                    <button
                        class=move || if current_tab.get() == "admin" { "nav-btn active admin-highlight" } else { "nav-btn admin-highlight" }
                        on:click=move |_| set_current_tab.set("admin".to_string())
                    >
                        "👑 管理者特権 (Admin)"
                    </button>
                    <button
                        class=move || if current_tab.get() == "crypto" { "nav-btn active" } else { "nav-btn" }
                        on:click=move |_| set_current_tab.set("crypto".to_string())
                    >
                        "🔐 NIP-04/44 暗号化実験室"
                    </button>
                    <button
                        class=move || if current_tab.get() == "leaderboard" { "nav-btn active" } else { "nav-btn" }
                        on:click=move |_| set_current_tab.set("leaderboard".to_string())
                    >
                        "🏆 Nostr ランキング"
                    </button>
                    <button
                        class=move || if current_tab.get() == "about" { "nav-btn active" } else { "nav-btn" }
                        on:click=move |_| set_current_tab.set("about".to_string())
                    >
                        "📖 私たちについて (About Us)"
                    </button>
                </nav>

                <div class="user-status-area">
                    <div
                        class="tier-badge"
                        style=move || format!("background: {};", user_tier.get().badge_color())
                    >
                        {move || user_tier.get().display_name()}
                    </div>

                    {move || {
                        if let Some(npub) = user_npub.get() {
                            let short_npub = if npub.len() > 14 {
                                format!("{}...{}", &npub[..8], &npub[npub.len() - 4..])
                            } else {
                                npub
                            };
                            view! {
                                <button class="user-btn logged-in" on:click=move |_| open_login_modal.set(true)>
                                    "🔑 " {short_npub}
                                </button>
                            }.into_view()
                        } else {
                            view! {
                                <button class="user-btn login" on:click=move |_| open_login_modal.set(true)>
                                    "⚡ Nostr ログイン / npub"
                                </button>
                            }.into_view()
                        }
                    }}
                </div>
            </div>
        </header>
    }
}
