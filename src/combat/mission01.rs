//! Authored encounters and optional rewards for the schematic payload mission.
use super::{
    CombatConfig, Encounter, Enemy, SpawnWarning, WaveConfig,
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

const ACTIVATION_RADIUS: f32 = 1200.;
const LANES: usize = 9;
const PATROL_LANE: usize = 8;
const PATROL_INTERVAL: f64 = 8.;
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
    // Lanes 0..6 retain every authored request. Repeat lanes 6..9 coalesce
    // missed intervals into at most one batch each; they never accrue a debt.
    pending: [VecDeque<EnemyKind>; LANES],
    cursor: usize,
    candidates: [usize; LANES],
    source_elapsed: Option<f64>,
    source_next: f64,
    patrol_elapsed: f64,
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
    waves: Res<WaveConfig>,
    encounter: Res<Encounter>,
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
        let elapsed = state
            .source_elapsed
            .map_or(0., |t| t + time.delta_secs_f64());
        let due = state.source_elapsed.is_none() || elapsed + 1e-7 >= state.source_next;
        state.source_elapsed = Some(elapsed);
        if due {
            // Compute the next deadline directly: a long frame never loops over
            // missed intervals or carries more than one batch per source.
            let (start, end, interval) = if elapsed + 1e-7 < 20. {
                (0., 20., 5.)
            } else if elapsed + 1e-7 < 40. {
                (20., 40., 4.)
            } else if elapsed + 1e-7 < 60. {
                (40., 60., 3.)
            } else {
                (60., f64::INFINITY, 2.)
            };
            state.source_next =
                (start + ((elapsed - start + 1e-7) / interval).floor() * interval + interval)
                    .min(end);
            for (lane, pending) in state.pending.iter_mut().enumerate().take(8).skip(6) {
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
    let before = state.patrol_elapsed;
    state.patrol_elapsed += time.delta_secs_f64();
    let patrol_due = state.patrol_elapsed + 1e-7 >= 3.
        && (before + 1e-7 < 3.
            || ((before - 3. + 1e-7) / PATROL_INTERVAL).floor()
                < ((state.patrol_elapsed - 3. + 1e-7) / PATROL_INTERVAL).floor());
    if patrol_due && state.pending[PATROL_LANE].is_empty() {
        state.pending[PATROL_LANE].extend(std::iter::repeat_n(EnemyKind::Chaser, SOURCE_BATCH));
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
    // Stop after a full unsuccessful round. Direct
    // admission has 220u drone clearance, shared terrain/body checks and the
    // map's visible dormant/source zones. No graph is built in this loop.
    let mut misses = 0;
    while occupied.len() < ENEMY_CAP && misses < LANES {
        let lane = state.cursor;
        state.cursor = (lane + 1) % LANES;
        let Some(&kind) = state.pending[lane].front() else {
            misses += 1;
            continue;
        };
        let patrol = lane == PATROL_LANE;
        let center = if patrol {
            // The Scout can fly closer to floor/ceiling than a banking enemy
            // hull fits. Keep the candidate's full hull inside legal airspace.
            player.translation.with_y(player.translation.y.clamp(
                arena.center().y - arena.half_size.y + half.y + 1.,
                arena.center().y + arena.half_size.y - half.y - 1.,
            ))
        } else {
            map::point(ANCHORS[lane].0, ANCHORS[lane].1)
        };
        let mut selected = None;
        for _ in 0..48 {
            let n = state.candidates[lane];
            state.candidates[lane] = (n + 1) % 48;
            let angle = (n % 16) as f32 * std::f32::consts::TAU / 16.;
            let radius = if patrol {
                550. + (n / 16) as f32 * 150.
            } else {
                360. + (n / 16) as f32 * 180.
            };
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
            if patrol {
                commands.spawn((
                    SpawnWarning {
                        ready_at: encounter.elapsed + waves.warning_seconds,
                        kind,
                        ..default()
                    },
                    Transform::from_translation(at),
                ));
            } else {
                spawn_enemy_kind(&mut commands, &combat, at, player.translation, kind);
            }
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
            .insert_resource(WaveConfig::default())
            .init_resource::<Encounter>()
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
    fn patrol_requests_four_nearby_warnings_at_three_then_every_eight_seconds() {
        let (mut app, drone) = app();
        tick(&mut app, 2.5);
        assert_eq!(
            app.world_mut()
                .query::<&SpawnWarning>()
                .iter(app.world())
                .count(),
            0
        );
        tick(&mut app, 0.5);
        let first: Vec<_> = app
            .world_mut()
            .query::<(Entity, &SpawnWarning, &Transform)>()
            .iter(app.world())
            .map(|(id, w, t)| (id, w.kind, t.translation))
            .collect();
        assert_eq!(first.len(), 4);
        for (id, kind, p) in first {
            assert_eq!(kind, EnemyKind::Chaser);
            assert!((549.9..=850.1).contains(&p.distance(map::start())));
            assert!(safe(
                p,
                spawn_half(&CombatConfig::default()),
                &map::arena(),
                &Transform::from_translation(map::start()),
                220.,
                &[],
                None,
                Some(&map::geometry())
            ));
            app.world_mut().despawn(id);
        }
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Choosing;
        tick(&mut app, 100.);
        assert_eq!(
            app.world_mut()
                .query::<&SpawnWarning>()
                .iter(app.world())
                .count(),
            0
        );
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Playing;
        let next = map::start() - Vec3::X * 1000.;
        at(&mut app, drone, next);
        tick(&mut app, 7.5);
        assert_eq!(
            app.world_mut()
                .query::<&SpawnWarning>()
                .iter(app.world())
                .count(),
            0
        );
        tick(&mut app, 0.5);
        let points: Vec<_> = app
            .world_mut()
            .query::<(&SpawnWarning, &Transform)>()
            .iter(app.world())
            .map(|(_, t)| t.translation)
            .collect();
        assert_eq!(points.len(), 4);
        assert!(
            points
                .iter()
                .all(|p| (549.9..=850.1).contains(&p.distance(next)))
        );
    }

    #[test]
    fn patrols_fit_the_arena_even_when_player_flies_at_floor_or_ceiling() {
        for y in [10., 290.] {
            let (mut app, drone) = app();
            at(&mut app, drone, map::start().with_y(y));
            tick(&mut app, 3.);
            let warnings: Vec<_> = app
                .world_mut()
                .query_filtered::<&Transform, With<SpawnWarning>>()
                .iter(app.world())
                .map(|t| t.translation)
                .collect();
            assert_eq!(warnings.len(), 4, "patrol at player altitude {y}");
            let half = spawn_half(&CombatConfig::default());
            assert!(warnings.iter().all(|p| {
                (p - map::arena().center())
                    .abs()
                    .cmple(map::arena().half_size - half)
                    .all()
            }));
        }
    }

    #[test]
    fn payload_sources_accelerate_at_twenty_second_boundaries_without_hitch_debt() {
        let (mut app, _) = app();
        app.world_mut().resource_mut::<ObjectiveRun>().visited[0] = true;
        tick(&mut app, 0.);
        assert_eq!(enemies(&mut app), 8);
        for (dt, expected) in [
            (5., 8),
            (5., 8),
            (5., 8),
            (5., 8),
            (3.5, 0),
            (0.5, 8),
            (16., 8),
            (2.5, 0),
            (0.5, 8),
            (17., 8),
            (1.5, 0),
            (0.5, 8),
            (10000., 8),
        ] {
            let ids: Vec<_> = app
                .world_mut()
                .query_filtered::<Entity, Or<(With<Enemy>, With<SpawnWarning>)>>()
                .iter(app.world())
                .collect();
            for id in ids {
                app.world_mut().despawn(id);
            }
            tick(&mut app, dt);
            assert_eq!(enemies(&mut app), expected, "source interval delta={dt}");
        }
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    }

    #[test]
    fn near_boundary_clock_tolerance_never_reissues_a_batch_on_zero_delta() {
        let (mut app, _) = app();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f64(2.99999995));
        app.update();
        tick(&mut app, 0.);
        assert_eq!(
            app.world_mut()
                .query::<&SpawnWarning>()
                .iter(app.world())
                .count(),
            4
        );
        app.world_mut().resource_mut::<ObjectiveRun>().visited[0] = true;
        tick(&mut app, 0.);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f64(19.99999995));
        app.update();
        tick(&mut app, 0.);
        assert_eq!(enemies(&mut app), 16);
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
            let outside = c + Vec3::X * (HOLDOUT_RADIUS + 200.);
            let paths = if excursion {
                vec![segment(c, outside, 0., 0.5), segment(outside, c, 0.5, 1.)]
            } else {
                vec![segment(
                    c - Vec3::X * (HOLDOUT_RADIUS + 200.),
                    outside,
                    0.,
                    1.,
                )]
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
    fn both_sources_and_patrol_keep_emitting_fairly_with_one_free_slot() {
        let (mut app, _) = app();
        for _ in 0..95 {
            app.world_mut().spawn((
                SpawnWarning::default(),
                Transform::from_translation(map::start()),
            ));
        }
        app.world_mut().resource_mut::<ObjectiveRun>().visited[0] = true;
        let mut counts = [0, 0, 0];
        let mut slowers = 0;
        for _ in 0..999 {
            tick(&mut app, 5.);
            let spawned: Vec<_> = app
                .world_mut()
                .query::<(Entity, Option<&Enemy>, Option<&SpawnWarning>, &Transform)>()
                .iter(app.world())
                .filter_map(|(id, e, w, t)| {
                    e.map(|e| (id, e.kind, t.translation, false)).or_else(|| {
                        w.filter(|w| w.ready_at > 0.)
                            .map(|w| (id, w.kind, t.translation, true))
                    })
                })
                .collect();
            assert_eq!(spawned.len(), 1);
            assert!(
                app.world()
                    .resource::<MissionEncounters>()
                    .pending
                    .iter()
                    .map(VecDeque::len)
                    .sum::<usize>()
                    <= 12
            );
            for (id, kind, p, patrol) in spawned {
                counts[if patrol { 2 } else { usize::from(p.x > 0.) }] += 1;
                slowers += usize::from(kind == crate::economy::runtime::EnemyKind::Slower);
                app.world_mut().despawn(id);
            }
        }
        assert_eq!(counts, [333, 333, 333]);
        assert!(slowers > 50);
    }

    #[test]
    fn all_fixed_requests_survive_saturation_and_share_slots_with_repeat_lanes() {
        let (mut app, drone) = app();
        let blockers: Vec<_> = (0..ENEMY_CAP)
            .map(|_| {
                app.world_mut()
                    .spawn((
                        SpawnWarning::default(),
                        Transform::from_translation(map::start()),
                    ))
                    .id()
            })
            .collect();
        for &(x, z) in &ANCHORS[..5] {
            at(&mut app, drone, map::point(x, z));
            tick(&mut app, 1.);
        }
        assert_eq!(
            app.world().resource::<MissionEncounters>().pending[..5]
                .iter()
                .map(VecDeque::len)
                .collect::<Vec<_>>(),
            [10, 10, 20, 7, 7]
        );
        at(&mut app, drone, map::start());
        app.world_mut().resource_mut::<ObjectiveRun>().visited[0] = true;
        app.world_mut().despawn(blockers[0]);
        for _ in 0..9 {
            tick(&mut app, 10000.);
            let spawned: Vec<_> = app
                .world_mut()
                .query::<(Entity, Option<&Enemy>, Option<&SpawnWarning>)>()
                .iter(app.world())
                .filter(|(_, e, w)| e.is_some() || w.is_some_and(|w| w.ready_at > 0.))
                .map(|(id, _, _)| id)
                .collect();
            assert_eq!(spawned.len(), 1);
            for id in spawned {
                app.world_mut().despawn(id);
            }
        }
        let state = app.world().resource::<MissionEncounters>();
        assert!(
            state.pending[..5]
                .iter()
                .zip([10, 10, 20, 7, 7])
                .all(|(queue, n)| queue.len() < n),
            "no authored group can be starved by repeating pressure"
        );
        assert!(
            state.pending[6..]
                .iter()
                .all(|queue| queue.len() <= SOURCE_BATCH)
        );
        for _ in 0..200 {
            tick(&mut app, 10000.);
            let spawned: Vec<_> = app
                .world_mut()
                .query::<(Entity, Option<&Enemy>, Option<&SpawnWarning>)>()
                .iter(app.world())
                .filter(|(_, e, w)| e.is_some() || w.is_some_and(|w| w.ready_at > 0.))
                .map(|(id, _, _)| id)
                .collect();
            assert_eq!(spawned.len(), 1);
            for id in spawned {
                app.world_mut().despawn(id);
            }
        }
        assert!(
            app.world().resource::<MissionEncounters>().pending[..5]
                .iter()
                .all(VecDeque::is_empty)
        );
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
        // Isolate boundary timing from terrain crossed by this synthetic long sweep.
        app.world_mut()
            .resource_mut::<WorldGeometry>()
            .solids
            .clear();
        let c = map::challenge();
        at(&mut app, drone, c);
        app.world_mut().resource_mut::<PlayerPath>().segments =
            vec![segment(c + Vec3::X * (HOLDOUT_RADIUS * 2.), c, 0., 1.)];
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
        assert_eq!(state.source_next, 0.);
        assert_eq!(state.patrol_elapsed, 0.);
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
    fn mission01_warning_activation_above_thirty_uses_its_cap_and_next_mission_restores_default() {
        use crate::{
            arena::DroneFlight,
            combat::PlayerHealth,
            mission::tests::{app as mission_app, launch, select_placeholder, tick as frame},
        };
        let mut app = mission_app();
        launch(&mut app);
        // Production pursuit bodies far from the patrol, player and gun range.
        for i in 0..30 {
            let p = map::point(80. + (i % 6) as f32 * 7., 85. + (i / 6) as f32 * 7.);
            app.world_mut().spawn((
                Enemy {
                    kind: EnemyKind::Chaser,
                    health: 20,
                    previous: p,
                    path: vec![],
                },
                Transform::from_translation(p),
                DroneFlight::default(),
            ));
        }
        frame(&mut app, 3., &[]);
        assert_eq!(enemies(&mut app), 30);
        assert_eq!(
            app.world_mut()
                .query::<&SpawnWarning>()
                .iter(app.world())
                .count(),
            4
        );
        frame(&mut app, 0.75, &[]);
        assert_eq!(
            enemies(&mut app),
            34,
            "patrol warnings must activate with more than30 live enemies below the Mission01cap"
        );
        assert_eq!(app.world().resource::<WaveConfig>().cap, 96);
        app.world_mut().resource_mut::<PlayerHealth>().current = 0;
        frame(&mut app, 0., &[]);
        // Outcome consumes its frame; release once to arm the menu action.
        frame(&mut app, 0., &[]);
        frame(&mut app, 0., &[KeyCode::Enter]);
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
        select_placeholder(&mut app, 1);
        launch(&mut app);
        assert!(!app.world().resource::<BlockoutRun>().enabled);
        assert_eq!(
            app.world().resource::<WaveConfig>().cap,
            WaveConfig::default().cap
        );
    }

    #[test]
    fn patrol_warning_activation_rechecks_player_and_restart_clears_pressure() {
        use crate::mission::tests::{app as mission_app, launch, tick as frame};
        let mut app = mission_app();
        launch(&mut app);
        frame(&mut app, 3., &[]);
        let warning = app
            .world_mut()
            .query::<&SpawnWarning>()
            .iter(app.world())
            .count();
        assert_eq!(warning, 4);
        let position = app
            .world_mut()
            .query_filtered::<&Transform, With<SpawnWarning>>()
            .iter(app.world())
            .next()
            .unwrap()
            .translation;
        let drone = app
            .world_mut()
            .query_filtered::<Entity, With<Drone>>()
            .single(app.world())
            .unwrap();
        at(&mut app, drone, position);
        frame(&mut app, 0.75, &[]);
        assert!(app.world().resource::<Encounter>().spawns.cancelled >= 1);
        assert!(enemies(&mut app) < 4);
        app.world_mut().resource_mut::<ObjectiveRun>().visited[0] = true;
        frame(&mut app, 60., &[]);
        frame(&mut app, 0., &[KeyCode::KeyR]);
        assert_eq!(enemies(&mut app), 0);
        assert_eq!(
            app.world_mut()
                .query::<&SpawnWarning>()
                .iter(app.world())
                .count(),
            0
        );
        let state = app.world().resource::<MissionEncounters>();
        assert_eq!(state.patrol_elapsed, 0.);
        assert_eq!(state.source_elapsed, None);
        assert!(state.pending.iter().all(VecDeque::is_empty));
    }

    #[test]
    #[ignore = "damage-enabled full-plugin Mission01 stationary and route measurement"]
    fn mission01_stationary_and_moving_pressure_probe() {
        use crate::{
            arena::DroneFlight,
            combat::{Encounter, PlayerHealth},
            energy::Energy,
            mission::tests::{app as mission_app, launch, tick as frame},
        };
        for moving in [false, true] {
            let mut app = mission_app();
            launch(&mut app);
            let drone = app
                .world_mut()
                .query_filtered::<Entity, With<Drone>>()
                .single(app.world())
                .unwrap();
            let route = map::route();
            let mut waypoint = 1;
            let mut first_warning = None;
            let mut first_near = None;
            let mut first_damage = None;
            let mut minimum_health = 100;
            let mut peak = 0;
            let mut active_frames = 0;
            let mut choice_armed = false;
            for _ in 0..60 * 65 + 100 {
                let phase = *app.world().resource::<GamePhase>();
                if matches!(phase, GamePhase::Dead | GamePhase::Survived) {
                    break;
                }
                let p = app.world().get::<Transform>(drone).unwrap().translation;
                let flight = app.world().get::<DroneFlight>(drone).unwrap();
                let keys = if phase == GamePhase::Choosing {
                    choice_armed = !choice_armed;
                    if choice_armed {
                        vec![]
                    } else {
                        vec![KeyCode::Backspace]
                    }
                } else if moving {
                    if p.distance(route[waypoint]) < 140. && waypoint + 1 < route.len() {
                        waypoint += 1;
                    }
                    choice_armed = false;
                    let delta = (route[waypoint] - p).with_y(0.);
                    let desired =
                        delta.normalize_or_zero() * 420_f32.min((200. * delta.length()).sqrt());
                    let acceleration = (desired - flight.velocity.with_y(0.)) * 3.
                        + flight.velocity.with_y(0.) * 0.25;
                    let local = Quat::from_rotation_y(-flight.heading) * acceleration;
                    let mut keys = Vec::new();
                    if local.x.abs() > 15. {
                        keys.push(if local.x > 0. {
                            KeyCode::KeyE
                        } else {
                            KeyCode::KeyQ
                        });
                    }
                    if local.z.abs() > 15. {
                        keys.push(if local.z > 0. {
                            KeyCode::KeyS
                        } else {
                            KeyCode::KeyW
                        });
                    }
                    keys
                } else {
                    vec![]
                };
                frame(&mut app, 1. / 60., &keys);
                if phase != GamePhase::Playing {
                    continue;
                }
                active_frames += 1;
                let elapsed = app.world().resource::<Encounter>().elapsed;
                let warning_count = app
                    .world_mut()
                    .query::<&SpawnWarning>()
                    .iter(app.world())
                    .count();
                let p = app.world().get::<Transform>(drone).unwrap().translation;
                let near = app
                    .world_mut()
                    .query_filtered::<&Transform, With<Enemy>>()
                    .iter(app.world())
                    .map(|t| t.translation.distance(p))
                    .fold(f32::INFINITY, f32::min);
                if warning_count > 0 {
                    first_warning.get_or_insert(elapsed);
                }
                if near <= 400. {
                    first_near.get_or_insert(elapsed);
                }
                let hp = app.world().resource::<PlayerHealth>().current;
                if hp < minimum_health {
                    first_damage.get_or_insert(elapsed);
                    minimum_health = hp;
                }
                peak = peak.max(enemies(&mut app));
                if elapsed >= 60. {
                    break;
                }
            }
            eprintln!(
                "M1 pressure moving={moving} scale={} overrides=none controls={} first_warning={first_warning:?} first_enemy_within400={first_near:?} first_damage={first_damage:?} min_hp={minimum_health} end={:.3}s phase={:?} peak={peak} kills={} energy={:.2} active_frames={active_frames} waypoint={waypoint}",
                map::SCALE,
                if moving {
                    "route pilot, decline upgrades"
                } else {
                    "no movement, decline upgrades"
                },
                app.world().resource::<Encounter>().elapsed,
                app.world().resource::<GamePhase>(),
                app.world().resource::<Encounter>().kills,
                app.world().resource::<Energy>().current
            );
            assert!(first_warning.is_some_and(|t| (3.0..3.05).contains(&t)));
            if !moving {
                assert!(
                    first_near.is_some_and(|t| (5.0..8.1).contains(&t)),
                    "early contact {first_near:?}"
                );
                assert!(
                    first_damage.is_some(),
                    "stationary real combat should deal damage"
                );
            }
        }
    }

    #[test]
    #[ignore = "isolated damage-enabled full-plugin beam and pursuit measurement"]
    fn mission01_beam_pursuit_pressure_probe() {
        use crate::{
            arena::DroneFlight,
            combat::control::{ControlAttack, ControlEffects},
            mission::tests::{app as mission_app, launch, tick as frame},
        };
        let mut app = mission_app();
        launch(&mut app);
        // Controlled combat fixture: disable shooting, place two enemies. All
        // movement, terrain, beam timing, health and damage remain production.
        app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
        let drone = app
            .world_mut()
            .query_filtered::<Entity, With<Drone>>()
            .single(app.world())
            .unwrap();
        let p = app.world().get::<Transform>(drone).unwrap().translation;
        let chaser = app
            .world_mut()
            .spawn((
                Enemy {
                    kind: EnemyKind::Chaser,
                    health: 20,
                    previous: p + Vec3::Z * 650.,
                    path: vec![],
                },
                Transform::from_translation(p + Vec3::Z * 650.),
                DroneFlight::default(),
            ))
            .id();
        app.world_mut().spawn((
            Enemy {
                kind: EnemyKind::Slower,
                health: 40,
                previous: p - Vec3::Z * 180.,
                path: vec![],
            },
            Transform::from_translation(p - Vec3::Z * 180.),
            DroneFlight::default(),
            ControlAttack::default(),
        ));
        let mut slow_frames = 0;
        let mut first = None;
        let mut last = None;
        for index in 0..180 {
            frame(
                &mut app,
                1. / 60.,
                if index < 80 { &[] } else { &[KeyCode::KeyW] },
            );
            if index >= 80
                && app
                    .world()
                    .resource::<ControlEffects>()
                    .movement_multiplier()
                    < 1.
            {
                slow_frames += 1;
                let player = app.world().get::<Transform>(drone).unwrap().translation;
                let enemy = app.world().get::<Transform>(chaser).unwrap().translation;
                let sample = (
                    player.distance(enemy),
                    app.world()
                        .get::<DroneFlight>(drone)
                        .unwrap()
                        .velocity
                        .with_y(0.)
                        .length(),
                    app.world()
                        .get::<DroneFlight>(chaser)
                        .unwrap()
                        .velocity
                        .with_y(0.)
                        .length(),
                );
                first.get_or_insert(sample);
                last = Some(sample);
            }
        }
        eprintln!(
            "M1 beam full-plugin overrides=target_range0, placedSlower180ahead+Chaser650behind controls=still1.333s thenW; slow_frames={slow_frames} first_gap_player_enemy_speed={first:?} last={last:?}"
        );
        assert!(slow_frames > 30);
        assert!(last.unwrap().0 < first.unwrap().0 - 40.);
        assert!(last.unwrap().1 <= 252.1);
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
        assert_eq!(enemies(&mut app), ENEMY_CAP);
        let mut micros = Vec::new();
        for _ in 0..480 {
            let before = std::time::Instant::now();
            frame(&mut app, 1. / 60., &[]);
            micros.push(before.elapsed().as_micros());
            assert_eq!(enemies(&mut app), ENEMY_CAP);
            assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
            assert!(
                app.world()
                    .resource::<MissionEncounters>()
                    .pending
                    .iter()
                    .map(VecDeque::len)
                    .sum::<usize>()
                    <= 12
            );
        }
        micros.sort_unstable();
        eprintln!(
            "Mission01 full ECS schedule, 96 live enemies, 480 frames at 60Hz: min={}us p50={}us p95={}us max={}us",
            micros[0], micros[240], micros[456], micros[479]
        );
    }
}
