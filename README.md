# 🍄 ASCII Mario 1985 (100% Rust WASM & Nostr NIP-04/44 Crypto VIP)

100% Rust（WASM + Leptos）で動作する完全なレトロ **ASCII マリオゲーム** と、**Nostr 分散プロトコル（nostr-sdk / nostro2）**・**公開鍵ホワイトリスト暗号化（NIP-04 / NIP-44）**・**Stripe 決済連動**・**Cloudflare Pages & GitHub Actions CI/CD** を統合した分散型ゲームアーケードプラットフォームです。

---

## 🌟 主な機能と特徴 (Features)

### 1. 🕹️ 100% Rust ASCII マリオ ゲームエンジン
- **60 FPS 物理演算エンジン**: 慣性、重力、可変ジャンプ、ブロック衝突判定、ダッシュ、しゃがみ、ファイアボール投擲。
- **レトロ ASCII アートレンダリング**: マリオ `m>` / `[M]>` / `[F*]>`、クリボー `[o_o]`、ノコノコ `<(OO)>`、ハテナブロック `[?]`、レンガ `[#]`、土管、旗、そして溶岩とクッパ `🦖 [[W_W]] 🔥`！
- **マルチステージ構成**:
  - `World 1-1`: 地上王国（無料プレイ）
  - `World 1-2`: 地下洞窟（無料プレイ）
  - `World 1-3`: キノコ木アスレチック（Proプラン限定）
  - `World 1-4`: クッパ城 溶岩ブリッジ＆クッパ戦（**Legendary VIP限定 / NIP-04・NIP-44 暗号化**）
- **8-Bit チップチューン内蔵 Web Audio シンセサイザー**: ジャンプ音、コイン音、パワーアップ音、踏みつけ音、ファイア音、クリアファンファーレ、ゲームオーバー音。
- **CRT レトロモニターシェーダー**: 走査線（Scanlines）トグル、カラーテーマ（Classic NES, Matrix Phosphor Green, 1980s Amber CRT, Cyberpunk Synthwave）。

---

### 2. ⚡ Nostr バックエンド ＆ 分散型アイデンティティ (`nostr-sdk` / `nostro2`)
- **中央集権ログイン不要**: ユーザーの公開鍵（`npub`）または NIP-07 ブラウザ拡張（Alby / nos2x）で認証。
- **Nostr 分散リレー同期**: `wss://relay.damus.io`, `wss://nos.lol`, `wss://relay.primal.net`, `wss://nostr.band` に接続。
- **Kind 30078 ハイスコア署名送信**: クリアタイム・コイン数・スコアを Schnorr 署名付き不変イベントとしてリレーへ保存。

---

### 3. 🔐 公開鍵ホワイトリスト ＋ 暗号化（NIP-04 / NIP-44）
```
[ Stripe 決済 (Card / Zap) ] 
        │
        ▼ (Webhook: customer_npub 添付)
[ Nostr 有料ホワイトリストに npub 登録 ]
        │
        ▼ (Diffie-Hellman ECDH 秘密共有鍵導出)
[ 有料ステージ (World 1-4 / VIPチート) を NIP-04 / NIP-44 で暗号化 ]
        │
        ▼ (Nostr リレーへ配信: Kind 4 / Kind 1059)
[ ユーザーブラウザ (WASM) : 自分の nsec で即時復号 & ゲーム解放！ ]
```
- **無料ユーザーからの完全遮蔽**: 暗号化された有料コンテンツは、ホワイトリスト登録された公開鍵に対応する秘密鍵（`nsec`）を持つユーザーのみ復号できるため、リレーを傍受しても無料ユーザーには中身を絶対に解読できません。
- **NIP-04**: AES-256-CBC 共有秘密暗号（`?iv=` 形式）。
- **NIP-44 v2**: HKDF-SHA256 + ChaCha20-Poly1305 認証付き AEAD 暗号（最新標準）。

---

### 4. 👑 管理者特権 (Admin Privileges) ＆ 有料プラン・サブスク
- **管理者特権ダッシュボード**:
  - 管理者公開鍵認証
  - 有料会員ホワイトリストのリアルタイム追加・プラン変更・無効化
  - 全 VIP ユーザー向け NIP-04/44 暗号化ステージのリレー一斉配信
  - 管理者専用ゴッドモード（無敵判定、落下死無効、常時ムーンジャンプ、無限ファイアボール）
- **有料プラン（Subscription Tiers）**:
  - 🆓 **Free Player**: $0 (World 1-1, 1-2)
  - 💎 **Pro Nostr Gamer**: $4.99/月 (World 1-3, ランキング登録, ムーンジャンプ)
  - ⭐ **Legendary 1-Up VIP**: $9.99/月 (World 1-4 クッパ城, NIP-04/44暗号解読, 無敵ゴッドモード, 無限ファイア)

---

### 5. ☁️ Cloudflare Pages ＆ GitHub Actions CI/CD
GitHub Actions ワークフロー (`.github/workflows/deploy.yml`) により、リポジトリへのプッシュ時に自動でテスト・Trunk WASM ビルドが行われ、Cloudflare Pages へエッジデプロイされます。

