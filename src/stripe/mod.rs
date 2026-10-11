//! Stripe 連携 — Cloudflare Pages Functions と協調
//! フロントは `create_checkout_session` を fetch で呼び、成功後は `verify_session` で pubkey を paid リストに登録。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCheckoutRequest {
    pub price_id: String,      // price_xxx
    pub npub: String,          // 購入者の npub
    pub pubkey_hex: String,    // 購入者の hex
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCheckoutResponse {
    pub url: String,           // Stripe Checkout URL (モックなら /success?mock=1)
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifySessionRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifySessionResponse {
    pub paid: bool,
    pub pubkey_hex: Option<String>,
    pub message: String,
}

/// Checkout セッションを作成 (Cloudflare Function へ POST)
pub async fn create_checkout_session(req: CreateCheckoutRequest) -> Result<CreateCheckoutResponse, String> {
    // 本番: gloo_net::http::Request::post("/api/create-checkout-session").json(&req).unwrap().send().await
    // デモではモックを返す
    if is_mock() {
        // 500ms 待ってモックURLを返す
        gloo_timers::future::TimeoutFuture::new(500).await;
        return Ok(CreateCheckoutResponse {
            url: format!("/pricing?mock_success=1&price={}", req.price_id),
            session_id: format!("cs_mock_{}", &req.pubkey_hex[..8]),
        });
    }
    // 本番 fetch
    let resp = gloo_net::http::Request::post("/api/create-checkout-session")
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("checkout failed: {}", resp.status()));
    }
    resp.json::<CreateCheckoutResponse>().await.map_err(|e| e.to_string())
}

pub async fn verify_session(session_id: &str) -> Result<VerifySessionResponse, String> {
    if is_mock() {
        gloo_timers::future::TimeoutFuture::new(300).await;
        return Ok(VerifySessionResponse {
            paid: true,
            pubkey_hex: None,
            message: "モック: 決済成功として扱います".into(),
        });
    }
    let req = VerifySessionRequest { session_id: session_id.into() };
    let resp = gloo_net::http::Request::post("/api/verify-session")
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<VerifySessionResponse>().await.map_err(|e| e.to_string())
}

fn is_mock() -> bool {
    // 本番では Cloudflare の環境変数で切り替えるが、フロントでは常にモック扱い
    // STRIPE_SECRET_KEY が無い場合はモック
    true
}

// ---- 価格ID ----
pub const PRICE_PREMIUM: &str = "price_premium_980";
pub const PRICE_GOD: &str = "price_god_2980";
