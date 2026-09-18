use super::*;
use crate::arena::Arena;
use crate::arena::{ArenaPlugin, Drone, DroneFlight};
use crate::combat::{CombatConfig, CombatPlugin, Encounter, Enemy, PlayerHealth, WaveConfig};
use crate::game::GamePhase;
use std::time::Duration;

fn app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins((ArenaPlugin, CombatPlugin));
    app.update();
    app.world_mut()
        .resource_mut::<WaveConfig>()
        .disable_authored_waves();
    app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
    app.world_mut()
        .resource_mut::<CombatConfig>()
        .separation_acceleration = 0.;
    app
}

fn spawn(app: &mut App, kind: EnemyKind, position: Vec3) -> Entity {
    let mut entity = app.world_mut().spawn((
        Enemy {
            kind,
            health: 80,
            previous: position,
            path: Vec::new(),
        },
        Transform::from_translation(position),
        DroneFlight {
            heading: -std::f32::consts::FRAC_PI_2,
            ..default()
        },
    ));
    if kind == EnemyKind::Rammer {
        entity.insert(Rammer::default());
    }
    entity.id()
}

fn step(app: &mut App, dt: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(dt));
    app.update();
}

#[test]
fn fast_pursuer_travels_faster_through_the_same_rotor_integrator() {
    let mut distances = Vec::new();
    for kind in [EnemyKind::Chaser, EnemyKind::Fast] {
        let mut app = app();
        let start = Vec3::new(-800., 90., -1400.);
        let id = spawn(&mut app, kind, start);
        for _ in 0..300 {
            step(&mut app, 1. / 60.);
        }
        distances.push(
            app.world()
                .get::<Transform>(id)
                .unwrap()
                .translation
                .distance(start),
        );
    }
    assert!(distances[1] > distances[0] + 30., "{distances:?}");
}

#[test]
fn rammer_delivers_exactly_two_physical_impacts_across_frame_rates() {
    for hz in [30, 60, 144] {
        let mut app = app();
        let id = spawn(&mut app, EnemyKind::Rammer, Vec3::new(-250., 90., 0.));
        let mut changes = Vec::new();
        let mut previous = 100;
        let mut phase_before = RamPhase::Approach;
        for frame in 0..hz * 30 {
            step(&mut app, 1. / hz as f32);
            let player_position = app
                .world_mut()
                .query_filtered::<&Transform, With<Drone>>()
                .single(app.world())
                .unwrap()
                .translation;
            if let Some(ram) = app.world().get::<Rammer>(id)
                && ram.phase != phase_before
            {
                println!(
                    "{hz}Hz t={:.2} phase={:?} position={:?} player={:?}",
                    frame as f32 / hz as f32,
                    ram.phase,
                    app.world().get::<Transform>(id).unwrap().translation,
                    player_position
                );
                phase_before = app.world().get::<Rammer>(id).unwrap().phase;
            }
            let current = app.world().resource::<PlayerHealth>().current;
            if current != previous {
                changes.push(frame as f32 / hz as f32);
                previous = current;
            }
            if app.world().get::<Enemy>(id).is_none() {
                break;
            }
        }
        assert_eq!(
            changes.len(),
            2,
            "{hz} Hz, events={changes:?}, phase={:?}",
            app.world().get::<Rammer>(id).map(|r| r.phase)
        );
        assert!(changes[1] - changes[0] >= 2.2, "{changes:?}");
        assert_eq!(app.world().resource::<PlayerHealth>().current, 80);
        assert_eq!(app.world().resource::<Encounter>().kills, 0);
        assert!(app.world().get::<Enemy>(id).is_none());
    }
}

#[test]
fn rammer_pauses_with_gameplay_and_restart_removes_its_state() {
    let mut app = app();
    let id = spawn(&mut app, EnemyKind::Rammer, Vec3::new(-200., 90., 0.));
    step(&mut app, 1. / 60.);
    let before = app.world().get::<Transform>(id).unwrap().translation;
    let elapsed = app.world().get::<Rammer>(id).unwrap().elapsed;
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Choosing;
    for _ in 0..60 {
        step(&mut app, 1. / 60.);
    }
    assert_eq!(app.world().get::<Rammer>(id).unwrap().elapsed, elapsed);
    assert_eq!(
        app.world().get::<Transform>(id).unwrap().translation,
        before
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyR);
    step(&mut app, 1. / 60.);
    assert!(app.world().get::<Enemy>(id).is_none());
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
}

