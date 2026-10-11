import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIST = path.join(__dirname, "dist");
const PORT = process.env.PORT || 3000;

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "application/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".ico": "image/x-icon",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".wasm": "application/wasm",
};

const server = http.createServer((req, res) => {
  // CORS & preview host allow
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET,POST,OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "content-type");
  // allow iframe embedding (Cloudflare preview needs it)
  res.setHeader("X-Frame-Options", "ALLOWALL");
  res.setHeader("Content-Security-Policy", "frame-ancestors *");

  if (req.method === "OPTIONS") { res.writeHead(204); return res.end(); }

  let url = new URL(req.url, `http://${req.headers.host}`);
  let pathname = url.pathname;

  // mock Stripe APIs for preview
  if (pathname === "/api/create-checkout-session" && req.method === "POST") {
    let body = "";
    req.on("data", chunk => body += chunk);
    req.on("end", () => {
      try {
        const j = JSON.parse(body || "{}");
        const hex = j.pubkey_hex || "demo";
        res.writeHead(200, { "content-type": "application/json" });
        res.end(JSON.stringify({ url: `/pricing?mock_success=1&session_id=cs_mock_${hex.slice(0,8)}`, session_id: `cs_mock_${hex.slice(0,8)}` }));
      } catch (e) {
        res.writeHead(400, { "content-type": "application/json" });
        res.end(JSON.stringify({ error: String(e) }));
      }
    });
    return;
  }
  if (pathname === "/api/verify-session" && req.method === "POST") {
    res.writeHead(200, { "content-type": "application/json" });
    res.end(JSON.stringify({ paid: true, message: "mock verified" }));
    return;
  }

  // static file serving from dist
  if (pathname === "/") pathname = "/index.html";
  let filePath = path.join(DIST, pathname);

  // SPA fallback: if not found, serve index.html
  if (!fs.existsSync(filePath) || fs.statSync(filePath).isDirectory()) {
    filePath = path.join(DIST, "index.html");
  }

  try {
    const data = fs.readFileSync(filePath);
    const ext = path.extname(filePath);
    res.writeHead(200, { "content-type": MIME[ext] || "application/octet-stream" });
    res.end(data);
  } catch (e) {
    res.writeHead(404, { "content-type": "text/plain" });
    res.end("404 Not Found: " + pathname);
  }
});

server.listen(Number(PORT), "0.0.0.0", () => {
  console.log(`[preview] serving ${DIST} on http://0.0.0.0:${PORT}`);
});
