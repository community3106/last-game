/**
 * Cloudflare Pages Function — POST /api/create-checkout-session
 * Stripe Checkout セッションを作成し、URLを返す。
 * 環境変数: STRIPE_SECRET_KEY, STRIPE_PRICE_PREMIUM, STRIPE_PRICE_GOD
 * モック: MOCK_STRIPE=true なら Stripe を呼ばずにダミーURLを返す。
 */

export async function onRequestPost(context) {
  const { request, env } = context;
  try {
    const body = await request.json();
    const { price_id, npub, pubkey_hex, success_url, cancel_url } = body;

    if (!pubkey_hex || !npub) {
      return json({ error: "npub / pubkey_hex required" }, 400);
    }

    // モックモード
    if (env.MOCK_STRIPE === "true" || !env.STRIPE_SECRET_KEY) {
      return json({
        url: `${success_url || "/pricing"}?mock_success=1&session_id=cs_mock_${pubkey_hex.slice(0,8)}`,
        session_id: `cs_mock_${pubkey_hex.slice(0,8)}_${Date.now()}`,
      });
    }

    // 本番: Stripe API を fetch で呼ぶ (Cloudflare Workers は Node Stripe SDK の代わりに fetch)
    const price = price_id || env.STRIPE_PRICE_PREMIUM;
    const params = new URLSearchParams({
      "payment_method_types[]": "card",
      mode: "subscription",
      success_url: success_url || `${new URL(request.url).origin}/pricing?session_id={CHECKOUT_SESSION_ID}`,
      cancel_url: cancel_url || `${new URL(request.url).origin}/pricing`,
      "line_items[0][price]": price,
      "line_items[0][quantity]": "1",
      "metadata[npub]": npub,
      "metadata[pubkey_hex]": pubkey_hex,
      "client_reference_id": pubkey_hex,
    });

    const stripeRes = await fetch("https://api.stripe.com/v1/checkout/sessions", {
      method: "POST",
      headers: {
        Authorization: `Bearer ${env.STRIPE_SECRET_KEY}`,
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: params.toString(),
    });

    const data = await stripeRes.json();
    if (!stripeRes.ok) {
      return json({ error: data.error?.message || "Stripe error", raw: data }, 400);
    }
    return json({ url: data.url, session_id: data.id });
  } catch (e) {
    return json({ error: e.message || String(e) }, 500);
  }
}

function json(obj, status = 200) {
  return new Response(JSON.stringify(obj), {
    status,
    headers: { "content-type": "application/json", "access-control-allow-origin": "*" },
  });
}

export async function onRequestOptions() {
  return new Response(null, {
    headers: {
      "access-control-allow-origin": "*",
      "access-control-allow-methods": "POST, OPTIONS",
      "access-control-allow-headers": "content-type",
    },
  });
}
