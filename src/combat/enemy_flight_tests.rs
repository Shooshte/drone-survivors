use super::*;
use crate::arena::{Arena, DroneFlight, world_half_extents};

#[test]
fn enemy_launch_accelerates_gradually_from_rest() {
    let (mut app, _) = empty_app();
    app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
    let from = START + Vec3::X * 200.;
    let target = enemy(&mut app, from, 100);
    step(&mut app, 0.1, &[]);
    let initial_distance = position(&app, target).distance(from);
    assert!(
        initial_distance > 0. && initial_distance < 2.,
        "initial distance: {initial_distance}"
    );
    let first = position(&app, target);
    step(&mut app, 0.1, &[]);
    assert!(position(&app, target).distance(first) > initial_distance * 1.5);
    println!(
        "enemy launch: first 0.1 s distance {initial_distance:.3}, second 0.1 s distance {:.3}, speed at 0.2 s {:.3}",
        position(&app, target).distance(first),
        app.world()
            .get::<DroneFlight>(target)
            .unwrap()
            .velocity
            .length()
    );
}

#[test]
fn immediate_forward_evasion_preserves_full_hull_for_two_seconds() {
    let (mut app, drone) = app();
    for frame in 0..240 {
        step(&mut app, 1. / 120., &[KeyCode::KeyW]);
        let health = app.world().resource::<PlayerHealth>().current;
        let enemies: Vec<_> = app
            .world_mut()
            .query_filtered::<&Transform, With<Enemy>>()
            .iter(app.world())
            .map(|t| t.translation)
            .collect();
        assert_eq!(
            health,
            100,
            "frame {frame}, drone {:?}, enemies {enemies:?}",
            position(&app, drone)
        );
    }
}

fn flying_app(from: Vec3, target: Vec3) -> (App, Entity, Entity) {
    let (mut app, drone) = empty_app();
    app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
    app.world_mut()
        .resource_mut::<PlayerHealth>()
        .invulnerable_until = f64::INFINITY;
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = target;
    let chaser = enemy(&mut app, from, 100);
    (app, drone, chaser)
}

#[test]
fn enemy_speed_attitude_and_yaw_have_slower_physical_limits() {
    let (mut app, drone, chaser) = flying_app(START, START + Vec3::new(200., 0., 100.));
    app.world_mut().resource_mut::<Arena>().half_size = Vec3::splat(100000.);
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = START + Vec3::new(10000., 0., 1000.);
    let mut peak_speed: f32 = 0.;
    for _ in 0..1200 {
        let before = *app.world().get::<DroneFlight>(chaser).unwrap();
        step(&mut app, 1. / 120., &[]);
        let flight = app.world().get::<DroneFlight>(chaser).unwrap();
        let speed = flight.velocity.with_y(0.).length();
        peak_speed = peak_speed.max(speed);
        assert!(speed <= 260.001);
        assert!(flight.tilt.length() <= 20_f32.to_radians() + 0.001);
        assert!(flight.tilt.distance(before.tilt) <= 150_f32.to_radians() / 120. + 0.001);
        let turn = (Quat::from_rotation_y(before.heading) * Vec3::NEG_Z)
            .angle_between(Quat::from_rotation_y(flight.heading) * Vec3::NEG_Z);
        assert!(turn <= 120_f32.to_radians() / 120. + 0.001);
    }
    assert!(peak_speed > 230., "peak {peak_speed}");
    let transform = app.world().get::<Transform>(chaser).unwrap();
    assert!((transform.rotation * Vec3::Y).distance(Vec3::Y) > 0.1);
}

#[test]
fn enemy_keeps_world_momentum_then_brakes_and_reverses_after_target_flip() {
    let (mut app, drone, chaser) = flying_app(START, START + Vec3::X * 400.);
    step(&mut app, 1., &[]);
    let before = *app.world().get::<DroneFlight>(chaser).unwrap();
    let position_before = position(&app, chaser);
    assert!(before.velocity.x > 50.);
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = START - Vec3::X * 400.;
    step(&mut app, 1. / 120., &[]);
    let first = *app.world().get::<DroneFlight>(chaser).unwrap();
    assert!(first.velocity.x > 0.);
    assert!(first.velocity.distance(before.velocity) < 5.);
    assert!(position(&app, chaser).x > position_before.x);
    step(&mut app, 2., &[]);
    assert!(app.world().get::<DroneFlight>(chaser).unwrap().velocity.x < -20.);
}

#[test]
fn enemy_arrives_at_stationary_targets_and_contains_rotated_corners() {
    for target in [
        START,
        Vec3::new(-445., 15., -225.),
        Vec3::new(445., 285., 225.),
        Vec3::new(200., 260., -100.),
    ] {
        let (mut app, _, chaser) = flying_app(Vec3::new(0., 150., 0.), target);
        for _ in 0..600 {
            step(&mut app, 1. / 30., &[]);
            let transform = app.world().get::<Transform>(chaser).unwrap();
            let half = world_half_extents(transform.rotation, Vec3::splat(14.));
            let arena = app.world().resource::<Arena>();
            assert!(
                (transform.translation - arena.center())
                    .abs()
                    .cmple(arena.half_size - half + Vec3::splat(0.001))
                    .all()
            );
        }
        assert!(
            position(&app, chaser).distance(target) < 2.,
            "target {target:?}, actual {:?}",
            position(&app, chaser)
        );
        assert!(
            app.world()
                .get::<DroneFlight>(chaser)
                .unwrap()
                .velocity
                .length()
                < 2.
        );
    }
}

