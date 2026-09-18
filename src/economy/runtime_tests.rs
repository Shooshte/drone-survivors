use super::*;
use bevy::ecs::system::RunSystemOnce;
use std::time::Duration;
fn fixture() -> App {
    let mut app = App::new();
    app.init_resource::<EconomyConfig>()
        .init_resource::<LootState>()
        .init_resource::<AttemptResources>()
        .init_resource::<CombatOutcomes>()
        .init_resource::<Time>()
        .init_resource::<MissionBoundary>()
        .insert_resource(GamePhase::Playing)
        .init_resource::<WorldGeometry>();
    app.world_mut()
        .spawn((Drone, Transform::from_xyz(0., 30., 0.)));
    app
}
fn run(app: &mut App) {
    app.world_mut().run_system_once(collect).unwrap();
}
fn pickup(app: &mut App, position: Vec3, cache: bool) -> Entity {
    let mut e = app.world_mut().spawn((
        Pickup {
            amount: Amounts {
                salvage: if cache { 0 } else { 1 },
                components: if cache { 1 } else { 0 },
            },
            radius: if cache { 50. } else { 100. },
            attracted: false,
        },
        Transform::from_translation(position),
    ));
    if cache {
        e.insert(ComponentCache);
    }
    e.id()
}
fn count(app: &mut App) -> usize {
    app.world_mut().query::<&Pickup>().iter(app.world()).count()
}

#[test]
fn new_enemy_kill_kinds_use_normal_drop_rules_without_duplicate_rewards() {
    let mut app = fixture();
    app.world_mut()
        .resource_mut::<EconomyConfig>()
        .chaser
        .chance_percent = 100;
    for kind in [EnemyKind::Fast, EnemyKind::Rammer] {
        let id = app.world_mut().spawn_empty().id();
        for _ in 0..2 {
            app.world_mut()
                .resource_mut::<CombatOutcomes>()
                .0
                .push(CombatOutcome::Hit {
                    entity: id,
                    kind,
                    position: Vec3::new(-300., 90., 0.),
                    killed: true,
                });
        }
    }
    app.world_mut().run_system_once(spawn_drops).unwrap();
    assert_eq!(count(&mut app), 2);
    app.world_mut().run_system_once(spawn_drops).unwrap();
    assert_eq!(count(&mut app), 2);
}
fn advance(app: &mut App, secs: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(secs));
}