#[test]
fn rammer_requires_warning_separation_and_two_distinct_accepted_impacts() {
    let config = VariantConfig::default();
    let arena = Arena::default();
    let mut ram = Rammer::default();
    let start = Vec3::new(-200., 90., 0.);
    let player = Vec3::new(0., 90., 0.);
    ram.plan(start, player, 100., &config, Vec3::splat(14.), &arena, None);
    assert_eq!(
        ram.phase,
        RamPhase::Windup,
        "a hitch cannot skip the warning"
    );
    let locked = ram.target;
    ram.plan(
        start,
        player + Vec3::Z * 100.,
        1.,
        &config,
        Vec3::splat(14.),
        &arena,
        None,
    );
    assert_eq!(ram.phase, RamPhase::Windup);
    ram.plan(
        start,
        player + Vec3::Z * 100.,
        0.,
        &config,
        Vec3::splat(14.),
        &arena,
        None,
    );
    assert_eq!(ram.phase, RamPhase::Charge);
    assert_eq!(ram.target, locked, "charge keeps its telegraphed line");
    assert!(!ram.impact(true));
    assert_eq!(ram.impacts, 1);
    assert_eq!(ram.phase, RamPhase::Retreat);
    assert!(
        !ram.impact(true),
        "lingering contact cannot consume another impact"
    );
    assert_eq!(ram.impacts, 1);
    ram.plan(
        player,
        player,
        100.,
        &config,
        Vec3::splat(14.),
        &arena,
        None,
    );
    assert_eq!(
        ram.phase,
        RamPhase::Retreat,
        "time alone cannot rearm while overlapping"
    );
    ram.plan(start, player, 0., &config, Vec3::splat(14.), &arena, None);
    assert_eq!(ram.phase, RamPhase::Windup);
    assert_eq!(ram.elapsed, 0.);
    ram.plan(start, player, 1., &config, Vec3::splat(14.), &arena, None);
    ram.plan(start, player, 0., &config, Vec3::splat(14.), &arena, None);
    assert!(ram.impact(true));
    assert_eq!(ram.impacts, 2);
}

#[test]
fn rammer_rejected_contact_retreats_without_spending_an_impact() {
    let mut ram = Rammer {
        phase: RamPhase::Charge,
        ..default()
    };
    assert!(!ram.impact(false));
    assert_eq!(ram.phase, RamPhase::Retreat);
    assert_eq!(ram.impacts, 0);
}

#[test]
fn rammer_shield_blocks_spend_hits_but_invulnerability_does_not() {
    use crate::energy::PowerFrame;
    use bevy::ecs::system::RunSystemOnce;
    let mut app = app();
    let id = spawn(
        &mut app,
        EnemyKind::Rammer,
        crate::arena::DRONE_START.translation,
    );
    app.world_mut().get_mut::<Rammer>(id).unwrap().phase = RamPhase::Charge;
    app.world_mut()
        .resource_mut::<PlayerHealth>()
        .invulnerable_until = 10.;
    app.world_mut()
        .run_system_once(crate::combat::lifecycle::contact_damage)
        .unwrap();
    assert_eq!(app.world().get::<Rammer>(id).unwrap().impacts, 0);
    assert_eq!(
        app.world().get::<Rammer>(id).unwrap().phase,
        RamPhase::Retreat
    );
    app.world_mut()
        .resource_mut::<PlayerHealth>()
        .invulnerable_until = 0.;
    app.world_mut().get_mut::<Rammer>(id).unwrap().phase = RamPhase::Charge;
    app.world_mut().resource_mut::<PowerFrame>().modules.enabled[1] = true;
    app.world_mut()
        .run_system_once(crate::combat::lifecycle::contact_damage)
        .unwrap();
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    assert_eq!(
        app.world().resource::<PowerFrame>().modules.shield.blocks,
        0
    );
    assert_eq!(app.world().get::<Rammer>(id).unwrap().impacts, 1);
    app.world_mut()
        .resource_mut::<PlayerHealth>()
        .invulnerable_until = 0.;
    app.world_mut()
        .run_system_once(crate::combat::lifecycle::contact_damage)
        .unwrap();
    assert_eq!(
        app.world().resource::<PlayerHealth>().current,
        100,
        "retreat contact is harmless"
    );
    app.world_mut().get_mut::<Rammer>(id).unwrap().phase = RamPhase::Charge;
    app.world_mut()
        .run_system_once(crate::combat::lifecycle::contact_damage)
        .unwrap();
    assert_eq!(app.world().resource::<PlayerHealth>().current, 90);
    assert!(app.world().get::<Enemy>(id).is_none());
    assert_eq!(app.world().resource::<Encounter>().kills, 0);
    assert!(
        app.world()
            .resource::<crate::combat::CombatOutcomes>()
            .0
            .iter()
            .all(|e| !matches!(e, crate::combat::CombatOutcome::Hit { killed: true, .. }))
    );
}

