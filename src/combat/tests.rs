use super::*;
use crate::arena::{ArenaPlugin, Drone};
use std::time::Duration;

const START: Vec3 = Vec3::new(0., 90., 0.);

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins((ArenaPlugin, CombatPlugin));
    app.update();
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    (app, drone)
}

fn empty_app() -> (App, Entity) {
    let (mut app, drone) = app();
    let entities: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, Or<(With<Enemy>, With<Projectile>)>>()
        .iter(app.world())
        .collect();
    for entity in entities {
        app.world_mut().despawn(entity);
    }
    *app.world_mut().resource_mut::<Weapon>() = Weapon::default();
    app.world_mut().resource_mut::<WaveConfig>().bursts.clear();
    // Keep the original collision/damage fixtures independent of scenario balance.
    app.world_mut()
        .resource_mut::<CombatConfig>()
        .contact_damage = 25;
    (app, drone)
}

fn step(app: &mut App, dt: f32, keys: &[KeyCode]) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
    for &key in keys {
        input.press(key);
    }
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(dt));
    app.update();
}

fn enemy(app: &mut App, position: Vec3, health: u32) -> Entity {
    app.world_mut()
        .spawn((
            Enemy {
                health,
                previous: position,
                path: Vec::new(),
            },
            Transform::from_translation(position),
            crate::arena::DroneFlight::default(),
        ))
        .id()
}

fn shot(app: &mut App, position: Vec3, velocity: Vec3, remaining: f32) -> Entity {
    app.world_mut()
        .spawn((
            Projectile {
                velocity,
                remaining,
            },
            Transform::from_translation(position),
        ))
        .id()
}

fn position(app: &App, entity: Entity) -> Vec3 {
    app.world().get::<Transform>(entity).unwrap().translation
}

fn count<T: Component>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .iter(app.world())
        .count()
}

fn quiet(app: &mut App) {
    let mut config = app.world_mut().resource_mut::<CombatConfig>();
    config.enemy_flight.max_horizontal_speed = 0.;
    config.target_range = 0.;
}

#[test]
fn starts_empty_before_first_warning_with_full_health() {
    let (mut app, _) = app();
    assert_eq!(count::<Enemy>(&mut app), 0);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
}

#[test]
fn chaser_accelerates_toward_player_in_three_dimensions() {
    for dt in [1_f32 / 30., 1. / 120.] {
        let (mut app, _) = empty_app();
        app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
        let from = START + Vec3::new(200., 100., 100.);
        let chaser = enemy(&mut app, from, 100);
        for _ in 0..(0.5 / dt).round() as usize {
            step(&mut app, dt, &[]);
        }
        let after = position(&app, chaser);
        assert!(after.distance(START) < from.distance(START));
        assert!(after.x < from.x && after.y < from.y && after.z < from.z);
        assert!(after.distance(from) < 60.);
    }
}

#[test]
fn chaser_reaches_contact_at_every_drone_boundary_without_leaving_arena() {
    for target in [
        Vec3::new(-445., 15., -225.),
        Vec3::new(445., 285., 225.),
        START,
    ] {
        let (mut app, drone) = empty_app();
        app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
        app.world_mut()
            .get_mut::<Transform>(drone)
            .unwrap()
            .translation = target;
        let chaser = enemy(&mut app, Vec3::new(0., 150., 0.), 100);
        step(&mut app, 10., &[]);
        let p = position(&app, chaser);
        assert!(p.cmpge(Vec3::new(-466., 14., -256.)).all());
        assert!(p.cmple(Vec3::new(466., 286., 256.)).all());
        assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
        step(&mut app, 0.1, &[]);
        assert!(position(&app, chaser).distance(p) < 0.01);
    }
}

