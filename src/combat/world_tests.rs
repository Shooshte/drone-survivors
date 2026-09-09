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
