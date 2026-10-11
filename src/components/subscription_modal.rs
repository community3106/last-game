use leptos::*;
use crate::nostr::whitelist::SubscriptionTier;

#[component]
pub fn SubscriptionModal(
    is_open: ReadSignal<bool>,
    set_is_open: WriteSignal<bool>,
    user_tier: ReadSignal<SubscriptionTier>,
    set_user_tier: WriteSignal<SubscriptionTier>,
    user_npub: ReadSignal<Option<String>>,
    set_user_npub: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (selected_plan, set_selected_plan) = create_signal("vip".to_string());
    let (input_npub, set_input_npub) = create_signal(String::new());
    let (payment_step, set_payment_step) = create_signal("plans".to_string()); // "plans", "stripe", "success"
    let (tx_hash, set_tx_hash) = create_signal(String::new());

    create_effect(move |_| {
        if let Some(npub) = user_npub.get() {
            set_input_npub.set(npub);
        } else if input_npub.get().is_empty() {
            set_input_npub.set("npub10pensourcerocks777nostrgamer999legendaryvipclubmember001".to_string());
        }
    });

    let handle_start_checkout = move |plan: &str| {
        set_selected_plan.set(plan.to_string());
        set_payment_step.set("stripe".to_string());
    };

    let handle_simulate_stripe_pay = move |_| {
        let npub = input_npub.get().trim().to_string();
        if npub.is_empty() {
            return;
        }
        let tier = match selected_plan.get().as_str() {
            "pro" => SubscriptionTier::ProGamer,
            "vip" => SubscriptionTier::LegendaryVip,
            _ => SubscriptionTier::ProGamer,
        };

        set_user_npub.set(Some(npub.clone()));
        set_user_tier.set(tier);
        set_tx_hash.set("stripe_ch_3Pz7x8A9K9200b8e7f101".to_string());
        set_payment_step.set("success".to_string());
    };

    view! {
        {move || if is_open.get() {
            view! {
                <div class="modal-backdrop" on:click=move |_| set_is_open.set(false)>
                    <div class="modal-card sub-modal" on:click=move |ev| ev.stop_propagation()>
                        <div class="modal-header">
                            <span class="modal-badge">"⭐ PREMIUM VIP ACCESS"</span>
                            <h2 class="modal-title">"有料プラン・サブスクリプション広告"</h2>
                            <button class="modal-close" on:click=move |_| set_is_open.set(false)>"✕"</button>
                        </div>

                        {move || match payment_step.get().as_str() {
                            "plans" => view! {
                                <div class="sub-plans-view">
                                    <p class="modal-desc">
                                        "Stripe決済であなたのNostr公開鍵（npub）を有料会員ホワイトリストに登録！有料限定ステージはNIP-04/44で暗号化されて配信されます。"
                                    </p>

                                    <!-- Plan Cards Grid -->
                                    <div class="plans-grid">
                                        <!-- Plan 1: Free -->
                                        <div class="plan-card">
                                            <div class="plan-tag">"無料"</div>
                                            <h3>"Free Player"</h3>
                                            <div class="plan-price">"$0"<span class="period">" / 永久"</span></div>
                                            <ul class="plan-features">
                                                <li>"✅ World 1-1 (地上)"</li>
                                                <li>"✅ World 1-2 (地下洞窟)"</li>
                                                <li>"❌ World 1-3 & 1-4 (ロック)"</li>
                                                <li>"❌ VIP暗号化チート (無効)"</li>
                                            </ul>
                                            <button class="plan-btn disabled" disabled=true>"現在の基本プラン"</button>
                                        </div>

                                        <!-- Plan 2: Pro -->
                                        <div class="plan-card featured-pro">
                                            <div class="plan-tag pro">"人気"</div>
                                            <h3>"Pro Nostr Gamer"</h3>
                                            <div class="plan-price">"$4.99"<span class="period">" / 月"</span></div>
                                            <ul class="plan-features">
                                                <li>"✅ World 1-1 & 1-2"</li>
                                                <li>"✅ World 1-3 キノコ木ステージ"</li>
                                                <li>"✅ Nostr リレーランキング登録"</li>
                                                <li>"✅ ムーンジャンプチート"</li>
                                            </ul>
                                            <button class="plan-btn pro" on:click=move |_| handle_start_checkout("pro")>
                                                "Proプランに加入 ($4.99)"
                                            </button>
                                        </div>

                                        <!-- Plan 3: Legendary VIP -->
                                        <div class="plan-card featured-vip">
                                            <div class="plan-tag vip">"最高峰 VIP"</div>
                                            <h3>"Legendary 1-Up VIP"</h3>
                                            <div class="plan-price">"$9.99"<span class="period">" / 月"</span></div>
                                            <ul class="plan-features">
                                                <li>"⭐ World 1-4 クッパ城 (NIP-04/44暗号化)"</li>
                                                <li>"⭐ 無敵ゴッドモード ＆ 無限ファイア"</li>
                                                <li>"⭐ 秘密のワープゾーン全開放"</li>
                                                <li>"⭐ Nostr 永久VIPバッジ付与"</li>
                                            </ul>
                                            <button class="plan-btn vip" on:click=move |_| handle_start_checkout("vip")>
                                                "⭐ VIPプランに加入 ($9.99)"
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }.into_view(),

                            "stripe" => view! {
                                <div class="stripe-checkout-view">
                                    <div class="stripe-header">
                                        <span class="stripe-logo">"💳 Stripe Powered Checkout"</span>
                                        <span class="plan-pill">{move || selected_plan.get().to_uppercase()}</span>
                                    </div>

                                    <div class="stripe-form">
                                        <div class="form-group">
                                            <label>"あなたの Nostr 公開鍵 (npub):"</label>
                                            <input
                                                type="text"
                                                class="retro-input"
                                                placeholder="npub1..."
                                                prop:value=input_npub
                                                on:input=move |ev| set_input_npub.set(event_target_value(&ev))
                                            />
                                            <small class="helper-text">"※ 決済完了後、このnpubが有料ホワイトリストに登録され、NIP-04/44暗号化キーが送信されます。"</small>
                                        </div>

                                        <div class="form-group">
                                            <label>"クレジットカード番号 (Stripe Test Mode):"</label>
                                            <input
                                                type="text"
                                                class="retro-input"
                                                value="4242 •••• •••• 4242"
                                                readonly=true
                                            />
                                        </div>

                                        <div class="form-row-2">
                                            <div class="form-group">
                                                <label>"有効期限:"</label>
                                                <input type="text" class="retro-input" value="12 / 28" readonly=true />
                                            </div>
                                            <div class="form-group">
                                                <label>"CVC:"</label>
                                                <input type="text" class="retro-input" value="999" readonly=true />
                                            </div>
                                        </div>

                                        <div class="stripe-actions">
                                            <button class="btn-secondary" on:click=move |_| set_payment_step.set("plans".to_string())>
                                                "戻る"
                                            </button>
                                            <button class="btn-stripe-pay" on:click=handle_simulate_stripe_pay>
                                                "💳 Stripeで決済を確定して公開鍵を登録"
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }.into_view(),

                            "success" => view! {
                                <div class="success-view">
                                    <div class="success-icon">"🎉"</div>
                                    <h3>"決済完了 ＆ ホワイトリスト登録成功！"</h3>
                                    <p>"Stripe決済が正常に完了し、あなたの公開鍵（npub）がNostr有料ホワイトリストに自動登録されました。"</p>
                                    <div class="code-box font-mono">
                                        <div>"ST_TX_ID: " {move || tx_hash.get()}</div>
                                        <div>"STATUS: PAID & WHITELISTED"</div>
                                        <div>"TIER: " {move || user_tier.get().display_name()}</div>
                                        <div>"NIP-04/44 CIPHER: WORLD 1-4 BOWSER CASTLE UNLOCKED"</div>
                                    </div>
                                    <button class="btn-primary" on:click=move |_| {
                                        set_is_open.set(false);
                                        set_payment_step.set("plans".to_string());
                                    }>
                                        "今すぐアンロックされたステージをプレイ！"
                                    </button>
                                </div>
                            }.into_view(),

                            _ => view! { <div></div> }.into_view(),
                        }}
                    </div>
                </div>
            }.into_view()
        } else {
            view! { <div></div> }.into_view()
        }}
    }
}
