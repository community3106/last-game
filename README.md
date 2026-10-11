# last-game — 100% Rust ASCII Mario

> Leptos (WASM) × Nostr × Stripe × Cloudflare Pages で作る、完全Rust製のASCIIマリオゲーム。

![Rust](https://img.shields.io/badge/100%25-Rust-orange?logo=rust)
![Leptos](https://img.shields.io/badge/Front-Leptos%20CSR-red)
![Nostr](https://img.shields.io/badge/Back-nostr--sdk-purple)
![License](https://img.shields.io/badge/license-MIT-blue)

---

## 概要

`last-game` は **100% Rust** で書かれたレトロASCIIマリオゲームです。

- **フロント**: [Leptos](https://leptos.dev/) CSR (WASM) — `leptos` as front
- **バック**: [nostr-sdk / nostr](https://github.com/rust-nostr/nostr) — `nostr-sdk` as back
- **認証・課金**: 公開鍵ホワイトリスト + NIP-04 / NIP-44 暗号化 + Stripe
- **インフラ**: Cloudflare Pages + GitHub Actions (`CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`)

```
[ Browser ] --WASM--> Leptos App --WSS--> Nostr Relays
      |                    |
      +-- Stripe Checkout --+-- Cloudflare Pages Functions (Rust/WASM)
                           +-- NIP-44復号 → 有料レベル表示
```

### 機能一覧

| ページ | 説明 |
|--------|------|
| `/` (Home) | ヒーロー、特徴、CTA。 games/about/pricing への導線 |
| `/games` | ゲーム一覧。無料ステージは誰でも、有料ステージは復号が必要 |
| `/play` | ASCIIマリオ本編 `<pre>` レンダリング。60FPS、キーボード操作 |
| `/about` | チーム紹介、技術スタック、Nostr思想 |
| `/pricing` | 無料 / Premium ¥980 / God Mode ¥2,980。Stripe Checkout連携 |
| `/admin` | 管理者特権。公開鍵ホワイトリストに含まれる `npub` のみアクセス可 |
| `/profile` | 自分の `npub` / `nsec` 管理、サブスク状態、復号テスト |

### 課金・暗号化フロー [4]

```
1. ユーザーが Stripe で決済
   ↓ webhook (Cloudflare Function)
2. ユーザーの公開鍵 (hex / npub) を「有料会員リスト」に登録
   → Nostr kind:30078 (parameterized replaceable) で publish
   → リレー: wss://relay.damus.io, wss://nos.lol, wss://relay.nostr.band
3. 有料コンテンツ（追加レベル、スキン、BGM等）は
   管理者が NIP-44 で `p` タグの鍵ごとに暗号化してリレーに投げる
   → 無料ユーザーには `content` が暗号文にしか見えない
4. フロントは `nostr-sdk` の `nip44::decrypt` で復号して描画
   フォールバックとして NIP-04 もサポート
```

---

## ASCII Mario 操作

| キー | 動作 |
|------|------|
| `←` `→` / `A` `D` | 左右移動 |
| `Space` / `W` / `↑` | ジャンプ |
| `R` | リスタート |
| `P` | ポーズ |

```
┌────────────────────────────────────────┐
│  SCORE 000340  LIVES ♥♥♡  STAGE 1-2    │
│                                        │
│          ○   ○                         │
│        ■■■ ■■■   G   ■■■               │
│                        ───             │
│     M              G                   │
│  ════════════════════════════════════   │
│  地面                                   │
└────────────────────────────────────────┘
  M = Mario  G = Goomba  ○ = Coin  ■ = Block
```

- 純Rustのゲームエンジン (`src/game/engine.rs`) は DOM 非依存でテスト可能
- 物理: 重力 `0.55`, ジャンプ初速 `-10.5`, 摩擦 `0.85`, 端数衝突判定
- レベルはJSONで定義 → 管理者が暗号化して配信

---

## 技術スタック

- **Rust 1.77+**, `wasm32-unknown-unknown`
- **Leptos 0.6 CSR** + `leptos_router` CSR
- **nostr-sdk 0.35** (`nip04`, `nip44` feature)
- **Trunk** で `index.html` → `dist/` ビルド
- **Cloudflare Pages** + **Wrangler** + **GitHub Actions**
- **Stripe Checkout** (Cloudflare Functions で Rust `worker` or JS shim)
- **Tailwind CSS** (CDN or CLI)

---

## ローカル開発

### 必要環境

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
# 任意: tailwind
npm install
```

### 起動

```bash
# フロント (WASM)
trunk serve --open

# ネイティブターミナル版 (crossterm)
cargo run --features terminal --bin last-game-terminal

# ビルド
trunk build --release
```

### 環境変数

`.env.example` を `.env` にコピー:

```ini
# Nostr
NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol,wss://relay.nostr.band
ADMIN_PUBKEYS=hex_pubkey1,hex_pubkey2  # 管理者ホワイトリスト (hex)
PAID_LIST_KIND=30078

# Stripe
STRIPE_PUBLISHABLE_KEY=pk_test_xxx
STRIPE_SECRET_KEY=sk_test_xxx
STRIPE_WEBHOOK_SECRET=whsec_xxx
STRIPE_PRICE_PREMIUM=price_xxx
STRIPE_PRICE_GOD=price_yyy

# Cloudflare
CLOUDFLARE_API_TOKEN=your_token
CLOUDFLARE_ACCOUNT_ID=your_account_id
```

---

## Nostr 詳細

### ホワイトリスト

`src/nostr/whitelist.rs`:

```rust
pub fn is_admin(pubkey: &PublicKey) -> bool {
  ADMIN_WHITELIST.contains(pubkey)
}
pub fn is_paid(pubkey: &PublicKey) -> bool {
  PAID_MEMBERS.contains(pubkey) // Nostr kind:30078 から同期
}
```

- ビルド時埋め込み (`ADMIN_PUBKEYS` env) + 実行時 Nostr イベント同期のハイブリッド

### 暗号化 (NIP-04 / NIP-44)

`src/nostr/crypto.rs`:

```rust
// NIP-44 (推奨, forward-secrecy あり)
let ct = nip44::encrypt(&sender_secret, &recipient_pubkey, plaintext, None)?;
let pt = nip44::decrypt(&recipient_secret, &sender_pubkey, ct)?;

// フォールバック NIP-04
let ct04 = nip04::encrypt(&secret, &pubkey, plaintext)?;
```

- 有料レベルは `kind:30078` + `d=level-2` + `p=<paid-pubkey>` で暗号化して publish
- クライアントは自分の `Keys` で該当イベントをフィルタリングし復号

---

## Stripe 連携

- フロントの「購入」ボタン → `POST /api/create-checkout-session` (Cloudflare Function)
- 成功後 `success.html?session_id=...` → `GET /api/verify-session` で `npub` を `paid` リストに追加
- Webhook: `POST /api/webhook` で `checkout.session.completed` を検証し Nostr に登録

> 開発時は `MOCK_STRIPE=true` でモック決済が使えます（Nostr への登録までスキップ可能）

---

## Cloudflare Pages & GitHub Actions

### Wrangler

`wrangler.toml`:

```toml
name = "last-game"
type = "javascript"
compatibility_date = "2024-10-01"
```

### Deploy Workflow

`.github/workflows/deploy.yml`:

- `trunk build --release` → `dist/` を `cloudflare/pages-action@v1` でデプロイ
- Secrets: `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`

GitHub リポジトリ → Settings → Secrets and variables → Actions に登録:

```
CLOUDFLARE_API_TOKEN = 先ほどのトークン
CLOUDFLARE_ACCOUNT_ID = アカウントID
```

### 手動デプロイ

```bash
wrangler pages deploy dist --project-name=last-game
```

---

## ディレクトリ構成

```
src/
  main.rs              # CSRエントリ (hydrate)
  lib.rs               # App re-export
  app.rs               # ルーティング
  components/
    navbar.rs          # ナビ + Nostr接続 + 管理者バッジ
    hero.rs
    game_view.rs       # ASCIIマリオ Leptos コンポーネント
    pricing.rs         # 有料プラン・サブスク広告
    about.rs
    admin.rs           # 管理者特権ページ
  game/
    engine.rs          # 純Rust 物理・衝突・スコア
    level.rs           # レベル定義 & JSON
    renderer.rs        # ASCII レンダリング
  nostr/
    mod.rs
    client.rs          # nostr-sdk Client ラッパ
    whitelist.rs
    crypto.rs          # NIP04/44
  stripe/
    mod.rs             # Checkout 生成・検証
functions/
  api/
    create-checkout-session.js
    verify-session.js
    webhook.js
```

---

## ライセンス

MIT

---

## 作者

community3106 / last-game
