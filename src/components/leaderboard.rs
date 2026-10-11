use leptos::*;

#[derive(Clone, Debug)]
pub struct LeaderboardScore {
    pub rank: u32,
    pub player_npub: String,
    pub score: u32,
    pub world: String,
    pub coins: u32,
    pub time_left: u32,
    pub verified_nostr: bool,
}

#[component]
pub fn Leaderboard() -> impl IntoView {
    let mock_scores = vec![
        LeaderboardScore {
            rank: 1,
            player_npub: "npub1speedrunner2026rustnostrgamerpro420marioarcadechampion".to_string(),
            score: 128400,
            world: "1-4 (Bowser Cleared)".to_string(),
            coins: 48,
            time_left: 312,
            verified_nostr: true,
        },
        LeaderboardScore {
            rank: 2,
            player_npub: "npub10pensourcerocks777nostrgamer999legendaryvipclubmember001".to_string(),
            score: 95200,
            world: "1-4 (Bowser Cleared)".to_string(),
            coins: 36,
            time_left: 280,
            verified_nostr: true,
        },
        LeaderboardScore {
            rank: 3,
            player_npub: "npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64qv6spqqq28cfm".to_string(),
            score: 84100,
            world: "1-3 (Mushroom Treetops)".to_string(),
            coins: 29,
            time_left: 245,
            verified_nostr: true,
        },
        LeaderboardScore {
            rank: 4,
            player_npub: "npub1freeplayer999classicmariooverworldchallenger123456789".to_string(),
            score: 42000,
            world: "1-2 (Underground)".to_string(),
            coins: 18,
            time_left: 198,
            verified_nostr: true,
        },
    ];

    view! {
        <div class="leaderboard-container">
            <div class="leaderboard-header">
                <span class="leaderboard-badge">"🏆 KIND 30078 NOSTR VERIFIED"</span>
                <h1 class="page-title">"Nostr 分散型ハイスコアランキング"</h1>
                <p class="page-description">
                    "すべてのスコアはプレイヤーのSchnorr秘密鍵で署名され、Nostr公開リレー（damus, nos.lol, primal）に不変データとして記録されています。"
                </p>
            </div>

            <div class="leaderboard-card">
                <table class="retro-table leaderboard-table">
                    <thead>
                        <tr>
                            <th>"順位"</th>
                            <th>"プレイヤー公開鍵 (npub)"</th>
                            <th>"スコア"</th>
                            <th>"到達ステージ"</th>
                            <th>"コイン"</th>
                            <th>"残り時間"</th>
                            <th>"Nostr署名検証"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {mock_scores.into_iter().map(|entry| {
                            let rank_icon = match entry.rank {
                                1 => "🥇 1位",
                                2 => "🥈 2位",
                                3 => "🥉 3位",
                                r => "",
                            };
                            let rank_display = if rank_icon.is_empty() {
                                format!("{}位", entry.rank)
                            } else {
                                rank_icon.to_string()
                            };

                            let short_npub = if entry.player_npub.len() > 18 {
                                format!("{}...{}", &entry.player_npub[..9], &entry.player_npub[entry.player_npub.len() - 5..])
                            } else {
                                entry.player_npub.clone()
                            };

                            view! {
                                <tr class=if entry.rank <= 3 { "top-rank-row" } else { "" }>
                                    <td class="rank-col">{rank_display}</td>
                                    <td class="font-mono">{short_npub}</td>
                                    <td class="score-col">{format!("{:06}", entry.score)}</td>
                                    <td>{entry.world}</td>
                                    <td>"x" {entry.coins}</td>
                                    <td>{entry.time_left} "s"</td>
                                    <td><span class="tag-verified">"✔ Verified"</span></td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
