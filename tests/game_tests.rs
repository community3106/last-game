use last_game::game::engine::{GameEngine, GameStatus, InputState};
use last_game::game::entities::{ItemType, MarioForm};
use last_game::game::level::LevelData;

#[test]
fn test_mario_engine_initialization() {
    let engine = GameEngine::new(1, 1);
    assert_eq!(engine.player.world, 1);
    assert_eq!(engine.player.level, 1);
    assert_eq!(engine.player.lives, 3);
    assert_eq!(engine.player.form, MarioForm::Small);
    assert_eq!(engine.status, GameStatus::Playing);
}

#[test]
fn test_mario_physics_tick_and_movement() {
    let mut engine = GameEngine::new(1, 1);
    let mut input = InputState::default();
    input.right = true;

    for _ in 0..10 {
        engine.tick(&input);
    }

    assert!(engine.player.x > 3.0, "Mario should move right when right input is pressed");
}

#[test]
fn test_level_data_load_world_1_1() {
    let level = LevelData::world_1_1();
    assert_eq!(level.world, 1);
    assert_eq!(level.level, 1);
    assert!(level.width >= 100);
    assert!(!level.is_vip_locked);
    assert!(!level.enemies.is_empty());
}

#[test]
fn test_vip_level_locked_status() {
    let level_1_4 = LevelData::world_1_4();
    assert_eq!(level_1_4.world, 1);
    assert_eq!(level_1_4.level, 4);
    assert!(level_1_4.is_vip_locked, "World 1-4 Bowser Castle must be VIP locked");
}
