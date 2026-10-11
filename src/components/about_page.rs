use leptos::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="about-page-container">
            <div class="about-hero">
                <span class="hero-tag">"100% RUST + LEPTOS + NOSTR + CLOUDFLARE PAGES"</span>
                <h1 class="about-title">"私たちについて (About Nostr ASCII Arcade)"</h1>
                <p class="about-lead">
                    "中央集権型サーバーのログインやパスワード管理を排除し、Nostrの暗号公開鍵（npub）とWebAssembly（WASM）によって実現された、世界初のエンドツーエンド暗号化レトロゲームアーケードです。"
                </p>
            </div>

            <div class="tech-grid">
                <!-- Tech Card 1: 100% Rust & Leptos -->
                <div class="tech-card">
                    <div class="tech-icon">"🦀"</div>
                    <h3>"100% Rust & Leptos Frontend"</h3>
                    <p>
                        "フロントエンドUIは最新のRustフルスタックフレームワーク<strong>Leptos</strong>（Fine-grained reactivity）を採用。Virtual DOMを介さず、ネイティブWASMによる超高速レンダリングと60 FPS物理演算エンジンをブラウザ上で実現しています。"
                    </p>
                    <ul class="tech-points">
                        <li>"ゼロコスト抽象化とメモリ安全性"</li>
                        <li>"Trunkによる高速WASMバンドル"</li>
                        <li>"Web Audio APIによる8-bitチップチューン内蔵シンセ"</li>
                    </ul>
                </div>

                <!-- Tech Card 2: Nostr Backend & Decentralization -->
                <div class="tech-card">
                    <div class="tech-icon">"⚡"</div>
                    <h3>"Nostr (nostr-sdk / nostro2) as Back"</h3>
                    <p>
                        "ユーザー認証・セッション・ランキング・チャット・コンテンツ配信はすべて分散型プロトコル<strong>Nostr</strong>で完結。検閲耐性のある公開リレーネットワークを通じてデータを同期します。"
                    </p>
                    <ul class="tech-points">
                        <li>"Ed25519 / Schnorr署名による自己主権型アイデンティティ (npub)"</li>
                        <li>"NIP-07 ブラウザ拡張機能 (Alby, nos2x) のシームレス連携"</li>
                        <li>"Kind 30078によるハイスコア分散保存"</li>
                    </ul>
                </div>

                <!-- Tech Card 3: NIP-04 & NIP-44 Whitelist Encryption -->
                <div class="tech-card">
                    <div class="tech-icon">"🔐"</div>
                    <h3>"公開鍵ホワイトリスト ＋ 暗号化（NIP-04 / NIP-44）"</h3>
                    <p>
                        "Stripeなどで決済を完了したユーザーの公開鍵（npub）を「有料会員ホワイトリスト」に自動登録。有料限定ステージ（World 1-4など）やチートデータは、購入者の公開鍵向けにDiffie-Hellman ECDH暗号化してリレーに投げるため、<strong>無料ユーザーには中身を絶対に解読できません</strong>。"
                    </p>
                    <ul class="tech-points">
                        <li>"NIP-04: AES-256-CBC 共有秘密暗号"</li>
                        <li>"NIP-44 v2: HKDF + ChaCha20-Poly1305 認証付き暗号"</li>
                        <li>"クライアントサイドでの秘密鍵（nsec）即時復号"</li>
                    </ul>
                </div>

                <!-- Tech Card 4: Cloudflare Pages & GitHub Actions -->
                <div class="tech-card">
                    <div class="tech-icon">"☁️"</div>
                    <h3>"Cloudflare Pages & GitHub Actions CI/CD"</h3>
                    <p>
                        "GitHub Actionsによる自動テスト・WASMビルドを経て、Cloudflare Pagesのエッジネットワークにグローバル即時デプロイ。<code>CLOUDFLARE_API_TOKEN</code> と <code>CLOUDFLARE_ACCOUNT_ID</code> による安全なCI/CDパイプラインを確立しています。"
                    </p>
                    <ul class="tech-points">
                        <li>"エッジサーバーによる低レイテンシ配信"</li>
                        <li>"セキュリティヘッダー（CSP, FrameGuard）適用"</li>
                        <li>"完全自動化されたプッシュ時デプロイ"</li>
                    </ul>
                </div>
            </div>

            <!-- Architecture Diagram -->
            <div class="arch-section">
                <h2>"📐 システムアーキテクチャ概要"</h2>
                <div class="arch-box">
                    <pre class="ascii-diagram">
"┌─────────────────────────┐         ┌───────────────────────────────┐"
"│  ユーザーブラウザ (WASM)  │         │   Stripe 決済ゲートウェイ      │"
"│  Leptos 100% Rust Frontend│         │  (Credit Card / Lightning Zap)│"
"└────────────┬────────────┘         └──────────────┬────────────────┘"
"             │                                     │ Webhook (npub 添付)"
"             │ ECDH 共有鍵復号                      ▼"
"             │ (nsec ⇄ Admin Pubkey) ┌───────────────────────────────┐"
"             ▼                       │   Nostr バックエンド (SDK)   │"
"┌─────────────────────────┐          │ - 公開鍵ホワイトリスト登録    │"
"│ NIP-04 / NIP-44 暗号化 │ ◀─────────│ - VIP ステージ暗号化配信      │"
"│ 有料ステージ (World 1-4)│  Relay   │ - Kind 4 / Kind 1059 送信     │"
"└─────────────────────────┘  Query   └───────────────────────────────┘"
"             ▲                                     ▲"
"             └─────────────────┬───────────────────┘"
"                               │"
"                  ┌────────────┴─────────────┐"
"                  │ Nostr 分散リレー (Relays) │"
"                  │ damus, nos.lol, primal   │"
"                  └──────────────────────────┘"
                    </pre>
                </div>
            </div>
        </div>
    }
}
