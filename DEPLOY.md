# Cloudflare Pages デプロイ手順 (日本語)

このリポジトリは `trunk build --release` で生成される `dist/` を Cloudflare Pages にデプロイします。
GitHub Actions が `CLOUDFLARE_API_TOKEN` / `CLOUDFLARE_ACCOUNT_ID` を使って自動デプロイします。

## 1. Cloudflare 側の準備

1. https://dash.cloudflare.com → Account Home → 右上のアカウントIDを控える → `CLOUDFLARE_ACCOUNT_ID`
2. My Profile → API Tokens → Create Token → **Edit Cloudflare Workers** テンプレート
   - Permissions: `Account / Cloudflare Pages / Edit`, `Zone / Zone / Read` (Pages のみなら Account のみで可)
   - Account Resources: 該当アカウント
   - Zone Resources: All zones
   - 生成 → `CLOUDFLARE_API_TOKEN`
3. Workers & Pages → Create application → **Pages** → Connect to Git でも良いが、今回は GitHub Actions から deploy するため `last-game` プロジェクトを手動作成
   ```bash
   npm i -g wrangler
   wrangler pages project create last-game --production-branch main
   ```

## 2. GitHub Secrets に登録

GitHub リポジトリ → Settings → Secrets and variables → Actions → New repository secret

| Secret | 値 |
|--------|----|
| `CLOUDFLARE_API_TOKEN` | 先ほどのトークン |
| `CLOUDFLARE_ACCOUNT_ID` | アカウントID |
| `ADMIN_PUBKEYS` | 管理者の hex pubkey をカンマ区切り (任意) |
| `NOSTR_RELAYS` | `wss://relay.damus.io,wss://nos.lol,...` (任意) |
| `STRIPE_SECRET_KEY` | `sk_live_...` (本番のみ) |
| `STRIPE_WEBHOOK_SECRET` | `whsec_...` |
| `STRIPE_PUBLISHABLE_KEY` | `pk_live_...` |

## 3. 初回デプロイ

```bash
# ローカルで確認
trunk build --release
npx wrangler pages deploy dist --project-name=last-game

# GitHub に push すれば Actions が自動で同じことを実行
git push origin arena/4b9050fc-last-game   # このセッションのブランチ
# または main にマージ
```

`main` への push / `arena/**` への push / PR で発火します (`.github/workflows/deploy.yml` 参照)。

## 4. 環境変数 (Pages の Variables)

Cloudflare Dashboard → Pages → last-game → Settings → Environment variables

- `NOSTR_RELAYS`
- `PAID_LIST_KIND=30078`
- `MOCK_STRIPE` (開発は `true`, 本番は `false`)
- `STRIPE_SECRET_KEY` / `STRIPE_WEBHOOK_SECRET` は **Encrypt** にチェック

Functions (`functions/api/*.js`) は `env` からこれらを読み取ります。

## 5. Stripe Webhook

Stripe Dashboard → Developers → Webhooks → Add endpoint

- URL: `https://last-game.pages.dev/api/webhook`
- Events: `checkout.session.completed`
- 生成された `whsec_...` を Cloudflare Secrets と GitHub Secrets 両方に登録

ローカルテスト:

```bash
stripe listen --forward-to localhost:8788/api/webhook
stripe trigger checkout.session.completed
```

## 6. トラブルシューティング

- **Pages が 404 を返す**: `public/_redirects` (`/* /index.html 200`) が `dist/` にコピーされているか確認。Trunk の pre_build hook がコピーします。
- **WASM が 404**: `Trunk.toml` の `data-wasm-opt="z"` を確認。`dist` に `.wasm` が含まれるか `ls dist` で確認。
- **Nostr で復号できない**: ブラウザの localStorage `nostr_pubkey_hex` と、リレーの `p` タグが一致しているか確認。`src/nostr/crypto.rs` の `encrypt_nip44_demo` を本番の `nostr::nips::nip44::encrypt` に切り替え。
