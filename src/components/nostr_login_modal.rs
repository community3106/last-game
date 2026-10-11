use leptos::*;
use crate::nostr::whitelist::{SubscriptionTier, WhitelistManager};

#[component]
pub fn NostrLoginModal(
    is_open: ReadSignal<bool>,
    set_is_open: WriteSignal<bool>,
    user_npub: ReadSignal<Option<String>>,
    set_user_npub: WriteSignal<Option<String>>,
    user_tier: ReadSignal<SubscriptionTier>,
    set_user_tier: WriteSignal<SubscriptionTier>,
) -> impl IntoView {
    let (input_key, set_input_key) = create_signal(String::new());
    let (login_msg, set_login_msg) = create_signal(String::new());

    let handle_login = move |_| {
        let key = input_key.get().trim().to_string();
        if key.is_empty() {
            set_login_msg.set("公開鍵（npub）を入力してください。".to_string());
            return;
        }

        let wm = WhitelistManager::default();
        let tier = wm.get_user_tier(&key);

        set_user_npub.set(Some(key.clone()));
        set_user_tier.set(tier);
        set_login_msg.set(format!("ログイン完了！ 検出プラン: {}", tier.display_name()));
        set_is_open.set(false);
    };

    let handle_generate_ephemeral = move |_| {
        let ephemeral_npub = "npub1retroplayer2026rustmarioarcadenewcomer888fastpass".to_string();
        set_user_npub.set(Some(ephemeral_npub));
        set_user_tier.set(SubscriptionTier::Free);
        set_is_open.set(false);
    };

    let handle_logout = move |_| {
        set_user_npub.set(None);
        set_user_tier.set(SubscriptionTier::Free);
        set_is_open.set(false);
    };

    view! {
        {move || if is_open.get() {
            view! {
                <div class="modal-backdrop" on:click=move |_| set_is_open.set(false)>
                    <div class="modal-card" on:click=move |ev| ev.stop_propagation()>
                        <div class="modal-header">
                            <span class="modal-badge">"⚡ NOSTR IDENTITY"</span>
                            <h2 class="modal-title">"Nostr ログイン / アカウント設定"</h2>
                            <button class="modal-close" on:click=move |_| set_is_open.set(false)>"✕"</button>
                        </div>

                        <div class="modal-body">
                            <p class="modal-desc">
                                "Nostrの公開鍵（npub）を入力すると、ホワイトリストの購入履歴と連携し、暗号化ステージが自動でアンロックされます。"
                            </p>

                            <div class="form-group">
                                <label>"Nostr 公開鍵 (npub / hex):"</label>
                                <input
                                    type="text"
                                    class="retro-input"
                                    placeholder="npub1..."
                                    prop:value=input_key
                                    on:input=move |ev| set_input_key.set(event_target_value(&ev))
                                />
                            </div>

                            <div class="login-actions">
                                <button class="btn-primary" on:click=handle_login>
                                    "npub でログイン"
                                </button>
                                <button class="btn-secondary" on:click=handle_generate_ephemeral>
                                    "⚡ ゲスト用一時キー生成"
                                </button>
                            </div>

                            {move || if user_npub.get().is_some() {
                                view! {
                                    <div class="logout-box">
                                        <button class="btn-danger" on:click=handle_logout>
                                            "ログアウト (ゲストに戻る)"
                                        </button>
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }}

                            {move || if !login_msg.get().is_empty() {
                                view! { <div class="status-msg">{login_msg.get()}</div> }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }}
                        </div>
                    </div>
                </div>
            }.into_view()
        } else {
            view! { <div></div> }.into_view()
        }}
    }
}
