use super::{CombatConfig, Enemy, enemies::spawn_enemy};
use crate::arena::{Arena, Drone, drone_world_half_extents, world_half_extents};
use crate::game::GamePhase;
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct WaveConfig {
    pub duration: f64,
    pub cap: usize,
    pub warning_seconds: f64,
    pub clearance: f32,
    pub bursts: Vec<(f64, usize)>,
    phases: Vec<WavePhase>,
}

#[derive(Debug, Clone)]
pub(crate) struct WavePhase {
    pub label: &'static str,
    pub start: f64,
    pub end: f64,
    pub lull_start: f64,
    first_warning: usize,
    burst_size: usize,
    interval: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WaveStatus {
    Active,
    Lull,
}

impl Default for WaveConfig {
    fn default() -> Self {
        let phases = vec![
            WavePhase {
                label: "OPENING",
                start: 0.,
                end: 30.,
                lull_start: 24.,
                first_warning: 3,
                burst_size: 5,
                interval: 8,
            },
            WavePhase {
                label: "PRESSURE I",
                start: 30.,
                end: 105.,
                lull_start: 99.,
                first_warning: 30,
                burst_size: 7,
                interval: 8,
            },
            WavePhase {
                label: "PRESSURE II",
                start: 105.,
                end: 165.,
                lull_start: 159.,
                first_warning: 105,
                burst_size: 8,
                interval: 6,
            },
            WavePhase {
                label: "PRESSURE III",
                start: 165.,
                end: 225.,
                lull_start: 219.,
                first_warning: 165,
                burst_size: 9,
                interval: 5,
            },
            WavePhase {
                label: "FINAL PUSH",
                start: 225.,
                end: 300.,
                lull_start: 294.,
                first_warning: 225,
                burst_size: 11,
                interval: 4,
            },
        ];
        let bursts = phases
            .iter()
            .flat_map(|phase| {
                (phase.first_warning..phase.lull_start as usize)
                    .step_by(phase.interval)
                    .map(|at| (at as f64, phase.burst_size))
            })
            .collect();
        Self {
            duration: phases.last().unwrap().end,
            cap: 30,
            warning_seconds: 0.75,
            clearance: 120.,
            bursts,
            phases,
        }
    }
}

impl WaveConfig {
    pub(super) fn disable_authored_waves(&mut self) {
        self.bursts.clear();
        self.phases.clear();
    }

    pub(crate) fn phase_at(&self, elapsed: f64) -> Option<&WavePhase> {
        self.phases
            .iter()
            .find(|phase| elapsed >= phase.start && elapsed < phase.end)
    }

