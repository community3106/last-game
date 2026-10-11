use leptos::*;
use crate::nostr::client::NostrContext;
use crate::nostr::whitelist::is_paid_hex;

#[component]
pub fn PricingPage() -> impl IntoView {
    let nostr = expect_context::<NostrContext>();
    let pubkey_hex = nostr.pubkey_hex;

    let checkout_loading = create_rw_signal::<Option<&'static str>>(None);
    let message = create_rw_signal::<Option<String>>(None);

    // モックCheckout: 実際は /api/create-checkout-session を叩く
    let start_checkout = move |plan: &'static str| {
        let hex_opt = pubkey_hex.get();
        if hex_opt.is_none() {
            message.set(Some("先に右上の「Nostrでログイン」で鍵を登録してください。Stripe決済後にその公開鍵が有料会員リストに登録されます。".into()));
            return;
        }
        checkout_loading.set(Some(plan));
        let hex = hex_opt.unwrap();
        // 擬似非同期: 1.2s後にモック成功
        let nostr_clone = nostr.clone();
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(900).await;
            // モック: ローカルストレージに paid フラグを保存し、Nostrへ擬似publish
            nostr_clone.mock_mark_paid(&hex);
            message.set(Some(format!("✓ {} プランが有効になりました！ (モック) — 公開鍵 {} が有料会員リストに登録され、NIP-44で暗号化されたレベルが復号可能になりました。 /play?level=2 をお試しください。", plan, &hex[..8])));
            checkout_loading.set(None);
        });
    };

    let verify_mock = move |_| {
        // 開発用: 手動で有料化
        if let Some(hex) = pubkey_hex.get() {
            nostr.mock_mark_paid(&hex);
            message.set(Some("✓ ローカルで有料フラグを付与しました。/play?level=2 が遊べます。".into()));
        } else {
            message.set(Some("鍵が未登録です。先にログインしてください。".into()));
        }
    };

    view! {
        <section class="max-w-6xl mx-auto px-6 py-10">
            // ヘッダ
            <div class="text-center max-w-2xl mx-auto">
                <span class="inline-flex items-center gap-2 text-[11px] px-3 py-1 rounded-full border border-amber-500/30 bg-amber-500/10 text-amber-300">有料プラン・サブスク広告</span>
                <h1 class="pixel-font text-xl md:text-2xl mt-4">PRICING</h1>
                <p class="mt-3 text-sm text-zinc-400 leading-relaxed">
                    無料で1-1は遊べます。<b class="text-white">Stripe</b>で決済すると、あなたの公開鍵 (<code class="px-1 py-0.5 bg-zinc-800 rounded text-xs">npub</code>) が
                    <b class="text-white">有料会員リスト</b>に登録され、<b class="text-white">NIP-44</b>で暗号化された追加ステージがリレーから復号できるようになります。
                </p>
            </div>

            // 3プラン
            <div class="mt-10 grid md:grid-cols-3 gap-6">
                <PlanCard
                    name="FREE"
                    price="¥0"
                    period="/ 永久"
                    features=vec!["ASCII MARIO 1-1 遊び放題", "コイン・スコア保存(ローカル)", "Nostrログイン不要"]
                    cta="今すぐ遊ぶ"
                    href="/play"
                    featured=false
                />
                <PlanCard
                    name="PREMIUM"
                    price="¥980"
                    period="/ 月"
                    features=vec!["全ステージ (1-2〜1-4) 解放", "NIP-44 暗号化レベルを復号", "限定スキン & BGM", "リレーに進捗を Nostr で保存"]
                    cta="Premium に登録"
                    on_click=move |_| start_checkout("PREMIUM")
                    loading=move || checkout_loading.get() == Some("PREMIUM")
                    featured=true
                    badge="人気"
                />
                <PlanCard
                    name="GOD MODE"
                    price="¥2,980"
                    period="/ 月"
                    features=vec!["Premium すべて +", "レベルエディタ & 自作面を暗号化配信", "管理者特権の一部を付与 (β)", "優先サポート & 限定リレー"]
                    cta="God Mode に登録"
                    on_click=move |_| start_checkout("GOD MODE")
                    loading=move || checkout_loading.get() == Some("GOD MODE")
                    featured=false
                    badge="管理者推奨"
                />
            </div>

            // メッセージ
            {move || message.get().map(|m| view!{
                <div class="mt-6 rounded-2xl border border-emerald-500/30 bg-emerald-500/10 p-4 text-sm text-emerald-200">{m}</div>
            })}

            // Stripe & Nostr 説明 + 広告
            <div class="mt-8 grid md:grid-cols-2 gap-6">
                <div class="rounded-2xl border border-zinc-800 bg-zinc-900/40 p-6">
                    <h3 class="font-bold flex items-center gap-2">💳 Stripe 決済フロー</h3>
                    <ol class="mt-3 space-y-2 text-sm text-zinc-400 list-decimal list-inside leading-relaxed">
                        <li><b class="text-zinc-200">Nostrでログイン</b> → 公開鍵を確定</li>
                        <li>「Premiumに登録」→ <code class="px-1 bg-zinc-800 rounded">POST /api/create-checkout-session</code> (Cloudflare Function)</li>
                        <li>Stripe Checkoutで決済 → <code>checkout.session.completed</code> webhook</li>
                        <li>Functionが <code>paid pubkey</code> を Nostr <code>kind:30078</code> に publish</li>
                        <li>フロントはリレーから有料リストを取得し、<code>nip44::decrypt</code>でレベル復号</li>
                    </ol>
                    <p class="mt-3 text-xs text-zinc-500">開発時は <code>MOCK_STRIPE=true</code> でモック決済が使えます。本番は <code>STRIPE_SECRET_KEY</code> を Cloudflare Secrets に登録。</p>
                </div>
                <div class="rounded-2xl border border-violet-500/20 bg-violet-500/10 p-6">
                    <h3 class="font-bold flex items-center gap-2">🔐 NIP-04 / NIP-44 暗号化</h3>
                    <p class="mt-2 text-sm text-zinc-300 leading-relaxed">
                        有料コンテンツは管理者の秘密鍵で <b>NIP-44</b> (推奨) により各有料会員の公開鍵向けに暗号化してリレーに投げます。
                        <code class="px-1 bg-black rounded">p</code>タグで宛先を指定するので、無料ユーザーや第三者は中身を読めません。
                        旧クライアント向けに <b>NIP-04</b> フォールバックも実装。
                    </p>
                    <pre class="mt-3 text-[11px] bg-black rounded-xl p-3 border border-zinc-800 overflow-x-auto">{r#"// 管理者: 暗号化してpublish
let ct = nip44::encrypt(&admin_keys.secret_key(), &paid_pubkey, level_json, None)?;
let event = EventBuilder::new(Kind::Custom(30078), ct, vec![Tag::public_key(paid_pubkey)])
    .custom_created_at(now).to_event(&admin_keys)?;

// 会員: 復号
let pt = nip44::decrypt(&my_keys.secret_key(), &admin_pubkey, event.content)?;"#}</pre>
                    <button on:click=verify_mock class="mt-3 text-xs px-3 py-1.5 rounded-full border border-violet-500/30 hover:bg-violet-500/20">🧪 モックで有料化を試す (開発用)</button>
                </div>
            </div>

            // 管理者特権 広告
            <div class="mt-6 rounded-2xl border border-amber-500/30 bg-gradient-to-r from-amber-500/10 to-orange-500/10 p-6 flex flex-col md:flex-row gap-4 items-start md:items-center">
                <div class="w-10 h-10 rounded-xl bg-amber-500 grid place-items-center font-black text-black">👑</div>
                <div class="flex-1">
                    <h3 class="font-bold text-sm">管理者特権</h3>
                    <p class="text-xs text-zinc-400 mt-1 leading-relaxed">
                        公開鍵ホワイトリスト (<code>ADMIN_PUBKEYS</code>) に登録された鍵だけが <a href="/admin" class="underline decoration-amber-500">/admin</a> にアクセス可能。
                        レベルの暗号化配信、会員管理、Stripe Webhookの検証ログを操作できます。
                    </p>
                </div>
                <a href="/admin" class="px-5 py-2.5 rounded-full bg-amber-500 text-black text-xs font-black hover:bg-amber-400">管理者ページ →</a>
            </div>

            // よくある質問
            <div class="mt-8 rounded-2xl border border-zinc-800 bg-zinc-900/30 p-6">
                <h3 class="font-bold text-sm">FAQ</h3>
                <details class="mt-3 group">
                    <summary class="cursor-pointer text-sm font-bold list-none flex justify-between">Q. Nostr鍵は安全ですか？ <span class="text-zinc-500 group-open:rotate-180 transition">⌄</span></summary>
                    <p class="mt-2 text-sm text-zinc-400">秘密鍵(nsec)はブラウザのlocalStorageにのみ保存し、リレーやサーバーには送信しません。NIP-07拡張があればそちらを優先します。</p>
                </details>
                <details class="mt-3 group">
                    <summary class="cursor-pointer text-sm font-bold list-none flex justify-between">Q. サブスクを解約するとどうなりますか？ <span class="text-zinc-500 group-open:rotate-180 transition">⌄</span></summary>
                    <p class="mt-2 text-sm text-zinc-400">Stripeで解約後、次回同期で有料リストから公開鍵が外れ、新しい暗号化レベルの復号が不可になります。既に復号済みのレベルはキャッシュされます。</p>
                </details>
            </div>
        </section>
    }
}

