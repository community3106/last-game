//! ASCII レンダリング — GameState → String ( <pre> に流す )

use crate::game::engine::{GameState, VIEW_H, VIEW_W};

pub fn render_ascii(state: &GameState) -> String {
    let mut out = String::with_capacity((VIEW_W + 1) * (VIEW_H + 4) * 2);

    // ヘッダ
    let lives_str = "♥".repeat(state.lives.max(0) as usize) + &"♡".repeat((3 - state.lives).max(0) as usize);
    let header = format!(
        " SCORE {:05}  COINS ○x{:02}  {}  {}  {} ",
        state.score,
        state.coins,
        state.level.name,
        lives_str,
        if state.won { "★GOAL★" } else if state.game_over { "GAME OVER" } else { "" }
    );
    out.push_str(&format!("┌{}┐\n", "─".repeat(VIEW_W)));
    // header をセンタリング気味に
    let pad = (VIEW_W as i32 - header.len() as i32).max(0) as usize / 2;
    out.push_str(&format!("│{}{}{}│\n", " ".repeat(pad), header, " ".repeat(VIEW_W - pad - header.len().min(VIEW_W))));

    // ビュー: カメラ位置から VIEW_W x VIEW_H を切り出し
    let cam = state.cam_x as i32;
    for vy in 0..VIEW_H as i32 {
        out.push('│');
        for vx in 0..VIEW_W as i32 {
            let tx = cam + vx;
            let ty = vy;

            // プレイヤー (1x1 → 丸め)
            let px = state.player_x as i32;
            let py = state.player_y as i32;
            // プレイヤーは2セル高で見せる (頭と体)
            let is_player = (px == tx && py == ty) || (px == tx && py + 1 == ty);

            // コイン
            let mut coin_here = false;
            for c in &state.coins_entities {
                if !c.taken && c.x == tx && c.y == ty {
                    coin_here = true;
                    break;
                }
            }
            // 敵
            let mut enemy_here = false;
            for e in &state.enemies {
                if e.alive && (e.x as i32) == tx && (e.y as i32) == ty {
                    enemy_here = true;
                    break;
                }
            }

            let ch = if is_player {
                // ジャンプ中は ^、着地は M
                if !state.on_ground { 'M' } else { 'M' }
            } else if enemy_here {
                'G' // Goomba
            } else if coin_here {
                '○'
            } else {
                match state.level.tile_at(tx, ty) {
                    1 => '═', // 地面
                    2 => '■', // ブロック
                    3 => '█', // 土管
                    9 => '⚑', // ゴール旗
                    _ => ' ',
                }
            };

            // 地面は下2行を強調
            let final_ch = if state.level.tile_at(tx, ty) == 1 && ty >= state.level.height as i32 - 2 {
                '▓'
            } else { ch };

            out.push(final_ch);
        }
        out.push_str("│\n");
    }
    out.push_str(&format!("└{}┘", "─".repeat(VIEW_W)));

    // デバッグ行
    // out.push_str(&format!("\n cam {} px {:.1},{:.1} vel {:.1},{:.1}", cam, state.player_x, state.player_y, state.vel_x, state.vel_y));

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::engine::{make_level_1, GameState, InputState};

    #[test]
    fn render_does_not_panic() {
        let l = make_level_1();
        let s = GameState::new(l);
        let out = render_ascii(&s);
        assert!(out.contains("SCORE"));
        assert!(out.contains('M'));
    }

    #[test]
    fn tick_moves_player() {
        let l = make_level_1();
        let mut s = GameState::new(l);
        let x0 = s.player_x;
        s.tick(InputState { right: true, ..Default::default() });
        assert!(s.player_x >= x0);
    }
}
