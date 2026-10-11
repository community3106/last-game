use leptos::*;
use leptos_meta::*;
use crate::components::*;
use crate::nostr::whitelist::SubscriptionTier;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let (current_tab, set_current_tab) = create_signal("games".to_string());
    let (user_tier, set_user_tier) = create_signal(SubscriptionTier::Free);
    let (user_npub, set_user_npub) = create_signal(None::<String>);
    let (is_sub_modal_open, set_is_sub_modal_open) = create_signal(false);
    let (is_login_modal_open, set_is_login_modal_open) = create_signal(false);

    view! {
        <Stylesheet id="leptos" href="/style.css"/>
        <Title text="100% Rust ASCII Mario • Nostr NIP-04/44 Crypto VIP"/>
        <Meta name="description" content="100% Rust WASM ASCII Mario Game with Leptos Frontend and Nostr NIP-04/44 Whitelist Encryption"/>

        <div class="retro-app-root">
            <!-- Top VIP Promo Banner -->
            <div class="vip-promo-banner" on:click=move |_| set_is_sub_modal_open.set(true)>
                <span class="banner-badge">"⭐ SPECIAL PROMO"</span>
                <span class="banner-text">
                    "【有料プラン・サブスク】Stripe決済で公開鍵（npub）をホワイトリスト登録！NIP-04/44暗号化World 1-4 クッパ城＆無敵ゴッドモードをアンロック！"
                </span>
                <button class="banner-cta">"今すぐ詳細を見る ▶"</button>
            </div>

            <!-- Navbar Header -->
            <Navbar
                current_tab=current_tab
                set_current_tab=set_current_tab
                user_tier=user_tier
                user_npub=user_npub
                open_login_modal=set_is_login_modal_open
                open_sub_modal=set_is_sub_modal_open
            />

            <!-- Main Tab Content -->
            <main class="main-content">
                {move || match current_tab.get().as_str() {
                    "games" => view! {
                        <div class="page-fade-in">
                            <GamesPage
                                user_tier=user_tier
                                open_sub_modal=set_is_sub_modal_open
                                set_current_tab=set_current_tab
                            />
                            <MarioGame
                                user_tier=user_tier
                                open_sub_modal=set_is_sub_modal_open
                            />
                        </div>
                    }.into_view(),

                    "play" => view! {
                        <div class="page-fade-in">
                            <MarioGame
                                user_tier=user_tier
                                open_sub_modal=set_is_sub_modal_open
                            />
                        </div>
                    }.into_view(),

                    "about" => view! {
                        <div class="page-fade-in">
                            <AboutPage />
                        </div>
                    }.into_view(),

                    "admin" => view! {
                        <div class="page-fade-in">
                            <AdminPage
                                user_tier=user_tier
                                set_user_tier=set_user_tier
                            />
                        </div>
                    }.into_view(),

                    "crypto" => view! {
                        <div class="page-fade-in">
                            <NipCryptoDemo />
                        </div>
                    }.into_view(),

                    "leaderboard" => view! {
                        <div class="page-fade-in">
                            <Leaderboard />
                        </div>
                    }.into_view(),

                    _ => view! {
                        <GamesPage
                            user_tier=user_tier
                            open_sub_modal=set_is_sub_modal_open
                            set_current_tab=set_current_tab
                        />
                    }.into_view(),
                }}
            </main>

            <!-- Modals -->
            <SubscriptionModal
                is_open=is_sub_modal_open
                set_is_open=set_is_sub_modal_open
                user_tier=user_tier
                set_user_tier=set_user_tier
                user_npub=user_npub
                set_user_npub=set_user_npub
            />

            <NostrLoginModal
                is_open=is_login_modal_open
                set_is_open=set_is_login_modal_open
                user_npub=user_npub
                set_user_npub=set_user_npub
                user_tier=user_tier
                set_user_tier=set_user_tier
            />

            <!-- Footer -->
            <Footer />
        </div>
    }
}