#[component]
fn PlanCard(
    name: &'static str,
    price: &'static str,
    period: &'static str,
    features: Vec<&'static str>,
    #[prop(optional)] cta: &'static str,
    #[prop(optional)] href: Option<&'static str>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(optional)] loading: Option<Signal<bool>>,
    #[prop(default=false)] featured: bool,
    #[prop(optional)] badge: Option<&'static str>,
) -> impl IntoView {
    let loading = loading.unwrap_or_else(|| Signal::derive(|| false));
    view! {
        <div class=format!("relative rounded-[20px] border p-6 flex flex-col {}", if featured { "border-violet-500/40 bg-gradient-to-b from-violet-600/15 to-zinc-900/60 shadow-glow scale-[1.02]" } else { "border-zinc-800 bg-zinc-900/40" })>
            {badge.map(|b| view!{ <span class="absolute -top-3 right-4 text-[10px] px-2.5 py-1 rounded-full bg-amber-500 text-black font-black tracking-widest">{b}</span> })}
            <h3 class="pixel-font text-[11px] tracking-widest text-zinc-400">{name}</h3>
            <div class="mt-3 flex items-baseline gap-1">
                <span class="text-3xl font-black">{price}</span><span class="text-xs text-zinc-500">{period}</span>
            </div>
            <ul class="mt-4 space-y-2 flex-1">
                {features.into_iter().map(|f| view!{ <li class="flex gap-2 text-sm text-zinc-300"><span class="text-emerald-400">✓</span> {f}</li> }).collect_view()}
            </ul>
            {if let Some(h) = href {
                view!{ <a href=h class="mt-6 block text-center py-3 rounded-full bg-white text-black text-sm font-black hover:bg-zinc-100"> {cta} </a> }.into_view()
            } else {
                let cb = on_click.unwrap_or_else(|| Callback::new(|_| ()));
                view!{
                    <button
                        on:click=move |_| cb.call(())
                        disabled=move || loading.get()
                        class=move || if loading.get() { "mt-6 w-full py-3 rounded-full bg-zinc-700 text-zinc-400 text-sm font-bold cursor-wait" } else if featured { "mt-6 w-full py-3 rounded-full bg-violet-600 hover:bg-violet-500 text-white text-sm font-black" } else { "mt-6 w-full py-3 rounded-full bg-zinc-800 hover:bg-zinc-700 text-white text-sm font-bold border border-zinc-700" }
                    >
                        {move || if loading.get() { "処理中…" } else { cta }}
                    </button>
                }.into_view()
            }}
        </div>
    }
}
