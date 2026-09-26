//! Authored encounters and optional rewards for the schematic payload mission.
use super::{
    CombatConfig, Enemy, SpawnWarning,
    enemies::spawn_enemy_kind,
    waves::{safe, spawn_half},
};
use crate::{
    arena::{Arena, Drone},
    economy::{Amounts, AttemptResources, runtime::EnemyKind},
    game::{GamePhase, GameplaySet},
    mission::{
        blockout::*,
        objectives::{ObjectiveKind, ObjectiveRun},
    },
    world::{MotionSegment, PlayerPath, WorldGeometry, mission01 as map, proximity_contact},
};
use bevy::prelude::*;
use std::collections::VecDeque;

const CAP: usize = 96;
const ACTIVATION_RADIUS: f32 = 1200.;
const SOURCE_INTERVAL: f64 = 5.;
const SOURCE_BATCH: usize = 4;
const HOLDOUT_WAVE: usize = 6;
const ANCHORS: [(f32, f32); 8] = [
    (640., 825.),
    (65., 470.),
    (515., 690.),
    (685., 115.),
    (860., 385.),
    (235., 855.),
    (80., 85.),
    (960., 65.),
];

#[derive(Resource, Default)]
struct MissionEncounters {
    activated: [bool; 5],
    // Lanes 0..6 retain every authored request. Repeat lanes 6..8 coalesce
    // missed intervals into at most one batch each; they never accrue a debt.
    pending: [VecDeque<EnemyKind>; 8],
    cursor: usize,
    candidates: [usize; 8],
    source_elapsed: Option<f64>,
    reward_credited: bool,
}
pub(super) fn install(app: &mut App) {
    app.init_resource::<MissionEncounters>()
        .add_systems(
            Update,
            reset
                .in_set(GameplaySet::Reset)
                .run_if(crate::game::reset_requested),
        )
        .add_systems(
            Update,
            update
                .after(super::rockets::fire)
                .in_set(GameplaySet::Combat)
                .run_if(crate::game::is_playing)
                .run_if(not(crate::game::reset_requested)),
        );
}
fn reset(mut state: ResMut<MissionEncounters>) {
    *state = default();
}
fn contact(
    segment: &MotionSegment,
    center: Vec3,
    radius: f32,
    world: &WorldGeometry,
) -> Option<f32> {
    proximity_contact(
        segment.start.with_y(center.y),
        segment.end.with_y(center.y),
        center,
        radius,
        Some(world),
    )
}
fn inside(p: Vec3, center: Vec3, radius: f32) -> bool {
    (p - center).with_y(0.).length_squared() <= radius * radius + 0.01
}
fn advance_holdout(
    run: &mut BlockoutRun,
    state: &mut MissionEncounters,
    segments: &[MotionSegment],
    seconds: f64,
    world: &WorldGeometry,
) {
    let center = map::challenge();
    for segment in segments {
        if matches!(run.holdout, Holdout::Forfeited | Holdout::Complete) {
            break;
        }
        let mut enter = 0.;
        if run.holdout == Holdout::Available {
            let Some(t) = contact(segment, center, HOLDOUT_RADIUS, world) else {
                continue;
            };
            enter = t;
            run.holdout = Holdout::Active {
                elapsed: 0.,
                waves: 0,
            };
        } else if !inside(segment.start, center, HOLDOUT_RADIUS) {
            run.holdout = Holdout::Forfeited;
            break;
        }
        let exiting = !inside(segment.end, center, HOLDOUT_RADIUS);
        let exit = if exiting {
            // Reverse entry is forward exit. LOS was already validated at
            // activation; crossing the circular boundary always consumes it.
            1. - proximity_contact(
                segment.end.with_y(center.y),
                segment.start.with_y(center.y),
                center,
                HOLDOUT_RADIUS,
                None,
            )
            .unwrap_or(1.)
        } else {
            1.
        };
        if let Holdout::Active {
            mut elapsed,
            mut waves,
        } = run.holdout
        {
            elapsed += seconds * f64::from((segment.to - segment.from) * (exit - enter).max(0.));
            while waves < 3 && elapsed + 1e-7 >= waves as f64 * 10. {
                state.pending[5].extend(std::iter::repeat_n(EnemyKind::Chaser, HOLDOUT_WAVE));
                waves += 1;
            }
            run.holdout = if elapsed + 1e-7 >= HOLDOUT_SECONDS {
                Holdout::Complete
            } else if exiting {
                Holdout::Forfeited
            } else {
                Holdout::Active { elapsed, waves }
            };
        }
    }
}
fn credit(resources: &mut AttemptResources, components: u64) -> bool {
    resources
        .collected
        .try_credit(Amounts {
            components,
            ..default()
        })
        .is_ok()
}
type Occupants<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        Option<&'static Enemy>,
        Option<&'static SpawnWarning>,
    ),
    Or<(With<Enemy>, With<SpawnWarning>)>,
