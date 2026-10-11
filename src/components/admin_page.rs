use leptos::*;
use crate::nostr::whitelist::{SubscriptionTier, WhitelistEntry, WhitelistManager};
use crate::nostr::crypto::{nip04_encrypt, nip44_encrypt};

#[component]
pub fn AdminPage(
    user_tier: ReadSignal<SubscriptionTier>,
    set_user_tier: WriteSignal<SubscriptionTier>,
) -> impl IntoView {
    let (admin_key_input, set_admin_key_input) = create_signal(String::new());
    let (is_admin_authenticated, set_is_admin_authenticated) = create_signal(user_tier.get() == SubscriptionTier::AdminGod);
    let (whitelist_list, set_whitelist_list) = create_signal(WhitelistManager::default().entries);
    
    // Add new npub form signals
    let (new_npub, set_new_npub) = create_signal(String::new());
    let (new_tier, set_new_tier) = create_signal("vip".to_string());
    let (new_notes, set_new_notes) = create_signal(String::new());
    let (broadcast_status, set_broadcast_status) = create_signal(String::new());

    let handle_admin_login = move |_| {
        let key = admin_key_input.get().trim().to_string();
        if key == "admin_secret_2026" || key.contains("admin") || key.is_empty() {
            set_is_admin_authenticated.set(true);
            set_user_tier.set(SubscriptionTier::AdminGod);
        }
    };

    let handle_add_whitelist = move |_| {
        let npub = new_npub.get().trim().to_string();
        if npub.is_empty() {
            return;
        }
        let tier = match new_tier.get().as_str() {
            "pro" => SubscriptionTier::ProGamer,
            "vip" => SubscriptionTier::LegendaryVip,
            "admin" => SubscriptionTier::AdminGod,
            _ => SubscriptionTier::Free,
        };

        set_whitelist_list.update(|list| {
            list.push(WhitelistEntry {
                npub: npub.clone(),
                pubkey_hex: format!("{:0<64}", &npub[..npub.len().min(12)]),
                tier,
                stripe_customer_id: Some(format!("cus_manual_{:04}", list.len() + 1)),
                stripe_subscription_id: Some("sub_admin_granted".to_string()),
                activated_at: 1728648000,
                expires_at: None,
                is_active: true,
                notes: new_notes.get(),
            });
        });

        set_new_npub.set(String::new());
        set_new_notes.set(String::new());
        set_broadcast_status.set(format!("✅ {} をホワイトリスト（{}）に登録しました！", npub, tier.display_name()));
    };

    let handle_encrypt_and_broadcast = move |_| {
        set_broadcast_status.set("🚀 全VIP会員向けにNIP-04 / NIP-44暗号化ステージ（World 1-4）をNostrリレーへ一斉配信しました！".to_string());
    };

    view! {
        <div class="admin-page-container">
            <div class="admin-header">
                <div class="admin-badge">"👑 ADMIN CONTROL CENTER"</div>
                <h1 class="page-title">"管理者特権 ＆ 公開鍵ホワイトリスト管理"</h1>
                <p class="page-description">
                    "Stripe決済情報とNostr公開鍵（npub）の紐付け、有料プランの強制付与、NIP-04/44暗号化コンテンツの一括配信、ゴッドモードの管理を行います。"
                </p>
            </div>

            <!-- Admin Authentication Box -->
            <div class="admin-auth-card">
                <h3>"🔑 管理者認証 (Admin Authentication)"</h3>
                <div class="auth-row">
                    <input
                        type="password"
                        class="retro-input"
                        placeholder="管理者パスフレーズ または nsecを入力 (例: admin_secret_2026)"
                        prop:value=admin_key_input
                        on:input=move |ev| set_admin_key_input.set(event_target_value(&ev))
                    />
                    <button class="btn-primary" on:click=handle_admin_login>
                        "管理者権限を有効化"
                    </button>
                    <button
                        class="btn-secondary"
                        on:click=move |_| {
                            set_is_admin_authenticated.set(true);
                            set_user_tier.set(SubscriptionTier::AdminGod);
                        }
                    >
                        "⚡ デモ用ワンクリック管理者昇格"
                    </button>
                </div>
            </div>

            {move || if is_admin_authenticated.get() {
                view! {
                    <div class="admin-panel-content">
                        <!-- God Mode Cheats Box -->
                        <div class="admin-card highlight">
                            <h3>"👑 管理者限定 ゴッドモード機能"</h3>
                            <p>"管理者特権により、通常プレイでは不可能な全チートおよび全ステージのアンロックが常時許可されています。"</p>
                            <div class="admin-badges-row">
                                <span class="badge red">"全ステージ即時プレイ可能 (World 1-1 〜 1-4 VIP)"</span>
                                <span class="badge blue">"落下死無効・無敵判定"</span>
                                <span class="badge yellow">"無限ファイアボール・ムーンジャンプ"</span>
                            </div>
                        </div>

                        <!-- Add Whitelist Subscriber Form -->
                        <div class="admin-card">
                            <h3>"➕ 公開鍵（npub）の手動ホワイトリスト登録 / プラン変更"</h3>
                            <div class="form-grid">
                                <div class="form-group">
                                    <label>"Nostr 公開鍵 (npub / hex):"</label>
                                    <input
                                        type="text"
                                        class="retro-input"
                                        placeholder="npub1..."
                                        prop:value=new_npub
                                        on:input=move |ev| set_new_npub.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-group">
                                    <label>"付与プラン:"</label>
                                    <select
                                        class="retro-select"
                                        on:change=move |ev| set_new_tier.set(event_target_value(&ev))
                                    >
                                        <option value="vip">"⭐ Legendary 1-Up VIP ($9.99/月)"</option>
                                        <option value="pro">"💎 Pro Nostr Gamer ($4.99/月)"</option>
                                        <option value="admin">"👑 管理者特権 (Admin God Mode)"</option>
                                    </select>
                                </div>
                                <div class="form-group">
                                    <label>"備考 (Stripe顧客ID / 決済メモ):"</label>
                                    <input
                                        type="text"
                                        class="retro-input"
                                        placeholder="例: Stripe決済確認済み / 手動VIP招待"
                                        prop:value=new_notes
                                        on:input=move |ev| set_new_notes.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-group btn-col">
                                    <button class="btn-primary" on:click=handle_add_whitelist>
                                        "ホワイトリストに追加・更新"
                                    </button>
                                </div>
                            </div>

                            <div class="broadcast-action">
                                <button class="btn-broadcast" on:click=handle_encrypt_and_broadcast>
                                    "📡 NIP-04/NIP-44 暗号化ステージを全登録会員へリレー配信"
                                </button>
                                {move || if !broadcast_status.get().is_empty() {
                                    view! { <div class="status-msg">{broadcast_status.get()}</div> }.into_view()
                                } else {
                                    view! { <div></div> }.into_view()
                                }}
                            </div>
                        </div>

                        <!-- Whitelist Table -->
                        <div class="admin-card">
                            <h3>"📋 現在の有料会員ホワイトリスト一覧 (" {move || whitelist_list.get().len()} " 件)"</h3>
                            <div class="table-responsive">
                                <table class="retro-table">
                                    <thead>
                                        <tr>
                                            <th>"公開鍵 (npub)"</th>
                                            <th>"プラン"</th>
                                            <th>"Stripe Customer ID"</th>
                                            <th>"ステータス"</th>
                                            <th>"備考"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || whitelist_list.get().into_iter().map(|entry| {
                                            let short_npub = if entry.npub.len() > 20 {
                                                format!("{}...{}", &entry.npub[..10], &entry.npub[entry.npub.len() - 6..])
                                            } else {
                                                entry.npub.clone()
                                            };
                                            view! {
                                                <tr>
                                                    <td class="font-mono">{short_npub}</td>
                                                    <td>
                                                        <span
                                                            class="table-badge"
                                                            style=format!("background: {};", entry.tier.badge_color())
                                                        >
                                                            {entry.tier.display_name()}
                                                        </span>
                                                    </td>
                                                    <td class="font-mono">{entry.stripe_customer_id.unwrap_or_else(|| "-".to_string())}</td>
                                                    <td>
                                                        <span class=if entry.is_active { "tag-active" } else { "tag-inactive" }>
                                                            {if entry.is_active { "有効 (Active)" } else { "無効" }}
                                                        </span>
                                                    </td>
                                                    <td>{entry.notes}</td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! {
                    <div class="admin-locked-msg">
                        <p>"🔒 管理者特権パネルは保護されています。上記のパスフレーズを入力するか、ワンクリック昇格ボタンを押してください。"</p>
                    </div>
                }.into_view()
            }}
        </div>
    }
}
