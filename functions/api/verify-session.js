/**
 * POST /api/verify-session
 * Stripe session_id を検証し、{paid, pubkey_hex} を返す。
 * 成功なら Nostr リレーに有料会員リストを publish する想定 (ここではログのみ)。
 */

export async function onRequestPost(context) {
  const { request, env } = context;
  try {
    const { session_id } = await request.json();
    if (!session_id) return json({ error: "session_id required" }, 400);

    // モック
    if (session_id.startsWith("cs_mock_") || env.MOCK_STRIPE === "true" || !env.STRIPE_SECRET_KEY) {
      // モックでは常に paid=true
      return json({ paid: true, pubkey_hex: null, message: "mock verified" });
    }

    // 本番: Stripe からセッションを取得
    const res = await fetch(`https://api.stripe.com/v1/checkout/sessions/${encodeURIComponent(session_id)}`, {
      headers: { Authorization: `Bearer ${env.STRIPE_SECRET_KEY}` },
    });
    const data = await res.json();
    if (!res.ok) return json({ error: data.error?.message || "Stripe fetch error" }, 400);

    const paid = data.payment_status === "paid" || data.status === "complete";
    const pubkey_hex = data.client_reference_id || data.metadata?.pubkey_hex || null;

    if (paid && pubkey_hex) {
      // ここで Nostr リレーに kind:30078 で有料リストを publish する
      // (Workers から WSS で nostr relay に publish するコードを書けるが、ここではログ)
      console.log(`[verify] paid pubkey=${pubkey_hex} session=${session_id}`);
      // TODO: publishPaidList(pubkey_hex, env.NOSTR_RELAYS)
    }

    return json({ paid, pubkey_hex, message: paid ? "verified" : "not paid" });
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
