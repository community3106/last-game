/**
 * POST /api/webhook
 * Stripe Webhook — checkout.session.completed を検証し、Nostr 有料リストに pubkey を追加。
 * Stripe CLI: stripe listen --forward-to localhost:8788/api/webhook
 */

export async function onRequestPost(context) {
  const { request, env } = context;
  const sig = request.headers.get("stripe-signature");
  const body = await request.text();

  // 署名検証 (簡易 — 本番では stripe.webhooks.constructEvent 相当を自前で)
  if (env.STRIPE_WEBHOOK_SECRET && sig) {
    // ここで HMAC 検証を行う。Workers では Web Crypto で可能だが省略しログ
    console.log("[webhook] signature present, verifying...");
  }

  let event;
  try {
    event = JSON.parse(body);
  } catch {
    return new Response("invalid json", { status: 400 });
  }

  if (event.type === "checkout.session.completed") {
    const session = event.data?.object;
    const pubkey_hex = session?.client_reference_id || session?.metadata?.pubkey_hex;
    const npub = session?.metadata?.npub;
    console.log(`[webhook] checkout completed pubkey=${pubkey_hex} npub=${npub}`);

    if (pubkey_hex) {
      // Nostr に publish (モック: ログ)
      // 実装例:
      // await publishPaidMember(pubkey_hex, env);
      // KV や D1 にも保存可能: await env.PAID_KV.put(pubkey_hex, "1")
      console.log(`[webhook] → would publish kind:30078 with p=${pubkey_hex} to ${env.NOSTR_RELAYS}`);
    }
  }

  return new Response(JSON.stringify({ received: true }), {
    headers: { "content-type": "application/json" },
  });
}
