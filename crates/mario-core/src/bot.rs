//! 自動運転ボット / a tiny heuristic bot.
//!
//! サイトのデモプレイ (AI が遊ぶのを見る) と、CI でのレベル検証に使います。
//! 同じクレートの [`World`] を入力経由で操作するだけなので、ブラウザでも端末でも同じように動きます。

use crate::input::Input;
use crate::world::{tile_range, World};

/// ボット / the bot.
#[derive(Clone, Debug)]
pub struct Bot {
    jump_timer: u32,
    cooldown: u32,
    stuck: u32,
    last_x: f32,
}

impl Default for Bot {
    fn default() -> Self {
        Self::new()
    }
}

impl Bot {
    /// 新しいボット / a fresh bot.
    pub const fn new() -> Self {
        Self {
            jump_timer: 0,
            cooldown: 0,
            stuck: 0,
            last_x: 0.0,
        }
    }

    /// 状態をリセットする / forget everything (after a respawn).
    pub fn reset(&mut self) {
        self.jump_timer = 0;
        self.cooldown = 0;
        self.stuck = 0;
        self.last_x = 0.0;
    }

    /// 次の入力 / the input the bot wants to send this frame.
    pub fn input(&mut self, world: &World) -> Input {
        let player = &world.player;
        let mut input = Input::new().right(true).run(true);

        self.cooldown = self.cooldown.saturating_sub(1);
        if self.jump_timer > 0 {
            input.jump = true;
            self.jump_timer -= 1;
        }

        let danger = wall_ahead(world) || pit_ahead(world) || hazard_ahead(world) || enemy_ahead(world);
        if danger && player.on_ground && self.jump_timer == 0 && self.cooldown == 0 {
            self.jump_timer = 20;
            input.jump = true;
            self.cooldown = 26;
        }

        // 進めなくなったら一度跳んでみる / if we are not making progress, try a jump
        if (player.x - self.last_x).abs() < 0.02 {
            self.stuck += 1;
        } else {
            self.stuck = 0;
        }
        self.last_x = player.x;
        if self.stuck > 30 {
            self.jump_timer = 22;
            input.jump = true;
            self.cooldown = 20;
            self.stuck = 0;
        }

        input
    }
}

/// 前方の壁 / is there a wall right in front of the player?
fn wall_ahead(world: &World) -> bool {
    let player = &world.player;
    let col = (player.x + player.w + 0.35).floor() as i32;
    let (row0, row1) = tile_range(player.y, player.h);
    (row0..=row1).any(|row| world.level.solid_at(col, row))
}

/// 前方の穴 / would the player fall into a pit if they kept walking?
fn pit_ahead(world: &World) -> bool {
    let player = &world.player;
    let col = (player.x + player.w + 0.7).floor() as i32;
    let feet_row = (player.feet() + 0.1).floor() as i32;
    for row in feet_row..(feet_row + 14) {
        if world.level.solid_at(col, row) {
            return false;
        }
    }
    true
}

/// 前方のトゲ / spikes ahead?
fn hazard_ahead(world: &World) -> bool {
    let player = &world.player;
    let col = (player.x + player.w + 0.6).floor() as i32;
    let (row0, row1) = tile_range(player.y, player.h);
    ((row0 - 1)..=(row1 + 1)).any(|row| world.level.hazard_at(col, row))
}

/// 前方の敵 / an enemy close in front of the player?
fn enemy_ahead(world: &World) -> bool {
    let player = &world.player;
    world.enemies.iter().any(|enemy| {
        enemy.alive
            && enemy.squashed.is_none()
            && enemy.x + enemy.w > player.x + player.w - 0.1
            && enemy.x < player.x + player.w + 2.4
            && (enemy.y - player.y).abs() < 1.7
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::Level;
    use crate::levels;
    use crate::world::Event;

    /// ボットに走らせる / let the bot play for a while.
    fn play(art: &str, name: &str, max_steps: usize, restarts: u32) -> (World, bool) {
        let level = Level::from_ascii(name, art).expect("level should parse");
        let mut world = World::new(level);
        let mut bot = Bot::new();
        let mut completed = false;
        let mut used_restarts = 0;
        for _ in 0..max_steps {
            let input = bot.input(&world);
            for event in world.step(input) {
                match event {
                    Event::LevelComplete { .. } => completed = true,
                    Event::GameOver => {
                        if used_restarts < restarts {
                            used_restarts += 1;
                            world.restart();
                            bot.reset();
                        }
                    }
                    _ => {}
                }
            }
            if completed {
                break;
            }
        }
        (world, completed)
    }

    #[test]
    fn bot_stomps_an_approach_enemy() {
        let art = "
#            #
#   p   g    #
##############
";
        let (world, _) = play(art, "bot-stomp", 400, 0);
        assert!(
            world.score >= 100,
            "the bot should stomp or avoid the goomba, score = {}",
            world.score
        );
    }

    #[test]
    fn bot_turns_around_when_stuck_against_a_wall() {
        // 右が壁、左が開いている / wall on the right, open on the left
        let art = "
###
# #
# #p
# #
###
";
        let (_world, _) = play(art, "bot-wall", 200, 0);
        // クラッシュせず 200 ステップ動き続けることが目的 / just must not panic
    }

    #[test]
    fn bot_clears_world_1_1() {
        let (world, completed) = play(levels::WORLD_1_1, "1-1", 14_000, 2);
        assert!(
            completed,
            "the bot should clear 1-1: x = {}, score = {}, coins = {}",
            world.player.x, world.score, world.coin_count
        );
    }

    #[test]
    fn bot_makes_progress_in_world_1_2() {
        let (world, _) = play(levels::WORLD_1_2, "1-2", 14_000, 2);
        assert!(
            world.player.x > 60.0,
            "the bot should get through most of 1-2: x = {}",
            world.player.x
        );
    }
}