#[test]
fn variant_sweeps_catch_crossings_but_reject_contact_across_a_wall() {
    use crate::world::{MotionSegment, PlayerPath, Solid, WorldGeometry};
    let config = CombatConfig::default();
    let enemy = Enemy {
        kind: EnemyKind::Fast,
        health: 20,
        previous: Vec3::new(-200., 90., 0.),
        path: vec![],
    };
    let target = Transform::from_xyz(200., 90., 0.);
    let player = Transform::from_xyz(-200., 90., 0.);
    let path = PlayerPath {
        segments: vec![MotionSegment {
            start: Vec3::new(200., 90., 0.),
            end: player.translation,
            from: 0.,
            to: 1.,
            half: crate::arena::DRONE_HALF_EXTENTS,
        }],
    };
    assert!(
        crate::combat::variant_contact::contact(
            &enemy,
            &target,
            &player,
            Some(&path),
            &config,
            None
        )
        .is_some()
    );
    let wall = WorldGeometry {
        solids: vec![Solid {
            center: Vec3::new(0., 150., 0.),
            half: Vec3::new(1., 150., 100.),
        }],
        hazard: None,
    };
    let enemy = Enemy {
        previous: Vec3::new(-16., 90., 0.),
        ..enemy
    };
    assert!(
        crate::combat::variant_contact::contact(
            &enemy,
            &Transform::from_xyz(-16., 90., 0.),
            &Transform::from_xyz(37., 90., 0.),
            None,
            &config,
            Some(&wall)
        )
        .is_none()
    );
}

#[test]
fn new_enemy_flight_and_committed_charges_respect_solid_geometry_at_all_rates() {
    use crate::world::{Solid, WorldGeometry};
    for hz in [30, 60, 144] {
        for kind in [EnemyKind::Fast, EnemyKind::Rammer] {
            let mut app = app();
            app.insert_resource(WorldGeometry {
                solids: vec![Solid {
                    center: Vec3::new(-100., 150., 0.),
                    half: Vec3::new(10., 150., 500.),
                }],
                hazard: None,
            });
            let id = spawn(&mut app, kind, Vec3::new(-250., 90., 0.));
            if let Some(mut ram) = app.world_mut().get_mut::<Rammer>(id) {
                ram.phase = RamPhase::Charge;
                ram.target = Vec3::new(240., 90., 0.);
            }
            for _ in 0..hz * 3 {
                step(&mut app, 1. / hz as f32);
                let t = app.world().get::<Transform>(id).unwrap();
                let half = crate::arena::world_half_extents(t.rotation, Vec3::splat(14.));
                assert!(
                    app.world()
                        .resource::<WorldGeometry>()
                        .solids
                        .iter()
                        .all(|s| !s.overlaps(t.translation, half)),
                    "{kind:?} at {hz} Hz"
                );
            }
            assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
        }
    }
}

#[test]
fn ordinary_waves_have_no_variant_roster() {
    let mut app = app();
    assert!(!app.world().contains_resource::<SpawnRoster>());
    app.world_mut().resource_mut::<WaveConfig>().bursts = vec![(0., 3)];
    for _ in 0..120 {
        step(&mut app, 1. / 60.);
    }
    let kinds: Vec<_> = app
        .world_mut()
        .query::<&Enemy>()
        .iter(app.world())
        .map(|e| e.kind)
        .collect();
    assert_eq!(kinds.len(), 3);
    assert!(kinds.iter().all(|k| *k == EnemyKind::Chaser));
}

