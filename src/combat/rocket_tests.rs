use super::*;
use crate::modules::{ModuleConfig, ModuleKind, Modules};

fn rocket(app: &mut App, position: Vec3, velocity: Vec3, remaining: f32) -> Entity {
    app.world_mut()
        .spawn((
            Projectile {
                velocity,
                remaining,
            },
            Transform::from_translation(position),
            rockets::Rocket,
        ))
        .id()
}

fn rocket_count(app: &mut App) -> usize {
    count::<rockets::Rocket>(app)
}

#[test]
fn launcher_targets_nearest_enemy_in_three_dimensions_with_stable_ties() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    app.world_mut().resource_mut::<ModuleConfig>().rocket_range = 120.;
    enemy(&mut app, START + Vec3::Y * 150., 100);
    let first = enemy(&mut app, START + Vec3::X * 100., 100);
    let second = enemy(&mut app, START - Vec3::X * 100., 100);

    step(&mut app, 0., &[KeyCode::Digit4]);

    let velocity = app
        .world_mut()
        .query_filtered::<&Projectile, With<rockets::Rocket>>()
        .single(app.world())
        .unwrap()
        .velocity;
    let expected = if first.to_bits() < second.to_bits() {
        Vec3::X
    } else {
        Vec3::NEG_X
    };
    assert!(velocity.normalize().distance(expected) < 0.001);
    assert_eq!(velocity.length(), 500.);
}

#[test]
fn launcher_cooldown_continues_while_off_without_banking_a_burst() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    enemy(&mut app, START + Vec3::X * 200., 10_000);
    {
        let mut config = app.world_mut().resource_mut::<ModuleConfig>();
        config.rocket_speed = 0.;
        config.rocket_lifetime = 20.;
        config.drains[ModuleKind::Rocket as usize] = 0.;
    }

    step(&mut app, 0., &[KeyCode::Digit4]);
    assert_eq!(rocket_count(&mut app), 1);
    assert_eq!(
        app.world().resource::<rockets::RocketLauncher>().ready_at,
        2.
    );

    step(&mut app, 0.5, &[KeyCode::Digit4]);
    step(&mut app, 1., &[]);
    step(&mut app, 0., &[KeyCode::Digit4]);
    assert_eq!(rocket_count(&mut app), 1);

    step(&mut app, 0.5, &[]);
    assert_eq!(rocket_count(&mut app), 2);
    step(&mut app, 10., &[]);
    assert_eq!(rocket_count(&mut app), 3);
    assert_eq!(
        app.world().resource::<rockets::RocketLauncher>().ready_at,
        14.
    );
}

#[test]
fn launcher_cadence_matches_across_frame_rates_and_ignores_overdrive() {
    for rate in [30, 120] {
        for overdrive in [false, true] {
            let (mut app, _) = empty_app();
            quiet(&mut app);
            enemy(&mut app, START + Vec3::X * 200., 10_000);
            {
                let mut config = app.world_mut().resource_mut::<ModuleConfig>();
                config.rocket_speed = 0.;
                config.rocket_lifetime = 10.;
                config.drains = [0.; 4];
            }
            let keys: &[KeyCode] = if overdrive {
                &[KeyCode::Digit1, KeyCode::Digit4]
            } else {
                &[KeyCode::Digit4]
            };
            step(&mut app, 0., keys);
            for _ in 0..rate * 4 {
                step(&mut app, 1. / rate as f32, &[]);
            }
            assert_eq!(
                rocket_count(&mut app),
                3,
                "rate={rate}, overdrive={overdrive}"
            );
        }
    }
}

