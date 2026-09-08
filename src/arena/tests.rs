use super::*;
use std::time::Duration;

fn test_app() -> (App, Entity) {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins(ArenaPlugin);
    app.update();
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    (app, drone)
}

fn step(app: &mut App, keys: &[KeyCode], seconds: f32) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
    for &key in keys {
        input.press(key);
    }
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(seconds));
    app.update();
}

fn position(app: &App, drone: Entity) -> Vec2 {
    app.world()
        .get::<Transform>(drone)
        .unwrap()
        .translation
        .truncate()
}

fn near(actual: Vec2, expected: Vec2) {
    assert!(
        (actual - expected).length() < 0.002,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn all_directions_and_both_keyboard_layouts_work() {
    for (keys, direction) in [
        (vec![KeyCode::KeyW], Vec2::Y),
        (vec![KeyCode::KeyS], Vec2::NEG_Y),
        (vec![KeyCode::KeyA], Vec2::NEG_X),
        (vec![KeyCode::KeyD], Vec2::X),
        (vec![KeyCode::ArrowUp], Vec2::Y),
        (vec![KeyCode::ArrowDown], Vec2::NEG_Y),
        (vec![KeyCode::ArrowLeft], Vec2::NEG_X),
        (vec![KeyCode::ArrowRight], Vec2::X),
        (vec![KeyCode::KeyW, KeyCode::KeyD], Vec2::new(1., 1.)),
        (vec![KeyCode::KeyW, KeyCode::KeyA], Vec2::new(-1., 1.)),
        (vec![KeyCode::KeyS, KeyCode::KeyD], Vec2::new(1., -1.)),
        (vec![KeyCode::KeyS, KeyCode::KeyA], Vec2::new(-1., -1.)),
    ] {
        let (mut app, drone) = test_app();
        let speed = app.world().resource::<Arena>().drone_speed;
        step(&mut app, &keys, 0.5);
        near(position(&app, drone), direction.normalize() * speed * 0.5);
    }
}

#[test]
fn duplicate_bindings_do_not_skew_diagonal_direction() {
    let (mut app, drone) = test_app();
    let speed = app.world().resource::<Arena>().drone_speed;
    step(
        &mut app,
        &[KeyCode::KeyW, KeyCode::ArrowUp, KeyCode::KeyD],
        0.5,
    );
    near(position(&app, drone), Vec2::ONE.normalize() * speed * 0.5);
}

#[test]
fn released_and_opposing_keys_do_not_move_the_drone() {
    let (mut app, drone) = test_app();
    step(&mut app, &[KeyCode::KeyD], 0.5);
    let before = position(&app, drone);
    step(&mut app, &[], 0.5);
    near(position(&app, drone), before);
    step(
        &mut app,
        &[
            KeyCode::KeyW,
            KeyCode::ArrowDown,
            KeyCode::KeyA,
            KeyCode::ArrowRight,
        ],
        0.5,
    );
    near(position(&app, drone), before);
}

#[test]
fn movement_depends_on_elapsed_time_not_frame_count() {
    let (mut slow, slow_drone) = test_app();
    let (mut fast, fast_drone) = test_app();
    for _ in 0..30 {
        step(&mut slow, &[KeyCode::KeyD], 1. / 30.);
    }
    for _ in 0..120 {
        step(&mut fast, &[KeyCode::KeyD], 1. / 120.);
    }
    near(position(&slow, slow_drone), position(&fast, fast_drone));
    near(
        position(&slow, slow_drone),
        Vec2::X * slow.world().resource::<Arena>().drone_speed,
    );
}

#[test]
fn entire_drone_stays_inside_every_edge_and_corner_after_a_long_frame() {
    for (keys, direction) in [
        (vec![KeyCode::KeyD], Vec2::X),
        (vec![KeyCode::KeyA], Vec2::NEG_X),
        (vec![KeyCode::KeyW], Vec2::Y),
        (vec![KeyCode::KeyS], Vec2::NEG_Y),
        (vec![KeyCode::KeyD, KeyCode::KeyW], Vec2::ONE),
        (vec![KeyCode::KeyA, KeyCode::KeyW], Vec2::new(-1., 1.)),
        (vec![KeyCode::KeyD, KeyCode::KeyS], Vec2::new(1., -1.)),
        (vec![KeyCode::KeyA, KeyCode::KeyS], -Vec2::ONE),
    ] {
        let (mut app, drone) = test_app();
        let arena = *app.world().resource::<Arena>();
        let limit = arena.half_size - Vec2::splat(DRONE_HALF_SIZE);
        step(&mut app, &keys, 10.);
        near(position(&app, drone), direction * limit);
        step(&mut app, &keys, 10.);
        near(position(&app, drone), direction * limit);
    }
}

#[test]
fn reset_restores_starting_transform_and_wins_over_movement_repeatedly() {
    let (mut app, drone) = test_app();
    let start = *app.world().get::<Transform>(drone).unwrap();
    let count = app.world().entities().len();
    for _ in 0..3 {
        step(&mut app, &[KeyCode::KeyD, KeyCode::KeyW], 1.);
        assert_ne!(position(&app, drone), start.translation.truncate());
        step(&mut app, &[KeyCode::KeyR, KeyCode::KeyD], 0.5);
        assert_eq!(*app.world().get::<Transform>(drone).unwrap(), start);
        assert_eq!(app.world().entities().len(), count);
    }
}

#[test]
fn holding_reset_does_not_reset_again_until_repressed() {
    let (mut app, drone) = test_app();
    step(&mut app, &[KeyCode::KeyR, KeyCode::KeyD], 0.5);
    near(position(&app, drone), Vec2::ZERO);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.update();
    assert!(position(&app, drone).x > 0.);
}