```yaml
- name: Publish to Cloudflare Pages
  uses: cloudflare/wrangler-action@v3
  with:
    apiToken: ${{ secrets.CLOUDFLARE_API_TOKEN }}
    accountId: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
    command: pages deploy dist --project-name=ascii-mario-nostr
```

#### 必要な GitHub Secrets 設定:
- `CLOUDFLARE_API_TOKEN` : Cloudflare API トークン (Cloudflare Pages 編集権限)
- `CLOUDFLARE_ACCOUNT_ID` : Cloudflare アカウント ID

---

## 🎮 操作方法 (Controls)

| キー | アクション |
| :--- | :--- |
| `←` / `A` | 左移動 (Move Left) |
| `→` / `D` | 右移動 (Move Right) |
| `↑` / `W` / `Z` / `Space` | ジャンプ (Jump - 長押しで高跳び) |
| `Shift` / `X` | ダッシュ / ファイアボール発射 (Run / Shoot Fireball) |
| `R` | ステージリトライ (Retry) |

---

## 📁 ディレクトリ構造

```
last-game/
├── Cargo.toml                     # Leptos, Nostr-SDK, Secp256k1, ChaCha20, AES 依存定義
├── Trunk.toml                     # Leptos WASM Trunk ビルド設定
├── wrangler.toml                  # Cloudflare Pages 設定
├── .github/
│   └── workflows/
│       ├── deploy.yml             # Cloudflare Pages CI/CD (CLOUDFLARE_API_TOKEN / ACCOUNT_ID)
│       └── test.yml               # Rust テスト & フォーマット CI
├── public/
│   ├── index.html                 # Web クライアント シェル
│   ├── style.css                  # レトロ CRT・ネオン・80年代アーケードスタイル
│   ├── app.js                     # 60 FPS インタラクティブゲーム & オーディオシンセ
│   ├── _headers                   # Cloudflare Pages セキュリティヘッダー
│   └── _routes.json               # Cloudflare Pages ルーティング
├── src/
│   ├── lib.rs                     # ライブラリルート
│   ├── main.rs                    # Leptos アプリケーションマウント
│   ├── app.rs                     # Leptos ルートコンポーネント & リアクティブシグナル
│   ├── components/                # Leptos UI コンポーネント群
│   │   ├── navbar.rs              # レトロアーケードヘッダー & Nostr 認証バッジ
│   │   ├── footer.rs              # リレーステータス & フッター
│   │   ├── mario_game.rs          # ASCII マリオターミナル & チートバー
│   │   ├── games_page.rs          # ゲームライブラリ一覧
│   │   ├── about_page.rs          # アーキテクチャ解説
│   │   ├── admin_page.rs          # 管理者特権 & ホワイトリスト管理
│   │   ├── subscription_modal.rs  # 有料サブスク & Stripe 決済シミュレーション
│   │   ├── nostr_login_modal.rs   # Nostr ログイン / ゲストキー生成
│   │   ├── nip_crypto_demo.rs     # NIP-04/44 暗号化・復号リアルタイム実験室
│   │   └── leaderboard.rs         # Nostr Kind 30078 ランキング
│   ├── game/                      # 100% Rust マリオゲームエンジン
│   │   ├── engine.rs              # 60 FPS 物理演算・衝突判定・ループ
│   │   ├── entities.rs            # マリオ、敵、アイテム、タイル定義
│   │   ├── level.rs               # World 1-1 〜 1-4 ASCII マップ定義
│   │   ├── renderer.rs            # ASCII フレームバッファ & HUD レンダラー
│   │   ├── audio.rs               # 8-Bit Web Audio シンセサイザー
│   │   └── cheat.rs               # VIP / 管理者チート状態
│   ├── nostr/                     # Nostr プロトコルモジュール
│   │   ├── crypto.rs              # NIP-04 (AES-CBC) & NIP-44 v2 (ChaCha20-Poly1305) 実装
│   │   ├── whitelist.rs           # 公開鍵ホワイトリストマネージャー
│   │   ├── events.rs              # Kind 1 / 4 / 1059 / 30078 イベントビルダー
│   │   ├── client.rs              # 分散リレー通信クライアント
│   │   └── nip07.rs               # NIP-07 ブラウザ拡張インターフェース
│   └── stripe/                    # Stripe 決済連動
│       └── webhook.rs             # checkout.session.completed -> npub ホワイトリスト処理
├── tests/                         # Rust ユニットテスト
│   ├── game_tests.rs              # 物理・ステージ読み込みテスト
│   ├── nostr_crypto_tests.rs      # NIP-04 / NIP-44 暗号化・復号テスト
│   └── whitelist_tests.rs         # ホワイトリスト & Stripe 処理テスト
└── server/
    └── preview_server.js          # プレビューサーバー (0.0.0.0:3000)
```

---

## 🛠️ ビルド & 実行方法 (Local Development)

### 1. Rust ユニットテストの実行
```bash
cargo test
```

### 2. Trunk による Leptos WASM ビルド
```bash
trunk build --release
```

### 3. ローカルプレビューサーバーの起動
```bash
node server/preview_server.js
# または
npm start
```
ブラウザで `http://localhost:3000` を開くと、インタラクティブな ASCII マリオゲームと全機能が利用可能です。

---

## 📄 ライセンス (License)
MIT OR Apache-2.0
