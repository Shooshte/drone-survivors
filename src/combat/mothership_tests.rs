use super::*;
use crate::{
    arena::{Arena, Drone},
    combat::{CombatConfig, Enemy, SpawnWarning, WaveConfig, waves},
    economy::runtime::EnemyKind,
    game::GamePhase,
    world::{Solid, WorldGeometry},
};
use std::time::Duration;

const PARENT_POSITION: Vec3 = Vec3::new(300., 90., 0.);

fn app(cap: usize) -> (App, Entity) {
    let mut app = App::new();
    let mut waves = WaveConfig::default();
    waves.cap = cap;
    waves.disable_authored_waves();
    app.init_resource::<Time>()
        .insert_resource(waves)
        .insert_resource(CombatConfig::default())
        .insert_resource(Arena::default())
        .insert_resource(GamePhase::Playing)
        .insert_resource(crate::combat::Encounter::default())
        .add_systems(
            Update,
            (waves::advance_clock, update, waves::update).chain(),
        );
    app.world_mut()
        .spawn((Drone, Transform::from_xyz(0., 90., 0.)));
    let parent = spawn_parent(&mut app, PARENT_POSITION);
    app.update();
    (app, parent)
}

fn spawn_parent(app: &mut App, position: Vec3) -> Entity {
    app.world_mut()
        .spawn((
            Enemy {
                kind: EnemyKind::Mothership,
                health: 100,
                previous: position,
                path: Vec::new(),
            },
            Transform::from_translation(position),
            Mothership::default(),
        ))
        .id()
}

fn step(app: &mut App, seconds: f64) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(seconds));
    app.update();
}

fn count<T: Component>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .iter(app.world())
        .count()
}

fn warning(app: &mut App) -> (Entity, EnemyKind, f64, SpawnParent, Vec3) {
    app.world_mut()
        .query::<(Entity, &SpawnWarning, &SpawnParent, &Transform)>()
        .single(app.world())
        .map(|(id, warning, parent, transform)| {
            (
                id,
                warning.kind,
                warning.ready_at,
                *parent,
                transform.translation,
            )
        })
        .unwrap()
}

#[test]
fn cadence_gives_every_supported_rate_a_full_warning_before_activation() {
    for hz in [30, 60, 144] {
        let (mut app, parent) = app(30);
        let frame = 1. / f64::from(hz);
        for _ in 0..6 * hz {
            step(&mut app, frame);
        }
        if count::<SpawnWarning>(&mut app) == 0 {
            step(&mut app, frame);
        }
        assert_eq!(count::<SpawnWarning>(&mut app), 1, "{hz} Hz admission");
        assert_eq!(count::<Enemy>(&mut app), 1, "{hz} Hz warning phase");
        let (_, kind, _, source, _) = warning(&mut app);
        assert_eq!(kind, EnemyKind::Chaser);
        assert_eq!(source, SpawnParent(parent));

        for _ in 0..(1.2 * f64::from(hz)).ceil() as usize - 1 {
            step(&mut app, frame);
        }
        assert_eq!(count::<Enemy>(&mut app), 1, "{hz} Hz full warning");
        step(&mut app, frame);
        assert_eq!(count::<SpawnWarning>(&mut app), 0, "{hz} Hz delivery");
        assert_eq!(count::<Enemy>(&mut app), 2, "{hz} Hz child");
        assert_eq!(
            app.world().resource::<crate::combat::Encounter>().spawns,
            waves::SpawnCounts {
                requested: 1,
                admitted: 1,
                activated: 1,
                ..default()
            }
        );
    }
}

#[test]
fn attempts_cycle_all_six_child_kinds_without_recursive_motherships() {
    let (mut app, _) = app(30);
    let expected = [
        EnemyKind::Chaser,
        EnemyKind::Fast,
        EnemyKind::Rammer,
        EnemyKind::Slower,
        EnemyKind::Jammer,
        EnemyKind::Bomber,
        EnemyKind::Chaser,
    ];
    for expected_kind in expected {
        step(&mut app, 6.);
        let (_, kind, _, _, _) = warning(&mut app);
        assert_eq!(kind, expected_kind);
        assert_ne!(kind, EnemyKind::Mothership);
        step(&mut app, 1.2);
    }
}

#[test]
fn cap_is_shared_by_live_enemies_parent_warnings_and_authored_waves() {
    let (mut app, _) = app(3);
    spawn_parent(&mut app, Vec3::new(-300., 90., 0.));
    app.update();
    app.world_mut().resource_mut::<WaveConfig>().bursts = vec![(6., 1)];

    step(&mut app, 6.);

    assert_eq!(count::<Enemy>(&mut app), 2);
    assert_eq!(count::<SpawnWarning>(&mut app), 1);
    assert_eq!(
        app.world().resource::<crate::combat::Encounter>().spawns,
        waves::SpawnCounts {
            requested: 3,
            admitted: 1,
            rejected_cap: 2,
            ..default()
        }
    );
}

#[test]
fn hitch_requests_only_one_child_and_restarts_the_interval_from_now() {
    let (mut app, parent) = app(30);

    step(&mut app, 20.);

    let (_, _, ready_at, _, _) = warning(&mut app);
    assert!((ready_at - 21.2).abs() < 1e-7);
    let mothership = app.world().get::<Mothership>(parent).unwrap();
    assert_eq!(mothership.ready_at, Some(26.));
    assert_eq!(
        app.world()
            .resource::<crate::combat::Encounter>()
            .spawns
            .requested,
        1
    );
}