#[test]
fn rammer_replans_retreat_when_player_follows_to_its_destination() {
    let config = VariantConfig::default();
    let arena = Arena::default();
    let mut ram = Rammer {
        phase: RamPhase::Charge,
        ..default()
    };
    ram.impact(true);
    let destination = ram.plan(
        Vec3::new(0., 90., 0.),
        Vec3::new(-50., 90., 0.),
        0.1,
        &config,
        Vec3::splat(14.),
        &arena,
        None,
    );
    let player = destination + Vec3::Z * 50.;
    let next = ram.plan(
        destination,
        player,
        2.,
        &config,
        Vec3::splat(14.),
        &arena,
        None,
    );
    assert_eq!(ram.phase, RamPhase::Retreat);
    assert!(
        next.distance(destination) > 100.,
        "rammer must keep retreating after the player follows it"
    );
    assert!(next.distance(player) >= config.rearm_distance);
    assert_eq!(ram.impacts, 1);
}

#[test]
fn missed_charge_retreats_without_spending_budget_and_terminal_states_freeze_flight() {
    let config = VariantConfig::default();
    let mut ram = Rammer {
        phase: RamPhase::Charge,
        elapsed: config.charge_seconds,
        ..default()
    };
    ram.plan(
        Vec3::new(200., 90., 0.),
        Vec3::new(0., 90., 0.),
        0.1,
        &config,
        Vec3::splat(14.),
        &Arena::default(),
        None,
    );
    assert_eq!(ram.phase, RamPhase::Retreat);
    assert_eq!(ram.impacts, 0);
    for phase in [GamePhase::Dead, GamePhase::Survived] {
        let mut app = app();
        let id = spawn(&mut app, EnemyKind::Rammer, Vec3::new(-200., 90., 0.));
        step(&mut app, 1. / 60.);
        let before = app.world().get::<Transform>(id).unwrap().translation;
        let elapsed = app.world().get::<Rammer>(id).unwrap().elapsed;
        *app.world_mut().resource_mut::<GamePhase>() = phase;
        step(&mut app, 3.);
        assert_eq!(app.world().get::<Rammer>(id).unwrap().elapsed, elapsed);
        assert_eq!(
            app.world().get::<Transform>(id).unwrap().translation,
            before
        );
    }
}

#[test]
fn rammer_retreat_can_leave_a_wall_contact_with_its_physical_hull() {
    let world = crate::world::WorldGeometry {
        solids: vec![crate::world::Solid {
            center: Vec3::new(0., 150., 0.),
            half: Vec3::new(10., 150., 500.),
        }],
        hazard: None,
    };
    let position = Vec3::new(-25., 90., 0.);
    let player = Vec3::new(-80., 90., 0.);
    let goal = retreat_target(
        position,
        player,
        240.,
        Vec3::splat(14.),
        &Arena::default(),
        Some(&world),
    );
    assert!(
        goal.distance(position) > 100.,
        "retreat must escape a wall contact, got {goal:?}"
    );
}

#[test]
fn following_a_retreating_rammer_still_allows_a_second_physical_warning() {
    for hz in [30, 60, 144] {
        let mut app = app();
        let id = spawn(&mut app, EnemyKind::Rammer, Vec3::new(-250., 90., 0.));
        let mut followed = false;
        let mut rearmed = false;
        for _ in 0..hz * 25 {
            step(&mut app, 1. / hz as f32);
            let ram = app.world().get::<Rammer>(id).unwrap();
            if !followed && let Some(goal) = ram.retreat_goal {
                let mut drone = app
                    .world_mut()
                    .query_filtered::<&mut Transform, With<Drone>>();
                drone.single_mut(app.world_mut()).unwrap().translation = goal + Vec3::Z * 50.;
                followed = true;
            } else if followed && ram.phase == RamPhase::Windup {
                let player = app
                    .world_mut()
                    .query_filtered::<&Transform, With<Drone>>()
                    .single(app.world())
                    .unwrap()
                    .translation;
                let enemy = app.world().get::<Transform>(id).unwrap().translation;
                assert!(
                    enemy.distance(player) >= 175.,
                    "rearmed without separation at {hz} Hz"
                );
                rearmed = true;
                break;
            }
        }
        assert!(followed && rearmed, "did not recover at {hz} Hz");
    }
}