#[test]
fn nearest_target_uses_altitude_range_and_stable_ties() {
    let (mut app, _) = empty_app();
    app.world_mut()
        .resource_mut::<CombatConfig>()
        .enemy_flight
        .max_horizontal_speed = 0.;
    enemy(&mut app, START + Vec3::Y * 150., 100);
    let first = enemy(&mut app, START + Vec3::X * 100., 100);
    let second = enemy(&mut app, START - Vec3::X * 100., 100);
    step(&mut app, 0., &[]);
    let velocity = app
        .world_mut()
        .query::<&Projectile>()
        .single(app.world())
        .unwrap()
        .velocity;
    let expected = if first.to_bits() < second.to_bits() {
        Vec3::X
    } else {
        Vec3::NEG_X
    };
    assert!(velocity.normalize().distance(expected) < 0.001);
}

#[test]
fn no_target_means_no_shots_or_accumulated_burst() {
    let (mut app, _) = empty_app();
    app.world_mut()
        .resource_mut::<CombatConfig>()
        .enemy_flight
        .max_horizontal_speed = 0.;
    let far = enemy(&mut app, START + Vec3::X * 450., 100);
    step(&mut app, 5., &[]);
    assert_eq!(count::<Projectile>(&mut app), 0);
    app.world_mut()
        .get_mut::<Transform>(far)
        .unwrap()
        .translation = START + Vec3::X * 100.;
    step(&mut app, 0., &[]);
    assert_eq!(count::<Projectile>(&mut app), 1);
    for _ in 0..5 {
        step(&mut app, 0., &[]);
    }
    assert_eq!(count::<Projectile>(&mut app), 1);
}

#[test]
fn projectile_keeps_direction_when_target_moves_and_misses_expire() {
    let (mut app, _) = empty_app();
    app.world_mut()
        .resource_mut::<CombatConfig>()
        .enemy_flight
        .max_horizontal_speed = 0.;
    let target = enemy(&mut app, START + Vec3::X * 100., 100);
    step(&mut app, 0., &[]);
    let id = app
        .world_mut()
        .query_filtered::<Entity, With<Projectile>>()
        .single(app.world())
        .unwrap();
    let velocity = app.world().get::<Projectile>(id).unwrap().velocity;
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation = START + Vec3::Y * 100.;
    step(&mut app, 0.1, &[]);
    assert_eq!(
        app.world().get::<Projectile>(id).unwrap().velocity,
        velocity
    );
    assert!(position(&app, id).distance(START + velocity * 0.1) < 0.01);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().health, 100);
    quiet(&mut app);
    step(&mut app, 1., &[]);
    assert!(app.world().get_entity(id).is_err());
}

#[test]
fn swept_shot_hits_first_crossed_enemy_only_even_on_a_long_frame() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    let behind = enemy(&mut app, START + Vec3::X * 140., 100);
    let first = enemy(&mut app, START + Vec3::X * 80., 100);
    let projectile = shot(&mut app, START, Vec3::X * 650., 1.);
    step(&mut app, 0.5, &[]);
    assert_eq!(app.world().get::<Enemy>(first).unwrap().health, 90);
    assert_eq!(app.world().get::<Enemy>(behind).unwrap().health, 100);
    assert!(app.world().get_entity(projectile).is_err());
}

#[test]
fn projectile_sweep_accounts_for_enemy_crossing_between_frames() {
    let (mut app, _) = empty_app();
    app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
    let target = enemy(&mut app, START + Vec3::Z * 100., 100);
    shot(
        &mut app,
        START + Vec3::new(-100., 0., 95.),
        Vec3::X * 400.,
        1.,
    );
    step(&mut app, 0.5, &[]);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().health, 90);
}

#[test]
fn expired_projectiles_cannot_hit_beyond_their_lifetime_and_outside_shots_are_removed() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    let target = enemy(&mut app, START + Vec3::X * 200., 100);
    let expired = shot(&mut app, START, Vec3::X * 650., 0.1);
    let outside = shot(&mut app, Vec3::new(470., 90., 0.), Vec3::X * 650., 1.);
    step(&mut app, 0.5, &[]);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().health, 100);
    assert!(app.world().get_entity(expired).is_err());
    assert!(app.world().get_entity(outside).is_err());
}

