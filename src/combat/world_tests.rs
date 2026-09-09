use super::*;
use crate::world::{Solid, WorldGeometry};

fn wall_app() -> (App, Entity) {
    let (mut app, drone) = empty_app();
    quiet(&mut app);
    app.insert_resource(WorldGeometry {
        solids: vec![Solid {
            center: Vec3::new(100., 150., 0.),
            half: Vec3::new(10., 150., 100.),
        }],
        hazard: None,
    });
    (app, drone)
}

#[test]
fn both_weapons_skip_occluded_nearest_target() {
    let (mut app, _) = wall_app();
    app.world_mut().resource_mut::<CombatConfig>().target_range = 400.;
    enemy(&mut app, START + Vec3::X * 180., 100);
    enemy(&mut app, START + Vec3::NEG_Z * 210., 100);
    step(&mut app, 0., &[KeyCode::Digit4]);
    let velocities: Vec<_> = app
        .world_mut()
        .query::<&Projectile>()
        .iter(app.world())
        .map(|p| p.velocity.normalize())
        .collect();
    assert_eq!(velocities.len(), 2);
    assert!(velocities.iter().all(|v| v.distance(Vec3::NEG_Z) < 0.001));
}

#[test]
fn wall_blocks_projectile_before_enemy_and_leaves_no_shot() {
    let (mut app, _) = wall_app();
    let target = enemy(&mut app, START + Vec3::X * 180., 100);
    let projectile = shot(&mut app, START, Vec3::X * 650., 1.);
    step(&mut app, 0.5, &[]);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().health, 100);
    assert!(app.world().get_entity(projectile).is_err());
}

#[test]
fn enemy_before_wall_is_hit_and_low_cover_can_be_shot_over() {
    let (mut app, _) = wall_app();
    let target = enemy(&mut app, START + Vec3::X * 60., 100);
    shot(&mut app, START, Vec3::X * 650., 1.);
    step(&mut app, 0.1, &[]);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().health, 90);
    app.world_mut().despawn(target);
    app.world_mut().resource_mut::<WorldGeometry>().solids[0] = Solid {
        center: Vec3::new(100., 20., 0.),
        half: Vec3::new(10., 20., 100.),
    };
    let high = enemy(&mut app, START + Vec3::X * 180., 100);
    shot(&mut app, START, Vec3::X * 650., 1.);
    step(&mut app, 0.5, &[]);
    assert_eq!(app.world().get::<Enemy>(high).unwrap().health, 90);
}

#[test]
fn rocket_explodes_on_terrain_and_splash_does_not_cross_cover() {
    let (mut app, _) = wall_app();
    let exposed = enemy(&mut app, Vec3::new(60., 90., 50.), 100);
    let covered = enemy(&mut app, Vec3::new(140., 90., 30.), 100);
    let projectile = shot(&mut app, START, Vec3::X * 500., 1.5);
    app.world_mut()
        .entity_mut(projectile)
        .insert(rockets::Rocket);
    step(&mut app, 0.3, &[]);
    assert!(app.world().get_entity(projectile).is_err());
    assert_eq!(app.world().get::<Enemy>(exposed).unwrap().health, 80);
    assert_eq!(app.world().get::<Enemy>(covered).unwrap().health, 100);
    assert!(
        app.world()
            .resource::<CombatOutcomes>()
            .0
            .iter()
            .any(|e| matches!(e, CombatOutcome::RocketExplosion { .. }))
    );
}

#[test]
fn enemy_can_reach_player_resting_above_low_cover() {
    for (target, from) in [
        (Vec3::new(-170., 75.01, -185.), Vec3::new(280., 90., -180.)),
        (Vec3::new(320., 85.01, 175.), Vec3::new(-280., 90., -180.)),
    ] {
        for rate in [30, 60, 144] {
            let (mut app, drone) = empty_app();
            app.insert_resource(crate::world::WorldGeometry::default());
            app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
            app.world_mut()
                .get_mut::<Transform>(drone)
                .unwrap()
                .translation = target;
            let chaser = enemy(&mut app, from, 1000);
            for _ in 0..15 * rate {
                step(&mut app, 1. / rate as f32, &[]);
            }
            let point = position(&app, chaser);
            assert!(
                point.distance(position(&app, drone)) < 80.,
                "{rate} FPS: enemy stuck at {point:?}; player {:?}",
                position(&app, drone)
            );
        }
    }
}
#[test]
fn zero_duration_wall_contact_cannot_hit_remote_projectile() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    let world = crate::world::WorldGeometry::default();
    let mut transform = Transform::from_xyz(94., 150., -180.);
    let mut flight = crate::arena::DroneFlight {
        velocity: Vec3::X * 50.,
        ..default()
    };
    let config = crate::arena::FlightConfig::default();
    let path = flight.step_in_world(
        &mut transform,
        &crate::arena::FlightInput {
            tilt: Vec2::ZERO,
            yaw: 0.,
            thrust: 1.,
        },
        &config,
        &crate::arena::Arena::default(),
        Vec3::splat(14.),
        1. / 120.,
        Some(&world),
    );
    let chaser = enemy(&mut app, transform.translation, 100);
    app.world_mut()
        .entity_mut(chaser)
        .remove::<crate::arena::DroneFlight>();
    app.world_mut().get_mut::<Enemy>(chaser).unwrap().path = path;
    app.insert_resource(world);
    let projectile = shot(&mut app, Vec3::new(-400., 90., 200.), Vec3::X * 650., 1.);
    step(&mut app, 1. / 120., &[]);
    assert_eq!(
        app.world().get::<Enemy>(chaser).unwrap().health,
        100,
        "remote enemy damaged by a missed projectile"
    );
    assert!(app.world().get_entity(projectile).is_ok());
}

#[test]
fn instantaneous_enemy_segments_preserve_real_contact_hits() {
    for (from, to, at) in [
        (Vec3::ZERO, Vec3::ZERO, 0.),
        (Vec3::NEG_Z * 30., Vec3::Z * 30., 0.5),
    ] {
        let (mut app, _) = empty_app();
        quiet(&mut app);
        let center = Vec3::new(0., 150., 0.);
        let chaser = enemy(&mut app, center + to, 100);
        app.world_mut()
            .entity_mut(chaser)
            .remove::<crate::arena::DroneFlight>();
        app.world_mut().get_mut::<Enemy>(chaser).unwrap().path =
            vec![crate::world::MotionSegment {
                start: center + from,
                end: center + to,
                from: at,
                to: at,
                half: Vec3::splat(14.),
            }];
        let projectile = shot(&mut app, center, Vec3::ZERO, 1.);
        step(&mut app, 0.1, &[]);
        assert_eq!(app.world().get::<Enemy>(chaser).unwrap().health, 90);
        assert!(app.world().get_entity(projectile).is_err());
    }
}
