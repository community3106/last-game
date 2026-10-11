use leptos::*;
use crate::nostr::client::NostrContext;
use crate::nostr::whitelist::{is_admin_hex, ADMIN_LIST, PAID_LIST};
use crate::nostr::crypto::{encrypt_nip44_demo, decrypt_nip44_demo};

#[component]
pub fn AdminPage() -> impl IntoView {
    let nostr = expect_context::<NostrContext>();
    let pubkey_hex = nostr.pubkey_hex;

    let is_admin = move || pubkey_hex.get().map(|h| is_admin_hex(&h)).unwrap_or(false);

    // 管理者用: 暗号化デモ
    let plain = create_rw_signal(String::from(r#"{"level":2,"name":"星の洞窟","blocks":[[1,1,1]],"coins":12}"#));
    let recipient = create_rw_signal(String::new());
    let encrypted = create_rw_signal::<Option<String>>(None);
    let decrypted = create_rw_signal::<Option<String>>(None);
    let encrypt_msg = create_rw_signal::<Option<String>>(None);

    let do_encrypt = move |_| {
        let pt = plain.get();
        let recip = recipient.get().trim().to_string();
        if recip.is_empty() {
            encrypt_msg.set(Some("宛先の公開鍵(hex/npub)を入力してください".into()));
            return;
        }
        // デモ: NIP-44風の擬似暗号化 (本番は nostr-sdk の nip44::encrypt を使う)
        match encrypt_nip44_demo(&pt, &recip) {
            Ok(ct) => {
                encrypted.set(Some(ct));
                encrypt_msg.set(Some("✓ 暗号化成功 — この内容を kind:30078 でリレーにpublishします (モック)".into()));
            }
            Err(e) => encrypt_msg.set(Some(format!("✗ {}", e))),
        }
    };
    let do_decrypt = move |_| {
        if let Some(ct) = encrypted.get() {
            match decrypt_nip44_demo(&ct) {
                Ok(pt) => decrypted.set(Some(pt)),
                Err(e) => encrypt_msg.set(Some(format!("復号失敗: {}", e))),
            }
        }
    };

    // 有料会員リスト (モック)
    let paid_list = PAID_LIST;
    let admin_list = ADMIN_LIST;

    view! {
        <section class="max-w-6xl mx-auto px-6 py-10">
            <div class="flex items-center gap-3">
                <span class="w-2 h-6 bg-amber-500 inline-block"></span>
                <h1 class="pixel-font text-lg">ADMIN — 管理者特権</h1>
                <span class="text-[11px] px-2.5 py-1 rounded-full border border-amber-500/30 bg-amber-500/10 text-amber-300">公開鍵ホワイトリストで保護</span>
            </div>

            // ゲート
            {move || {
                if is_admin() {
                    view!{
                        <div class="mt-4 rounded-2xl border border-emerald-500/30 bg-emerald-500/10 p-4 text-sm text-emerald-200 flex items-center gap-2">
                            <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                            管理者として認証済み — 公開鍵 <code class="px-1 bg-black rounded">{pubkey_hex.get().unwrap_or_default()[..8].to_string()}</code>…
                        </div>
                    }.into_view()
                } else {
                    view!{
                        <div class="mt-4 rounded-2xl border border-red-500/30 bg-red-500/10 p-6">
                            <h3 class="font-bold text-red-300">⛔ アクセス拒否</h3>
                            <p class="mt-2 text-sm text-zinc-300 leading-relaxed">
                                このページは <b>公開鍵ホワイトリスト</b> に登録された管理者のみがアクセスできます。<br/>
                                右上の「Nostrでログイン」から管理者鍵でログインしてください。<br/>
                                <span class="text-xs text-zinc-500">ヒント: 環境変数 <code>ADMIN_PUBKEYS</code> に hex をカンマ区切りで設定。デモでは任意の鍵で「デモ鍵で試す」→ 手動で is_admin を true にできます (開発用)。</span>
                            </p>
                            <div class="mt-4 flex gap-2">
                                <button on:click=move |_| {
                                    // 開発用バイパス: 現在の鍵を管理者リストに追加 (ローカルモック)
                                    if let Some(h) = pubkey_hex.get() {
                                        crate::nostr::whitelist::mock_add_admin(&h);
                                    }
                                } class="px-4 py-2 rounded-full bg-amber-500 text-black text-xs font-black">🧪 開発用: 自分を管理者にする</button>
                                <a href="/pricing" class="px-4 py-2 rounded-full border border-zinc-700 text-xs">有料プランを見る</a>
                            </div>
                        </div>
                    }.into_view()
                }
            }}

            // 管理パネル本体 (管理者のみ表示、ただしデモでは常に表示して中身を隠す)
            <div class=move || if is_admin() { "mt-6 grid lg:grid-cols-2 gap-6" } else { "mt-6 grid lg:grid-cols-2 gap-6 opacity-40 pointer-events-none" }>
                // 左: 会員管理
                <div class="rounded-2xl border border-zinc-800 bg-zinc-900/40 p-6">
                    <h3 class="font-bold">有料会員リスト (kind: 30078)</h3>
                    <p class="text-xs text-zinc-500 mt-1">Stripe webhook 後にここに公開鍵が追加されます。NIP-44 で各鍵向けに暗号化して配信。</p>

                    <div class="mt-4">
                        <h4 class="text-xs font-bold tracking-widest text-zinc-400">ADMIN WHITELIST</h4>
                        <ul class="mt-2 space-y-1.5">
                            {move || admin_list.get().iter().map(|k| view!{
                                <li class="text-xs font-mono px-2.5 py-1.5 rounded-lg bg-black border border-zinc-800 flex justify-between">
                                    <span>{&k[..16]}…</span><span class="text-amber-400">👑 admin</span>
                                </li>
                            }).collect_view()}
                        </ul>
                    </div>
                    <div class="mt-4">
                        <h4 class="text-xs font-bold tracking-widest text-zinc-400">PAID MEMBERS</h4>
                        {move || {
                            let list = paid_list.get();
                            if list.is_empty() {
                                view!{ <p class="mt-2 text-xs text-zinc-500">まだ有料会員がいません。/pricing でモック決済を試してください。</p> }.into_view()
                            } else {
                                view!{
                                    <ul class="mt-2 space-y-1.5">
                                        {list.iter().map(|k| view!{
                                            <li class="text-xs font-mono px-2.5 py-1.5 rounded-lg bg-black border border-zinc-800 flex justify-between">
                                                <span>{&k[..16]}…</span><span class="text-violet-400">★ paid</span>
                                            </li>
                                        }).collect_view()}
                                    </ul>
                                }.into_view()
                            }
                        }}
                    </div>

                    <div class="mt-4 flex gap-2">
                        <button on:click=move |_| {
                            // モック: ダミー会員を追加
                            crate::nostr::whitelist::mock_add_paid("f" .repeat(64));
                        } class="text-xs px-3 py-1.5 rounded-full border border-zinc-700 hover:bg-zinc-800">+ ダミー追加</button>
                        <button on:click=move |_| crate::nostr::whitelist::mock_clear_paid() class="text-xs px-3 py-1.5 rounded-full border border-zinc-700 hover:bg-zinc-800">クリア</button>
                    </div>
                </div>

                // 右: 暗号化配信
                <div class="rounded-2xl border border-zinc-800 bg-zinc-900/40 p-6">
                    <h3 class="font-bold">有料コンテンツを暗号化して配信</h3>
                    <p class="text-xs text-zinc-500 mt-1">NIP-44 (推奨) / NIP-04 フォールバック。宛先の公開鍵ごとに暗号化し、リレーに <code>kind:30078 + pタグ</code> で publish。</p>

                    <label class="mt-4 block text-xs font-bold">レベルJSON (平文)</label>
                    <textarea
                        prop:value=move || plain.get()
                        on:input=move |ev| plain.set(event_target_value(&ev))
                        rows=4
                        class="mt-1 w-full px-3 py-2 rounded-xl bg-black border border-zinc-700 text-xs font-mono focus:outline-none focus:border-violet-600"
                    ></textarea>

                    <label class="mt-3 block text-xs font-bold">宛先 公開鍵 (hex / npub)</label>
                    <input
                        prop:value=move || recipient.get()
                        on:input=move |ev| recipient.set(event_target_value(&ev))
                        placeholder="例: npub1... または hex"
                        class="mt-1 w-full px-3 py-2 rounded-xl bg-black border border-zinc-700 text-xs font-mono focus:outline-none focus:border-violet-600"
                    />

                    <div class="mt-3 flex gap-2">
                        <button on:click=do_encrypt class="px-4 py-2 rounded-full bg-violet-600 hover:bg-violet-500 text-white text-xs font-bold">🔐 NIP-44で暗号化</button>
                        <button on:click=do_decrypt class="px-4 py-2 rounded-full border border-zinc-700 text-xs">🔓 復号テスト</button>
                    </div>

                    {move || encrypt_msg.get().map(|m| view!{ <p class="mt-3 text-xs p-2.5 rounded-xl bg-black border border-zinc-800">{m}</p> })}
                    {move || encrypted.get().map(|ct| view!{
                        <div class="mt-3">
                            <p class="text-xs font-bold">暗号文 (リレーに投げる content)</p>
                            <pre class="mt-1 text-[10px] p-3 rounded-xl bg-black border border-zinc-800 overflow-x-auto whitespace-pre-wrap break-all">{ct}</pre>
                        </div>
                    })}
                    {move || decrypted.get().map(|pt| view!{
                        <div class="mt-3">
                            <p class="text-xs font-bold">復号結果</p>
                            <pre class="mt-1 text-xs p-3 rounded-xl bg-emerald-950 border border-emerald-800 overflow-x-auto">{pt}</pre>
                        </div>
                    })}

                    <div class="mt-4 rounded-xl bg-amber-950/40 border border-amber-800 p-3 text-xs leading-relaxed text-amber-200">
                        <b>本番コード例:</b> <code>nostr-sdk</code> の <code>nip44::encrypt(&sender_keys.secret_key(), &recipient_pubkey, plaintext, None)</code> を使用。
                        失敗時は <code>nip04::encrypt</code> にフォールバック。
                    </div>
                </div>
            </div>

            // Stripe ログ
            <div class="mt-6 rounded-2xl border border-zinc-800 bg-zinc-900/30 p-6">
                <h3 class="font-bold text-sm">Stripe Webhook ログ (モック)</h3>
                <pre class="mt-3 text-xs bg-black rounded-xl p-4 border border-zinc-800 overflow-x-auto leading-relaxed">{r#"[webhook] POST /api/webhook  — checkout.session.completed
  → verify signature with STRIPE_WEBHOOK_SECRET
  → extract pubkey from metadata.npub
  → publish to Nostr kind:30078 (paid list)
  → 200 OK

[pages] CLOUDFLARE_API_TOKEN=***  CLOUDFLARE_ACCOUNT_ID=***
  → wrangler pages deploy dist --project-name=last-game"#}</pre>
            </div>
        </section>
    }
}