#[test]
fn depletion_stops_new_launches_but_an_in_flight_rocket_continues() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    enemy(&mut app, START + Vec3::X * 200., 10_000);
    {
        let mut config = app.world_mut().resource_mut::<ModuleConfig>();
        config.rocket_interval = 0.1;
        config.rocket_speed = 1.;
        config.rocket_lifetime = 20.;
    }

    step(&mut app, 0., &[KeyCode::Digit4]);
    let launched = app
        .world_mut()
        .query_filtered::<Entity, With<rockets::Rocket>>()
        .single(app.world())
        .unwrap();
    step(&mut app, 10., &[]);

    assert!(!app.world().resource::<Modules>().active(ModuleKind::Rocket));
    assert_eq!(rocket_count(&mut app), 1);
    assert!(position(&app, launched).x > START.x);
}

#[test]
fn rocket_splash_is_inclusive_in_three_dimensions_and_hits_each_enemy_once() {
    let mut app = collision_app();
    let direct = enemy(&mut app, START + Vec3::X * 50., 100);
    // With 14-unit enemy half-size and 3-unit projectile radius, impact is x=33.
    let inclusive = enemy(&mut app, START + Vec3::new(33., 70., 0.), 100);
    let lethal = enemy(&mut app, START + Vec3::new(33., -50., 0.), 20);
    let outside = enemy(&mut app, START + Vec3::new(33., 0., 70.01), 100);
    rocket(&mut app, START, Vec3::X * 100., 1.5);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(1.));
    app.update();

    assert_eq!(app.world().get::<Enemy>(direct).unwrap().health, 80);
    assert_eq!(app.world().get::<Enemy>(inclusive).unwrap().health, 80);
    assert!(app.world().get_entity(lethal).is_err());
    assert_eq!(app.world().get::<Enemy>(outside).unwrap().health, 100);
    assert_eq!(app.world().resource::<Encounter>().kills, 1);
    let explosions: Vec<_> = app
        .world()
        .resource::<CombatOutcomes>()
        .0
        .iter()
        .filter_map(|outcome| match outcome {
            CombatOutcome::RocketExplosion { position, radius } => Some((*position, *radius)),
            _ => None,
        })
        .collect();
    assert_eq!(explosions.len(), 1);
    assert!(explosions[0].0.distance(START + Vec3::X * 33.) < 0.01);
    assert_eq!(explosions[0].1, 70.);
}

#[test]
fn splash_uses_enemy_centers_at_the_actual_impact_time() {
    let mut app = collision_app();
    let direct = enemy(&mut app, START + Vec3::X * 50., 100);
    let moving = app
        .world_mut()
        .spawn((
            Enemy {
                health: 100,
                previous: START + Vec3::new(33., 30., 0.),
                path: vec![enemies::FlightSegment {
                    start: START + Vec3::new(33., 30., 0.),
                    end: START + Vec3::new(33., 120., 0.),
                    from: 0.,
                    to: 1.,
                    half: Vec3::splat(14.),
                }],
            },
            Transform::from_translation(START + Vec3::new(33., 120., 0.)),
        ))
        .id();
    rocket(&mut app, START, Vec3::X * 100., 1.5);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(1.));

    app.update();

    assert_eq!(app.world().get::<Enemy>(direct).unwrap().health, 80);
    assert_eq!(app.world().get::<Enemy>(moving).unwrap().health, 80);
}

fn collision_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<crate::arena::Arena>()
        .init_resource::<CombatConfig>()
        .init_resource::<Encounter>()
        .init_resource::<CombatOutcomes>()
        .init_resource::<ModuleConfig>()
        .add_systems(Update, weapon::advance_projectiles);
    app
}

#[test]
fn missed_rockets_expire_without_exploding_even_after_the_module_is_disabled() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    let id = rocket(&mut app, START, Vec3::X * 100., 0.25);

    step(&mut app, 0.2, &[]);
    assert!(app.world().get_entity(id).is_ok());
    assert_eq!(position(&app, id), START + Vec3::X * 20.);
    step(&mut app, 0.1, &[]);

    assert!(app.world().get_entity(id).is_err());
    assert!(
        app.world()
            .resource::<CombatOutcomes>()
            .0
            .iter()
            .all(|outcome| !matches!(outcome, CombatOutcome::RocketExplosion { .. }))
    );
}