#[test]
fn admission_requires_player_arena_and_world_clearance() {
    for obstruction in [0, 1, 2] {
        let (mut app, parent) = app(30);
        match obstruction {
            0 => {
                let world = app.world_mut();
                let mut drones = world.query_filtered::<&mut Transform, With<Drone>>();
                drones.single_mut(world).unwrap().translation = PARENT_POSITION;
            }
            1 => {
                app.world_mut().resource_mut::<Arena>().half_size = Vec3::splat(50.);
            }
            2 => {
                app.insert_resource(WorldGeometry {
                    solids: vec![Solid {
                        center: PARENT_POSITION,
                        half: Vec3::new(200., 100., 200.),
                    }],
                    hazard: None,
                });
            }
            _ => unreachable!(),
        }
        step(&mut app, 6.);
        assert_eq!(
            count::<SpawnWarning>(&mut app),
            0,
            "obstruction {obstruction}"
        );
        assert_eq!(
            app.world()
                .resource::<crate::combat::Encounter>()
                .spawns
                .rejected_space,
            1
        );
        assert_eq!(
            app.world().get::<Mothership>(parent).unwrap().ready_at,
            Some(12.)
        );
    }
}

#[test]
fn activation_revalidates_clearance_and_parent_liveness() {
    for dead_parent in [false, true] {
        let (mut app, parent) = app(30);
        step(&mut app, 6.);
        let (_, _, _, _, position) = warning(&mut app);
        if dead_parent {
            app.world_mut().get_mut::<Enemy>(parent).unwrap().health = 0;
        } else {
            let world = app.world_mut();
            let mut drones = world.query_filtered::<&mut Transform, With<Drone>>();
            drones.single_mut(world).unwrap().translation = position;
        }

        step(&mut app, 1.2);

        assert_eq!(count::<SpawnWarning>(&mut app), 0);
        let live = app
            .world_mut()
            .query::<&Enemy>()
            .iter(app.world())
            .filter(|enemy| enemy.health > 0)
            .count();
        assert_eq!(live, usize::from(!dead_parent));
        assert_eq!(
            app.world()
                .resource::<crate::combat::Encounter>()
                .spawns
                .cancelled,
            1
        );
        assert_eq!(
            app.world()
                .resource::<crate::combat::Encounter>()
                .spawns
                .activated,
            0
        );
    }
}

#[test]
fn parent_death_cancels_warning_before_its_delivery_deadline() {
    let (mut app, parent) = app(30);
    step(&mut app, 6.);
    app.world_mut().get_mut::<Enemy>(parent).unwrap().health = 0;

    step(&mut app, 0.);

    assert_eq!(count::<SpawnWarning>(&mut app), 0);
    assert_eq!(
        app.world()
            .resource::<crate::combat::Encounter>()
            .spawns
            .cancelled,
        1
    );
}

#[test]
fn dead_parent_warning_releases_shared_capacity_in_the_death_frame() {
    let (mut app, dead_parent) = app(2);
    step(&mut app, 6.);
    app.world_mut()
        .get_mut::<Enemy>(dead_parent)
        .unwrap()
        .health = 0;
    let live_parent = spawn_parent(&mut app, Vec3::new(-300., 90., 0.));
    app.world_mut()
        .get_mut::<Mothership>(live_parent)
        .unwrap()
        .ready_at = Some(6.);

    step(&mut app, 0.);

    let (_, _, _, source, _) = warning(&mut app);
    assert_eq!(source, SpawnParent(live_parent));
    assert_eq!(
        app.world().resource::<crate::combat::Encounter>().spawns,
        waves::SpawnCounts {
            requested: 2,
            admitted: 2,
            cancelled: 1,
            ..default()
        }
    );
}

#[test]
fn delivered_child_survives_parent_despawn() {
    let (mut app, parent) = app(30);
    step(&mut app, 6.);
    step(&mut app, 1.2);
    app.world_mut().despawn(parent);

    step(&mut app, 0.);

    assert_eq!(count::<Enemy>(&mut app), 1);
    let child = app
        .world_mut()
        .query::<&Enemy>()
        .single(app.world())
        .unwrap();
    assert_eq!(child.kind, EnemyKind::Chaser);
}

#[test]
fn nonplaying_phase_does_not_initialize_or_advance_cadence() {
    let (mut app, parent) = app(30);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Choosing;
    app.world_mut()
        .get_mut::<Mothership>(parent)
        .unwrap()
        .ready_at = None;

    step(&mut app, 10.);

    assert_eq!(
        app.world().get::<Mothership>(parent).unwrap().ready_at,
        None
    );
    assert_eq!(count::<SpawnWarning>(&mut app), 0);
}

#[test]
fn terminal_phase_cancels_parent_linked_warning_through_shared_delivery() {
    let (mut app, _) = app(30);
    step(&mut app, 6.);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;

    step(&mut app, 0.);

    assert_eq!(count::<SpawnWarning>(&mut app), 0);
    assert_eq!(
        app.world()
            .resource::<crate::combat::Encounter>()
            .spawns
            .cancelled,
        1
    );
}

#[test]
fn flight_target_holds_inside_standoff_and_approaches_outside_it() {
    let player = Vec3::ZERO;
    assert_eq!(
        flight_target(Vec3::new(350., 0., 0.), player),
        Vec3::new(350., 0., 0.)
    );
    assert_eq!(flight_target(Vec3::new(351., 0., 0.), player), player);
}