#[test]
fn lethal_projectile_removes_enemy_before_contact_damage() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    let target = enemy(&mut app, START, 10);
    shot(&mut app, START, Vec3::X * 650., 1.);
    step(&mut app, 0.01, &[]);
    assert!(app.world().get_entity(target).is_err());
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
}

#[test]
fn simultaneous_and_sustained_contact_share_one_invulnerability_window() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    for _ in 0..3 {
        enemy(&mut app, START, 100);
    }
    step(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
    step(&mut app, 0.74, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
    step(&mut app, 0.02, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 50);
}

#[test]
fn separating_prevents_damage_after_invulnerability_expires() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    let target = enemy(&mut app, START, 100);
    step(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation += Vec3::X * 100.;
    step(&mut app, 1., &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
}

#[test]
fn death_clamps_health_and_freezes_all_gameplay_until_restart() {
    let (mut app, drone) = empty_app();
    quiet(&mut app);
    app.world_mut().resource_mut::<PlayerHealth>().current = 10;
    enemy(&mut app, START, 100);
    step(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Dead);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 0);
    let target = enemy(&mut app, START + Vec3::X * 200., 100);
    let projectile = shot(&mut app, START + Vec3::Y * 80., Vec3::X * 650., 1.);
    let before = position(&app, target);
    app.world_mut()
        .resource_mut::<CombatConfig>()
        .enemy_flight
        .max_horizontal_speed = 150.;
    app.world_mut().resource_mut::<CombatConfig>().target_range = 400.;
    step(&mut app, 2., &[KeyCode::KeyD, KeyCode::Space]);
    assert_eq!(position(&app, drone), START);
    assert_eq!(position(&app, target), before);
    assert_eq!(position(&app, projectile), START + Vec3::Y * 80.);
    assert_eq!(
        app.world().get::<Projectile>(projectile).unwrap().remaining,
        1.
    );
    assert_eq!(app.world().resource::<PlayerHealth>().current, 0);
    assert_eq!(count::<Projectile>(&mut app), 1);
}

#[test]
fn repeated_reset_restores_encounter_and_wins_over_movement_and_combat() {
    let (mut app, drone) = app();
    for phase in [GamePhase::Playing, GamePhase::Dead, GamePhase::Playing] {
        *app.world_mut().resource_mut::<GamePhase>() = phase;
        app.world_mut().resource_mut::<PlayerHealth>().current = 1;
        app.world_mut()
            .resource_mut::<PlayerHealth>()
            .invulnerable_until = 999.;
        app.world_mut()
            .get_mut::<Transform>(drone)
            .unwrap()
            .translation = Vec3::new(100., 200., 50.);
        let stale_enemy = enemy(&mut app, START, 100);
        let stale_shot = shot(&mut app, START, Vec3::X * 650., 1.);
        step(
            &mut app,
            1.,
            &[KeyCode::KeyR, KeyCode::KeyD, KeyCode::Space],
        );
        assert_eq!(position(&app, drone), START);
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
        assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
        assert_eq!(
            app.world().resource::<PlayerHealth>().invulnerable_until,
            0.
        );
        assert_eq!(count::<Enemy>(&mut app), 0);
        assert_eq!(count::<Projectile>(&mut app), 0);
        assert_eq!(count::<Drone>(&mut app), 1);
        assert!(app.world().get_entity(stale_enemy).is_err());
        assert!(app.world().get_entity(stale_shot).is_err());
        step(&mut app, 0., &[]);
        assert_eq!(count::<Projectile>(&mut app), 0);
    }
}

#[test]
fn clearing_the_encounter_keeps_movement_available_and_stops_firing() {
    let (mut app, drone) = empty_app();
    quiet(&mut app);
    let target = enemy(&mut app, START + Vec3::X * 80., 10);
    shot(&mut app, START, Vec3::X * 650., 1.);
    step(&mut app, 0.2, &[]);
    assert!(app.world().get_entity(target).is_err());
    assert_eq!(count::<Enemy>(&mut app), 0);
    step(&mut app, 0.25, &[KeyCode::Space]);
    assert!(position(&app, drone).y > START.y);
    assert_eq!(count::<Projectile>(&mut app), 0);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
}

#[test]
fn firing_cadence_and_retargeting_are_consistent_across_frame_rates() {
    for rate in [30, 120] {
        let (mut app, _) = empty_app();
        app.world_mut()
            .resource_mut::<CombatConfig>()
            .enemy_flight
            .max_horizontal_speed = 0.;
        let near = enemy(&mut app, START + Vec3::X * 100., 10);
        let far = enemy(&mut app, START + Vec3::Y * 150., 100);
        for _ in 0..rate * 2 {
            step(&mut app, 1. / rate as f32, &[]);
        }
        assert!(app.world().get_entity(near).is_err());
        assert_eq!(app.world().get::<Enemy>(far).unwrap().health, 70);
    }
}

#[test]
fn presentation_reuses_assets_and_hud_across_death_and_repeated_restart() {
    use super::scene::{CombatHud, CombatScenePlugin};
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .add_plugins((ArenaPlugin, CombatPlugin, CombatScenePlugin));
    app.update();
    assert_eq!(count::<CombatHud>(&mut app), 1);
    assert_eq!(count::<Mesh3d>(&mut app), 0); // Quiet opening.
    let meshes = app.world().resource::<Assets<Mesh>>().len();
    let materials = app.world().resource::<Assets<StandardMaterial>>().len();
    let entity_count = app.world().entities().count_spawned();
    for _ in 0..3 {
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
        app.world_mut().resource_mut::<PlayerHealth>().current = 0;
        step(&mut app, 0., &[]);
        let text = app
            .world_mut()
            .query_filtered::<&Text, With<CombatHud>>()
            .single(app.world())
            .unwrap();
        assert!(text.0.contains("DESTROYED"));
        assert!(text.0.contains("R"));
        step(&mut app, 0., &[KeyCode::KeyR]);
        step(&mut app, 0., &[]);
        assert_eq!(count::<CombatHud>(&mut app), 1);
        assert_eq!(count::<Mesh3d>(&mut app), 0);
        assert_eq!(app.world().entities().count_spawned(), entity_count);
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), meshes);
        assert_eq!(
            app.world().resource::<Assets<StandardMaterial>>().len(),
            materials
        );
        let text = app
            .world_mut()
            .query_filtered::<&Text, With<CombatHud>>()
            .single(app.world())
            .unwrap();
        assert!(text.0.contains("100 / 100"));
        assert!(!text.0.contains("DESTROYED"));
    }
}