#[test]
fn enemy_steering_matches_across_frame_rates_and_long_frames() {
    let mut results = vec![];
    for rate in [1, 30, 60, 120, 144] {
        let (mut app, drone, chaser) =
            flying_app(Vec3::new(-200., 50., 100.), Vec3::new(250., 240., -150.));
        for _ in 0..rate * 2 {
            step(&mut app, 1. / rate as f32, &[]);
        }
        app.world_mut()
            .get_mut::<Transform>(drone)
            .unwrap()
            .translation = Vec3::new(-300., 30., -180.);
        for _ in 0..rate * 2 {
            step(&mut app, 1. / rate as f32, &[]);
        }
        results.push((
            position(&app, chaser),
            *app.world().get::<DroneFlight>(chaser).unwrap(),
        ));
    }
    for (position, flight) in &results {
        assert!(position.distance(results[0].0) < 2., "results {results:?}");
        assert!(
            flight.velocity.distance(results[0].1.velocity) < 2.,
            "results {results:?}"
        );
    }
}

#[test]
fn enemy_death_freezes_flight_and_restart_spawns_level_at_rest_facing_player() {
    let (mut app, _, chaser) = flying_app(START + Vec3::X * 200., START);
    step(&mut app, 0.75, &[]);
    let before = *app.world().get::<DroneFlight>(chaser).unwrap();
    let transform = *app.world().get::<Transform>(chaser).unwrap();
    assert!(before.velocity.length() > 20. && before.tilt.length() > 0.);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
    step(&mut app, 4., &[]);
    assert_eq!(*app.world().get::<DroneFlight>(chaser).unwrap(), before);
    assert_eq!(*app.world().get::<Transform>(chaser).unwrap(), transform);
    step(&mut app, 1., &[KeyCode::KeyR]);
    for (flight, transform) in app
        .world_mut()
        .query_filtered::<(&DroneFlight, &Transform), With<Enemy>>()
        .iter(app.world())
    {
        assert_eq!(flight.velocity, Vec3::ZERO);
        assert_eq!(flight.tilt, Vec2::ZERO);
        let toward = (START - transform.translation).with_y(0.).normalize();
        assert!((transform.rotation * Vec3::NEG_Z).distance(toward) < 0.001);
    }
}

#[test]
fn rotated_enemy_envelope_is_shared_by_contact_and_projectiles() {
    for projectile in [false, true] {
        let (mut app, _) = empty_app();
        quiet(&mut app);
        let center = START + Vec3::X * if projectile { 100. } else { 54. };
        let chaser = enemy(&mut app, center, 100);
        app.world_mut()
            .get_mut::<DroneFlight>(chaser)
            .unwrap()
            .heading = std::f32::consts::FRAC_PI_4;
        if projectile {
            shot(
                &mut app,
                center + Vec3::new(-50., 0., 21.),
                Vec3::X * 400.,
                1.,
            );
            step(&mut app, 0.2, &[]);
            assert_eq!(app.world().get::<Enemy>(chaser).unwrap().health, 90);
        } else {
            step(&mut app, 0., &[]);
            assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
        }
    }
}

#[test]
fn clipped_projectile_chooses_first_hit_across_static_and_flying_targets() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    let first = enemy(&mut app, START + Vec3::X * 80., 100);
    app.world_mut().entity_mut(first).remove::<DroneFlight>();
    let second = enemy(&mut app, START + Vec3::X * 120., 100);
    shot(&mut app, START, Vec3::X * 650., 0.5);
    step(&mut app, 1., &[]);
    assert_eq!(app.world().get::<Enemy>(first).unwrap().health, 90);
    assert_eq!(app.world().get::<Enemy>(second).unwrap().health, 100);
}

#[test]
fn curved_enemy_path_hits_projectile_away_from_whole_frame_chord() {
    let from = Vec3::new(-200., 120., 120.);
    let target = Vec3::new(-300., 120., -200.);
    let prepare = || {
        let (mut app, drone, chaser) = flying_app(from, target);
        app.world_mut()
            .get_mut::<DroneFlight>(chaser)
            .unwrap()
            .velocity = Vec3::X * 200.;
        (app, drone, chaser)
    };
    let (mut reference, _, chaser) = prepare();
    step(&mut reference, 1., &[]);
    let crossing = position(&reference, chaser);
    step(&mut reference, 1., &[]);
    let end = position(&reference, chaser);
    assert!(
        super::super::collision::segment_box(from, end, crossing, Vec3::splat(28.)).is_none(),
        "path must curve away from chord: {from:?}, {crossing:?}, {end:?}"
    );
    for lifetime in [0.05, 2.] {
        let (mut app, _, chaser) = prepare();
        shot(&mut app, crossing, Vec3::ZERO, lifetime);
        step(&mut app, 2., &[]);
        assert_eq!(
            app.world().get::<Enemy>(chaser).unwrap().health,
            if lifetime < 1. { 100 } else { 90 }
        );
    }
}