#[test]
fn configured_enemy_rolls_and_duplicate_deaths_are_bounded() {
    let rule = DropRule {
        chance_percent: 25,
        amount: 1,
    };
    assert_eq!((0..100).map(|r| rule.amount_for_roll(r)).sum::<u64>(), 25);
    assert_eq!(
        DropRule {
            chance_percent: 50,
            amount: 3
        }
        .amount_for_roll(49),
        3
    );
    assert_eq!(
        DropRule {
            chance_percent: 50,
            amount: 3
        }
        .amount_for_roll(50),
        0
    );
    let mut app = fixture();
    app.world_mut()
        .resource_mut::<EconomyConfig>()
        .chaser
        .chance_percent = 100;
    let id = app.world_mut().spawn_empty().id();
    app.world_mut()
        .resource_mut::<CombatOutcomes>()
        .0
        .push(CombatOutcome::Hit {
            entity: id,
            position: Vec3::new(200., 90., 0.),
            killed: true,
            kind: EnemyKind::Chaser,
        });
    for _ in 0..3 {
        app.world_mut().run_system_once(spawn_drops).unwrap();
    }
    assert_eq!(count(&mut app), 1);
    let position = app
        .world_mut()
        .query_filtered::<&Transform, With<Pickup>>()
        .single(app.world())
        .unwrap()
        .translation;
    assert_eq!(position, Vec3::new(200., 6., 0.));
    assert_eq!(app.world().resource::<LootState>().seen.len(), 1);
    app.world_mut()
        .resource_mut::<EconomyConfig>()
        .chaser
        .chance_percent = 0;
    let id = app.world_mut().spawn_empty().id();
    app.world_mut()
        .resource_mut::<CombatOutcomes>()
        .0
        .push(CombatOutcome::Hit {
            entity: id,
            position: Vec3::ZERO,
            killed: true,
            kind: EnemyKind::Chaser,
        });
    app.world_mut().run_system_once(spawn_drops).unwrap();
    assert_eq!(count(&mut app), 1);
}
#[test]
fn salvage_attracts_in_three_dimensions_and_only_credits_on_arrival() {
    let mut app = fixture();
    let near = pickup(&mut app, Vec3::new(80., 6., 0.), false);
    let far = pickup(&mut app, Vec3::new(0., 131., 0.), false);
    advance(&mut app, 0.01);
    run(&mut app);
    assert!(app.world().get::<Pickup>(near).unwrap().attracted);
    assert!(!app.world().get::<Pickup>(far).unwrap().attracted);
    assert_eq!(
        app.world().resource::<AttemptResources>().collected.salvage,
        0
    );
    advance(&mut app, 1.);
    run(&mut app);
    run(&mut app);
    assert!(app.world().get_entity(near).is_err());
    assert_eq!(
        app.world().resource::<AttemptResources>().collected.salvage,
        1
    );
    assert!(app.world().get_entity(far).is_ok());
}
#[test]
fn pickups_do_not_expire_and_freeze_during_choices_or_terminal_frames() {
    let mut app = fixture();
    let id = pickup(&mut app, Vec3::new(-400., 6., 0.), false);
    advance(&mut app, 300.);
    run(&mut app);
    assert!(app.world().get_entity(id).is_ok());
    *app.world_mut().get_mut::<Transform>(id).unwrap() = Transform::from_xyz(0., 6., 0.);
    for phase in [
        GamePhase::Choosing,
        GamePhase::Dead,
        GamePhase::Survived,
        GamePhase::Hub,
        GamePhase::Briefing,
        GamePhase::MissionSelect,
    ] {
        *app.world_mut().resource_mut::<GamePhase>() = phase;
        run(&mut app);
        assert_eq!(
            app.world().resource::<AttemptResources>().collected,
            Amounts::default()
        );
        assert_eq!(
            app.world().get::<Transform>(id).unwrap().translation,
            Vec3::new(0., 6., 0.)
        );
    }
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Playing;
    app.world_mut().resource_mut::<MissionBoundary>().reset = true;
    run(&mut app);
    assert_eq!(
        app.world().resource::<AttemptResources>().collected,
        Amounts::default()
    );
}
#[test]
fn caches_collect_once_at_radius_and_solid_cover_blocks_collection() {
    let mut app = fixture();
    let near = pickup(&mut app, Vec3::new(0., 30., 50.), true);
    let far = pickup(&mut app, Vec3::new(0., 30., 50.01), true);
    run(&mut app);
    run(&mut app);
    assert!(app.world().get_entity(near).is_err());
    assert!(app.world().get_entity(far).is_ok());
    assert_eq!(
        app.world()
            .resource::<AttemptResources>()
            .collected
            .components,
        1
    );
    app.world_mut()
        .resource_mut::<WorldGeometry>()
        .solids
        .push(crate::world::Solid {
            center: Vec3::new(0., 30., 25.),
            half: Vec3::new(10., 30., 1.),
        });
    let blocked = pickup(&mut app, Vec3::new(0., 30., 49.), false);
    advance(&mut app, 1.);
    run(&mut app);
    assert!(!app.world().get::<Pickup>(blocked).unwrap().attracted);
}
#[test]
fn drop_projection_rests_on_floor_or_top_of_cover() {
    let world = WorldGeometry::default();
    assert_eq!(ground_position(Vec3::new(0., 200., 0.), Some(&world)).y, 6.);
    let above = crate::world::layout::LOW_COVER_LEFT + Vec3::Y * 100.;
    assert_eq!(ground_position(above, Some(&world)).y, 66.);
}
#[test]
fn authored_caches_are_reachable_and_reset_without_accumulation() {
    let mut app = crate::mission::tests::app();
    app.world_mut().init_resource::<WorldGeometry>();
    for _ in 0..3 {
        if *app.world().resource::<GamePhase>() == GamePhase::Hub {
            crate::mission::tests::launch(&mut app);
        } else {
            crate::mission::tests::tick(&mut app, 0., &[KeyCode::KeyR]);
        }
        assert_eq!(count(&mut app), 3);
        let world = WorldGeometry::default();
        for position in app.world().resource::<EconomyConfig>().caches {
            let target = position + Vec3::Y * 20.;
            assert!(world.clear_body(target, target, crate::arena::DRONE_HALF_EXTENTS));
            assert!(
                crate::world::navigation::next_point(
                    &world,
                    crate::arena::DRONE_START.translation,
                    target,
                    crate::arena::DRONE_HALF_EXTENTS
                )
                .is_some()
            );
        }
        let drone = app
            .world_mut()
            .query_filtered::<Entity, With<Drone>>()
            .single(app.world())
            .unwrap();
        for position in app.world().resource::<EconomyConfig>().caches {
            app.world_mut()
                .get_mut::<Transform>(drone)
                .unwrap()
                .translation = position + Vec3::Y * 20.;
            crate::mission::tests::tick(&mut app, 0., &[]);
            if *app.world().resource::<GamePhase>() == GamePhase::Choosing {
                crate::mission::tests::tick(&mut app, 0., &[]);
                crate::mission::tests::tick(&mut app, 0., &[KeyCode::Backspace]);
                crate::mission::tests::tick(&mut app, 0., &[]);
            }
        }
        assert_eq!(
            app.world()
                .resource::<AttemptResources>()
                .collected
                .components,
            3
        );
        assert_eq!(count(&mut app), 0);
    }
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
    crate::mission::tests::tick(&mut app, 0., &[]);
    assert_eq!(count(&mut app), 0);
    assert_eq!(
        app.world()
            .resource::<crate::mission::Campaign>()
            .wallet
            .components,
        0
    );
}

#[test]
fn seeded_loot_sequence_repeats_after_restart_and_duplicate_facts_do_not_shift_it() {
    let mut app = fixture();
    let mut sequences = Vec::new();
    for duplicate in [false, true] {
        app.world_mut().run_system_once(reset).unwrap();
        let mut sequence = Vec::new();
        for index in 0..32 {
            let id = app.world_mut().spawn_empty().id();
            let outcome = || CombatOutcome::Hit {
                entity: id,
                position: Vec3::new(index as f32 * 10., 90., 200.),
                killed: true,
                kind: EnemyKind::Chaser,
            };
            app.world_mut().resource_mut::<CombatOutcomes>().0 = vec![outcome()];
            if duplicate {
                app.world_mut()
                    .resource_mut::<CombatOutcomes>()
                    .0
                    .push(outcome());
            }
            let before = count(&mut app);
            app.world_mut().run_system_once(spawn_drops).unwrap();
            sequence.push(count(&mut app) - before);
        }
        assert!(sequence.contains(&0) && sequence.contains(&1));
        sequences.push(sequence);
    }
    assert_eq!(sequences[0], sequences[1]);
}

#[path = "discovery_tests.rs"]
mod discoveries;