#[test]
fn briefly_losing_target_does_not_cancel_the_current_shot_cooldown() {
    let (mut app, _) = empty_app();
    app.world_mut()
        .resource_mut::<CombatConfig>()
        .enemy_flight
        .max_horizontal_speed = 0.;
    let target = enemy(&mut app, START + Vec3::X * 100., 100);
    step(&mut app, 0., &[]);
    assert_eq!(count::<Projectile>(&mut app), 1);
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation = START + Vec3::X * 450.;
    step(&mut app, 0.1, &[]);
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation = START + Vec3::X * 100.;
    step(&mut app, 0., &[]);
    assert_eq!(count::<Projectile>(&mut app), 1);
    step(&mut app, 0.4, &[]);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().health, 90);
    assert_eq!(count::<Projectile>(&mut app), 1);
}

#[test]
fn yaw_and_tilt_change_player_contact_bounds_on_every_axis() {
    use crate::arena::{DroneFlight, drone_world_half_extents};
    for (heading, tilt) in [
        (std::f32::consts::FRAC_PI_2, Vec2::ZERO),
        (0.7, Vec2::new(0.2, 0.3)),
    ] {
        let rotation = Quat::from_rotation_y(heading)
            * Quat::from_scaled_axis(Vec3::new(-tilt.y, 0., -tilt.x));
        let half = drone_world_half_extents(rotation);
        for axis in 0..3 {
            for sign in [-1., 1.] {
                for inside in [false, true] {
                    let (mut app, drone) = empty_app();
                    quiet(&mut app);
                    *app.world_mut().get_mut::<DroneFlight>(drone).unwrap() = DroneFlight {
                        heading,
                        tilt,
                        ..default()
                    };
                    let mut offset = Vec3::ZERO;
                    offset[axis] = sign * (half[axis] + 14. + if inside { -0.1 } else { 0.1 });
                    enemy(&mut app, START + offset, 100);
                    step(&mut app, 0., &[]);
                    assert_eq!(
                        app.world().resource::<PlayerHealth>().current,
                        if inside { 75 } else { 100 },
                        "axis {axis}, sign {sign}, inside {inside}"
                    );
                }
            }
        }
    }
}

