use leptos::*;
use crate::nostr::crypto::{nip04_decrypt, nip04_encrypt, nip44_decrypt, nip44_encrypt, KeyPair};

#[component]
pub fn NipCryptoDemo() -> impl IntoView {
    let admin_keys = KeyPair::default_admin();
    let vip_keys = KeyPair::demo_vip_user();

    let (scheme, set_scheme) = create_signal("nip44".to_string());
    let (plain_input, set_plain_input) = create_signal(
        r#"{
  "stage": "World 1-4 Bowser Castle",
  "secret_boss": "Bowser Dragon Fire Breath",
  "lava_axe_x": 85,
  "vip_cheats": ["god_mode", "moon_jump", "infinite_fireballs"]
}"#.to_string()
    );
    let (ciphertext_output, set_ciphertext_output) = create_signal(String::new());
    let (decrypted_output, set_decrypted_output) = create_signal(String::new());
    let (crypto_status, set_crypto_status) = create_signal(String::new());

    let handle_encrypt = move |_| {
        let plain = plain_input.get();
        let is_nip44 = scheme.get() == "nip44";

        let result = if is_nip44 {
            nip44_encrypt(&plain, &admin_keys.secret_key_hex, &vip_keys.public_key_hex)
        } else {
            nip04_encrypt(&plain, &admin_keys.secret_key_hex, &vip_keys.public_key_hex)
        };

        match result {
            Ok(cipher) => {
                set_ciphertext_output.set(cipher);
                set_crypto_status.set("✅ 管理者鍵で暗号化成功！ リレー上の無料ユーザーには解読不可能な暗号文になりました。".to_string());
            }
            Err(e) => {
                set_crypto_status.set(format!("❌ 暗号化エラー: {}", e));
            }
        }
    };

    let handle_decrypt = move |_| {
        let cipher = ciphertext_output.get();
        if cipher.is_empty() {
            set_crypto_status.set("先に暗号化を実行してください。".to_string());
            return;
        }

        let is_nip44 = scheme.get() == "nip44";
        let result = if is_nip44 {
            nip44_decrypt(&cipher, &vip_keys.secret_key_hex, &admin_keys.public_key_hex)
        } else {
            nip04_decrypt(&cipher, &vip_keys.secret_key_hex, &admin_keys.public_key_hex)
        };

        match result {
            Ok(decrypted) => {
                set_decrypted_output.set(decrypted);
                set_crypto_status.set("🎉 VIP会員の秘密鍵で完全復号に成功！ ゲームエンジンにステージデータが展開されました。".to_string());
            }
            Err(e) => {
                set_crypto_status.set(format!("❌ 復号エラー: {}", e));
            }
        }
    };

    view! {
        <div class="crypto-demo-container">
            <div class="crypto-header">
                <span class="crypto-badge">"🔐 ZERO-KNOWLEDGE CONTENT DELIVERY"</span>
                <h1 class="page-title">"NIP-04 / NIP-44 暗号化 ＆ 復号リアルタイム実験室"</h1>
                <p class="page-description">
                    "Nostrの公開鍵暗号化（Diffie-Hellman ECDH）の仕組みを体感できます。サーバー側（管理者）で暗号化された有料コンテンツは、ホワイトリスト登録されたVIPユーザーのみが手元の秘密鍵で復号できます。"
                </p>
            </div>

            <!-- Algorithm Selector -->
            <div class="crypto-card">
                <div class="scheme-selector">
                    <label class="scheme-option">
                        <input
                            type="radio"
                            name="scheme"
                            value="nip44"
                            checked=move || scheme.get() == "nip44"
                            on:change=move |_| set_scheme.set("nip44".to_string())
                        />
                        <span class="scheme-title">"✨ NIP-44 v2 (ChaCha20-Poly1305 + HKDF)"</span>
                        <span class="scheme-desc">"最新のNostr暗号化標準。認証付きAEAD暗号、パディング長隠蔽による高耐性。"</span>
                    </label>

                    <label class="scheme-option">
                        <input
                            type="radio"
                            name="scheme"
                            value="nip04"
                            checked=move || scheme.get() == "nip04"
                            on:change=move |_| set_scheme.set("nip04".to_string())
                        />
                        <span class="scheme-title">"📜 NIP-04 (AES-256-CBC)"</span>
                        <span class="scheme-desc">"Nostr初期のDM暗号化方式。IVを?iv=形式で付与。"</span>
                    </label>
                </div>
            </div>

            <!-- Key Pair Information -->
            <div class="crypto-grid-2">
                <div class="crypto-card">
                    <h3>"🔑 送信者 (管理者 / Server Admin)"</h3>
                    <div class="key-field">
                        <label>"Admin npub:"</label>
                        <input type="text" class="retro-input readonly" value="npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64qv6spqqq28cfm" readonly=true />
                    </div>
                    <div class="key-field">
                        <label>"Admin Public Hex:"</label>
                        <input type="text" class="retro-input readonly" value="3bf0c63fcb93463407af97b5e0918838bce40f29c164a521551232443e3aa116" readonly=true />
                    </div>
                </div>

                <div class="crypto-card">
                    <h3>"🔑 受信者 (Stripe決済済み VIP会員)"</h3>
                    <div class="key-field">
                        <label>"User npub:"</label>
                        <input type="text" class="retro-input readonly" value="npub10pensourcerocks777nostrgamer999legendaryvipclubmember001" readonly=true />
                    </div>
                    <div class="key-field">
                        <label>"User Public Hex:"</label>
                        <input type="text" class="retro-input readonly" value="79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798" readonly=true />
                    </div>
                </div>
            </div>

            <!-- Live Encryption & Decryption Workspace -->
            <div class="crypto-workspace">
                <div class="workspace-col">
                    <h3>"1. 有料ステージ平文 (Plaintext JSON)"</h3>
                    <textarea
                        class="retro-textarea"
                        rows="8"
                        prop:value=plain_input
                        on:input=move |ev| set_plain_input.set(event_target_value(&ev))
                    ></textarea>
                    <button class="btn-primary full-width" on:click=handle_encrypt>
                        "🔐 管理者秘密鍵で暗号化してリレーへ投げる"
                    </button>
                </div>

                <div class="workspace-col">
                    <h3>"2. リレー上の暗号文 (Ciphertext Base64)"</h3>
                    <textarea
                        class="retro-textarea cipher"
                        rows="8"
                        prop:value=ciphertext_output
                        readonly=true
                        placeholder="暗号化を実行すると、ここに暗号文が表示されます（無料ユーザーに見えるデータ）"
                    ></textarea>
                    <button class="btn-secondary full-width" on:click=handle_decrypt>
                        "🔓 VIP会員秘密鍵で復号してゲームへ展開"
                    </button>
                </div>

                <div class="workspace-col">
                    <h3>"3. 復号されたデータ (Decrypted JSON)"</h3>
                    <textarea
                        class="retro-textarea success"
                        rows="8"
                        prop:value=decrypted_output
                        readonly=true
                        placeholder="復号を実行すると、ここに元の平文が完全復元されます"
                    ></textarea>
                </div>
            </div>

            {move || if !crypto_status.get().is_empty() {
                view! { <div class="crypto-alert">{crypto_status.get()}</div> }.into_view()
            } else {
                view! { <div></div> }.into_view()
            }}
        </div>
    }
}