>;
#[allow(clippy::too_many_arguments)]
fn update(
    mut commands: Commands,
    time: Res<Time>,
    phase: Res<GamePhase>,
    run: Option<ResMut<BlockoutRun>>,
    mut state: ResMut<MissionEncounters>,
    objective: Option<Res<ObjectiveRun>>,
    resources: Option<ResMut<AttemptResources>>,
    path: Option<Res<PlayerPath>>,
    world: Option<Res<WorldGeometry>>,
    arena: Res<Arena>,
    combat: Res<CombatConfig>,
    player: Single<&Transform, With<Drone>>,
    occupants: Occupants,
) {
    let (Some(mut run), Some(mut resources), Some(world)) = (run, resources, world) else {
        return;
    };
    if !run.enabled || *phase != GamePhase::Playing {
        return;
    }
    let fallback = [MotionSegment {
        start: player.translation,
        end: player.translation,
        from: 0.,
        to: 1.,
        half: Vec3::ZERO,
    }];
    let segments = path
        .as_ref()
        .filter(|p| !p.segments.is_empty())
        .map_or(&fallback[..], |p| p.segments.as_slice());
    for (i, (east, south)) in ANCHORS.iter().enumerate().take(5) {
        if !state.activated[i]
            && segments
                .iter()
                .any(|s| contact(s, map::point(*east, *south), ACTIVATION_RADIUS, &world).is_some())
        {
            state.activated[i] = true;
            let chasers = match i {
                0 | 1 => 10,
                2 => 20,
                _ => 5,
            };
            state.pending[i].extend(std::iter::repeat_n(EnemyKind::Chaser, chasers));
            if i >= 3 {
                state.pending[i].extend([EnemyKind::Slower; 2]);
            }
        }
    }
    advance_holdout(
        &mut run,
        &mut state,
        segments,
        time.delta_secs_f64(),
        &world,
    );
    if run.holdout == Holdout::Complete
        && !state.reward_credited
        && credit(&mut resources, HOLDOUT_COMPONENTS)
    {
        state.reward_credited = true;
    }
    if !run.hidden_collected
        && segments
            .iter()
            .any(|s| contact(s, map::hidden(), HIDDEN_RADIUS, &world).is_some())
        && credit(&mut resources, HIDDEN_COMPONENTS)
    {
        run.hidden_collected = true;
    }
    if objective
        .as_ref()
        .is_some_and(|o| o.kind == ObjectiveKind::Payload && o.ready())
    {
        let due = match state.source_elapsed.as_mut() {
            None => {
                state.source_elapsed = Some(0.);
                true
            }
            Some(elapsed) => {
                *elapsed += time.delta_secs_f64();
                if *elapsed >= SOURCE_INTERVAL {
                    *elapsed %= SOURCE_INTERVAL;
                    true
                } else {
                    false
                }
            }
        };
        if due {
            for (lane, pending) in state.pending.iter_mut().enumerate().skip(6) {
                // Refill only an empty repeat batch, retaining its mixed order
                // when capacity is scarce instead of dropping the slower.
                if pending.is_empty() {
                    for i in 0..SOURCE_BATCH {
                        pending.push_back(if lane == 7 && i == SOURCE_BATCH - 1 {
                            EnemyKind::Slower
                        } else {
                            EnemyKind::Chaser
                        });
                    }
                }
            }
        }
    }
    let half = spawn_half(&combat);
    let mut occupied: Vec<_> = occupants
        .iter()
        .filter(|(_, _, enemy, warning)| {
            enemy.is_some_and(|e| e.health > 0) || warning.is_some_and(|w| !w.cancelled)
        })
        .map(|(id, t, _, _)| (id, t.translation, half))
        .collect();
    // At most 96 admissions, with at most 48 candidates checked per attempt.
    // Stop after eight unsuccessful lanes. Direct
    // admission has 220u drone clearance, shared terrain/body checks and the
    // map's visible dormant/source zones. No graph is built in this loop.
    let mut misses = 0;
    while occupied.len() < CAP && misses < 8 {
        let lane = state.cursor;
        state.cursor = (lane + 1) % 8;
        let Some(&kind) = state.pending[lane].front() else {
            misses += 1;
            continue;
        };
        let center = map::point(ANCHORS[lane].0, ANCHORS[lane].1);
        let mut selected = None;
        for _ in 0..48 {
            let n = state.candidates[lane];
            state.candidates[lane] = (n + 1) % 48;
            let angle = (n % 16) as f32 * std::f32::consts::TAU / 16.;
            let radius = 360. + (n / 16) as f32 * 180.;
            let at = center + Vec3::new(angle.cos() * radius, 0., angle.sin() * radius);
            if safe(
                at,
                half,
                &arena,
                &player,
                220.,
                &occupied,
                None,
                Some(&world),
            ) {
                selected = Some(at);
                break;
            }
        }
        if let Some(at) = selected {
            state.pending[lane].pop_front();
            spawn_enemy_kind(&mut commands, &combat, at, player.translation, kind);
            occupied.push((Entity::PLACEHOLDER, at, half));
            misses = 0;
        } else {
            misses += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        arena::Drone,
        combat::{CombatConfig, Enemy, SpawnWarning},
        economy::AttemptResources,
        game::GamePhase,
        mission::{
            blockout::{BlockoutRun, Holdout},
            objectives::{ObjectiveKind, ObjectiveRun},
        },
        world::{MotionSegment, PlayerPath, mission01 as map},
    };
    fn app() -> (App, Entity) {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .insert_resource(GamePhase::Playing)
            .insert_resource(BlockoutRun {
                enabled: true,
                ..default()
            })
            .init_resource::<MissionEncounters>()
            .init_resource::<AttemptResources>()
            .init_resource::<PlayerPath>()
            .insert_resource(ObjectiveRun {
                kind: ObjectiveKind::Payload,
                ..default()
            })
            .insert_resource(CombatConfig::default())
            .insert_resource(map::arena())
            .insert_resource(map::geometry())
            .add_systems(Update, update.run_if(crate::game::is_playing));
        let drone = app
            .world_mut()
            .spawn((Drone, Transform::from_translation(map::start())))
            .id();
        (app, drone)
    }
    fn tick(app: &mut App, dt: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(dt));
        app.update();
    }
    fn at(app: &mut App, drone: Entity, p: Vec3) {
        app.world_mut()
            .get_mut::<Transform>(drone)
            .unwrap()
            .translation = p;
        app.world_mut()
            .resource_mut::<PlayerPath>()
            .segments
            .clear();
    }
    fn enemies(app: &mut App) -> usize {
        app.world_mut().query::<&Enemy>().iter(app.world()).count()
    }
    #[test]
    fn fixed_groups_activate_once_with_exact_counts_and_mixtures() {
        let (mut app, drone) = app();
        tick(&mut app, 1.);
        assert_eq!(enemies(&mut app), 0);
        for (p, total) in [
            (map::point(640., 825.), 10),
            (map::point(65., 470.), 20),
            (map::point(515., 690.), 40),
            (map::point(685., 115.), 47),
            (map::point(860., 385.), 54),
        ] {
            at(&mut app, drone, p);
            for _ in 0..8 {
                tick(&mut app, 1.);
            }
            assert_eq!(enemies(&mut app), total);
        }
        assert_eq!(
            app.world_mut()
                .query::<&Enemy>()
                .iter(app.world())
                .filter(|e| e.kind == crate::economy::runtime::EnemyKind::Slower)
                .count(),
            4
        );
    }
    #[test]
    fn holdout_three_waves_survivors_pause_and_single_reward() {
        let (mut app, drone) = app();
        at(&mut app, drone, map::challenge());
        tick(&mut app, 1.);
        assert!(matches!(
            app.world().resource::<BlockoutRun>().holdout,
            Holdout::Active { waves: 1, .. }
        ));
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Choosing;
        tick(&mut app, 50.);
        assert!(
            matches!(app.world().resource::<BlockoutRun>().holdout,Holdout::Active{elapsed,waves:1} if elapsed==1.)
        );
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Playing;
        tick(&mut app, 29.);
        assert_eq!(
            app.world().resource::<BlockoutRun>().holdout,
            Holdout::Complete
        );
        assert_eq!(enemies(&mut app), 18);
        assert_eq!(
            app.world()
                .resource::<AttemptResources>()
                .collected
                .components,
            5
        );
        tick(&mut app, 30.);
        assert_eq!(
            app.world()
                .resource::<AttemptResources>()
                .collected
                .components,
            5
        );
    }
    fn segment(a: Vec3, b: Vec3, from: f32, to: f32) -> MotionSegment {
        MotionSegment {
            start: a,
            end: b,
            from,
            to,
            half: Vec3::ZERO,
        }
    }
    #[test]
    fn swept_outside_crossing_and_inside_excursion_consume_holdout() {
        for excursion in [false, true] {
            let (mut app, drone) = app();
            let c = map::challenge();
            let outside = c + Vec3::X * 3000.;
            let paths = if excursion {
                vec![segment(c, outside, 0., 0.5), segment(outside, c, 0.5, 1.)]
            } else {
                vec![segment(c - Vec3::X * 3000., outside, 0., 1.)]
            };
            at(&mut app, drone, if excursion { c } else { outside });
            app.world_mut().resource_mut::<PlayerPath>().segments = paths;
            tick(&mut app, 1.);
            assert_eq!(
                app.world().resource::<BlockoutRun>().holdout,
                Holdout::Forfeited
            );
            at(&mut app, drone, c);
            tick(&mut app, 60.);
            assert_eq!(
                app.world()
                    .resource::<AttemptResources>()
                    .collected
                    .components,
                0
            );
        }
    }
    #[test]
    fn hidden_reward_sweeps_once_and_does_not_wrap_overflow() {
        let (mut app, drone) = app();
        let c = map::hidden();
        app.world_mut()
            .resource_mut::<AttemptResources>()
            .collected
            .components = u64::MAX;
        at(&mut app, drone, c);
        tick(&mut app, 1.);
        assert!(!app.world().resource::<BlockoutRun>().hidden_collected);
        app.world_mut()
            .resource_mut::<AttemptResources>()
            .collected
            .components = 0;
        at(&mut app, drone, c + Vec3::X * 500.);
        app.world_mut().resource_mut::<PlayerPath>().segments =
            vec![segment(c - Vec3::X * 500., c + Vec3::X * 500., 0., 1.)];
        tick(&mut app, 1.);
        tick(&mut app, 1.);
        assert_eq!(
            app.world()
                .resource::<AttemptResources>()
                .collected
                .components,
            3
        );
    }
    #[test]
    fn saturated_cap_retains_fixed_requests_until_slots_return() {
        let (mut app, drone) = app();
        let warnings: Vec<_> = (0..96)
            .map(|_| {
                app.world_mut()
                    .spawn((
                        SpawnWarning::default(),
                        Transform::from_translation(map::start()),
                    ))
                    .id()
            })
            .collect();
        at(&mut app, drone, map::point(515., 690.));
        tick(&mut app, 1.);
        assert_eq!(enemies(&mut app), 0);
        at(&mut app, drone, map::start());
        for id in warnings {
            app.world_mut().despawn(id);
        }
        for _ in 0..10 {
            tick(&mut app, 1.);
        }
        assert_eq!(enemies(&mut app), 20);
    }
    #[test]
    fn both_repeat_sources_keep_emitting_fairly_with_one_free_slot() {
        let (mut app, _) = app();
        for _ in 0..95 {
            app.world_mut().spawn((
                SpawnWarning::default(),
                Transform::from_translation(map::start()),
            ));
        }
        app.world_mut().resource_mut::<ObjectiveRun>().visited[0] = true;
        let mut counts = [0, 0];
        let mut slowers = 0;
        for _ in 0..1000 {
            tick(&mut app, 5.);
            let spawned: Vec<_> = app
                .world_mut()
                .query::<(Entity, &Enemy, &Transform)>()
                .iter(app.world())
                .map(|(id, e, t)| (id, e.kind, t.translation))
                .collect();
            assert_eq!(spawned.len(), 1);
            assert!(
                app.world()
                    .resource::<MissionEncounters>()
                    .pending
                    .iter()
                    .map(VecDeque::len)
                    .sum::<usize>()
                    <= 8
            );
            for (id, kind, p) in spawned {
                counts[usize::from(p.x > 0.)] += 1;
                slowers += usize::from(kind == crate::economy::runtime::EnemyKind::Slower);
                app.world_mut().despawn(id);
            }
        }
        assert_eq!(counts, [500, 500]);
        assert!(slowers > 50);
    }

    #[test]
    fn temporary_space_block_keeps_every_fixed_request() {
        let (mut app, drone) = app();
        let center = map::point(515., 690.);
        let warnings: Vec<_> = (0..96)
            .map(|_| {
                app.world_mut()
                    .spawn((
                        SpawnWarning::default(),
                        Transform::from_translation(map::start()),
                    ))
                    .id()
            })
            .collect();
        at(&mut app, drone, center);
        tick(&mut app, 1.);
        for id in warnings {
            app.world_mut().despawn(id);
        }
        at(&mut app, drone, map::start());
        app.world_mut()
            .resource_mut::<WorldGeometry>()
            .solids
            .push(crate::world::Solid {
                center,
                half: Vec3::new(1000., 300., 1000.),
            });
        for _ in 0..10 {
            tick(&mut app, 1.);
        }
        assert_eq!(enemies(&mut app), 0);
        app.world_mut().resource_mut::<WorldGeometry>().solids.pop();
        tick(&mut app, 1.);
        assert_eq!(enemies(&mut app), 20);
    }
    #[test]
    fn holdout_entry_credits_only_inside_fraction_and_retry_overflow_once() {
        let (mut app, drone) = app();
        let c = map::challenge();
        at(&mut app, drone, c);
        app.world_mut().resource_mut::<PlayerPath>().segments =
            vec![segment(c + Vec3::X * 5600., c, 0., 1.)];
        tick(&mut app, 10.);
        assert!(
            matches!(app.world().resource::<BlockoutRun>().holdout,Holdout::Active{elapsed,waves:1} if (elapsed-5.).abs()<0.0001)
        );
        app.world_mut()
            .resource_mut::<AttemptResources>()
            .collected
            .components = u64::MAX;
        at(&mut app, drone, c);
        tick(&mut app, 25.);
        assert_eq!(
            app.world().resource::<BlockoutRun>().holdout,
            Holdout::Complete
        );
        assert_eq!(
            app.world()
                .resource::<AttemptResources>()
                .collected
                .components,
            u64::MAX
        );
        app.world_mut()
            .resource_mut::<AttemptResources>()
            .collected
            .components = 0;
        tick(&mut app, 1.);
        tick(&mut app, 1.);
        assert_eq!(
            app.world()
                .resource::<AttemptResources>()
                .collected
                .components,
            5
        );
    }
    #[test]
    fn resetting_encounters_clears_activation_backlog_and_reward_receipt() {
        use bevy::ecs::system::RunSystemOnce;
        let (mut app, drone) = app();
        at(&mut app, drone, map::challenge());
        tick(&mut app, 30.);
        assert!(app.world().resource::<MissionEncounters>().reward_credited);
        app.world_mut().run_system_once(reset).unwrap();
        let state = app.world().resource::<MissionEncounters>();
        assert!(!state.reward_credited);
        assert_eq!(state.activated, [false; 5]);
        assert!(state.pending.iter().all(VecDeque::is_empty));
        assert_eq!(state.source_elapsed, None);
        // Baseline owns BlockoutRun and the economy resource reset.
        *app.world_mut().resource_mut::<BlockoutRun>() = BlockoutRun {
            enabled: true,
            ..default()
        };
        *app.world_mut().resource_mut::<AttemptResources>() = default();
        tick(&mut app, 30.);
        assert_eq!(
            app.world()
                .resource::<AttemptResources>()
                .collected
                .components,
            5
        );
    }
    #[test]
    fn mission01_full_lifecycle_source_kill_earns_xp_and_restart_restores_holdout() {
        use crate::mission::tests::{app as mission_app, launch, tick as frame};
        let mut app = mission_app();
        launch(&mut app);
        let drone = app
            .world_mut()
            .query_filtered::<Entity, With<Drone>>()
            .single(app.world())
            .unwrap();
        app.world_mut().resource_mut::<ObjectiveRun>().visited[0] = true;
        frame(&mut app, 0., &[]);
        assert_eq!(enemies(&mut app), 8);
        let (victim, position) = app
            .world_mut()
            .query::<(Entity, &Enemy, &Transform)>()
            .iter(app.world())
            .next()
            .map(|(id, _, t)| (id, t.translation))
            .unwrap();
        app.world_mut().get_mut::<Enemy>(victim).unwrap().health = 10;
        app.world_mut().spawn((
            super::super::Projectile {
                velocity: Vec3::ZERO,
                remaining: 1.,
            },
            Transform::from_translation(position),
        ));
        frame(&mut app, 0., &[]);
        assert_eq!(app.world().resource::<super::super::Encounter>().kills, 1);
        assert!(
            app.world()
                .resource::<crate::upgrades::UpgradeRun>()
                .total_xp
                > 0
        );
        at(&mut app, drone, map::challenge());
        frame(&mut app, 0., &[]);
        assert!(matches!(
            app.world().resource::<BlockoutRun>().holdout,
            Holdout::Active { .. }
        ));
        at(&mut app, drone, map::start());
        frame(&mut app, 0., &[]);
        assert_eq!(
            app.world().resource::<BlockoutRun>().holdout,
            Holdout::Forfeited
        );
        frame(&mut app, 0., &[KeyCode::KeyR]);
        assert_eq!(
            app.world().resource::<BlockoutRun>().holdout,
            Holdout::Available
        );
        assert_eq!(enemies(&mut app), 0);
        assert!(
            app.world()
                .resource::<MissionEncounters>()
                .pending
                .iter()
                .all(VecDeque::is_empty)
        );
        assert!(!app.world().resource::<ObjectiveRun>().ready());
        assert_eq!(
            app.world()
                .resource::<crate::upgrades::UpgradeRun>()
                .total_xp,
            0
        );
        at(&mut app, drone, map::challenge());
        frame(&mut app, 0., &[]);
        assert!(matches!(
            app.world().resource::<BlockoutRun>().holdout,
            Holdout::Active { waves: 1, .. }
        ));
    }
    #[test]
    #[ignore = "opt-in full-schedule 96-enemy performance probe"]
    fn mission01_population_cap_full_schedule_performance_probe() {
        use crate::mission::tests::{app as mission_app, launch, tick as frame};
        let mut app = mission_app();
        launch(&mut app);
        {
            let mut combat = app.world_mut().resource_mut::<CombatConfig>();
            combat.contact_damage = 0;
            combat.target_range = 0.;
        }
        app.world_mut().resource_mut::<ObjectiveRun>().visited[0] = true;
        for _ in 0..70 {
            frame(&mut app, 1., &[]);
        }
        assert_eq!(enemies(&mut app), CAP);
        let mut micros = Vec::new();
        for _ in 0..480 {
            let before = std::time::Instant::now();
            frame(&mut app, 1. / 60., &[]);
            micros.push(before.elapsed().as_micros());
            assert_eq!(enemies(&mut app), CAP);
            assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
            assert!(
                app.world()
                    .resource::<MissionEncounters>()
                    .pending
                    .iter()
                    .map(VecDeque::len)
                    .sum::<usize>()
                    <= 8
            );
        }
        micros.sort_unstable();
        eprintln!(
            "Mission01 full ECS schedule, 96 live enemies, 480 frames at 60Hz: min={}us p50={}us p95={}us max={}us",
            micros[0], micros[240], micros[456], micros[479]
        );
    }
}
