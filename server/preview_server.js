const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = process.env.PORT || 3000;
const PUBLIC_DIR = path.join(__dirname, '..', 'public');

const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.js': 'application/javascript; charset=utf-8',
  '.wasm': 'application/wasm',
  '.json': 'application/json; charset=utf-8',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.ico': 'image/x-icon',
  '.svg': 'image/svg+xml',
};

const server = http.createServer((req, res) => {
  // Allow all origins for preview proxy
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');

  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  // Parse URL
  let parsedUrl = req.url.split('?')[0];
  if (parsedUrl === '/') {
    parsedUrl = '/index.html';
  }

  // Stripe webhook simulation endpoint
  if (parsedUrl === '/api/stripe/webhook' && req.method === 'POST') {
    let body = '';
    req.on('data', chunk => { body += chunk; });
    req.on('end', () => {
      try {
        const payload = JSON.parse(body || '{}');
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({
          status: 'success',
          whitelisted_npub: payload.metadata?.nostr_npub || 'npub1...',
          tier: 'legendary_vip',
          nip04_encrypted_event_id: 'event_04_' + Date.now(),
          nip44_encrypted_event_id: 'event_44_' + Date.now(),
        }));
      } catch (e) {
        res.writeHead(400, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: 'Invalid JSON' }));
      }
    });
    return;
  }

  // Whitelist query endpoint
  if (parsedUrl === '/api/nostr/whitelist') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({
      admin_pubkey: '3bf0c63fcb93463407af97b5e0918838bce40f29c164a521551232443e3aa116',
      subscribers: [
        { npub: 'npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64qv6spqqq28cfm', tier: 'AdminGod' },
        { npub: 'npub10pensourcerocks777nostrgamer999legendaryvipclubmember001', tier: 'LegendaryVip' },
        { npub: 'npub1speedrunner2026rustnostrgamerpro420marioarcadechampion', tier: 'ProGamer' },
      ]
    }));
    return;
  }

  // Static File Serving
  let filePath = path.join(PUBLIC_DIR, parsedUrl);
  if (!fs.existsSync(filePath) || fs.statSync(filePath).isDirectory()) {
    filePath = path.join(PUBLIC_DIR, 'index.html');
  }

  const ext = path.extname(filePath).toLowerCase();
  const contentType = MIME_TYPES[ext] || 'application/octet-stream';

  fs.readFile(filePath, (err, content) => {
    if (err) {
      res.writeHead(404, { 'Content-Type': 'text/plain' });
      res.end('404 Not Found');
      return;
    }
    res.writeHead(200, { 'Content-Type': contentType });
    res.end(content);
  });
});

server.listen(PORT, '0.0.0.0', () => {
  console.log(`[RUST NOSTR MARIO ARCADE] Preview Server running at http://0.0.0.0:${PORT}`);
});