    pub(crate) fn status_at(&self, elapsed: f64) -> Option<WaveStatus> {
        self.phase_at(elapsed).map(|phase| {
            if elapsed >= phase.lull_start {
                WaveStatus::Lull
            } else {
                WaveStatus::Active
            }
        })
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpawnCounts {
    pub requested: usize,
    pub admitted: usize,
    pub rejected_cap: usize,
    pub rejected_space: usize,
    pub skipped_hitch: usize,
    pub skipped_terminal: usize,
    pub activated: usize,
    pub cancelled: usize,
}

#[derive(Resource, Default)]
pub(crate) struct Encounter {
    pub elapsed: f64,
    pub kills: u32,
    pub next_burst: usize,
    pub candidate: usize,
    pub spawns: SpawnCounts,
}

#[derive(Component, Default)]
pub(super) struct SpawnWarning {
    pub ready_at: f64,
    pub cancelled: bool,
}

pub(super) fn advance_clock(time: Res<Time>, config: Res<WaveConfig>, mut run: ResMut<Encounter>) {
    run.elapsed = (run.elapsed + time.delta_secs_f64()).min(config.duration);
}

pub(super) fn finish(config: Res<WaveConfig>, run: Res<Encounter>, mut phase: ResMut<GamePhase>) {
    if *phase == GamePhase::Playing && run.elapsed + 1e-7 >= config.duration {
        *phase = GamePhase::Survived;
    }
}

// Conservative for any heading/tilt at activation, including a moving player.
fn spawn_half(config: &CombatConfig) -> Vec3 {
    Vec3::splat(config.enemy_half_size * 3_f32.sqrt())
}

#[allow(clippy::too_many_arguments)]
fn safe(
    position: Vec3,
    half: Vec3,
    arena: &Arena,
    player: &Transform,
    clearance: f32,
    occupied: &[(Entity, Vec3, Vec3)],
    ignore: Option<Entity>,
    world: Option<&crate::world::WorldGeometry>,
) -> bool {
    let fits = (position - arena.center())
        .abs()
        .cmple(arena.half_size - half)
        .all();
    let gap =
        ((position - player.translation).abs() - half - drone_world_half_extents(player.rotation))
            .max(Vec3::ZERO)
            .length();
    let navigable = world.is_none_or(|world| {
        !world.solids.iter().any(|s| s.overlaps(position, half))
            && !world.hazard.is_some_and(|s| s.overlaps(position, half))
            && crate::world::navigation::next_point(world, position, player.translation, half)
                .is_some()
    });
    fits && navigable
        && gap >= clearance
        && occupied.iter().all(|(id, p, h)| {
            Some(*id) == ignore
                || (position - *p)
                    .abs()
                    .cmpgt(half + *h + Vec3::splat(1.))
                    .any()
        })
}

// 4 sides x 5 offsets x 3 altitudes, permuted to alternate sides and heights.
fn candidate(index: usize, arena: &Arena, half: Vec3) -> Vec3 {
    let n = (index % 60) * 37 % 60;
    let side = n % 4;
    let along = ((n / 4) % 5) as f32 / 4. * 1.6 - 0.8;
    let height = (n / 20) as f32 / 2. * 0.7 + 0.15;
    let extent = (arena.half_size - half - Vec3::splat(2.)).max(Vec3::ZERO);
    let mut p = arena.center();
    p.y += (height * 2. - 1.) * extent.y;
    if side < 2 {
        p.x = if side == 0 { -extent.x } else { extent.x };
        p.z = along * extent.z;
    } else {
        p.z = if side == 2 { -extent.z } else { extent.z };
        p.x = along * extent.x;
    }
    p
}

type Occupants<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        Option<&'static Enemy>,
        Option<&'static mut SpawnWarning>,
    ),
    Or<(With<Enemy>, With<SpawnWarning>)>,
>;

#[allow(clippy::too_many_arguments)]
pub(super) fn update(
    mut commands: Commands,
    config: Res<WaveConfig>,
    combat: Res<CombatConfig>,
    arena: Res<Arena>,
    world: Option<Res<crate::world::WorldGeometry>>,
    player: Single<&Transform, With<Drone>>,
    phase: Res<GamePhase>,
    mut run: ResMut<Encounter>,
    mut occupants: Occupants,
) {
    let mut warnings = Vec::new();
    let mut occupied = Vec::new();
    let half = spawn_half(&combat);
    let mut live = 0;
    for (id, transform, enemy, warning) in &mut occupants {
        if let Some(mut w) = warning {
            if w.cancelled {
                continue;
            }
            if matches!(*phase, GamePhase::Dead | GamePhase::Survived) {
                // Despawns are deferred; a second cleanup must see this decision
                // immediately so it cannot count or queue the same warning twice.
                w.cancelled = true;
            }
            warnings.push((id, transform.translation, w.ready_at));
            occupied.push((id, transform.translation, half));
        } else if enemy.is_some_and(|e| e.health > 0) {
            live += 1;
            occupied.push((
                id,
                transform.translation,
                world_half_extents(transform.rotation, Vec3::splat(combat.enemy_half_size)),
            ));
        }
    }
    warnings.sort_by_key(|(id, _, _)| id.to_bits());
    if *phase == GamePhase::Choosing {
        return;
    }
    if *phase != GamePhase::Playing {
        // The outcome wins over spawning, but due requests still need an outcome
        // in the report (including a hitch crossing the final authored burst).
        while let Some(&(at, count)) = config.bursts.get(run.next_burst) {
            if run.elapsed + 1e-7 < at {
                break;
            }
            run.spawns.requested += count;
            run.spawns.skipped_terminal += count;
            run.next_burst += 1;
        }
        run.spawns.cancelled += warnings.len();
        for (id, _, _) in warnings {
            commands.entity(id).despawn();
        }
        return;
    }
    let mut pending = warnings.len();
    for (id, position, ready_at) in warnings {
        if run.elapsed + 1e-7 < ready_at {
            continue;
        }
        commands.entity(id).despawn();
        pending -= 1;
        if live < config.cap
            && safe(
                position,
                half,
                &arena,
                &player,
                config.clearance,
                &occupied,
                Some(id),
                world.as_deref(),
            )
        {
            spawn_enemy(&mut commands, &combat, position, player.translation);
            live += 1;
            run.spawns.activated += 1;
        } else {
            occupied.retain(|(other, _, _)| *other != id);
            run.spawns.cancelled += 1;
        }
    }
    let mut burst = None;
    while let Some(&(at, count)) = config.bursts.get(run.next_burst) {
        if run.elapsed + 1e-7 < at {
            break;
        }
        run.spawns.requested += count;
        if let Some(discarded) = burst.replace(count) {
            run.spawns.skipped_hitch += discarded;
        }
        run.next_burst += 1;
    }
    let Some(requested) = burst else {
        return;
    };
    let count = requested.min(config.cap.saturating_sub(live + pending));
    run.spawns.rejected_cap += requested - count;
    for offset in 0..count {
        let mut selected = None;
        for _ in 0..60 {
            let position = candidate(run.candidate, &arena, half);
            run.candidate = (run.candidate + 1) % 60;
            if safe(
                position,
                half,
                &arena,
                &player,
                config.clearance,
                &occupied,
                None,
                world.as_deref(),
            ) {
                selected = Some(position);
                break;
            }
        }
        let Some(position) = selected else {
            run.spawns.rejected_space += count - offset;
            break;
        };
        let id = commands
            .spawn((
                SpawnWarning {
                    ready_at: run.elapsed + config.warning_seconds,
                    ..default()
                },
                Transform::from_translation(position),
            ))
            .id();
        occupied.push((id, position, half));
        run.spawns.admitted += 1;
    }
}

#[cfg(test)]
mod terrain_tests {
    use super::*;
    use crate::world::{Solid, WorldGeometry};
    #[test]
    fn warning_creation_and_activation_reject_solid_hazard_and_disconnected_locations() {
        let mut app = App::new();
        app.insert_resource(WaveConfig::default())
            .insert_resource(CombatConfig::default())
            .insert_resource(Arena::default())
            .insert_resource(GamePhase::Playing)
            .insert_resource(Encounter {
                elapsed: 3.,
                ..default()
            })
            .insert_resource(WorldGeometry {
                solids: vec![Solid {
                    center: Vec3::new(0., 150., 0.),
                    half: Vec3::new(480., 150., 270.),
                }],
                hazard: None,
            })
            .add_systems(Update, update);
        app.world_mut()
            .spawn((Drone, Transform::from_xyz(-280., 90., 0.)));
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&SpawnWarning>()
                .iter(app.world())
                .count(),
            0
        );
        // A warning that was valid before geometry changed must also be rejected.
        app.world_mut().spawn((
            SpawnWarning {
                ready_at: 3.,
                ..default()
            },
            Transform::from_xyz(280., 90., 0.),
        ));
        app.update();
        assert_eq!(
            app.world_mut().query::<&Enemy>().iter(app.world()).count(),
            0
        );
    }
    #[test]
    fn spawn_candidates_require_hazard_clearance_and_a_connected_route() {
        let arena = Arena::default();
        let player = Transform::from_xyz(-280., 90., 0.);
        let position = Vec3::new(280., 90., 0.);
        let half = spawn_half(&CombatConfig::default());
        for world in [
            WorldGeometry {
                solids: Vec::new(),
                hazard: Some(Solid {
                    center: position,
                    half: Vec3::splat(30.),
                }),
            },
            WorldGeometry {
                solids: vec![Solid {
                    center: Vec3::new(0., 150., 0.),
                    half: Vec3::new(5., 150., 270.),
                }],
                hazard: None,
            },
        ] {
            assert!(!safe(
                position,
                half,
                &arena,
                &player,
                120.,
                &[],
                None,
                Some(&world)
            ));
        }
        assert!(safe(
            position,
            half,
            &arena,
            &player,
            120.,
            &[],
            None,
            Some(&WorldGeometry::default())
        ));
    }
}
