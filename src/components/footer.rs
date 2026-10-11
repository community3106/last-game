use leptos::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="retro-footer">
            <div class="footer-container">
                <div class="footer-section">
                    <h4>"🕹️ RUST ASCII MARIO 1985"</h4>
                    <p>"Pure 100% Rust WASM Game Engine running directly in your browser. Powered by Leptos reactive framework & Nostr decentralized protocol."</p>
                </div>

                <div class="footer-section">
                    <h4>"📡 NOSTR RELAYS STATUS"</h4>
                    <div class="relay-list">
                        <span class="relay-tag"><span class="dot green"></span> "relay.damus.io"</span>
                        <span class="relay-tag"><span class="dot green"></span> "nos.lol"</span>
                        <span class="relay-tag"><span class="dot green"></span> "relay.primal.net"</span>
                        <span class="relay-tag"><span class="dot green"></span> "nostr.band"</span>
                    </div>
                </div>

                <div class="footer-section">
                    <h4>"☁️ CLOUDFLARE PAGES & CI/CD"</h4>
                    <p>"Automated GitHub Actions deployment with CLOUDFLARE_API_TOKEN & CLOUDFLARE_ACCOUNT_ID. Edge distributed globally."</p>
                </div>
            </div>

            <div class="footer-bottom">
                <span>"© 2026 Rust Nostr Arcade • 100% Open Source Rust • End-to-End Encrypted Gaming"</span>
            </div>
        </footer>
    }
}