#[test]
fn automatic_aim_ignores_heading_and_flight_runs_before_combat() {
    use crate::arena::DroneFlight;
    for heading in [0., std::f32::consts::FRAC_PI_2, std::f32::consts::PI] {
        let (mut app, drone) = empty_app();
        app.world_mut()
            .resource_mut::<CombatConfig>()
            .enemy_flight
            .max_horizontal_speed = 0.;
        *app.world_mut().get_mut::<DroneFlight>(drone).unwrap() = DroneFlight {
            heading,
            velocity: Vec3::new(20., 10., -10.),
            ..default()
        };
        let target = START + Vec3::new(-100., 60., 50.);
        enemy(&mut app, target, 100);
        step(&mut app, 0.1, &[]);
        let (shot, transform) = app
            .world_mut()
            .query::<(&Projectile, &Transform)>()
            .single(app.world())
            .unwrap();
        let current = position(&app, drone);
        assert!(current.distance(START) > 1.);
        assert!(transform.translation.distance(current) < 0.001);
        assert!(
            shot.velocity
                .normalize()
                .distance((target - current).normalize())
                < 0.001
        );
    }
}

#[test]
fn death_freezes_existing_velocity_and_tilt_then_restart_clears_them() {
    use crate::arena::DroneFlight;
    let (mut app, drone) = empty_app();
    quiet(&mut app);
    step(
        &mut app,
        0.3,
        &[KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyD, KeyCode::Space],
    );
    let before = *app.world().get::<DroneFlight>(drone).unwrap();
    let transform = *app.world().get::<Transform>(drone).unwrap();
    assert_ne!(before, DroneFlight::default());
    app.world_mut().resource_mut::<PlayerHealth>().current = 1;
    enemy(&mut app, transform.translation, 100);
    step(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Dead);
    step(
        &mut app,
        10.,
        &[
            KeyCode::KeyS,
            KeyCode::KeyQ,
            KeyCode::KeyA,
            KeyCode::ShiftLeft,
        ],
    );
    assert_eq!(*app.world().get::<DroneFlight>(drone).unwrap(), before);
    assert_eq!(*app.world().get::<Transform>(drone).unwrap(), transform);
    step(
        &mut app,
        1.,
        &[KeyCode::KeyR, KeyCode::KeyW, KeyCode::Space],
    );
    assert_eq!(
        *app.world().get::<DroneFlight>(drone).unwrap(),
        DroneFlight::default()
    );
    assert_eq!(
        *app.world().get::<Transform>(drone).unwrap(),
        Transform::from_translation(START)
    );
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
}

#[path = "enemy_flight_tests.rs"]
mod enemy_flight_tests;

#[path = "wave_tests.rs"]
mod wave_tests;

#[path = "feedback_tests.rs"]
mod feedback_tests;

#[path = "validation_tests.rs"]
mod validation_tests;

#[path = "energy_tests.rs"]
mod energy_tests;

#[path = "module_tests.rs"]
mod module_tests;

#[path = "rocket_tests.rs"]
mod rocket_tests;

#[path = "world_tests.rs"]
mod world_tests;

#[path = "hazard_tests.rs"]
mod hazard_tests;
#[path = "upgrade_tests.rs"]
mod upgrade_tests;
